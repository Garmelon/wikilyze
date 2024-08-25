use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use crate::data::adjacency_list::AdjacencyList;
use crate::data::store;

pub fn reexport(
    from: &Path,
    to: &Path,
    in_parens: Option<bool>,
    in_structure: Option<bool>,
) -> io::Result<()> {
    eprintln!(">> Import");
    let mut from = BufReader::new(File::open(from)?);
    let mut data = store::read_adjacency_list(&mut from)?;

    eprintln!(">> Consistency check");
    data.check_consistency();

    if in_parens.is_some() || in_structure.is_some() {
        eprintln!(">> Filtering");

        let mut data2 = AdjacencyList::default();
        for (page_idx, page) in data.pages() {
            data2.push_page(page.data.clone());
            for (_, link) in data.links(page_idx) {
                if in_parens.is_some_and(|v| v != link.data.in_parens()) {
                    continue;
                }

                if in_structure.is_some_and(|v| v != link.data.in_structure()) {
                    continue;
                }

                data2.push_link(link.to, link.data);
            }
        }

        data = data2;
    }

    eprintln!(">> Export");
    let mut to = BufWriter::new(File::create(to)?);
    store::write_adjacency_list(&data, &mut to)?;

    Ok(())
}
