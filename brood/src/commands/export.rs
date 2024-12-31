use std::{io, path::PathBuf};

use crate::data::Data;

#[derive(Debug, clap::Parser)]
pub struct Cmd {
    out: PathBuf,
}

impl Cmd {
    pub fn run(self, data: Data) -> io::Result<()> {
        println!(">> Export");
        data.write_to_file(&self.out)?;

        Ok(())
    }
}
