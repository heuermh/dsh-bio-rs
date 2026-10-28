/*

    dsh-bio-rs  Rewrite of dishevelled.org bio in rust
    Copyright (c) 2026 held jointly by the individual authors.

    This library is free software; you can redistribute it and/or modify it
    under the terms of the GNU Lesser General Public License as published
    by the Free Software Foundation; either version 3 of the License, or (at
    your option) any later version.

    This library is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; with out even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this library;  if not, write to the Free Software Foundation,
    Inc., 59 Temple Place, Suite 330, Boston, MA 02111-1307  USA.

    > http://www.fsf.org/licensing/licenses/lgpl.html
    > http://www.opensource.org/licenses/lgpl-license.php

*/
use std::error::Error;

use duckdb::Appender;
use duckdb::appender_params_from_iter;
use duckdb::types::{ToSqlOutput, ValueRef};

use noodles::vcf;

use crate::cli::VcfColumnArgs;
use crate::sql;

const MISSING: &str = ".";
const PASS: &str = "PASS";
const GT: &str = "GT";

/// Variants table structure and row values for VCF records
///
/// To avoid building a list value per cell, which dominates appender time, each VARCHAR[] column
/// is appended as raw VARCHAR text borrowed from the record, or NULL for an empty list, and split
/// into a list when selecting from the table.
pub struct VcfTable {
    info_fields: Vec<String>,
    info_columns: Vec<String>,
    sample_indices: Vec<usize>,
    format_fields: Vec<String>,
    sample_columns: Vec<String>,
    multiallelic: bool,
}

impl VcfTable {
    /// Create a new variants table structure, checking that each sample is present in the header
    pub fn new(header: &vcf::Header, args: &VcfColumnArgs) -> Result<Self, Box<dyn Error>> {
        let name = |s: &str| {
            if args.lowercase {
                s.to_lowercase()
            } else {
                s.to_string()
            }
        };

        let info_columns = args
            .info_fields
            .iter()
            .map(|field| format!("{}{}", args.info_prefix, name(field)))
            .collect();

        let mut sample_indices = Vec::with_capacity(args.samples.len());
        let mut sample_columns = Vec::with_capacity(args.samples.len() * args.format_fields.len());
        for sample in &args.samples {
            let index = header
                .sample_names()
                .get_index_of(sample)
                .ok_or_else(|| format!("sample {} not found in VCF header", sample))?;
            sample_indices.push(index);
            for field in &args.format_fields {
                sample_columns.push(format!(
                    "{}{}_{}",
                    args.sample_prefix,
                    name(sample),
                    name(field)
                ));
            }
        }

        Ok(Self {
            info_fields: args.info_fields.clone(),
            info_columns,
            sample_indices,
            format_fields: args.format_fields.clone(),
            sample_columns,
            multiallelic: args.multiallelic,
        })
    }

    /// Return the CREATE TABLE SQL for the specified table name, with list columns as VARCHAR
    pub fn create_table_sql(&self, table: &str) -> String {
        let mut sql = format!(
            "CREATE TABLE {} (chrom VARCHAR, pos LONG, ref VARCHAR, alt VARCHAR, qual DOUBLE, filters_applied BOOLEAN, filters_passed BOOLEAN, filters_failed VARCHAR",
            table
        );
        for column in self.info_columns.iter().chain(&self.sample_columns) {
            sql.push_str(", ");
            sql.push_str(&sql::identifier(column));
            sql.push_str(" VARCHAR");
        }
        sql.push(')');
        sql
    }

    /// Return the SELECT SQL for the specified table name, splitting list columns into VARCHAR[]
    pub fn select_sql(&self, table: &str) -> String {
        let alt = if self.multiallelic {
            split_sql("alt", ',')
        } else {
            String::from("alt")
        };
        let mut sql = format!(
            "SELECT chrom, pos, ref, {}, qual, filters_applied, filters_passed, {}",
            alt,
            split_sql("filters_failed", ';')
        );
        for column in self.info_columns.iter().chain(&self.sample_columns) {
            sql.push_str(", ");
            sql.push_str(&split_sql(&sql::identifier(column), ','));
        }
        sql.push_str(" FROM ");
        sql.push_str(table);
        sql
    }

    /// Append a row for the specified record
    pub fn append_row(
        &self,
        record: &vcf::Record,
        appender: &mut Appender<'_>,
    ) -> Result<(), Box<dyn Error>> {
        // field wrappers must outlive the row, which borrows from them
        let alternate_bases = record.alternate_bases();
        let filters = record.filters();
        let info = record.info();
        let samples = record.samples();

        let mut row: Vec<ToSqlOutput<'_>> =
            Vec::with_capacity(8 + self.info_columns.len() + self.sample_columns.len());

        let position = record.variant_start().transpose()?;
        let pos = position.map(usize::from).unwrap_or_default();
        row.push(text(record.reference_sequence_name()));
        row.push(ToSqlOutput::Borrowed(ValueRef::BigInt(i64::try_from(pos)?)));
        row.push(text(record.reference_bases()));

        // alternate alleles, one per row unless multiallelic
        let alt = alternate_bases.as_ref();
        if self.multiallelic {
            row.push(list_text(alt));
        } else if alt.contains(',') {
            return Err(format!(
                "multiallelic variants not supported without --multiallelic, found alternate alleles {} at {}:{}",
                alt,
                record.reference_sequence_name(),
                pos
            )
            .into());
        } else {
            row.push(text(alt));
        }

        // widen via the shortest decimal representation, so a QUAL of 29.6 stays 29.6
        let qual = record
            .quality_score()
            .transpose()?
            .map(|q| q.to_string().parse::<f64>().unwrap_or(f64::from(q)));
        row.push(qual.map_or(NULL, |q| ToSqlOutput::Borrowed(ValueRef::Double(q))));

        // filters_applied, filters_passed, filters_failed
        let (applied, passed, failed) = match filters.as_ref() {
            "" | MISSING => (false, false, NULL),
            PASS => (true, true, NULL),
            failed => (true, false, text(failed)),
        };
        row.push(ToSqlOutput::Borrowed(ValueRef::Boolean(applied)));
        row.push(ToSqlOutput::Borrowed(ValueRef::Boolean(passed)));
        row.push(failed);

        self.push_info_values(info.as_ref(), &mut row);
        self.push_sample_values(samples.as_ref(), &mut row)?;

        appender.append_row(appender_params_from_iter(row))?;
        Ok(())
    }

    /// Push one value per INFO field: a flag is "true", a missing value is NULL
    fn push_info_values<'r>(&self, info: &'r str, row: &mut Vec<ToSqlOutput<'r>>) {
        let base = row.len();
        row.resize(base + self.info_fields.len(), NULL);
        if matches!(info, "" | MISSING) {
            return;
        }
        for entry in info.split(';') {
            let (key, value) = entry.split_once('=').unwrap_or((entry, "true"));
            if let Some(i) = self.info_fields.iter().position(|field| field == key) {
                row[base + i] = list_text(value);
            }
        }
    }

    /// Push one value per sample and FORMAT field: GT, when first, is kept as is, a missing value
    /// is NULL
    fn push_sample_values<'r>(
        &self,
        samples: &'r str,
        row: &mut Vec<ToSqlOutput<'r>>,
    ) -> Result<(), Box<dyn Error>> {
        if self.sample_indices.is_empty() || self.format_fields.is_empty() {
            return Ok(());
        }

        let keys = samples.split('\t').next().unwrap_or_default();
        let key_count = keys.split(':').count();
        let gt_first = keys.split(':').next() == Some(GT);

        for &sample_index in &self.sample_indices {
            let sample = samples
                .split('\t')
                .nth(1 + sample_index)
                .unwrap_or_default();
            if sample.split(':').count() > key_count {
                return Err(format!("too many genotype fields in {}", samples).into());
            }
            for field in &self.format_fields {
                let key_index = keys.split(':').position(|key| key == field);
                let value = key_index.and_then(|k| sample.split(':').nth(k).map(|v| (k, v)));
                row.push(match value {
                    Some((0, gt)) if gt_first => text(gt),
                    Some((_, v)) => list_text(v),
                    None => NULL,
                });
            }
        }
        Ok(())
    }
}

const NULL: ToSqlOutput<'static> = ToSqlOutput::Borrowed(ValueRef::Null);

/// Borrow text as a VARCHAR value
fn text(s: &str) -> ToSqlOutput<'_> {
    ToSqlOutput::Borrowed(ValueRef::Text(s.as_bytes()))
}

/// Borrow text for a list column, where an empty or missing (".") field is NULL, an empty list
fn list_text(s: &str) -> ToSqlOutput<'_> {
    match s {
        "" | MISSING => NULL,
        _ => text(s),
    }
}

/// Return SQL splitting a list column on the specified delimiter, with NULL as an empty list
fn split_sql(column: &str, delimiter: char) -> String {
    format!(
        "coalesce(string_split({}, '{}'), []::VARCHAR[]) AS {}",
        column, delimiter, column
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sql() {
        assert_eq!(
            split_sql("\"AF\"", ','),
            "coalesce(string_split(\"AF\", ','), []::VARCHAR[]) AS \"AF\""
        );
    }
}
