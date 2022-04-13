use crate::prelude::*;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Config {
    device_class: String,
    name: String,
    state_topic: String,
    state_class: String,
    value_template: String,
    unit_of_measurement: String,
    unique_id: String,
}

impl Config {
    pub fn all(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
    ) -> Result<Vec<mqtt::Message>> {
        let r = vec![
            Self::battery(inverter, mqtt_config, "Battery Percentage", 1)?,
            Self::voltage(inverter, mqtt_config, "Voltage (PV String 1)", 1)?,
            Self::voltage(inverter, mqtt_config, "Voltage (PV String 2)", 1)?,
            Self::voltage(inverter, mqtt_config, "Voltage (PV String 3)", 1)?,
            Self::voltage(inverter, mqtt_config, "Battery Voltage", 1)?,
            Self::power(inverter, mqtt_config, "Power (PV Array)", 1)?,
            Self::power(inverter, mqtt_config, "Power (PV String 1)", 1)?,
            Self::power(inverter, mqtt_config, "Power (PV String 2)", 1)?,
            Self::power(inverter, mqtt_config, "Power (PV String 3)", 1)?,
            Self::power(inverter, mqtt_config, "Battery Charge (Watts)", 1)?,
            Self::power(inverter, mqtt_config, "Battery Discharge (Watts)", 1)?,
            Self::power(inverter, mqtt_config, "Power from Grid", 1)?,
            Self::power(inverter, mqtt_config, "Power to Grid", 1)?,
            Self::energy(inverter, mqtt_config, "PV Generation (Today)", 2)?,
            Self::energy(inverter, mqtt_config, "PV Generation (Today) (String 1)", 2)?,
            Self::energy(inverter, mqtt_config, "PV Generation (Today) (String 2)", 2)?,
            Self::energy(inverter, mqtt_config, "PV Generation (Today) (String 3)", 2)?,
            Self::energy(inverter, mqtt_config, "Total Battery Charge", 2)?,
            Self::energy(inverter, mqtt_config, "Total Battery Discharge", 2)?,
            Self::energy(inverter, mqtt_config, "Power from Grid (All time)", 2)?,
            Self::energy(inverter, mqtt_config, "Power to Grid (All time)", 2)?,
            Self::temperature(inverter, mqtt_config, "Inverter Temperature", 2)?,
            Self::temperature(inverter, mqtt_config, "Radiator 1 Temperature", 2)?,
            Self::temperature(inverter, mqtt_config, "Radiator 2 Temperature", 2)?,
        ];

        Ok(r)
    }

    fn battery(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        input: u16,
    ) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "battery".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "%".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/{}",
                mqtt_config.namespace, inverter.datalog, input
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog, name),
            name: format!("{} {}", inverter.datalog, name),
        };

        Ok(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        })
    }

    fn power(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        input: u16,
    ) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "power".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "W".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/{}",
                mqtt_config.namespace, inverter.datalog, input
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog, name),
            name: format!("{} {}", inverter.datalog, name),
        };

        Ok(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        })
    }

    fn energy(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        input: u16,
    ) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "energy".to_owned(),
            state_class: "total_increasing".to_owned(),
            unit_of_measurement: "kWh".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/{}",
                mqtt_config.namespace, inverter.datalog, input
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog, name),
            name: format!("{} {}", inverter.datalog, name),
        };

        Ok(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        })
    }

    fn voltage(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        input: u16,
    ) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "voltage".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "V".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/{}",
                mqtt_config.namespace, inverter.datalog, input
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog, name),
            name: format!("{} {}", inverter.datalog, name),
        };

        Ok(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),

            payload: serde_json::to_string(&config)?,
        })
    }

    fn temperature(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        input: u16,
    ) -> Result<mqtt::Message> {
        let config = Self {
            device_class: "temperature".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "°C".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/{}",
                mqtt_config.namespace, inverter.datalog, input
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog, name),
            name: format!("{} {}", inverter.datalog, name),
        };

        Ok(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        })
    }
}
