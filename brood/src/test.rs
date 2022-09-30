use std::io::{self, BufReader};

use crate::data::SlimAdjacencyList;

pub fn test() -> io::Result<()> {
    eprintln!("IMPORT");
    let data: SlimAdjacencyList = ciborium::de::from_reader(BufReader::new(io::stdin())).unwrap();
    // let data: SlimAdjacencyList =
    //     simd_json::serde::from_reader(BufReader::new(io::stdin())).unwrap();
    let data = data.to_alist();

    eprintln!("CONSISTENCY CHECK");
    let range = 0..data.pages.len() as u32;
    for link in &data.links {
        if !range.contains(&link.to) {
            eprintln!("Invalid link detected!");
        }
    }

    Ok(())
}
