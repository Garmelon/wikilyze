use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

use crate::data::store;

pub fn run(datafile: &Path) -> io::Result<()> {
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = store::read_adjacency_list(&mut databuf)?;

    for (page_idx, page) in data.pages() {
        if page.data.redirect {
            for link_idx in data.link_range(page_idx) {
                let target_page = data.page(data.link(link_idx).to);
                println!("{:?} -> {:?}", page.data.title, target_page.data.title);
            }
        } else {
            println!("{:?}", page.data.title);
        }
    }

    Ok(())
}
