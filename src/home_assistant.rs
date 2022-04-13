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
            Self::battery(inverter, mqtt_config, "soc", 1)?,
            Self::voltage(inverter, mqtt_config, "v_pv_1", 1)?,
            Self::voltage(inverter, mqtt_config, "v_pv_2", 1)?,
            Self::voltage(inverter, mqtt_config, "v_pv_3", 1)?,
            Self::voltage(inverter, mqtt_config, "v_bat", 1)?,
            Self::power(inverter, mqtt_config, "p_pv", 1)?,
            Self::power(inverter, mqtt_config, "p_pv_1", 1)?,
            Self::power(inverter, mqtt_config, "p_pv_2", 1)?,
            Self::power(inverter, mqtt_config, "p_pv_3", 1)?,
            Self::power(inverter, mqtt_config, "p_charge", 1)?,
            Self::power(inverter, mqtt_config, "p_discharge", 1)?,
            Self::power(inverter, mqtt_config, "p_to_user", 1)?,
            Self::power(inverter, mqtt_config, "p_to_grid", 1)?,
            Self::energy(inverter, mqtt_config, "e_pv_all", 2)?,
            Self::energy(inverter, mqtt_config, "e_pv_all_1", 2)?,
            Self::energy(inverter, mqtt_config, "e_pv_all_2", 2)?,
            Self::energy(inverter, mqtt_config, "e_pv_all_3", 2)?,
            Self::energy(inverter, mqtt_config, "e_chg_all", 2)?,
            Self::energy(inverter, mqtt_config, "e_dischg_all", 2)?,
            Self::energy(inverter, mqtt_config, "e_to_user_all", 2)?,
            Self::energy(inverter, mqtt_config, "e_to_grid_all", 2)?,
            Self::temperature(inverter, mqtt_config, "t_inner", "Inverter Temperature", 2)?,
            Self::temperature(inverter, mqtt_config, "t_rad_1", "Radiator 1 Temperature", 2)?,
            Self::temperature(inverter, mqtt_config, "t_rad_2", "Radiator 2 Temperature", 2)?,
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
        label: &str,
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
            name: format!("{}", label),
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
