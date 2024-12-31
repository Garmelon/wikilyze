use std::{collections::HashSet, io};

use thousands::Separable;

use crate::{
    data::Data,
    util::{self, TitleNormalizer},
};

/// Show info about a specific article.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    title: String,

    /// Print links in more detail.
    #[arg(long, short)]
    links: bool,
}

impl Cmd {
    pub fn run(self, data: Data) -> io::Result<()> {
        let normalizer = TitleNormalizer::new();

        println!(">> Locate article");
        let mut node = util::locate_title(&normalizer, &data, &self.title);

        loop {
            let page = &data.pages[node.usize()];

            const W_LABEL: usize = 12;
            const W_NUM: usize = 11;

            println!();

            println!("{:>W_LABEL$}: {}", "Title", page.title);

            println!(
                "{:>W_LABEL$}: {}",
                "Title (norm)",
                normalizer.normalize(&page.title)
            );

            println!("{:>W_LABEL$}: {}", "Redirect", page.redirect);

            println!("{:>W_LABEL$}: {:>W_NUM$}", "ID", page.id);

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "Length",
                page.length.separate_with_underscores()
            );

            let outlinks = data.graph.edge_slice(node).to_vec();
            let inlinks = data
                .graph
                .edges()
                .filter(|(_, target)| *target == node)
                .map(|(source, _)| source)
                .collect::<Vec<_>>();

            let outlinks_set = outlinks.iter().copied().collect::<HashSet<_>>();
            let inlinks_set = inlinks.iter().copied().collect::<HashSet<_>>();
            let twins_set = outlinks_set
                .intersection(&inlinks_set)
                .copied()
                .collect::<HashSet<_>>();

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "Links (out)",
                outlinks.len().separate_with_underscores()
            );

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "unique",
                outlinks_set.len().separate_with_underscores()
            );

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "Links (in)",
                inlinks.len().separate_with_underscores()
            );

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "unique",
                inlinks_set.len().separate_with_underscores()
            );

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "Twins",
                twins_set.len().separate_with_underscores()
            );

            if self.links {
                let mut twin_pages = twins_set
                    .iter()
                    .map(|n| &data.pages[n.usize()])
                    .collect::<Vec<_>>();

                let mut outlink_only_pages = outlinks_set
                    .difference(&twins_set)
                    .map(|n| &data.pages[n.usize()])
                    .collect::<Vec<_>>();

                let mut inlink_only_pages = inlinks_set
                    .difference(&twins_set)
                    .map(|n| &data.pages[n.usize()])
                    .collect::<Vec<_>>();

                twin_pages.sort_by_key(|p| &p.title);
                outlink_only_pages.sort_by_key(|p| &p.title);
                inlink_only_pages.sort_by_key(|p| &p.title);

                println!();
                println!("Twins ({}):", twin_pages.len().separate_with_underscores());
                for page in twin_pages {
                    println!("{}", util::fmt_page(page));
                }

                println!();
                println!(
                    "Only outlinks ({}):",
                    outlink_only_pages.len().separate_with_underscores()
                );
                for page in outlink_only_pages {
                    println!("{}", util::fmt_page(page));
                }

                println!();
                println!(
                    "Only inlinks ({}):",
                    inlink_only_pages.len().separate_with_underscores()
                );
                for page in inlink_only_pages {
                    println!("{}", util::fmt_page(page));
                }
            }

            node = match data.redirect_target(node) {
                Some(target) => target,
                None => break,
            };
        }

        Ok(())
    }
}
