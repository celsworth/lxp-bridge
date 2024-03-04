use crate::prelude::*;
use lxp::packet::Register;

use serde::{Serialize, Serializer};

// ValueTemplate {{{
#[derive(Clone, Debug, PartialEq)]
pub enum ValueTemplate {
    None,
    Default, // "{{ value_json.$key }}"
    String(String),
}
impl ValueTemplate {
    pub fn from_default(key: &str) -> Self {
        Self::String(format!("{{{{ value_json.{} }}}}", key))
    }
    pub fn is_none(&self) -> bool {
        *self == Self::None
    }
    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }
}
impl Serialize for ValueTemplate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueTemplate::String(str) => serializer.serialize_str(str),
            _ => unreachable!(),
        }
    }
} // }}}

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

// https://www.home-assistant.io/integrations/sensor.mqtt/
#[derive(Clone, Debug, Serialize)]
pub struct Entity<'a> {
    // this is not serialised into the JSON output, just used as a transient store to
    // work out what unique_id and topic should be
    #[serde(skip)]
    key: &'a str, // for example, soc

    unique_id: &'a str, // lxp_XXXX_soc
    name: &'a str,      // really more of a label? for example, "State of Charge"

    state_topic: &'a str,

    // these are all skipped in the output if None. this lets us use the same struct for
    // different types of entities, just our responsibility to make sure a sane set of attributes
    // are populated. Could make subtypes to enforce the various attributes being set for different
    // HA entity types but I think its not worth the extra complexity.
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_category: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_class: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_class: Option<&'a str>,
    #[serde(skip_serializing_if = "ValueTemplate::is_none")]
    value_template: ValueTemplate,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit_of_measurement: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<&'a str>,

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
            icon: None,
            value_template: ValueTemplate::Default, // "{{ value_json.$key }}"
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

        let current = Entity {
            device_class: Some("current"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("A"),
            ..base.clone()
        };

        let energy = Entity {
            device_class: Some("energy"),
            state_class: Some("total_increasing"),
            unit_of_measurement: Some("kWh"),
            ..base.clone()
        };

        let temperature = Entity {
            device_class: Some("temperature"),
            state_class: Some("measurement"),
            unit_of_measurement: Some("°C"),
            ..base.clone()
        };

        // now each entry in here should only have to specify specific overrides for each key.
        // if we have multiple things sharing keys, consider whether to make a new variable to
        // inherit from.
        let sensors = [
            Entity {
                key: "status",
                name: "Status",
                state_topic: &format!(
                    "{}/{}/input/0/parsed",
                    self.mqtt_config.namespace(),
                    self.inverter.datalog()
                ),
                value_template: ValueTemplate::None,
                ..base.clone()
            },
            Entity {
                key: "soc",
                name: "State of Charge",
                device_class: Some("battery"),
                state_class: Some("measurement"),
                unit_of_measurement: Some("%"),
                ..base.clone()
            },
            Entity {
                key: "fault_code",
                name: "Fault Code",
                entity_category: Some("diagnostic"),
                state_topic: &format!(
                    "{}/{}/input/fault_code/parsed",
                    self.mqtt_config.namespace(),
                    self.inverter.datalog()
                ),
                value_template: ValueTemplate::None,
                icon: Some("mdi:alert"),
                ..base.clone()
            },
            Entity {
                key: "warning_code",
                name: "Warning Code",
                entity_category: Some("diagnostic"),
                state_topic: &format!(
                    "{}/{}/input/warning_code/parsed",
                    self.mqtt_config.namespace(),
                    self.inverter.datalog()
                ),
                value_template: ValueTemplate::None,
                icon: Some("mdi:alert-outline"),
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
                key: "v_eps_r",
                name: "EPS Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_gen",
                name: "Generator Voltage",
                ..voltage.clone()
            },
            Entity {
                key: "v_eps_l1",
                name: "EPS Voltage L1",
                ..voltage.clone()
            },
            Entity {
                key: "v_eps_l2",
                name: "EPS Voltage L2",
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
                key: "f_gen",
                name: "Generator Frequency",
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
                key: "s_eps_l1",
                name: "Apparent EPS Power L1",
                device_class: Some("apparent_power"),
                unit_of_measurement: Some("VA"),
                ..power.clone()
            },
            Entity {
                key: "s_eps_l2",
                name: "Apparent EPS Power L2",
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
            Entity {
                key: "p_to_user",
                name: "Power from Grid",
                ..power.clone()
            },
            Entity {
                key: "p_to_grid",
                name: "Power to Grid",
                ..power.clone()
            },
            Entity {
                key: "p_eps",
                name: "Active EPS Power",
                ..power.clone()
            },
            Entity {
                key: "p_inv",
                name: "Inverter Power",
                ..power.clone()
            },
            Entity {
                key: "p_rec",
                name: "AC Charge Power",
                ..power.clone()
            },
            Entity {
                key: "p_gen",
                name: "Generator Power",
                ..power.clone()
            },
            Entity {
                key: "p_eps_l1",
                name: "EPS Power L1",
                ..power.clone()
            },
            Entity {
                key: "p_eps_l2",
                name: "EPS Power L2",
                ..power.clone()
            },
            Entity {
                key: "e_pv_all",
                name: "PV Generation (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_all_1",
                name: "PV Generation (All time) (String 1)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_all_2",
                name: "PV Generation (All time) (String 2)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_all_3",
                name: "PV Generation (All time) (String 3)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day",
                name: "PV Generation (Today))",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day_1",
                name: "PV Generation (Today) (String 1)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day_2",
                name: "PV Generation (Today) (String 2)",
                ..energy.clone()
            },
            Entity {
                key: "e_pv_day_3",
                name: "PV Generation (Today) (String 3)",
                ..energy.clone()
            },
            Entity {
                key: "e_chg_all",
                name: "Battery Charge (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_chg_day",
                name: "Battery Charge (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_dischg_all",
                name: "Battery Discharge (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_dischg_day",
                name: "Battery Discharge (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_user_all",
                name: "Energy from Grid (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_user_day",
                name: "Energy from Grid (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_grid_all",
                name: "Energy to Grid (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_to_grid_day",
                name: "Energy to Grid (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_all",
                name: "Energy from EPS (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_day",
                name: "Energy from EPS (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_rec_all",
                name: "Energy of AC Charging (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_rec_day",
                name: "Energy of AC Charging (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_inv_all",
                name: "Energy of Inverter (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_inv_day",
                name: "Energy of Inverter (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_gen_all",
                name: "Energy of Generator (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_gen_day",
                name: "Energy of Generator (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l1_all",
                name: "Energy of EPS L1 (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l1_day",
                name: "Energy of EPS L1  (Today)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l2_all",
                name: "Energy of EPS L2 (All time)",
                ..energy.clone()
            },
            Entity {
                key: "e_eps_l2_day",
                name: "Energy of EPS L2  (Today)",
                ..energy.clone()
            },
            Entity {
                key: "t_inner",
                name: "Inverter Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "t_rad_1",
                name: "Radiator 1 Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "t_rad_2",
                name: "Radiator 2 Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "t_bat",
                name: "Battery Temperature",
                ..temperature.clone()
            },
            Entity {
                key: "max_chg_curr",
                name: "Max Charge Current",
                ..current.clone()
            },
            Entity {
                key: "max_dischg_curr",
                name: "Max Discharge Current",
                ..current.clone()
            },
            Entity {
                key: "min_cell_voltage",
                name: "Min Cell Voltage (BMS)",
                ..voltage.clone()
            },
            Entity {
                key: "max_cell_voltage",
                name: "Max Cell Voltage (BMS)",
                ..voltage.clone()
            },
            Entity {
                key: "min_cell_temp",
                name: "Min Cell Temperature (BMS)",
                ..temperature.clone()
            },
            Entity {
                key: "max_cell_temp",
                name: "Max Cell Temperature (BMS)",
                ..temperature.clone()
            },
            Entity {
                key: "runtime",
                name: "Total Runtime",
                entity_category: Some("diagnostic"),
                device_class: Some("duration"),
                state_class: Some("total_increasing"),
                unit_of_measurement: Some("s"),
                ..base.clone()
            },
        ];

        sensors
            .map(|sensor| {
                // fill in unique_id and value_template (if default) which are derived from key
                let mut sensor = Entity {
                    unique_id: &self.unique_id(sensor.key),
                    ..sensor
                };
                if sensor.value_template.is_default() {
                    sensor.value_template = ValueTemplate::from_default(sensor.key);
                }

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
            self.switch("ac_charge", "AC Charge")?,
            self.switch("charge_priority", "Charge Priority")?,
            self.switch("forced_discharge", "Forced Discharge")?,
            self.number_percent(Register::ChargePowerPercentCmd, "System Charge Rate (%)")?,
            self.number_percent(Register::DischgPowerPercentCmd, "System Discharge Rate (%)")?,
            self.number_percent(Register::AcChargePowerCmd, "AC Charge Rate (%)")?,
            self.number_percent(Register::AcChargeSocLimit, "AC Charge Limit %")?,
            self.number_percent(Register::ChargePriorityPowerCmd, "Charge Priority Rate (%)")?,
            self.number_percent(Register::ChargePrioritySocLimit, "Charge Priority Limit %")?,
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
                register as u16,
            ),
            command_topic: format!(
                "{}/cmd/{}/set/hold/{}",
                self.mqtt_config.namespace(),
                self.inverter.datalog(),
                register as u16,
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
