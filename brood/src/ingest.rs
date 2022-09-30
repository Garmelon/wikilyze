use std::io::{self, BufRead, BufReader};

use serde::Deserialize;

#[derive(Deserialize)]
struct JsonPage {
    ns: u16,
    id: u32,
    title: String,
    redirect: Option<String>,
    #[serde(default)]
    links: Vec<(String, u32, u32)>,
}
pub fn ingest() -> io::Result<()> {
    let stdin = BufReader::new(io::stdin());

    let mut n_pages = 0;
    let mut n_links = 0;

    for line in stdin.lines() {
        let json_page = serde_json::from_str::<JsonPage>(&line?)?;

        n_pages += 1;
        n_links += json_page.links.len();

        if n_pages % 100_000 == 0 {
            eprintln!("{n_pages}");
        }
    }

    eprintln!("{n_pages} - {n_links}");

    Ok(())
}
