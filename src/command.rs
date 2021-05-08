use crate::prelude::*;

#[derive(Debug)]
pub enum Command {
    ReadHold(u16),
    ChargeRate(u16),
    DischargeRate(u16),
    AcCharge(bool),
    ForcedDischarge(bool),
    AcChargeRate(u16),
    AcChargeSocLimit(u16),
    DischargeCutoffSocLimit(u16),
}

impl TryFrom<Message> for Command {
    type Error = anyhow::Error;

    fn try_from(message: Message) -> Result<Command> {
        let parts: Vec<&str> = message.topic.split('/').collect();
        let parts = &parts[2..]; // drop lxp/cmd

        match parts {
            // read_input
            ["read_hold", register] => Ok(Command::ReadHold(register.parse()?)),
            // set_hold
            ["ac_charge"] => Ok(Command::AcCharge(message.payload_bool())),

            ["forced_discharge"] => Ok(Command::ForcedDischarge(message.payload_bool())),

            ["charge_pct"] | ["charge_rate_pct"] => {
                Ok(Command::ChargeRate(message.payload_percent()))
            }
            ["discharge_pct"] | ["discharge_rate_pct"] => {
                Ok(Command::DischargeRate(message.payload_percent()))
            }
            ["ac_charge_rate_pct"] => Ok(Command::AcChargeRate(message.payload_percent())),

            ["charge_amount_pct"] | ["ac_charge_soc_limit_pct"] => {
                Ok(Command::AcChargeSocLimit(message.payload_percent()))
            }

            ["discharge_cutoff_soc_limit_pct"] => {
                Ok(Command::DischargeCutoffSocLimit(message.payload_percent()))
            }

            [..] => Err(anyhow!("unhandled: {:?}", parts)),
        }
    }
}

impl Command {
    pub fn mqtt_topic(&self) -> &str {
        match self {
            Command::ReadHold(_) => "read_hold",
            Command::AcCharge(_) => "ac_charge",
            Command::ForcedDischarge(_) => "forced_discharge",
            Command::ChargeRate(_) => "charge_rate_pct",
            Command::DischargeRate(_) => "discharge_rate_pct",
            Command::AcChargeRate(_) => "ac_charge_rate_pct",
            Command::AcChargeSocLimit(_) => "ac_charge_soc_limit_pct",
            Command::DischargeCutoffSocLimit(_) => "discharge_cutoff_soc_limit_pct",
        }
    }
}
