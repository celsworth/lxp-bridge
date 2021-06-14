use crate::prelude::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub inverters: Vec<Inverter>,
    pub mqtt: Mqtt,
    pub influx: Influx,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Inverter {
    pub host: String,
    pub port: u16,
    #[serde(deserialize_with = "de_serial")]
    pub serial: Serial,
    #[serde(deserialize_with = "de_serial")]
    pub datalog: Serial,
}

fn de_serial<'de, D>(deserializer: D) -> Result<Serial, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    raw.parse().map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct Mqtt {
    pub host: String,
    #[serde(default = "Config::default_mqtt_port")]
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,

    #[serde(default = "Config::default_mqtt_namespace")]
    pub namespace: String,
}

#[derive(Debug, Deserialize)]
pub struct Influx {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,

    pub database: String,
    pub measurement: String,
}

impl Config {
    pub fn new(file: String) -> Result<Self> {
        let content = std::fs::read_to_string(file)?;
        let config = serde_yaml::from_str(&content)?;

        Ok(config)
    }

    // find the inverter in our config for the given message.
    pub fn inverter_for_message(&self, message: &mqtt::Message) -> Option<Inverter> {
        // TODO is this ok()? sufficient? might be throwing away an error
        let r = message.split_cmd_topic().ok()?;

        // search for inverter datalog in our config
        self.inverters
            .iter()
            .cloned()
            .find(|i| i.datalog == r.datalog)
    }

    fn default_mqtt_port() -> u16 {
        1883
    }
    fn default_mqtt_namespace() -> String {
        "lxp".to_string()
    }
}
