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
) -> Result<(), Box<dyn Error>> {
    let mut db = Connection::open_in_memory()?;

    // create vcf reader, reading the header first for sample names
    info!("Reading variants from {}", input_vcf_path.display());
    let mut reader = vcf::io::Reader::new(input::open(&input_vcf_path)?);
    let header = reader.read_header()?;
    let table = VcfTable::new(&header, columns)?;

    // create variants table
    info!("Creating variants table");
    db.execute_batch(&table.create_table_sql("variants"))?;

    {
        // start transaction for appender
        let mut tx = db.transaction()?;
        tx.set_drop_behavior(DropBehavior::Commit);

        // create appender
        let mut appender = tx.appender("variants")?;

        // append each vcf record, reusing a single record buffer
        let mut record = vcf::Record::default();
        while reader.read_record(&mut record)? != 0 {
            table.append_row(&record, &mut appender)?;
        }
    }

    // allow parallel, lower-memory copy
    db.execute_batch("SET preserve_insertion_order = false")?;

    // copy to parquet, splitting list columns
    let copy_sql = format!(
        "COPY ({}) TO {} (FORMAT 'parquet', COMPRESSION 'zstd', OVERWRITE_OR_IGNORE 1, ROW_GROUP_SIZE {}, PER_THREAD_OUTPUT)",
        table.select_sql("variants"),
        sql::string_literal(&output_parquet_file)?,
        row_group_size
    );

    info!(
        "Copying variants to {} in Parquet format",
        output_parquet_file.display()
    );
    db.execute_batch(&copy_sql)?;

    Ok(())
}
