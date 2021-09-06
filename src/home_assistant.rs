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
    pub fn all(inverter: &config::Inverter, mqtt_namespace: &str) -> Result<Vec<mqtt::Message>> {
        let r = vec![
            Self::power(inverter, mqtt_namespace, "p_pv", 1)?,
            Self::power(inverter, mqtt_namespace, "p_pv_1", 1)?,
            Self::power(inverter, mqtt_namespace, "p_pv_2", 1)?,
            Self::power(inverter, mqtt_namespace, "p_pv_3", 1)?,
            Self::power(inverter, mqtt_namespace, "p_charge", 1)?,
            Self::power(inverter, mqtt_namespace, "p_discharge", 1)?,
            Self::power(inverter, mqtt_namespace, "p_to_user", 1)?,
            Self::power(inverter, mqtt_namespace, "p_to_grid", 1)?,
            Self::energy(inverter, mqtt_namespace, "e_pv_all", 2)?,
            Self::energy(inverter, mqtt_namespace, "e_pv_all_1", 2)?,
            Self::energy(inverter, mqtt_namespace, "e_pv_all_2", 2)?,
            Self::energy(inverter, mqtt_namespace, "e_pv_all_3", 2)?,
            Self::energy(inverter, mqtt_namespace, "e_chg_all", 2)?,
            Self::energy(inverter, mqtt_namespace, "e_dischg_all", 2)?,
            Self::energy(inverter, mqtt_namespace, "e_to_user_all", 2)?,
            Self::energy(inverter, mqtt_namespace, "e_to_grid_all", 2)?,
        ];

        Ok(r)
    }

    fn power(
        inverter: &config::Inverter,
        mqtt_namespace: &str,
        name: &str,
        input: u16,
    ) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "power".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "W".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!("{}/{}/inputs/{}", mqtt_namespace, inverter.datalog, input),
            unique_id: format!("lxp_{}_{}", inverter.datalog, name),
            name: format!("{} {}", inverter.datalog, name),
        };

        Ok(mqtt::Message {
            topic: Self::topic(inverter, name),
            payload: Self::payload(&config)?,
        })
    }

    fn energy(
        inverter: &config::Inverter,
        mqtt_namespace: &str,
        name: &str,
        input: u16,
    ) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "energy".to_owned(),
            state_class: "total_increasing".to_owned(),
            unit_of_measurement: "kWh".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!("{}/{}/inputs/{}", mqtt_namespace, inverter.datalog, input),
            unique_id: format!("lxp_{}_{}", inverter.datalog, name),
            name: format!("{} {}", inverter.datalog, name),
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
