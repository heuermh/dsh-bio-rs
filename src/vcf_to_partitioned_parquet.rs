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
use std::fs;
use std::path::PathBuf;

use duckdb::{Connection, DropBehavior};

use noodles::vcf;

use log::info;

use crate::cli::VcfColumnArgs;
use crate::vcf_table::VcfTable;
use crate::{input, sql};

pub fn run(
    input_vcf_path: PathBuf,
    output_parquet_file: PathBuf,
    columns: &VcfColumnArgs,
    row_group_size: u64,
    partition_size: u64,
) -> Result<(), Box<dyn Error>> {
    // validate parameters
    if partition_size == 0 {
        return Err("partition size must be greater than zero".into());
    }

    let mut db = Connection::open_in_memory()?;

    // allow parallel, lower-memory copy
    db.execute_batch("SET preserve_insertion_order = false")?;

    info!("Reading variants from {}", input_vcf_path.display());

    // create vcf reader, reading the header first for sample names
    let mut reader = vcf::io::Reader::new(input::open(&input_vcf_path)?);
    let header = reader.read_header()?;
    let vcf_table = VcfTable::new(&header, columns)?;

    sql::string_literal(&output_parquet_file)?;
    fs::create_dir_all(&output_parquet_file)?;

    // reuse a single record buffer across all partitions
    let mut record = vcf::Record::default();

    let mut rows: u64 = 0;
    let mut first_row: u64 = 0;
    loop {
        // create variants table for this partition
        let table = format!("v{}", first_row);
        db.execute_batch(&vcf_table.create_table_sql(&table))?;

        let mut partition_rows: u64 = 0;
        let mut done = false;
        {
            // start transaction for appender
            let mut tx = db.transaction()?;
            tx.set_drop_behavior(DropBehavior::Commit);

            // create appender
            let mut appender = tx.appender(&table)?;

            // append vcf records until the partition is full
            while partition_rows < partition_size {
                if reader.read_record(&mut record)? == 0 {
                    done = true;
                    break;
                }
                vcf_table.append_row(&record, &mut appender)?;
                partition_rows += 1;
            }
            rows += partition_rows;
        }

        // copy partition to parquet, skipping an empty trailing partition unless there are no rows at all
        if partition_rows > 0 || rows == 0 {
            let partition_file =
                output_parquet_file.join(format!("part-{}-{}.parquet", first_row, rows));
            info!(
                "Copying {} variants to {} in Parquet format",
                partition_rows,
                partition_file.display()
            );
            db.execute_batch(&format!(
                "COPY ({}) TO {} (FORMAT 'parquet', COMPRESSION 'zstd', OVERWRITE_OR_IGNORE 1, ROW_GROUP_SIZE {})",
                vcf_table.select_sql(&table),
                sql::string_literal(&partition_file)?,
                row_group_size
            ))?;
        }
        db.execute_batch(&format!("DROP TABLE {}", table))?;

        // stop at EOF, or when the input ends exactly on a partition boundary
        if done {
            break;
        }
        first_row = rows + 1;
    }

    Ok(())
}
