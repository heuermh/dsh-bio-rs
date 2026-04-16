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
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Rewrite of dishevelled.org bio in rust
#[derive(Parser)]
#[command(version, about, long_about = None, term_width = 120)]
struct DshBio {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args)]
struct FastaToParquetArgs {
    /// Input FASTA path [default <stdin>]
    #[arg(short, long, value_name = "FASTA")]
    input_fasta_path: Option<PathBuf>,

    /// Output Parquet file, will be created as a directory, overwriting if necessary
    #[arg(short, long, value_name = "PARQUET")]
    output_parquet_file: PathBuf,

    /// Input FASTA alphabet { dna, protein }
    #[arg(short = 'e', long, default_value_t = String::from("dna"))]
    alphabet: String,

    /// Row group size
    #[arg(short = 'g', long, default_value_t = 122880)]
    row_group_size: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert DNA or protein sequences in FASTA format to Parquet format
    FastaToParquet(FastaToParquetArgs),

    /// Convert DNA sequences in FASTQ format to Parquet format
    FastqToParquet {
        // empty
    },
    /// Convert variants in VCF format to Parquet format
    VcfToParquet {
        // empty
    },
}

fn main() {
    let dsh = DshBio::parse();

    match &dsh.command {
        Commands::FastaToParquet(args) => match &args.input_fasta_path {
            Some(p) => println!(
                "FASTA to Parquet, {} {} {} {}",
                p.display(),
                args.output_parquet_file.display(),
                args.alphabet,
                args.row_group_size
            ),
            _ => println!(
                "FASTA to Parquet, {} {} {} {}",
                "<stdin>",
                args.output_parquet_file.display(),
                args.alphabet,
                args.row_group_size
            ),
        },
        Commands::FastqToParquet {} => {
            println!("FASTQ to Parquet");
        }
        Commands::VcfToParquet {} => {
            println!("VCF to Parquet");
        }
    }
}
