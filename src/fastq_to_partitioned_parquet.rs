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

use bstr::ByteSlice;

use duckdb::{Connection, DropBehavior, params};

use noodles::fastq;

use log::info;

use crate::{input, sql};

pub fn run(
    input_fastq_path: PathBuf,
    output_parquet_file: PathBuf,
    row_group_size: u64,
    partition_size: u64,
) -> Result<(), Box<dyn Error>> {
    // validate arguments
    if partition_size == 0 {
        return Err("partition size must be greater than zero".into());
    }

    let mut db = Connection::open_in_memory()?;

    // allow parallel, lower-memory copy
    db.execute_batch("SET preserve_insertion_order = false")?;

    sql::string_literal(&output_parquet_file)?;
    fs::create_dir_all(&output_parquet_file)?;

    info!("Reading sequences from {}", input_fastq_path.display());

    // create fastq reader
    let mut reader = fastq::io::Reader::new(input::open(&input_fastq_path)?);

    // reuse a single record buffer across all partitions
    let mut record = fastq::Record::default();

    let mut rows: u64 = 0;
    let mut first_row: u64 = 0;
    loop {
        // create sequence table for this partition
        let table = format!("s{}", first_row);
        db.execute_batch(&format!(
            "CREATE TABLE {} (name VARCHAR, description VARCHAR, sequence VARCHAR, quality VARCHAR, length INTEGER, alphabet VARCHAR)",
            table
        ))?;

        let mut partition_rows: u64 = 0;
        let mut done = false;
        {
            // start transaction for appender
            let mut tx = db.transaction()?;
            tx.set_drop_behavior(DropBehavior::Commit);

            // create appender
            let mut appender = tx.appender(&table)?;

            // append fastq records until the partition is full
            while partition_rows < partition_size {
                if reader.read_record(&mut record)? == 0 {
                    done = true;
                    break;
                }
                let name = record.name().to_str_lossy();
                let description = record.description().to_str_lossy();
                let sequence = record.sequence().to_str()?;
                let quality = record.quality_scores().to_str()?;
                let length = sequence.len();

                appender.append_row(params![
                    name,
                    description,
                    sequence,
                    quality,
                    length,
                    "dna"
                ])?;
                partition_rows += 1;
            }
            rows += partition_rows;
        }

        // copy partition to parquet, skipping an empty trailing partition unless there are no rows at all
        if partition_rows > 0 || rows == 0 {
            let partition_file =
                output_parquet_file.join(format!("part-{}-{}.parquet", first_row, rows));
            info!(
                "Copying {} sequences to {} in Parquet format",
                partition_rows,
                partition_file.display()
            );
            db.execute_batch(&format!(
                "COPY {} TO {} (FORMAT 'parquet', COMPRESSION 'zstd', OVERWRITE_OR_IGNORE 1, ROW_GROUP_SIZE {})",
                table,
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
