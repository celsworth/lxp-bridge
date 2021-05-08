use crate::prelude::*;

#[derive(Debug)]
pub enum Command {
    Invalid(Message),
    ChargeRate(u16),
    DischargeRate(u16),
    AcCharge(bool),
    ForcedDischarge(bool),
    AcChargeRate(u16),
    AcChargeSocLimit(u16),
    DischargeCutoffSocLimit(u16),
}

impl From<Message> for Command {
    fn from(message: Message) -> Command {
        let parts: Vec<&str> = message.topic.split('/').collect();

        match parts.last() {
            // read_hold ? json payload?
            // set_hold ? json payload?
            // read_input
            //
            Some(&"ac_charge") => Command::AcCharge(message.payload_bool()),
            Some(&"forced_discharge") => Command::ForcedDischarge(message.payload_bool()),

            Some(&"charge_pct") | Some(&"charge_rate_pct") => {
                Command::ChargeRate(message.payload_percent())
            }
            Some(&"discharge_pct") | Some(&"discharge_rate_pct") => {
                Command::DischargeRate(message.payload_percent())
            }

            Some(&"ac_charge_rate_pct") => Command::AcChargeRate(message.payload_percent()),

            Some(&"charge_amount_pct") | Some(&"ac_charge_soc_limit_pct") => {
                Command::AcChargeSocLimit(message.payload_percent())
            }

            Some(&"discharge_cutoff_soc_limit_pct") => {
                Command::DischargeCutoffSocLimit(message.payload_percent())
            }

            _ => Command::Invalid(message),
        }
    }
}

impl Command {
    pub fn mqtt_topic(&self) -> &str {
        match self {
            Command::AcCharge(_) => "ac_charge",
            Command::ForcedDischarge(_) => "forced_discharge",
            Command::ChargeRate(_) => "charge_rate_pct",
            Command::DischargeRate(_) => "discharge_rate_pct",
            Command::AcChargeRate(_) => "ac_charge_rate_pct",
            Command::AcChargeSocLimit(_) => "ac_charge_soc_limit_pct",
            Command::DischargeCutoffSocLimit(_) => "discharge_cutoff_soc_limit_pct",
            Command::Invalid(_) => "",
        }
    }
}
