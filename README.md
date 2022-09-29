# Wikilyze

Analyze wikipedia dumps obtained from
<https://dumps.wikimedia.org/backup-index.html>.

## Structure

This project has two main parts, `sift` and `brood`.

`sift` is written in Python and sifts through the wikipedia dump, parsing and
analyzing individual articles and printing interesting data.

`brood` is written in Rust and analyzes the data obtained by `sift`.
