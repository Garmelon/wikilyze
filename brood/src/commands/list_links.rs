use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use crate::{
    data::{
        adjacency_list::AdjacencyList,
        info::{LinkInfo, PageInfo},
        store,
    },
    util,
};

fn links_from(data: &AdjacencyList<PageInfo, LinkInfo>, idx: u32) -> HashSet<u32> {
    data.links(idx).map(|(_, ld)| ld.to).collect()
}

fn links_to(data: &AdjacencyList<PageInfo, LinkInfo>, idx: u32) -> HashSet<u32> {
    let mut links = HashSet::<u32>::new();
    for (pi, _) in data.pages() {
        for (_, ld) in data.links(pi) {
            if ld.to == idx {
                links.insert(pi);
                continue;
            }
        }
    }
    links
}

fn print_links(data: &AdjacencyList<PageInfo, LinkInfo>, name: &str, links: &HashSet<u32>) {
    let mut links = links
        .iter()
        .map(|pi| {
            let page = data.page(*pi);
            (&page.data.title as &str, page.data.redirect)
        })
        .collect::<Vec<_>>();

    links.sort();

    println!(">> {name} ({}):", links.len());
    for (title, redirect) in links {
        if redirect {
            println!("v {title}");
        } else {
            println!("- {title}");
        }
    }
}

pub fn run(datafile: &Path, page: &str) -> io::Result<()> {
    println!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = store::read_adjacency_list(&mut databuf)?;

    println!(">> Locate page");
    let idx = util::resolve_redirects(&data, util::find_index_of_title(&data.pages, page));
    println!("Page: {:?}", data.page(idx).data.title);

    println!(">> Find links");
    let from = links_from(&data, idx);
    let to = links_to(&data, idx);
    let twins = from.intersection(&to).copied().collect::<HashSet<_>>();
    let twinless_from = from.difference(&twins).copied().collect::<HashSet<_>>();
    let twinless_to = to.difference(&twins).copied().collect::<HashSet<_>>();

    println!();
    print_links(&data, "From", &from);

    println!();
    print_links(&data, "To", &to);

    println!();
    print_links(&data, "Twins", &twins);

    println!();
    print_links(&data, "From without twins", &twinless_from);

    println!();
    print_links(&data, "To without twins", &twinless_to);

    Ok(())
}
