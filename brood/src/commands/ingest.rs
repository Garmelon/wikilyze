use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::{self, BufRead, BufReader, Seek},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thousands::Separable;

use crate::{
    data::{Data, Link, Page},
    graph::NodeIdx,
    util::{Counter, TitleNormalizer},
};

#[derive(Deserialize)]
struct JsonPage {
    id: u32,
    title: String,
    length: u32,
    links: Vec<(String, u32, u32, u8)>,
    redirect: Option<String>,
}

fn read_titles(r: &mut BufReader<File>) -> io::Result<Vec<String>> {
    let mut counter = Counter::new();
    let mut titles = vec![];

    for line in r.lines() {
        counter.tick();
        let page = serde_json::from_str::<JsonPage>(&line?).unwrap();
        titles.push(page.title);
    }

    counter.done();
    Ok(titles)
}

/// Returns a map from normalized title to the index in the brood data where the
/// article will appear.
///
/// Titles in the title list are not always unique. When multiple identical
/// titles appear, all but one have to be discarded. Originally, I tried to be
/// smart and keep the last occurrence (under the assumption that its data would
/// be the newest), but this led to index-based bugs. Because of this, I now
/// keep the first occurrence.
fn compute_title_lookup(
    normalizer: &TitleNormalizer,
    titles: &[String],
) -> HashMap<String, (u32, u32)> {
    let mut counter = Counter::new();
    let mut title_lookup = HashMap::<String, (u32, u32)>::new();

    for (sift_i, title) in titles.iter().enumerate() {
        counter.tick();

        // The index where this article will appear in the final list, assuming
        // it is not a duplicate. For ownership reasons, we compute this here
        // instead of inside the Entry::Vacant branch of the following match.
        let brood_i = title_lookup.len();

        match title_lookup.entry(normalizer.normalize(title)) {
            Entry::Vacant(entry) => {
                entry.insert((sift_i as u32, brood_i as u32));
            }
            Entry::Occupied(entry) => {
                let prev_sift_i = entry.get().0;
                let prev = &titles[prev_sift_i as usize];
                if prev == title {
                    println!("  {title:?} ({prev_sift_i}) occurs again at {sift_i}");
                } else {
                    println!(
                        "  {prev:?} ({prev_sift_i}) and {title:?} ({sift_i}) both normalize to {:?}",
                        normalizer.normalize(title)
                    );
                }
            }
        }
    }

    counter.done();
    title_lookup
}

fn read_page_data(
    normalizer: &TitleNormalizer,
    title_lookup: &HashMap<String, (u32, u32)>,
    r: &mut BufReader<File>,
) -> io::Result<Data> {
    let mut counter = Counter::new();
    let mut data = Data::new();

    for (i, line) in r.lines().enumerate() {
        counter.tick();
        let page = serde_json::from_str::<JsonPage>(&line?).unwrap();
        let normalized = normalizer.normalize(&page.title);

        let (sift_i, _) = title_lookup[&normalized];
        if i as u32 != sift_i {
            // Articles may occur multiple times, and this is not the instance
            // of the article we should keep.
            println!("  Skipping {:?} ({i}) in favor of {sift_i}", page.title);
            continue;
        }

        data.graph.add_node();
        data.pages.push(Page {
            id: page.id,
            title: page.title,
            length: page.length,
            redirect: page.redirect.is_some(),
        });

        let mut page_links = page.links;
        if let Some(target) = page.redirect {
            page_links.clear();
            let len = target.len() as u32;
            page_links.push((target, 0, len, 0));
        }

        for (target, start, len, flags) in page_links {
            if let Some((_, brood_i)) = title_lookup.get(&normalizer.normalize(&target)) {
                data.graph.add_edge(NodeIdx(*brood_i));
                data.links.push(Link { start, len, flags });
            }
        }
    }

    counter.done();
    Ok(data)
}

/// Convert sift data to brood data.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    /// The sift data file to ingest.
    data: PathBuf,
}

impl Cmd {
    pub fn run(&self, brood_data: &Path) -> io::Result<()> {
        let normalizer = TitleNormalizer::new();

        println!(">> First pass");
        let mut sift_data = BufReader::new(File::open(&self.data)?);

        println!("> Reading titles");
        let titles = read_titles(&mut sift_data)?;

        println!("> Computing title index lookup table");
        let title_lookup = compute_title_lookup(&normalizer, &titles);
        drop(titles); // Don't hoard memory

        println!(">> Second pass");
        sift_data.seek(io::SeekFrom::Start(0))?;

        println!("> Reading page data");
        let data = read_page_data(&normalizer, &title_lookup, &mut sift_data)?;
        assert_eq!(data.pages.len(), title_lookup.len());
        drop(title_lookup); // Don't hoard memory
        drop(sift_data); // No longer needed

        println!("> Checking consistency");
        data.check_consistency();

        println!(">> Export");
        println!(
            "Pages: {:>13}",
            data.pages.len().separate_with_underscores()
        );
        println!(
            "Links: {:>13}",
            data.links.len().separate_with_underscores()
        );
        data.write_to_file(brood_data)?;

        Ok(())
    }
}
