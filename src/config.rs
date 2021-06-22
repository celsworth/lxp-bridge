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
    #[serde(default = "Config::default_inverter_enabled")]
    pub enabled: bool,

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
    #[serde(default = "Config::default_mqtt_enabled")]
    pub enabled: bool,

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
    #[serde(default = "Config::default_influx_enabled")]
    pub enabled: bool,

    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,

    pub database: String,
}

impl Config {
    pub fn new(file: String) -> Result<Self> {
        let content = std::fs::read_to_string(file)?;
        let config = serde_yaml::from_str(&content)?;

        Ok(config)
    }

    pub fn enabled_inverters(&self) -> impl Iterator<Item = &Inverter> {
        self.inverters.iter().filter(|inverter| inverter.enabled)
    }

    // find the inverter(s) in our config for the given message.
    pub fn inverters_for_message(&self, message: &mqtt::Message) -> Result<Vec<Inverter>> {
        use mqtt::SerialOrAll::*;

        let inverters = self.enabled_inverters();

        let r = match message.split_cmd_topic()? {
            All => inverters.cloned().collect(),
            Serial(datalog) => inverters
                .filter(|i| i.datalog == datalog)
                .cloned()
                .collect(),
        };

        Ok(r)
    }

    fn default_inverter_enabled() -> bool {
        true
    }

    fn default_mqtt_enabled() -> bool {
        true
    }
    fn default_mqtt_port() -> u16 {
        1883
    }
    fn default_mqtt_namespace() -> String {
        "lxp".to_string()
    }

    fn default_influx_enabled() -> bool {
        true
    }
}
