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
    pub fn p_pv(inverter: &config::Inverter) -> Result<mqtt::Message> {
        let p_pv = Self {
            device_class: "power".to_owned(),
            name: format!("{} p_pv", inverter.datalog),
            state_topic: format!("lxp/{}/inputs/1", inverter.datalog),
            state_class: "measurement".to_owned(),
            value_template: "{{ value_json.p_pv }}".to_owned(),
            unit_of_measurement: "W".to_owned(),
            unique_id: format!("{}_p_pv", inverter.datalog),
        };

        Ok(mqtt::Message {
            topic: Self::topic(inverter, "p_pv"),
            payload: Self::payload(&p_pv)?,
        })
    }

    fn topic(inverter: &config::Inverter, key: &str) -> String {
        format!("homeassistant/sensor/{}_{}/config", inverter.datalog, key)
    }

    fn payload(stuff: &Self) -> Result<String> {
        Ok(serde_json::to_string(stuff)?)
    }
}
