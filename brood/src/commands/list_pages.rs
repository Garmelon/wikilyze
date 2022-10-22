use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::data::AdjacencyList;

pub fn run(datafile: &Path) -> io::Result<()> {
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = AdjacencyList::read(&mut databuf)?;

    for (page_idx, page) in data.pages.iter().enumerate() {
        if page.data.redirect {
            for link_idx in data.link_range(page_idx as u32) {
                let target_page = data.page(data.link(link_idx).to);
                println!("{:?} -> {:?}", page.data.title, target_page.data.title);
            }
        } else {
            println!("{:?}", page.data.title);
        }
    }

    Ok(())
}
