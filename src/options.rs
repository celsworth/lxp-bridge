use crate::prelude::*;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Options {
    /// Config file to read
    #[structopt(short = "c", long = "config", default_value = "config.yaml")]
    pub config_file: String,
}

impl Options {
    pub fn new() -> Result<Self, Error> {
        let r = Self::from_args();

        Ok(r)
    }
}
