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

use duckdb::types::Value;

use noodles::vcf;

use crate::cli::VcfColumnArgs;
use crate::sql;

const MISSING: &str = ".";
const PASS: &str = "PASS";
const GT: &str = "GT";

/// Variants table structure and row values for VCF records
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

    /// Return the CREATE TABLE SQL for the specified table name
    pub fn create_table_sql(&self, table: &str) -> String {
        let alt_type = if self.multiallelic {
            "VARCHAR[]"
        } else {
            "VARCHAR"
        };
        let mut sql = format!(
            "CREATE TABLE {} (chrom VARCHAR, pos LONG, ref VARCHAR, alt {}, qual DOUBLE, filters_applied BOOLEAN, filters_passed BOOLEAN, filters_failed VARCHAR[]",
            table, alt_type
        );
        for column in self.info_columns.iter().chain(&self.sample_columns) {
            sql.push_str(", ");
            sql.push_str(&sql::identifier(column));
            sql.push_str(" VARCHAR[]");
        }
        sql.push(')');
        sql
    }

    /// Append the column values for the specified record to row
    pub fn append_row_values(
        &self,
        record: &vcf::Record,
        row: &mut Vec<Value>,
    ) -> Result<(), Box<dyn Error>> {
        let position = record.variant_start().transpose()?;
        let pos = position.map(usize::from).unwrap_or_default();
        row.push(Value::Text(record.reference_sequence_name().to_string()));
        row.push(Value::BigInt(pos as i64));
        row.push(Value::Text(record.reference_bases().to_string()));

        // alternate alleles, one per row unless multiallelic
        let alternate_bases = record.alternate_bases();
        let alt = split(alternate_bases.as_ref(), ',');
        if self.multiallelic {
            row.push(list(alt));
        } else {
            match alt.as_slice() {
                [] => row.push(Value::Text(String::new())),
                [allele] => row.push(Value::Text(allele.to_string())),
                _ => {
                    return Err(format!(
                            "multiallelic variants not supported without --multiallelic, found alternate alleles {} at {}:{}",
                            alt.join(","),
                            record.reference_sequence_name(),
                            pos
                        ).into());
                }
            }
        }

        // widen via the shortest decimal representation, so a QUAL of 29.6 stays 29.6
        let qual = record
            .quality_score()
            .transpose()?
            .map(|q| q.to_string().parse::<f64>().unwrap_or(f64::from(q)));
        row.push(qual.map_or(Value::Null, Value::Double));

        // filters_applied, filters_passed, filters_failed
        let filters = record.filters();
        let filters = split(filters.as_ref(), ';');
        let (applied, passed, failed) = match filters.as_slice() {
            [] => (false, false, Vec::new()),
            [PASS] => (true, true, Vec::new()),
            _ => (true, false, filters),
        };
        row.push(Value::Boolean(applied));
        row.push(Value::Boolean(passed));
        row.push(list(failed));

        self.append_info_values(record.info().as_ref(), row);
        self.append_sample_values(record.samples().as_ref(), row)
    }

    /// Append one list per INFO field: a flag is ["true"], a missing value is []
    fn append_info_values(&self, info: &str, row: &mut Vec<Value>) {
        let mut values: Vec<Vec<&str>> = vec![Vec::new(); self.info_fields.len()];
        for entry in split(info, ';') {
            let (key, value) = entry.split_once('=').unwrap_or((entry, "true"));
            if let Some(i) = self.info_fields.iter().position(|field| field == key) {
                values[i] = split(value, ',');
            }
        }
        row.extend(values.into_iter().map(list));
    }

    /// Append one list per sample and FORMAT field: GT, when first, is a single value, a
    /// missing value is []
    fn append_sample_values(
        &self,
        samples: &str,
        row: &mut Vec<Value>,
    ) -> Result<(), Box<dyn Error>> {
        if self.sample_indices.is_empty() || self.format_fields.is_empty() {
            return Ok(());
        }

        let mut columns = samples.split('\t');
        let keys: Vec<&str> = columns
            .next()
            .map(|s| s.split(':').collect())
            .unwrap_or_default();
        let columns: Vec<&str> = columns.collect();
        let key_indices: Vec<Option<usize>> = self
            .format_fields
            .iter()
            .map(|field| keys.iter().position(|key| key == field))
            .collect();

        for &sample_index in &self.sample_indices {
            let values: Vec<&str> = columns
                .get(sample_index)
                .map(|s| s.split(':').collect())
                .unwrap_or_default();
            if values.len() > keys.len() {
                return Err(format!("too many genotype fields in {}", samples).into());
            }
            for &key_index in &key_indices {
                let value = match key_index.and_then(|k| values.get(k).map(|v| (k, *v))) {
                    Some((0, gt)) if keys[0] == GT => vec![gt],
                    Some((_, v)) => split(v, ','),
                    None => Vec::new(),
                };
                row.push(list(value));
            }
        }
        Ok(())
    }
}

/// Split a field into values, where an empty or missing (".") field has no values
fn split(field: &str, delimiter: char) -> Vec<&str> {
    match field {
        "" | MISSING => Vec::new(),
        _ => field.split(delimiter).collect(),
    }
}

/// Convert values to a VARCHAR[] list value
fn list(values: Vec<&str>) -> Value {
    Value::List(
        values
            .into_iter()
            .map(|v| Value::Text(v.to_string()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        assert!(split("", ',').is_empty());
        assert!(split(".", ',').is_empty());
        assert_eq!(split("A", ','), vec!["A"]);
        assert_eq!(split("A,C", ','), vec!["A", "C"]);
    }
}
