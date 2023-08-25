use crate::prelude::*;
use lxp::packet::Register;

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
pub struct Diagnostic {
    entity_category: String,
    name: String,
    state_topic: String,
    unique_id: String,
    device: Device,
    availability: Availability,
}

// https://www.home-assistant.io/integrations/sensor.mqtt/
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

// https://www.home-assistant.io/integrations/switch.mqtt/
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

// https://www.home-assistant.io/integrations/number.mqtt/
#[derive(Debug, Serialize)]
pub struct Number {
    name: String,
    state_topic: String,
    command_topic: String,
    value_template: String,
    unique_id: String,
    device: Device,
    availability: Availability,
    min: f64,
    max: f64,
    step: f64,
    unit_of_measurement: String,
}

// https://www.home-assistant.io/integrations/text.mqtt/
#[derive(Debug, Serialize)]
pub struct Text {
    name: String,
    state_topic: String,
    command_topic: String,
    command_template: String,
    value_template: String,
    unique_id: String,
    device: Device,
    availability: Availability,
    pattern: String,
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
            self.diagnostic("status", "Status", "input/0/parsed")?,
            self.diagnostic("internal_fault", "Internal Fault", "input/6/parsed")?,
            self.diagnostic("fault_code", "Fault Code", "input/fault_code/parsed")?,
            self.diagnostic("warning_code", "Warning Code", "input/warning_code/parsed")?,
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
            self.power("p_battery", "Battery Power (discharge is negative)")?,
            self.power("p_charge", "Battery Charge")?,
            self.power("p_discharge", "Battery Discharge")?,
            self.power("p_grid", "Grid Power (export is negative)")?,
            self.power("p_to_user", "Power from Grid")?,
            self.power("p_to_grid", "Power to Grid")?,
            self.power("p_eps", "Active EPS Power")?,
            self.power("p_inv", "Inverter Power")?,
            self.power("p_rec", "AC Charge Power")?,
            self.energy("e_pv_all", "PV Generation (All time)")?,
            self.energy("e_pv_all_1", "PV Generation (All time) (String 1)")?,
            self.energy("e_pv_all_2", "PV Generation (All time) (String 2)")?,
            self.energy("e_pv_all_3", "PV Generation (All time) (String 3)")?,
            self.energy("e_pv_day", "PV Generation (Today)")?,
            self.energy("e_pv_day_1", "PV Generation (Today) (String 1)")?,
            self.energy("e_pv_day_2", "PV Generation (Today) (String 2)")?,
            self.energy("e_pv_day_3", "PV Generation (Today) (String 3)")?,
            self.energy("e_chg_all", "Battery Charge (All time)")?,
            self.energy("e_chg_day", "Battery Charge (Today)")?,
            self.energy("e_dischg_all", "Battery Discharge (All time)")?,
            self.energy("e_dischg_day", "Battery Discharge (Today)")?,
            self.energy("e_to_user_all", "Energy from Grid (All time)")?,
            self.energy("e_to_user_day", "Energy from Grid (Today)")?,
            self.energy("e_to_grid_all", "Energy to Grid (All time)")?,
            self.energy("e_to_grid_day", "Energy to Grid (Today)")?,
            self.energy("e_eps_all", "Energy from EPS (All time)")?,
            self.energy("e_eps_day", "Energy from EPS (Today)")?,
            self.energy("e_rec_all", "Energy of AC charging (All time)")?,
            self.energy("e_rec_day", "Energy of AC charging (Today)")?,
            self.energy("e_inv_all", "Energy of Inverter (All time)")?,
            self.energy("e_inv_day", "Energy of Inverter (Today)")?,
            self.temperature("t_inner", "Inverter Temperature")?,
            self.temperature("t_rad_1", "Radiator 1 Temperature")?,
            self.temperature("t_rad_2", "Radiator 2 Temperature")?,
            self.switch("ac_charge", "AC Charge")?,
            self.switch("charge_priority", "Charge Priority")?,
            self.switch("forced_discharge", "Forced Discharge")?,
            self.number_percent(Register::ChargePowerPercentCmd, "System Charge Rate (%)")?,
            self.number_percent(Register::DischgPowerPercentCmd, "System Discharge Rate (%)")?,
            // TODO: is this one actually a percentage?
            // self.number_percent(
            //    Register::AcChargePowerCmd,
            //    "Grid Charge Rate (%)",
            // )?,
            self.number_percent(Register::AcChargeSocLimit, "AC Charge Limit %")?,
            self.number_percent(Register::ForcedChargeSocLimit, "Forced Charge Limit %")?,
            self.number_percent(Register::ForcedDischgSocLimit, "Forced Discharge Limit %")?,
            self.number_percent(Register::DischgCutOffSocEod, "Discharge Cutoff %")?,
            self.number_percent(
                Register::EpsDischgCutoffSocEod,
                "Discharge Cutoff for EPS %",
            )?,
            self.number_percent(
                Register::AcChargeStartSocLimit,
                "Charge From AC Lower Limit %",
            )?,
            self.number_percent(
                Register::AcChargeEndSocLimit,
                "Charge From AC Upper Limit %",
            )?,
            self.time_range("ac_charge/1", "AC Charge Timeslot 1")?,
            self.time_range("ac_charge/2", "AC Charge Timeslot 2")?,
            self.time_range("ac_charge/3", "AC Charge Timeslot 3")?,
            self.time_range("ac_first/1", "AC First Timeslot 1")?,
            self.time_range("ac_first/2", "AC First Timeslot 2")?,
            self.time_range("ac_first/3", "AC First Timeslot 3")?,
            self.time_range("charge_priority/1", "Charge Priority Timeslot 1")?,
            self.time_range("charge_priority/2", "Charge Priority Timeslot 2")?,
            self.time_range("charge_priority/3", "Charge Priority Timeslot 3")?,
            self.time_range("forced_discharge/1", "Forced Discharge Timeslot 1")?,
            self.time_range("forced_discharge/2", "Forced Discharge Timeslot 2")?,
            self.time_range("forced_discharge/3", "Forced Discharge Timeslot 3")?,
        ];

        Ok(r)
    }

    fn ha_discovery_topic(&self, kind: &str, name: &str) -> String {
        format!(
            "{}/{}/lxp_{}/{}/config",
            self.mqtt_config.homeassistant().prefix(),
            kind,
            self.inverter.datalog(),
            // The forward slash is used in some names (e.g. ac_charge/1) but
            // has semantic meaning in MQTT, so must be changed
            name.replace('/', "_"),
        )
    }

    fn apparent_power(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "apparent_power", "measurement", "VA")
    }

    fn battery(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "battery", "measurement", "%")
    }

    fn duration(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "duration", "total_increasing", "s")
    }

    fn frequency(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "frequency", "measurement", "Hz")
    }

    fn power(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "power", "measurement", "W")
    }

    fn energy(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "energy", "total_increasing", "kWh")
    }

    fn voltage(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "voltage", "measurement", "V")
    }

    fn temperature(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, "temperature", "measurement", "°C")
    }

    fn diagnostic(&self, name: &str, label: &str, topic_suffix: &str) -> Result<mqtt::Message> {
        let config = Diagnostic {
            entity_category: "diagnostic".to_owned(),
            state_topic: format!(
                "{}/{}/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                topic_suffix
            ),
            unique_id: format!("lxp_{}_{}", self.inverter.datalog(), name),
            name: label.to_string(),
            device: self.device(),
            availability: self.availability(),
        };

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("sensor", name),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    fn sensor(
        &self,
        name: &str,
        label: &str,
        device_class: &str,
        state_class: &str,
        unit_of_measurement: &str,
    ) -> Result<mqtt::Message> {
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

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("sensor", name),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    fn switch(&self, name: &str, label: &str) -> Result<mqtt::Message> {
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

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("switch", name),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    fn number_percent(&self, register: Register, label: &str) -> Result<mqtt::Message> {
        let config = Number {
            name: label.to_string(),
            state_topic: format!(
                "{}/{}/hold/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                register as i16,
            ),
            command_topic: format!(
                "{}/cmd/{}/set/hold/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                register as i16,
            ),
            value_template: "{{ float(value) }}".to_string(),
            unique_id: format!("lxp_{}_number_{:?}", self.inverter.datalog(), register),
            device: self.device(),
            availability: self.availability(),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            unit_of_measurement: "%".to_string(),
        };

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("number", &format!("{:?}", register)),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
    }

    // Models a time range as an MQTT Text field taking values like: 00:00-23:59
    fn time_range(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        let config = Text {
            name: label.to_string(),
            state_topic: format!(
                "{}/{}/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                name,
            ),
            command_topic: format!(
                "{}/cmd/{}/set/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                name,
            ),
            command_template: r#"{% set parts = value.split("-") %}{"start":"{{ parts[0] }}", "end":"{{ parts[1] }}"}"#.to_string(),
            value_template: r#"{{ value_json["start"] }}-{{ value_json["end"] }}"#.to_string(),
            unique_id: format!("lxp_{}_text_{}", self.inverter.datalog(), name),
            device: self.device(),
            availability: self.availability(),
            pattern: r"([01]?[0-9]|2[0-3]):[0-5][0-9]-([01]?[0-9]|2[0-3]):[0-5][0-9]".to_string(),
        };

        Ok(mqtt::Message {
            topic: self.ha_discovery_topic("text", name),
            retain: true,
            payload: serde_json::to_string(&config)?,
        })
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
