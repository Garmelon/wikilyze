use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use petgraph::{
    algo,
    graph::NodeIndex,
    visit::{EdgeRef, IntoNodeReferences},
    Graph,
};

use crate::{
    data::{
        info::{LinkInfo, PageInfo},
        store,
    },
    util::{self, normalize_link},
};

pub fn find_index_of_title(graph: &Graph<PageInfo, LinkInfo>, title: &str) -> NodeIndex {
    let title = util::normalize_link(title);
    graph
        .node_references()
        .find(|(_, nw)| normalize_link(&nw.title) == title)
        .map(|(ni, _)| ni)
        .expect("invalid title")
}

pub fn resolve_redirects(graph: &Graph<PageInfo, LinkInfo>, mut page: NodeIndex) -> NodeIndex {
    loop {
        if graph.node_weight(page).unwrap().redirect {
            if let Some(link) = graph.edges(page).next() {
                page = link.target();
                continue;
            }
        }
        return page;
    }
}

pub fn path(datafile: &Path, from: &str, to: &str) -> io::Result<()> {
    println!(">> Import");
    let mut databuf = BufReader::new(File::open(datafile)?);
    let graph = store::read_petgraph(&mut databuf)?;

    println!(">> Locate from and to");
    let start = resolve_redirects(&graph, find_index_of_title(&graph, from));
    let goal = resolve_redirects(&graph, find_index_of_title(&graph, to));
    println!("From: {:?}", graph.node_weight(start).unwrap().title);
    println!("To:   {:?}", graph.node_weight(goal).unwrap().title);

    println!(">> Find path");
    let Some((cost, path)) = algo::astar(
        &graph,
        start,
        |n| n == goal,
        |e| !graph.node_weight(e.source()).unwrap().redirect as u32,
        |_| 0,
    ) else {
        println!("No path found");
        return Ok(());
    };

    println!("Path found (cost {cost}, length {}):", path.len());
    for page in path {
        let page = graph.node_weight(page).unwrap();
        if page.redirect {
            println!(" v {:?}", page.title);
        } else {
            println!(" - {:?}", page.title);
        }
    }

    Ok(())
}
