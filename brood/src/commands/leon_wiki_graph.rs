use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::data::AdjacencyList;

#[derive(Deserialize)]
struct Article {
    title: String,
    url: String,
    language: String,
}

#[derive(Serialize)]
struct NodeRow {
    id: u32,
    label: String,
}

#[derive(Serialize)]
struct EdgeRow {
    source: u32,
    target: u32,
}

pub fn run(datafile: &Path, articlesfile: &Path, language: &str) -> io::Result<()> {
    let mut databuf = BufReader::new(File::open(datafile)?);
    let data = AdjacencyList::read(&mut databuf)?;

    let articlesbuf = BufReader::new(File::open(articlesfile)?);
    let articles: Vec<Article> =
        serde_json::from_reader(articlesbuf).expect("failed to parse articles file");

    let titles = articles
        .into_iter()
        .filter(|a| a.language == language)
        .map(|a| a.title)
        .collect::<HashSet<_>>();

    let page_ids = data
        .pages
        .split_last()
        .unwrap()
        .1
        .iter()
        .enumerate()
        .filter(|(_, p)| titles.contains(&p.data.title))
        .map(|(i, _)| i as u32)
        .collect::<Vec<_>>();

    let mut node_rows = vec![];
    for i in &page_ids {
        let page = data.page(*i);
        let row = NodeRow {
            id: *i,
            label: page.data.title.clone(),
        };
        node_rows.push(row);
    }

    let mut edge_rows = vec![];
    for i in &page_ids {
        let links = data
            .link_range(*i)
            .map(|li| data.link(li).to)
            .filter(|to| page_ids.contains(to))
            .collect::<HashSet<_>>();

        for to in links {
            let row = EdgeRow {
                source: *i,
                target: to,
            };
            edge_rows.push(row);
        }
    }

    let node_writer = BufWriter::new(File::create("nodes.csv")?);
    let mut node_writer = csv::Writer::from_writer(node_writer);
    for node in node_rows {
        node_writer.serialize(node).unwrap();
    }

    let edge_writer = BufWriter::new(File::create("edges.csv")?);
    let mut edge_writer = csv::Writer::from_writer(edge_writer);
    for edge in edge_rows {
        edge_writer.serialize(edge).unwrap();
    }

    Ok(())
}
