use crate::prelude::*;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ConfigDevice {
    manufacturer: String,
    name: String,
    identifiers: [String; 1],
    // model: String, // TODO: provide inverter model
}

#[derive(Debug, Serialize)]
pub struct Config {
    device_class: String,
    name: String,
    state_topic: String,
    state_class: String,
    value_template: String,
    unit_of_measurement: String,
    unique_id: String,
    device: ConfigDevice,
}

impl Config {
    pub fn all(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
    ) -> Result<Vec<mqtt::Message>> {
        let r = vec![
            Self::battery(inverter, mqtt_config, "soc", "Battery Percentage")?,
            Self::voltage(inverter, mqtt_config, "v_pv", "Voltage (PV Array)")?,
            Self::voltage(inverter, mqtt_config, "v_pv_1", "Voltage (PV String 1)")?,
            Self::voltage(inverter, mqtt_config, "v_pv_2", "Voltage (PV String 2)")?,
            Self::voltage(inverter, mqtt_config, "v_pv_3", "Voltage (PV String 3)")?,
            Self::voltage(inverter, mqtt_config, "v_bat", "Battery Voltage")?,
            Self::voltage(inverter, mqtt_config, "v_ac_r", "Grid Voltage")?,
            Self::frequency(inverter, mqtt_config, "f_ac", "Grid Frequency")?,
            Self::frequency(inverter, mqtt_config, "f_eps", "EPS Frequency")?,
            Self::power(inverter, mqtt_config, "p_pv", "Power (PV Array)")?,
            Self::power(inverter, mqtt_config, "p_pv_1", "Power (PV String 1)")?,
            Self::power(inverter, mqtt_config, "p_pv_2", "Power (PV String 2)")?,
            Self::power(inverter, mqtt_config, "p_pv_3", "Power (PV String 3)")?,
            Self::power(inverter, mqtt_config, "p_charge", "Battery Charge")?,
            Self::power(inverter, mqtt_config, "p_discharge", "Battery Discharge")?,
            Self::power(inverter, mqtt_config, "p_to_user", "Power from Grid")?,
            Self::power(inverter, mqtt_config, "p_to_grid", "Power to Grid")?,
            Self::power(inverter, mqtt_config, "p_eps", "Active EPS Power")?,
            Self::power(inverter, mqtt_config, "p_charge", "Battery Charge Power")?,
            Self::power(inverter, mqtt_config, "p_discharge", "Battery Discharge Power")?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all",
                "PV Generation (All time)",
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all_1",
                "PV Generation (All time) (String 1)",
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all_2",
                "PV Generation (All time) (String 2)",
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_pv_all_3",
                "PV Generation (All time) (String 3)",
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_chg_all",
                "Battery Charge (All time)",
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_dischg_all",
                "Battery Discharge (All time)",
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_to_user_all",
                "Energy from Grid (All time)",
            )?,
            Self::energy(
                inverter,
                mqtt_config,
                "e_to_grid_all",
                "Energy to Grid (All time)",
            )?,
            Self::temperature(inverter, mqtt_config, "t_inner", "Inverter Temperature")?,
            Self::temperature(inverter, mqtt_config, "t_rad_1", "Radiator 1 Temperature")?,
            Self::temperature(inverter, mqtt_config, "t_rad_2", "Radiator 2 Temperature")?,
        ];

        // drop all None
        Ok(r.into_iter().flatten().collect())
    }

    fn battery(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(mqtt_config.homeassistant().sensors(), name) {
            return Ok(None);
        }

        let config = Self {
            device_class: "battery".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "%".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/all",
                mqtt_config.namespace(),
                inverter.datalog()
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog(), name),
            name: label.to_string(),
            device: Self::device(inverter),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/lxp_{}/{}/config",
                mqtt_config.homeassistant().prefix(),
                inverter.datalog(),
                name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn frequency(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(mqtt_config.homeassistant().sensors(), name) {
            return Ok(None);
        }

        let config = Self {
            device_class: "frequency".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "Hz".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/all",
                mqtt_config.namespace(),
                inverter.datalog()
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog(), name),
            name: label.to_string(),
            device: Self::device(inverter),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/lxp_{}/{}/config",
                mqtt_config.homeassistant().prefix(),
                inverter.datalog(),
                name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn power(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(mqtt_config.homeassistant().sensors(), name) {
            return Ok(None);
        }

        let config = Self {
            device_class: "power".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "W".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/all",
                mqtt_config.namespace(),
                inverter.datalog()
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog(), name),
            name: label.to_string(),
            device: Self::device(inverter),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/lxp_{}/{}/config",
                mqtt_config.homeassistant().prefix(),
                inverter.datalog(),
                name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn energy(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(mqtt_config.homeassistant().sensors(), name) {
            return Ok(None);
        }

        let config = Self {
            device_class: "energy".to_owned(),
            state_class: "total_increasing".to_owned(),
            unit_of_measurement: "kWh".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/all",
                mqtt_config.namespace(),
                inverter.datalog()
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog(), name),
            name: label.to_string(),
            device: Self::device(inverter),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/lxp_{}/{}/config",
                mqtt_config.homeassistant().prefix(),
                inverter.datalog(),
                name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn voltage(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(mqtt_config.homeassistant().sensors(), name) {
            return Ok(None);
        }

        let config = Self {
            device_class: "voltage".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "V".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/all",
                mqtt_config.namespace(),
                inverter.datalog()
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog(), name),
            name: label.to_string(),
            device: Self::device(inverter),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/lxp_{}/{}/config",
                mqtt_config.homeassistant().prefix(),
                inverter.datalog(),
                name
            ),

            payload: serde_json::to_string(&config)?,
        }))
    }

    fn temperature(
        inverter: &config::Inverter,
        mqtt_config: &config::Mqtt,
        name: &str,
        label: &str,
    ) -> Result<Option<mqtt::Message>> {
        if !Self::sensor_enabled(mqtt_config.homeassistant().sensors(), name) {
            return Ok(None);
        }

        let config = Self {
            device_class: "temperature".to_owned(),
            state_class: "measurement".to_owned(),
            unit_of_measurement: "°C".to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/all",
                mqtt_config.namespace(),
                inverter.datalog()
            ),
            unique_id: format!("lxp_{}_{}", inverter.datalog(), name),
            name: label.to_string(),
            device: Self::device(inverter),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/lxp_{}/{}/config",
                mqtt_config.homeassistant().prefix(),
                inverter.datalog(),
                name
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

    fn device(inverter: &config::Inverter) -> ConfigDevice {
        ConfigDevice {
            identifiers: [format!("lxp_{}", inverter.datalog())],
            manufacturer: "LuxPower".to_owned(),
            name: format!("lxp_{}", inverter.datalog()),
        }
    }
}
