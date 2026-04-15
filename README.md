# dsh-bio-rs
Rewrite of dishevelled.org bio in rust

[![rewrites.bio - Follows best practice principles for rewriting bioinformatics tools](https://rewrites.bio/badges/rewrites-bio-flat.svg)](https://rewrites.bio)


### Initial scope

```bash
$ dsh-bio-rs --help
usage:
dsh-bio-rs [command] [args]

commands:
  fasta-to-parquet                convert DNA or protein sequences in FASTA format to Parquet format
  fasta-to-partitioned-parquet    convert DNA or protein sequences in FASTA format to partitioned Parquet format
  fastq-to-parquet                convert DNA sequences in FASTQ format to Parquet format
  fastq-to-partitioned-parquet    convert DNA sequences in FASTQ format to partitioned Parquet format
  vcf-to-parquet                  convert variants in VCF format to Parquet format
  vcf-to-partitioned-parquet      convert variants in VCF format to partitioned Parquet format
```

### Features

* Command line arguments via `clap`
* TUI support via `rich_rust`, `richrs`, or `ratatui`
* Bioinformatics formats support via `noodles` or `oxbow`
* Parquet output format support via `duckdb-rs`
* UDFs via `duckdb-rs` or TBD
* Cloud storage support via TBD
* Performance benchmarks via TBD
* Validation benchmarks via TBD
