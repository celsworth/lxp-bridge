use crate::prelude::*;
use lxp::packet::Register;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Availability {
    topic: String,
}

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
pub struct Entity<'a> {
    // this is not serialised into the JSON output, just used as a transient store to
    // work out what unique_id and topic should be
    #[serde(skip)]
    key: &'a str, // for example, soc

    unique_id: &'a str, // lxp_XXXX_soc
    name: &'a str,      // really more of a label? for example, "State of Charge"

    state_topic: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_category: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_class: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_template: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_of_measurement: Option<&'a str>,
    device: Device,
    availability: Availability,
}

// https://www.home-assistant.io/integrations/sensor.mqtt/
#[derive(Debug, Serialize)]
pub struct Sensor {
    name: String,
    state_topic: String,
    unique_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_of_measurement: Option<String>,
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

    pub fn sensors(&self) -> Vec<mqtt::Message> {
        let base = Entity {
            key: &String::default(),
            unique_id: &String::default(),
            name: &String::default(),
            entity_category: None,
            device_class: None,
            state_class: None,
            unit_of_measurement: None,
            value_template: None,
            // TODO: might change this to an enum that defaults to InputsAll but can be replaced
            // with a string for a specific topic?
            state_topic: &format!(
                "{}/{}/inputs/all",
                self.mqtt_config.namespace(),
                self.inverter.datalog()
            ),
            device: self.device(),
            availability: self.availability(),
        };

        let voltage = Entity {
            device_class: Some("voltage"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("V"),
            ..base.clone()
        };

        let frequency = Entity {
            device_class: Some("frequency"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("Hz"),
            ..base.clone()
        };

        let power = Entity {
            device_class: Some("power"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("W"),
            ..base.clone()
        };

        // now each entry in here should only have to specify specific overrides for each key.
        // if we have multiple things sharing keys, consider whether to make a new variable to
        // inherit from.
        let sensors = [
            Entity {
                key: "soc",
                name: "State of Charge",
                device_class: Some("battery"),
                state_class: Some("measurement"),
                unit_of_measurement: Some("%"),
                ..base.clone()
            },
            Entity {
                key: "v_bat",
                name: "Battery Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_ac_r",
                name: "Grid Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_pv_1",
                name: "PV Voltage (String 1)",
                ..voltage.clone()
            },
            Entity {
                key: "v_pv_2",
                name: "PV Voltage (String 2)",
                ..voltage.clone()
            },
            Entity {
                key: "v_pv_3",
                name: "PV Voltage (String 3)",
                ..voltage.clone()
            },
            Entity {
                key: "f_ac",
                name: "Grid Frequency",
                ..frequency.clone()
            },
            Entity {
                key: "f_eps",
                name: "EPS Frequency",
                ..frequency.clone()
            },
            Entity {
                key: "s_eps",
                name: "Apparent EPS Power",
                device_class: Some("apparent_power"),
                unit_of_measurement: Some("VA"),
                ..power.clone()
            },
            Entity {
                key: "p_pv",
                name: "PV Power (Array)",
                ..power.clone()
            },
            Entity {
                key: "p_pv_1",
                name: "PV Power (String 1)",
                ..power.clone()
            },
            Entity {
                key: "p_pv_2",
                name: "PV Power (String 2)",
                ..power.clone()
            },
            Entity {
                key: "p_pv_3",
                name: "PV Power (String 3)",
                ..power.clone()
            },
            Entity {
                key: "p_battery",
                name: "Battery Power (discharge is negative)",
                ..power.clone()
            },
            Entity {
                key: "p_charge",
                name: "Battery Charge",
                ..power.clone()
            },
            Entity {
                key: "p_discharge",
                name: "Battery Discharge",
                ..power.clone()
            },
            Entity {
                key: "p_grid",
                name: "Grid Power (export is negative)",
                ..power.clone()
            },
        ];

        sensors
            .map(|sensor| {
                // fill in unique_id and value_template (if not set) which are derived from key
                let f = &format!("{{{{ value_json.{} }}}}", sensor.key);
                let value_template = sensor.value_template.or_else(|| Some(f));
                let sensor = Entity {
                    unique_id: &self.unique_id(sensor.key),
                    value_template,
                    ..sensor
                };

                mqtt::Message {
                    topic: self.ha_discovery_topic("sensor", sensor.key),
                    retain: true,
                    payload: serde_json::to_string(&sensor).unwrap(),
                }
            })
            .to_vec()
    }

    pub fn all(&self) -> Result<Vec<mqtt::Message>> {
        let mut r = vec![
            // nota diagnostic
            self.diagnostic("status", "Status", "input/0/parsed")?,
            self.diagnostic("fault_code", "Fault Code", "input/fault_code/parsed")?,
            self.diagnostic("warning_code", "Warning Code", "input/warning_code/parsed")?,
            // is a diagnostic
            self.duration("runtime", "Total Runtime")?,
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

        r.append(&mut self.sensors());

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
        self.sensor(
            name,
            label,
            Some("apparent_power"),
            Some("measurement"),
            Some("VA"),
        )
    }

    fn duration(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(
            name,
            label,
            Some("duration"),
            Some("total_increasing"),
            Some("s"),
        )
    }

    fn power(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(name, label, Some("power"), Some("measurement"), Some("W"))
    }

    fn energy(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(
            name,
            label,
            Some("energy"),
            Some("total_increasing"),
            Some("kWh"),
        )
    }

    fn temperature(&self, name: &str, label: &str) -> Result<mqtt::Message> {
        self.sensor(
            name,
            label,
            Some("temperature"),
            Some("measurement"),
            Some("°C"),
        )
    }

    fn diagnostic(&self, name: &str, label: &str, topic_suffix: &str) -> Result<mqtt::Message> {
        let config = Sensor {
            entity_category: Some("diagnostic".to_owned()),
            device_class: None,
            state_class: None,
            unit_of_measurement: None,
            value_template: None,
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
        device_class: Option<&str>,
        state_class: Option<&str>,
        unit_of_measurement: Option<&str>,
    ) -> Result<mqtt::Message> {
        let config = Sensor {
            entity_category: None,
            device_class: device_class.map(String::from),
            state_class: state_class.map(String::from),
            unit_of_measurement: unit_of_measurement.map(String::from),
            value_template: Some(format!("{{{{ value_json.{} }}}}", name)),
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

    fn unique_id(&self, name: &str) -> String {
        format!("lxp_{}_{}", self.inverter.datalog(), name)
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
