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

//use bstr::ByteSlice;

use duckdb::{Connection, DropBehavior, params};

use noodles::vcf;

use log::info;

pub fn run(
    input_vcf_path: PathBuf,
    output_parquet_file: PathBuf,
    row_group_size: u64,
) -> Result<(), Box<dyn Error>> {
    let mut db = Connection::open_in_memory()?;

    // create variants table
    info!("Creating variants table");
    let create_table_sql = "CREATE TABLE variants (chrom VARCHAR, pos LONG, ref VARCHAR, alt VARCHAR, qual DOUBLE, filters_applied BOOLEAN, filters_passed BOOLEAN, filters_failed VARCHAR[]";
    db.execute_batch(create_table_sql)?;

    info!("Reading variants from {}", input_vcf_path.display());
    {
        // start transaction for appender
        let mut tx = db.transaction()?;
        tx.set_drop_behavior(DropBehavior::Commit);

        // create appender
        let mut appender = tx.appender("variants")?;

        // create vcf reader
        let mut reader = vcf::io::reader::Builder::default().build_from_path(input_vcf_path)?;
        reader.read_header()?;

        // append each vcf record
        for result in reader.records() {
            let record = result?;
            let position = record.variant_start().transpose()?;

            let chrom = record.reference_sequence_name();
            let pos = position.map(usize::from).unwrap_or_default();
            let reference = record.reference_bases();
            let alt: Option<String> = None;
            let qual: Option<String> = None;
            let filters_applied: Vec<u8> = Vec::new();
            let filters_passed: Vec<u8> = Vec::new();
            let filters_failed: Vec<u8> = Vec::new();

            appender.append_row(params![chrom, pos, reference, alt, qual, filters_applied, filters_passed, filters_failed])?;
        }
    }

    // copy to parquet
    let copy_sql = format!(
        "COPY variants TO '{}' (FORMAT 'parquet', COMPRESSION 'zstd', OVERWRITE_OR_IGNORE 1, ROW_GROUP_SIZE {}, PER_THREAD_OUTPUT)",
        output_parquet_file.to_string_lossy(),
        row_group_size
    );

    info!(
        "Copying variants to {} in Parquet format",
        output_parquet_file.display()
    );
    db.execute_batch(&copy_sql)?;

    Ok(())
}
