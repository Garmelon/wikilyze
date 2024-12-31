use std::{io, path::Path};

use thousands::Separable;

use crate::{
    data::Data,
    util::{self, TitleNormalizer},
};

/// Show info about a specific article.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    title: String,
}

impl Cmd {
    pub fn run(self, data: &Path) -> io::Result<()> {
        let normalizer = TitleNormalizer::new();

        println!(">> Import");
        let data = Data::read_from_file(data)?;

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

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "Links (out)",
                data.graph
                    .edge_range(node)
                    .len()
                    .separate_with_underscores()
            );

            println!(
                "{:>W_LABEL$}: {:>W_NUM$}",
                "Links (in)",
                data.graph
                    .edges()
                    .filter(|(_, target)| *target == node)
                    .count()
                    .separate_with_underscores()
            );

            node = match data.redirect_target(node) {
                Some(target) => target,
                None => break,
            };
        }

        Ok(())
    }
}
