use crate::prelude::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub inverter: Inverter,
    pub mqtt: Mqtt,
}

#[derive(Debug, Deserialize)]
pub struct Inverter {
    pub host: String,
    pub port: u16,
    pub serial: String,
    pub datalog: String,
}

#[derive(Debug, Deserialize)]
pub struct Mqtt {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Config {
    pub fn new(file: String) -> Result<Self> {
        let content = std::fs::read_to_string(file)?;
        let config = serde_yaml::from_str(&content)?;

        Ok(config)
    }
}
