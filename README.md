# noodles squab

[![GitHub Actions status](https://github.com/zaeleus/noodles-squab/workflows/CI/badge.svg)](https://github.com/zaeleus/noodles-squab/actions)

**noodles squab** performs gene expression quantification by counting the
number of aligned records that intersects a set of features. Output can be the
raw counts or normalized counts in TPM (transcripts per million) or FPKM
(fragments per kilobase per million mapped reads).

The original goal of this project is to provide a faster alternative to
[htseq-count]. It uses the same counting rules and outputs a compatible data
table.

[htseq-count]: https://htseq.readthedocs.io/en/master/count.html

## Installation

Install [Rust] and use `cargo` to install `noodles-squab`.

```
$ cargo install --git https://github.com/zaeleus/noodles-squab.git
```

[Rust]: https://www.rust-lang.org/tools/install


## Usage

```
noodles-squab unknown (d1da57f85 2020-03-01)

USAGE:
    noodles-squab [FLAGS] [OPTIONS] <bam> --annotations <file> --output <file>

FLAGS:
    -h, --help                          Prints help information
    -V, --version                       Prints version information
    -v, --verbose                       Use verbose logging
        --with-nonunique-records        Count nonunique records (BAM data tag NH > 1)
        --with-secondary-records        Count secondary records (BAM flag 0x100)
        --with-supplementary-records    Count supplementary records (BAM flag 0x800)

OPTIONS:
    -a, --annotations <file>            Input annotations file (GTF/GFFv2)
    -i, --id <str>                      Feature attribute to use as the feature identity [default: gene_id]
        --min-mapq <u8>                 Minimum mapping quality to consider an alignment [default: 10]
        --normalize <str>               Quantification normalization method [possible values: fpkm, tpm]
    -o, --output <file>                 Output destination for feature counts
        --strand-specification <str>    Strand specification [default: auto]  [possible values: none, forward, reverse,
                                        auto]
        --threads <uint>                Force a specific number of threads
    -t, --type <str>                    Feature type to count [default: exon]

ARGS:
    <bam>    Input alignment file
```

The default output (`--quantification-method count`) is a tab-delimited text
file with two columns: the feature identifier (string) and the number of reads
(integer) from the input alignment that overlap it. This file is compatible as
output from htseq-count, meaning it includes statistics in the trailer.

## Limitations

  * Counts are taken only as the union of matched feature sets, i.e., reads that
    overlap any part of the feature is considered once.
  * For paired end alignments, a read that matches itself before a mate is
    found replaces the previously known record.

## References

  * Wagner, G.P., Kin, K. & Lynch, V.J. Measurement of mRNA abundance using
    RNA-seq data: RPKM measure is inconsistent among samples. _Theory Biosci_.
    **131**, 281–285 (2012). https://doi.org/10.1007/s12064-012-0162-3
