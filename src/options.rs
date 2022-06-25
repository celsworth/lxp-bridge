use crate::prelude::*;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version)]
pub struct Options {
    /// Config file to read
    #[clap(short = 'c', long = "config", default_value = "config.yaml")]
    pub config_file: String,
}

impl Options {
    pub fn new() -> Result<Self, Error> {
        let r = Self::parse();

        Ok(r)
    }
}
