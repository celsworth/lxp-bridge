use crate::prelude::*;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Availability {
    topic: String,
}

#[derive(Debug, Serialize)]
pub struct Device {
    manufacturer: String,
    name: String,
    identifiers: [String; 1],
    // model: String, // TODO: provide inverter model
}

pub struct Config {
    inverter: config::Inverter,
    mqtt_config: config::Mqtt,
}

#[derive(Debug, Serialize)]
pub struct Sensor {
    device_class: String,
    name: String,
    state_topic: String,
    state_class: String,
    value_template: String,
    unit_of_measurement: String,
    unique_id: String,
    device: Device,
    availability: Availability,
}

#[derive(Debug, Serialize)]
pub struct Switch {
    name: String,
    state_topic: String,
    command_topic: String,
    value_template: String,
    unique_id: String,
    device: Device,
    availability: Availability,
}

impl Config {
    pub fn new(inverter: &config::Inverter, mqtt_config: &config::Mqtt) -> Self {
        Self {
            inverter: inverter.clone(),
            mqtt_config: mqtt_config.clone(),
        }
    }

    pub fn all(&self) -> Result<Vec<mqtt::Message>> {
        let r = vec![
            self.apparent_power("s_eps", "Apparent EPS Power")?,
            self.battery("soc", "Battery Percentage")?,
            self.duration("runtime", "Total Runtime")?,
            self.voltage("v_pv_1", "Voltage (PV String 1)")?,
            self.voltage("v_pv_2", "Voltage (PV String 2)")?,
            self.voltage("v_pv_3", "Voltage (PV String 3)")?,
            self.voltage("v_bat", "Battery Voltage")?,
            self.voltage("v_ac_r", "Grid Voltage")?,
            self.frequency("f_ac", "Grid Frequency")?,
            self.frequency("f_eps", "EPS Frequency")?,
            self.power("p_pv", "Power (PV Array)")?,
            self.power("p_pv_1", "Power (PV String 1)")?,
            self.power("p_pv_2", "Power (PV String 2)")?,
            self.power("p_pv_3", "Power (PV String 3)")?,
            self.power("p_charge", "Battery Charge")?,
            self.power("p_discharge", "Battery Discharge")?,
            self.power("p_to_user", "Power from Grid")?,
            self.power("p_to_grid", "Power to Grid")?,
            self.power("p_eps", "Active EPS Power")?,
            self.power("p_charge", "Battery Charge Power")?,
            self.power("p_discharge", "Battery Discharge Power")?,
            self.energy("e_pv_all", "PV Generation (All time)")?,
            self.energy("e_pv_all_1", "PV Generation (All time) (String 1)")?,
            self.energy("e_pv_all_2", "PV Generation (All time) (String 2)")?,
            self.energy("e_pv_all_3", "PV Generation (All time) (String 3)")?,
            self.energy("e_chg_all", "Battery Charge (All time)")?,
            self.energy("e_dischg_all", "Battery Discharge (All time)")?,
            self.energy("e_to_user_all", "Energy from Grid (All time)")?,
            self.energy("e_to_grid_all", "Energy to Grid (All time)")?,
            self.temperature("t_inner", "Inverter Temperature")?,
            self.temperature("t_rad_1", "Radiator 1 Temperature")?,
            self.temperature("t_rad_2", "Radiator 2 Temperature")?,
            self.switch("ac_charge", "AC Charge")?,
            self.switch("charge_priority", "Charge Priority")?,
            self.switch("forced_discharge", "Forced Discharge")?,
        ];

        // drop all None
        Ok(r.into_iter().flatten().collect())
    }

    fn apparent_power(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "apparent_power", "measurement", "VA")
    }

    fn battery(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "battery", "measurement", "%")
    }

    fn duration(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "duration", "total_increasing", "s")
    }

    fn frequency(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "frequency", "measurement", "Hz")
    }

    fn power(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "power", "measurement", "W")
    }

    fn energy(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "energy", "total_increasing", "kWh")
    }

    fn voltage(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "voltage", "measurement", "V")
    }

    fn temperature(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        self.sensor(name, label, "temperature", "measurement", "°C")
    }

    fn sensor(
        &self,
        name: &str,
        label: &str,
        device_class: &str,
        state_class: &str,
        unit_of_measurement: &str,
    ) -> Result<Option<mqtt::Message>> {
        if !enabled(self.mqtt_config.homeassistant().sensors(), name) {
            return Ok(None);
        }

        let config = Sensor {
            device_class: device_class.to_owned(),
            state_class: state_class.to_owned(),
            unit_of_measurement: unit_of_measurement.to_owned(),
            value_template: format!("{{{{ value_json.{} }}}}", name),
            state_topic: format!(
                "{}/{}/inputs/all",
                self.mqtt_config.namespace(),
                self.inverter.datalog()
            ),
            unique_id: format!("lxp_{}_{}", self.inverter.datalog(), name),
            name: label.to_string(),
            device: self.device(),
            availability: self.availability(),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/sensor/lxp_{}/{}/config",
                self.mqtt_config.homeassistant().prefix(),
                self.inverter.datalog(),
                name
            ),

            payload: serde_json::to_string(&config)?,
        }))
    }

    fn switch(&self, name: &str, label: &str) -> Result<Option<mqtt::Message>> {
        if !enabled(self.mqtt_config.homeassistant().switches(), name) {
            return Ok(None);
        }

        let config = Switch {
            value_template: format!("{{{{ value_json.{}_en }}}}", name),
            state_topic: format!(
                "{}/{}/hold/21/bits",
                self.mqtt_config.namespace(),
                self.inverter.datalog()
            ),
            command_topic: format!(
                "{}/cmd/{}/set/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                name
            ),
            unique_id: format!("lxp_{}_{}", self.inverter.datalog(), name),
            name: label.to_string(),
            device: self.device(),
            availability: self.availability(),
        };

        Ok(Some(mqtt::Message {
            topic: format!(
                "{}/switch/lxp_{}/{}/config",
                self.mqtt_config.homeassistant().prefix(),
                self.inverter.datalog(),
                name
            ),
            payload: serde_json::to_string(&config)?,
        }))
    }

    fn device(&self) -> Device {
        Device {
            identifiers: [format!("lxp_{}", self.inverter.datalog())],
            manufacturer: "LuxPower".to_owned(),
            name: format!("lxp_{}", self.inverter.datalog()),
        }
    }

    fn availability(&self) -> Availability {
        Availability {
            topic: format!("{}/LWT", self.mqtt_config.namespace()),
        }
    }
}

fn enabled(list: &[String], name: &str) -> bool {
    // this is rather suboptimal but it only gets run at startup so not time critical
    list.iter()
        .map(|s| s.replace(' ', ""))
        .any(|s| s == "all" || s == name)
}
