# dsh-bio-rs
Rewrite of dishevelled.org bio in rust

[![rewrites.bio - Follows best practice principles for rewriting bioinformatics tools](https://rewrites.bio/badges/rewrites-bio-flat.svg)](https://rewrites.bio)


### Usage

```bash
$ dsh-bio --help
Rewrite of dishevelled.org bio in rust

Usage: dsh-bio [OPTIONS] <COMMAND>

Commands:
  fasta-to-parquet              Convert DNA or protein sequences in FASTA format to Parquet format
  fasta-to-partitioned-parquet  Convert DNA or protein sequences in FASTA format to partitioned Parquet format
  fastq-to-parquet              Convert DNA sequences in FASTQ format to Parquet format
  fastq-to-partitioned-parquet  Convert DNA sequences in FASTQ format to partitioned Parquet format
  vcf-to-parquet                Convert variants in VCF format to Parquet format
  vcf-to-partitioned-parquet    Convert variants in VCF format to partitioned Parquet format
  help                          Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Show additional logging messages (-v, -vv, -vvv)
  -h, --help        Print help
  -V, --version     Print version
```


### Commands

#### fasta-to-parquet

Convert DNA or protein sequences in FASTA format to Parquet format.

```bash
$ dsh-bio fasta-to-parquet --help
Convert DNA or protein sequences in FASTA format to Parquet format

Usage: dsh-bio fasta-to-parquet [OPTIONS] --output-parquet-file <PARQUET>

Options:
  -i, --input-fasta-path <FASTA>         Input FASTA path [default <stdin>]
  -o, --output-parquet-file <PARQUET>    Output Parquet file, will be created as a directory, overwriting if necessary
  -e, --alphabet <ALPHABET>              Input FASTA alphabet { dna, protein } [default: dna]
  -g, --row-group-size <ROW_GROUP_SIZE>  Row group size [default: 122880]
  -v, --verbose...                       Show additional logging messages (-v, -vv, -vvv)
  -h, --help                             Print help
```

#### fasta-to-partitioned-parquet

Convert DNA or protein sequences in FASTA format to partitioned Parquet format.

```bash
$ dsh-bio fasta-to-partitioned-parquet --help
Convert DNA or protein sequences in FASTA format to partitioned Parquet format

Usage: dsh-bio fasta-to-partitioned-parquet [OPTIONS] --output-parquet-file <PARQUET>

Options:
  -i, --input-fasta-path <FASTA>         Input FASTA path [default <stdin>]
  -o, --output-parquet-file <PARQUET>    Output Parquet file, will be created as a directory, overwriting if necessary
  -e, --alphabet <ALPHABET>              Input FASTA alphabet { dna, protein } [default: dna]
  -g, --row-group-size <ROW_GROUP_SIZE>  Row group size [default: 122880]
  -p, --partition-size <PARTITION_SIZE>  Partition size, in number of rows [default: 1228800]
  -v, --verbose...                       Show additional logging messages (-v, -vv, -vvv)
  -h, --help                             Print help
```

#### fastq-to-parquet

Convert DNA sequences in FASTQ format to Parquet format.

```bash
$ dsh-bio fastq-to-parquet --help
Convert DNA sequences in FASTQ format to Parquet format

Usage: dsh-bio fastq-to-parquet [OPTIONS] --output-parquet-file <PARQUET>

Options:
  -i, --input-fastq-path <FASTQ>         Input FASTQ path [default <stdin>]
  -o, --output-parquet-file <PARQUET>    Output Parquet file, will be created as a directory, overwriting if necessary
  -g, --row-group-size <ROW_GROUP_SIZE>  Row group size [default: 122880]
  -v, --verbose...                       Show additional logging messages (-v, -vv, -vvv)
  -h, --help                             Print help
```

#### fastq-to-partitioned-parquet

Convert DNA sequences in FASTQ format to partitioned Parquet format.

```bash
$ dsh-bio fastq-to-partitioned-parquet --help
Convert DNA sequences in FASTQ format to partitioned Parquet format

Usage: dsh-bio fastq-to-partitioned-parquet [OPTIONS] --output-parquet-file <PARQUET>

Options:
  -i, --input-fastq-path <FASTQ>         Input FASTQ path [default <stdin>]
  -o, --output-parquet-file <PARQUET>    Output Parquet file, will be created as a directory, overwriting if necessary
  -g, --row-group-size <ROW_GROUP_SIZE>  Row group size [default: 122880]
  -p, --partition-size <PARTITION_SIZE>  Partition size, in number of rows [default: 1228800]
  -v, --verbose...                       Show additional logging messages (-v, -vv, -vvv)
  -h, --help                             Print help
```

#### vcf-to-parquet

Convert variants in VCF format to Parquet format.

```bash
$ dsh-bio vcf-to-parquet --help
Convert variants in VCF format to Parquet format

Usage: dsh-bio vcf-to-parquet [OPTIONS] --output-parquet-file <PARQUET>

Options:
  -i, --input-vcf-path <VCF>             Input VCF path [default <stdin>]
  -o, --output-parquet-file <PARQUET>    Output Parquet file, will be created as a directory, overwriting if necessary
  -n, --info-fields <FIELD>              List of INFO fields to include, if any
  -r, --info-prefix <PREFIX>             Prefix for INFO field column names [default: ""]
  -s, --samples <SAMPLE>                 List of samples to include, if any
  -x, --sample-prefix <PREFIX>           Prefix for sample FORMAT field column names [default: ""]
  -f, --format-fields <FIELD>            List of FORMAT fields to include for each sample, if any
  -w, --lowercase                        Lowercase fields and samples in column names
  -m, --multiallelic                     Allow multiallelic records
  -g, --row-group-size <ROW_GROUP_SIZE>  Row group size [default: 122880]
  -v, --verbose...                       Show additional logging messages (-v, -vv, -vvv)
  -h, --help                             Print help
```

#### vcf-to-partitioned-parquet

Convert variants in VCF format to partitioned Parquet format.

```bash
$ dsh-bio vcf-to-partitioned-parquet --help
Convert variants in VCF format to partitioned Parquet format

Usage: dsh-bio vcf-to-partitioned-parquet [OPTIONS] --output-parquet-file <PARQUET>

Options:
  -i, --input-vcf-path <VCF>             Input VCF path [default <stdin>]
  -o, --output-parquet-file <PARQUET>    Output Parquet file, will be created as a directory, overwriting if necessary
  -n, --info-fields <FIELD>              List of INFO fields to include, if any
  -r, --info-prefix <PREFIX>             Prefix for INFO field column names [default: ""]
  -s, --samples <SAMPLE>                 List of samples to include, if any
  -x, --sample-prefix <PREFIX>           Prefix for sample FORMAT field column names [default: ""]
  -f, --format-fields <FIELD>            List of FORMAT fields to include for each sample, if any
  -w, --lowercase                        Lowercase fields and samples in column names
  -m, --multiallelic                     Allow multiallelic records
  -g, --row-group-size <ROW_GROUP_SIZE>  Row group size [default: 122880]
  -p, --partition-size <PARTITION_SIZE>  Partition size, in number of rows [default: 1228800]
  -v, --verbose...                       Show additional logging messages (-v, -vv, -vvv)
  -h, --help                             Print help
```
