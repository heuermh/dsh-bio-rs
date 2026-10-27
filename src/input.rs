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
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::NonZero;
use std::path::Path;
use std::thread;

use flate2::bufread::MultiGzDecoder;

use noodles::bgzf;

use log::debug;

const BUFFER_SIZE: usize = 1 << 20;

/// gzip ID1, ID2, and CM (deflate)
const GZIP_MAGIC: [u8; 3] = [0x1f, 0x8b, 0x08];

/// gzip FLG bit indicating an extra field is present
const GZIP_FEXTRA: u8 = 0x04;

/// BGZF extra subfield identifier SI1, SI2, at bytes 12-13
const BGZF_SUBFIELD_ID: [u8; 2] = *b"BC";

/// Compression method of an input file, detected from its header
#[derive(Debug, PartialEq)]
pub enum Compression {
    Bgzf,
    Gzip,
    None,
}

/// Detect the compression method from the first bytes of a file: gzip magic 1f 8b 08, and
/// for BGZF the FEXTRA flag with a "BC" extra subfield at bytes 12-13
pub fn detect(header: &[u8]) -> Compression {
    if !header.starts_with(&GZIP_MAGIC) {
        return Compression::None;
    }
    let fextra = header.get(3).is_some_and(|flags| flags & GZIP_FEXTRA != 0);
    if fextra && header.get(12..14) == Some(&BGZF_SUBFIELD_ID[..]) {
        Compression::Bgzf
    } else {
        Compression::Gzip
    }
}

/// Open an input file for buffered reading, decompressing BGZF or plain gzip as detected
/// from the file header rather than its extension
pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Box<dyn BufRead>> {
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, File::open(path.as_ref())?);
    let compression = detect(reader.fill_buf()?);
    debug!(
        "Detected {:?} compression for {}",
        compression,
        path.as_ref().display()
    );

    Ok(match compression {
        // BGZF blocks are independent, so decompress them on all available cores
        Compression::Bgzf => {
            let worker_count = thread::available_parallelism().unwrap_or(NonZero::<usize>::MIN);
            Box::new(bgzf::io::MultithreadedReader::with_worker_count(
                worker_count,
                reader,
            ))
        }
        // multi-member decoder, as gzip files may be concatenated
        Compression::Gzip => Box::new(BufReader::with_capacity(
            BUFFER_SIZE,
            MultiGzDecoder::new(reader),
        )),
        Compression::None => Box::new(reader),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Read, Write};

    use flate2::write::GzEncoder;

    const FASTA: &[u8] = b">sq0\nACGT\n>sq1\nNNNN\n";

    fn read_all(path: &Path) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        open(path)?.read_to_end(&mut buf)?;
        Ok(buf)
    }

    #[test]
    fn test_detect() {
        let bgzf_header = [
            0x1f, 0x8b, 0x08, 0x04, 0, 0, 0, 0, 0, 0xff, 0x06, 0x00, b'B', b'C',
        ];
        let gzip_header = [0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0, 0xff];
        assert_eq!(detect(&bgzf_header), Compression::Bgzf);
        assert_eq!(detect(&gzip_header), Compression::Gzip);
        assert_eq!(detect(&GZIP_MAGIC), Compression::Gzip);
        let fextra_header = [
            0x1f, 0x8b, 0x08, 0x04, 0, 0, 0, 0, 0, 0xff, 0x06, 0x00, b'X', b'Y',
        ];
        assert_eq!(detect(&fextra_header), Compression::Gzip);
        assert_eq!(detect(FASTA), Compression::None);
        assert_eq!(detect(&[]), Compression::None);
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let dir = tempfile::tempdir()?;

        let plain = dir.path().join("plain.fa");
        std::fs::write(&plain, FASTA)?;

        let gzip = dir.path().join("gzip.fa.gz");
        let mut encoder = GzEncoder::new(File::create(&gzip)?, flate2::Compression::default());
        encoder.write_all(FASTA)?;
        encoder.finish()?;

        let bgzf = dir.path().join("bgzf.fa.bgz");
        let mut writer = bgzf::io::Writer::new(File::create(&bgzf)?);
        writer.write_all(FASTA)?;
        writer.finish()?;

        assert_eq!(read_all(&plain)?, FASTA);
        assert_eq!(read_all(&gzip)?, FASTA);
        assert_eq!(read_all(&bgzf)?, FASTA);
        Ok(())
    }
}
