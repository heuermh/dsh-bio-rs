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

use bstr::ByteSlice;

use duckdb::{Connection, DropBehavior, params};

use noodles::fasta;

use log::info;

pub fn run(
    input_fasta_path: PathBuf,
    output_parquet_file: PathBuf,
    alphabet: String,
    row_group_size: u64,
) -> Result<(), Box<dyn Error>> {
    let mut db = Connection::open_in_memory()?;

    // initialize parquet extension
    info!("Initializing duckdb parquet extension");
    db.execute_batch("INSTALL parquet; LOAD parquet;")?;

    // create sequence table
    info!("Creating sequence table");
    let create_table_sql = "CREATE TABLE sequences (name VARCHAR, description VARCHAR, sequence VARCHAR, length INTEGER, alphabet VARCHAR)";
    db.execute_batch(create_table_sql)?;

    info!("Reading sequences from {}", input_fasta_path.display());
    {
        // start transaction for appender
        let mut tx = db.transaction()?;
        tx.set_drop_behavior(DropBehavior::Commit);

        // create appender
        let mut appender = tx.appender("sequences")?;

        // create fasta reader
        let mut reader = fasta::io::reader::Builder.build_from_path(input_fasta_path)?;

        // append each fasta record
        for result in reader.records() {
            let record = result?;
            let name = record.name();
            let description = record.description().map(|s| s.to_str_lossy());
            let sequence = record.sequence().as_ref();
            let length = sequence.len();

            appender.append_row(params![name, description, sequence, length, alphabet])?;
        }
    }

    // copy to parquet
    let copy_sql = format!(
        "COPY sequences TO '{}' (FORMAT 'parquet', COMPRESSION 'zstd', OVERWRITE_OR_IGNORE 1, ROW_GROUP_SIZE {}, PER_THREAD_OUTPUT)",
        output_parquet_file.to_string_lossy(),
        row_group_size
    );

    info!(
        "Copying sequences to {} in Parquet format",
        output_parquet_file.display()
    );
    db.execute_batch(&copy_sql)?;

    Ok(())
}
