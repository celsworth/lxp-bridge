use crate::prelude::*;

use serde::Deserialize;
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator}; //, OneOrMany;

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub inverters: Vec<Inverter>,
    //#[serde_as(deserialize_as = "OneOrMany<_>")]
    pub mqtt: Mqtt,
    //#[serde_as(deserialize_as = "OneOrMany<_>")]
    pub influx: Influx,
    #[serde(default = "Vec::new")]
    pub databases: Vec<Database>,

    pub scheduler: Option<Scheduler>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Inverter {
    #[serde(default = "Config::default_enabled")]
    pub enabled: bool,

    pub host: String,
    pub port: u16,
    #[serde(deserialize_with = "de_serial")]
    pub serial: Serial,
    #[serde(deserialize_with = "de_serial")]
    pub datalog: Serial,

    pub heartbeats: Option<bool>,
}

fn de_serial<'de, D>(deserializer: D) -> Result<Serial, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    raw.parse().map_err(serde::de::Error::custom)
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct HomeAssistant {
    #[serde(default = "Config::default_enabled")]
    pub enabled: bool,

    #[serde(default = "Config::default_mqtt_homeassistant_prefix")]
    pub prefix: String,

    #[serde(default = "Config::default_mqtt_homeassistant_sensors")]
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    pub sensors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Mqtt {
    #[serde(default = "Config::default_enabled")]
    pub enabled: bool,

    pub host: String,
    #[serde(default = "Config::default_mqtt_port")]
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,

    #[serde(default = "Config::default_mqtt_namespace")]
    pub namespace: String,

    #[serde(default = "Config::default_mqtt_homeassistant")]
    pub homeassistant: HomeAssistant,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Influx {
    #[serde(default = "Config::default_enabled")]
    pub enabled: bool,

    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,

    pub database: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    #[serde(default = "Config::default_enabled")]
    pub enabled: bool,

    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Scheduler {
    #[serde(default = "Config::default_enabled")]
    pub enabled: bool,

    pub timesync: Crontab,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Crontab {
    #[serde(default = "Config::default_enabled")]
    pub enabled: bool,

    pub cron: String,
}

pub struct ConfigWrapper {
    config: Rc<RefCell<Config>>,
}

impl Clone for ConfigWrapper {
    fn clone(&self) -> Self {
        Self {
            config: Rc::clone(&self.config),
        }
    }
}

impl ConfigWrapper {
    pub fn new(file: String) -> Result<Self> {
        let config = Rc::new(RefCell::new(Config::new(file)?));

        Ok(Self { config })
    }

    pub fn inverters(&self) -> Ref<Vec<Inverter>> {
        Ref::map(self.config.borrow(), |b| &b.inverters)
    }

    pub fn enabled_inverters(&self) -> Vec<Inverter> {
        self.inverters()
            .iter()
            .filter(|inverter| inverter.enabled)
            .cloned()
            .collect()
    }

    pub fn inverters_for_message(&self, message: &mqtt::Message) -> Result<Vec<Inverter>> {
        use mqtt::SerialOrAll::*;

        let inverters = self.enabled_inverters();

        let r = match message.split_cmd_topic()? {
            All => inverters,
            Serial(datalog) => inverters
                .iter()
                .filter(|i| i.datalog == datalog)
                .cloned()
                .collect(),
        };

        Ok(r)
    }

    pub fn mqtt(&self) -> Ref<Mqtt> {
        Ref::map(self.config.borrow(), |b| &b.mqtt)
    }

    pub fn influx(&self) -> Ref<Influx> {
        Ref::map(self.config.borrow(), |b| &b.influx)
    }

    pub fn influx_mut(&self) -> RefMut<Influx> {
        RefMut::map(self.config.borrow_mut(), |b: &mut Config| &mut b.influx)
    }

    pub fn databases(&self) -> Ref<Vec<Database>> {
        Ref::map(self.config.borrow(), |b| &b.databases)
    }

    pub fn databases_ref(&self) -> Ref<Vec<Database>> {
        Ref::map(self.config.borrow(), |b| &b.databases)
    }

    pub fn databases_mut(&self) -> RefMut<Vec<Database>> {
        RefMut::map(self.config.borrow_mut(), |b: &mut Config| &mut b.databases)
    }

    pub fn enabled_database_count(&self) -> usize {
        self.databases()
            .iter()
            .filter(|database| database.enabled)
            .count()
    }

    pub fn enabled_databases(&self) -> Vec<Database> {
        self.databases()
            .iter()
            .filter(|database| database.enabled)
            .cloned()
            .collect()
    }

    pub fn scheduler(&self) -> Ref<Option<Scheduler>> {
        Ref::map(self.config.borrow(), |b| &b.scheduler)
    }
}

impl Config {
    pub fn new(file: String) -> Result<Self> {
        let content = std::fs::read_to_string(&file)
            .map_err(|err| anyhow!("error reading {}: {}", file, err))?;

        Ok(serde_yaml::from_str(&content)?)
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

    fn default_mqtt_port() -> u16 {
        1883
    }
    fn default_mqtt_namespace() -> String {
        "lxp".to_string()
    }

    fn default_mqtt_homeassistant() -> HomeAssistant {
        HomeAssistant {
            enabled: Self::default_enabled(),
            prefix: Self::default_mqtt_homeassistant_prefix(),
            sensors: Self::default_mqtt_homeassistant_sensors(),
        }
    }

    fn default_mqtt_homeassistant_prefix() -> String {
        "homeassistant".to_string()
    }
    fn default_mqtt_homeassistant_sensors() -> Vec<String> {
        // by default, use the special-case string of "all" rather than list them all out
        vec!["all".to_string()]
    }

    fn default_enabled() -> bool {
        true
    }

    pub fn enabled_databases(&self) -> impl Iterator<Item = &Database> {
        self.databases.iter().filter(|database| database.enabled)
    }
}
