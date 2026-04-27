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
mod cli;
mod fasta_to_parquet;
mod fastq_to_parquet;
mod vcf_to_parquet;

use clap::Parser;

use cli::{Cli, Command};

use env_logger::Env;

fn main() {
    let cli = Cli::parse();

    // map log filter level from `--verbose` count
    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    // override with RUST_LOG environment variable, if present
    env_logger::Builder::from_env(Env::default().default_filter_or(filter)).init();

    // todo: accept None or - for stdin
    match cli.command {
        Command::FastaToParquet(args) => {
            let _result = fasta_to_parquet::run(
                args.input_fasta_path.unwrap(),
                args.output_parquet_file,
                args.alphabet,
                args.row_group_size,
            );
        }
        Command::FastqToParquet(args) => {
            let _result = fastq_to_parquet::run(
                args.input_fastq_path.unwrap(),
                args.output_parquet_file,
                args.row_group_size,
            );
        }
        Command::VcfToParquet(args) => {
            let _result = vcf_to_parquet::run(
                args.input_vcf_path.unwrap(),
                args.output_parquet_file,
                args.row_group_size,
            );
        }
    }
}
