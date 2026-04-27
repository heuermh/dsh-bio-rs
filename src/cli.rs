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
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Show additional logging messages (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand)]
pub enum Command {
    /// Convert DNA or protein sequences in FASTA format to Parquet format
    FastaToParquet(FastaToParquetArgs),

    /// Convert DNA sequences in FASTQ format to Parquet format
    FastqToParquet(FastqToParquetArgs),

    /// Convert variants in VCF format to Parquet format
    VcfToParquet(VcfToParquetArgs),
}

#[derive(Args)]
pub struct FastaToParquetArgs {
    /// Input FASTA path [default <stdin>]
    #[arg(short, long, value_name = "FASTA")]
    pub input_fasta_path: Option<PathBuf>,

    /// Output Parquet file, will be created as a directory, overwriting if necessary
    #[arg(short, long, value_name = "PARQUET")]
    pub output_parquet_file: PathBuf,

    /// Input FASTA alphabet { dna, protein }
    #[arg(short = 'e', long, default_value_t = String::from("dna"))]
    pub alphabet: String,

    /// Row group size
    #[arg(short = 'g', long, default_value_t = 122880)]
    pub row_group_size: u64,
}

#[derive(Args)]
pub struct FastqToParquetArgs {
    /// Input FASTQ path [default <stdin>]
    #[arg(short, long, value_name = "FASTQ")]
    pub input_fastq_path: Option<PathBuf>,

    /// Output Parquet file, will be created as a directory, overwriting if necessary
    #[arg(short, long, value_name = "PARQUET")]
    pub output_parquet_file: PathBuf,

    /// Row group size
    #[arg(short = 'g', long, default_value_t = 122880)]
    pub row_group_size: u64,
}

#[derive(Args)]
pub struct VcfToParquetArgs {
    /// Input VCF path [default <stdin>]
    #[arg(short, long, value_name = "VCF")]
    pub input_vcf_path: Option<PathBuf>,

    /// Output Parquet file, will be created as a directory, overwriting if necessary
    #[arg(short, long, value_name = "PARQUET")]
    pub output_parquet_file: PathBuf,

    /// Row group size
    #[arg(short = 'g', long, default_value_t = 122880)]
    pub row_group_size: u64,
}
