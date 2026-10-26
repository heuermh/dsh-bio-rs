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
use std::ops::{Deref, DerefMut};
use std::path::Path;

use duckdb::Connection;

use log::info;

use tempfile::TempDir;

/// DuckDB connection, optionally backed by a database file in a temporary directory.
pub struct Database {
    // field order matters: the connection must be dropped before its temporary directory is removed
    connection: Connection,
    _temp_dir: Option<TempDir>,
}

impl Deref for Database {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        &self.connection
    }
}

impl DerefMut for Database {
    fn deref_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

/// Open an in-memory database, or if `temp_dir` is specified, a database file in a new
/// temporary directory under `temp_dir`, removed when the returned database is dropped.
pub fn open(temp_dir: Option<&Path>) -> Result<Database, Box<dyn Error>> {
    match temp_dir {
        None => Ok(Database {
            connection: Connection::open_in_memory()?,
            _temp_dir: None,
        }),
        Some(dir) => {
            let temp_dir = tempfile::Builder::new().prefix("dsh-bio-").tempdir_in(dir)?;
            let db_path = temp_dir.path().join("dsh-bio.duckdb");
            let spill_path = temp_dir.path().join("spill");
            info!("Opening temporary database {}", db_path.display());

            let connection = Connection::open(&db_path)?;
            connection.execute_batch(&format!(
                "SET temp_directory = '{}'",
                spill_path.to_string_lossy().replace('\'', "''")
            ))?;

            Ok(Database {
                connection,
                _temp_dir: Some(temp_dir),
            })
        }
    }
}
