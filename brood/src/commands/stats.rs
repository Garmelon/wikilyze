mod redirects;

use std::io;

use thousands::Separable;

use crate::data::Data;

#[derive(Debug, clap::Parser)]
enum Command {
    Redirects(redirects::Cmd),
}

/// Show interesting stats.
#[derive(Debug, clap::Parser)]
pub struct Cmd {
    #[command(subcommand)]
    command: Option<Command>,
}

impl Cmd {
    pub fn run(self, data: Data) -> io::Result<()> {
        if let Some(cmd) = self.command {
            return match cmd {
                Command::Redirects(cmd) => cmd.run(data),
            };
        }

        println!();

        const W_LABEL: usize = 14;
        const W_NUM: usize = 11;

        let n_pages = data.pages.len();
        let n_redirects = data.pages.iter().filter(|p| p.redirect).count();
        let n_articles = n_pages - n_redirects;

        println!(
            "{:>W_LABEL$}: {:>W_NUM$}",
            "Pages",
            n_pages.separate_with_underscores()
        );

        println!(
            "{:>W_LABEL$}: {:>W_NUM$}",
            "Articles",
            n_articles.separate_with_underscores()
        );

        println!(
            "{:>W_LABEL$}: {:>W_NUM$}",
            "Redirects",
            n_redirects.separate_with_underscores()
        );

        println!();
        println!(
            "{:>W_LABEL$}: {:>W_NUM$}",
            "Links",
            data.links.len().separate_with_underscores()
        );

        println!(
            "{:>W_LABEL$}: {:>W_NUM$}",
            "in parens",
            data.links
                .iter()
                .filter(|l| l.in_parens())
                .count()
                .separate_with_underscores()
        );

        println!(
            "{:>W_LABEL$}: {:>W_NUM$}",
            "in structures",
            data.links
                .iter()
                .filter(|l| l.in_structure())
                .count()
                .separate_with_underscores()
        );

        println!(
            "{:>W_LABEL$}: {:>W_NUM$}",
            "pg eligible",
            data.links
                .iter()
                .filter(|l| !l.in_parens() && !l.in_structure())
                .count()
                .separate_with_underscores()
        );

        Ok(())
    }
}
