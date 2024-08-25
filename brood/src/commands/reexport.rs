use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use crate::data::store;

pub fn reexport(from: &Path, to: &Path) -> io::Result<()> {
    eprintln!(">> Import");
    let mut from = BufReader::new(File::open(from)?);
    let data = store::read_adjacency_list(&mut from)?;

    eprintln!(">> Consistency check");
    data.check_consistency();

    eprintln!(">> Export");
    let mut to = BufWriter::new(File::create(to)?);
    store::write_adjacency_list(&data, &mut to)?;

    Ok(())
}
