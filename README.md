# Wikilyze

Analyze wikipedia dumps obtained from
<https://dumps.wikimedia.org/backup-index.html>.

## Structure

This project has two main parts, `sift` and `brood`.

### Sift

`sift` is written in Python and sifts through a wikipedia article dump
(`*-pages-articles.xml.bz2`), parsing and analyzing individual articles and
printing interesting data.

It takes a (decompressed) XML article dump on stdin. For each article in the
dump, it prints a single-line JSON object to stdout.

### Brood

`brood` is written in Rust and analyzes the data obtained by `sift`.
