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
            Self::battery(inverter, mqtt_config, "soc", "Battery Percentage", 1)?,
            Self::voltage(inverter, mqtt_config, "v_pv", "Voltage (PV Array)", 1)?,
            Self::voltage(inverter, mqtt_config, "v_pv_1", "Voltage (PV String 1)", 1)?,
            Self::voltage(inverter, mqtt_config, "v_pv_2", "Voltage (PV String 2)", 1)?,
            Self::voltage(inverter, mqtt_config, "v_pv_3", "Voltage (PV String 3)", 1)?,
            Self::voltage(inverter, mqtt_config, "v_bat", "Battery Voltage", 1)?,
            Self::power(inverter, mqtt_config, "p_pv", "Power (PV Array)", 1)?,
            Self::power(inverter, mqtt_config, "p_pv_1", "Power (PV String 1)", 1)?,
            Self::power(inverter, mqtt_config, "p_pv_2", "Power (PV String 2)", 1)?,
            Self::power(inverter, mqtt_config, "p_pv_3", "Power (PV String 3)", 1)?,
            Self::power(inverter, mqtt_config, "p_charge", "Battery Charge", 1)?,
            Self::power(inverter, mqtt_config, "p_discharge", "Battery Discharge", 1)?,
            Self::power(inverter, mqtt_config, "p_to_user", "Power from Grid", 1)?,
            Self::power(inverter, mqtt_config, "p_to_grid", "Power to Grid", 1)?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all",
                "PV Generation (All time)",
                2,
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all_1",
                "PV Generation (All time) (String 1)",
                2,
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all_2",
                "PV Generation (All time) (String 2)",
                2,
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all_3",
                "PV Generation (All time) (String 3)",
                2,
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_chg_all",
                "Battery Charge (All time)",
                2,
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_dischg_all",
                "Battery Discharge (All time)",
                2,
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_to_user_all",
                "Energy from Grid (All time)",
                2,
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_to_grid_all",
                "Energy to Grid (All time)",
                2,
            )?,
            Self::temperature(inverter, mqtt_config, "t_inner", "Inverter Temperature", 2)?,
            Self::temperature(
                inverter,
                mqtt_config,
                "t_rad_1",
                "Radiator 1 Temperature",
                2,
            )?,
            Self::temperature(
                inverter,
                mqtt_config,
                "t_rad_2",
                "Radiator 2 Temperature",
                2,
            )?,
        ];

        // drop all None
        Ok(r.into_iter().flatten().collect())
    }

    fn battery(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
        input: u16,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(&mqtt_config.homeassistant.sensors, name) {
            return Ok(None);
        }

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
            name: label.to_string(),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn power(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
        input: u16,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(&mqtt_config.homeassistant.sensors, name) {
            return Ok(None);
        }

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
            name: label.to_string(),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn energy(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
        input: u16,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(&mqtt_config.homeassistant.sensors, name) {
            return Ok(None);
        }

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
            name: label.to_string(),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn voltage(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
        input: u16,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(&mqtt_config.homeassistant.sensors, name) {
            return Ok(None);
        }

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
            name: label.to_string(),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),

            payload: serde_json::to_string(&config)?,
        }))
    }

    fn temperature(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
        input: u16,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(&mqtt_config.homeassistant.sensors, name) {
            return Ok(None);
        }

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
            name: label.to_string(),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/{}_{}/config",
                mqtt_config.homeassistant.prefix, inverter.datalog, name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn sensor_enabled(sensors: &[String], name: &str) -> bool {
        // this is rather suboptimal but it only gets run at startup so not time critical
        sensors
            .iter()
            .map(|s| s.replace(' ', ""))
            .any(|s| s == "all" || s == name)
    }
}
