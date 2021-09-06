use crate::prelude::*;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Config {
    device_class: String, // energy (Wh) power (W) / temperature (C) / battery (SOC%) / voltage
    name: String,
    state_topic: String,
    state_class: String,    // measurement or total_increasing (energy)
    value_template: String, // value_json.p_pv or whatever
    unit_of_measurement: String,
    unique_id: String,
}

impl Config {
    pub fn all(inverter: &config::Inverter) -> Result<Vec<mqtt::Message>> {
        let r = vec![
            Self::power(inverter, "p_pv", 1)?,
            Self::power(inverter, "p_chg", 1)?,
            Self::power(inverter, "p_dischg", 1)?,
            Self::power(inverter, "p_to_user", 1)?,
            Self::power(inverter, "p_to_grid", 1)?,
            Self::energy(inverter, "e_pv_all", 2)?,
            Self::energy(inverter, "e_chg_all", 2)?,
            Self::energy(inverter, "e_dischg_all", 2)?,
            Self::energy(inverter, "e_to_user_all", 2)?,
            Self::energy(inverter, "e_to_grid_all", 2)?,
        ];

        Ok(r)
    }

    fn power(inverter: &config::Inverter, name: &str, input: u16) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "power".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "W".to_owned(),
            value_template: format!("{{ value_json.{} }}", name),
            state_topic: format!("lxp/{}/inputs/{}", inverter.datalog, input),
            unique_id: format!("{}_{}", inverter.datalog, name),
            name: name.to_owned(),
        };

        Ok(mqtt::Message {
            topic: Self::topic(inverter, name),
            payload: Self::payload(&config)?,
        })
    }

    fn energy(inverter: &config::Inverter, name: &str, input: u16) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "energy".to_owned(),
            state_class: "total_increasing".to_owned(),
            unit_of_measurement: "kWh".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!("lxp/{}/inputs/{}", inverter.datalog, input),
            unique_id: format!("{}_{}", inverter.datalog, name),
            name: name.to_owned(),
        };

        Ok(mqtt::Message {
            topic: Self::topic(inverter, name),
            payload: Self::payload(&config)?,
        })
    }

    fn topic(inverter: &config::Inverter, key: &str) -> String {
        format!("homeassistant/sensor/{}_{}/config", inverter.datalog, key)
    }

    fn payload(stuff: &Self) -> Result<String> {
        Ok(serde_json::to_string(stuff)?)
    }
}
