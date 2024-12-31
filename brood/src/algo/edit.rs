use std::mem;

use crate::{
    data::{Data, Link},
    graph::NodeIdx,
    util,
};

pub fn retain_edges(data: &mut Data, f: impl Fn(&Link) -> bool) {
    let mut links = mem::take(&mut data.links).into_iter();
    let graph = mem::take(&mut data.graph);

    for node in graph.nodes() {
        data.graph.add_node();

        for edge in graph.edge_slice(node) {
            let link = links.next().unwrap();
            if f(&link) {
                data.links.push(link);
                data.graph.add_edge(*edge);
            }
        }
    }
}

pub fn resolve_redirects(data: &mut Data) {
    // Permutation from input node to input node
    let mut perm_redirect = vec![NodeIdx::NONE; data.pages.len()];
    for node in data.graph.nodes() {
        perm_redirect[node.usize()] = util::resolve_redirects(data, node);
    }

    // Permutation from input node to final node
    let mut perm_retain = vec![NodeIdx::NONE; data.pages.len()];
    let mut perm_retain_count = NodeIdx(0);
    for (i, page) in data.pages.iter().enumerate() {
        if !page.redirect {
            perm_retain[i] = perm_retain_count;
            perm_retain_count += 1;
        }
    }

    let mut pages = mem::take(&mut data.pages).into_iter();
    let mut links = mem::take(&mut data.links).into_iter();
    let graph = mem::take(&mut data.graph);

    for node in graph.nodes() {
        let page = pages.next().unwrap();
        let new_node = perm_retain[node.usize()];

        if new_node == NodeIdx::NONE {
            // Skip all edges
            for _ in graph.edge_slice(node) {
                links.next().unwrap();
            }
            continue;
        }

        data.pages.push(page);
        data.graph.add_node();

        for edge in graph.edge_slice(node) {
            let link = links.next().unwrap();
            let new_edge = perm_retain[perm_redirect[edge.usize()].usize()];

            if new_edge == NodeIdx::NONE {
                continue;
            }

            data.links.push(link);
            data.graph.add_edge(new_edge);
        }
    }
}
