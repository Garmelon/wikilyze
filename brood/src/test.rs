use std::io::{self, BufReader};

use crate::data::AdjacencyList;

pub fn test() -> io::Result<()> {
    eprintln!("IMPORT");
    let data: AdjacencyList = ciborium::de::from_reader(BufReader::new(io::stdin())).unwrap();

    eprintln!("CONSISTENCY CHECK");
    let range = 0..data.pages.len() as u32;
    for link in &data.links {
        if !range.contains(&link.to) {
            eprintln!("Invalid link detected!");
        }
    }

    Ok(())
}
