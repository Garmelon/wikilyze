use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use crate::{
    algo::Dijkstra,
    data::{info::PageInfo, store},
    graph::{Graph, NodeIdx},
    util,
};

pub fn find_index_of_title(pages: &[PageInfo], title: &str) -> NodeIdx {
    let title = util::normalize_link(title);
    pages
        .iter()
        .enumerate()
        .find(|(_, p)| util::normalize_link(&p.title) == title)
        .map(|(i, _)| NodeIdx::new(i))
        .expect("invalid title")
}

pub fn resolve_redirects(pages: &[PageInfo], graph: &Graph, mut page: NodeIdx) -> NodeIdx {
    loop {
        if pages[page.usize()].redirect {
            if let Some(next) = graph.edges_for(page).first() {
                page = *next;
                continue;
            }
        }

        return page;
    }
}

pub fn path(datafile: &Path, start: &str, goal: &str) -> io::Result<()> {
    println!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let (pages, _links, graph) = store::read_graph(&mut databuf)?;

    println!(">> Locate from and to");
    let start = resolve_redirects(&pages, &graph, find_index_of_title(&pages, start));
    let goal = resolve_redirects(&pages, &graph, find_index_of_title(&pages, goal));
    println!("Start: {:?}", pages[start.usize()].title);
    println!("Goal:  {:?}", pages[goal.usize()].title);

    println!(">> Find path");
    println!("> Preparing dijkstra");
    let mut dijkstra = Dijkstra::new(&graph);
    println!("> Running dijkstra");
    dijkstra.run(
        start,
        |node| node == goal,
        |source, _edge, _target| !pages[source.usize()].redirect as u32,
    );

    if dijkstra.cost(goal) == u32::MAX {
        println!("No path found");
        return Ok(());
    }

    println!("> Collecting path");
    let path = dijkstra.path(goal);
    let cost = dijkstra.cost(goal);
    println!("Path found (cost {cost}, length {}):", path.len());
    for page in path {
        let info = &pages[page.usize()];
        if info.redirect {
            println!(" v {:?}", info.title);
        } else {
            println!(" - {:?}", info.title);
        }
    }

    Ok(())
}
