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
use std::path::Path;

/// Quote a path as a SQL string literal, doubling any single quotes
pub fn string_literal(path: &Path) -> Result<String, Box<dyn Error>> {
    let s = path
        .to_str()
        .ok_or_else(|| format!("path is not valid UTF-8: {}", path.display()))?;
    Ok(format!("'{}'", s.replace('\'', "''")))
}

/// Quote a name as a SQL identifier, doubling any double quotes
pub fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_literal() -> Result<(), Box<dyn Error>> {
        assert_eq!(string_literal(Path::new("out.parquet"))?, "'out.parquet'");
        assert_eq!(
            string_literal(Path::new("/data/bob's run.parquet"))?,
            "'/data/bob''s run.parquet'"
        );
        Ok(())
    }

    #[test]
    fn test_identifier() {
        assert_eq!(identifier("AF"), "\"AF\"");
        assert_eq!(identifier("HG001-x"), "\"HG001-x\"");
        assert_eq!(identifier("a\"b"), "\"a\"\"b\"");
    }

    #[cfg(unix)]
    #[test]
    fn test_string_literal_invalid_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let path = Path::new(OsStr::from_bytes(b"out\xff.parquet"));
        assert!(string_literal(path).is_err());
    }
}
