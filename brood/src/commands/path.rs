use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::data::AdjacencyList;
use crate::util;

pub fn path(datafile: &Path, from: &str, to: &str) -> io::Result<()> {
    eprintln!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let mut data = AdjacencyList::read(&mut databuf)?.change_page_data(f32::INFINITY);

    eprintln!(">> Locate from and to");
    let from = util::normalize_link(from);
    let to = util::normalize_link(to);
    let (from_i, from_p) = data
        .pages
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.redirect)
        .find(|(_, p)| util::normalize_link(&p.title) == from)
        .unwrap_or_else(|| panic!("no article called {from}"));
    let (to_i, to_p) = data
        .pages
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.redirect)
        .find(|(_, p)| util::normalize_link(&p.title) == to)
        .unwrap_or_else(|| panic!("no article called {to}"));
    dbg!(from_i, from_p, to_i, to_p);

    Ok(())
}
