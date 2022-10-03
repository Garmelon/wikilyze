use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use crate::data::AdjacencyList;

pub fn reexport(from: &Path, to: &Path) -> io::Result<()> {
    eprintln!(">> Import");
    let from = BufReader::new(File::open(from)?);
    let data = AdjacencyList::read(from)?;

    eprintln!(">> Export");
    let to = BufWriter::new(File::create(to)?);
    data.write(to)?;

    Ok(())
}
