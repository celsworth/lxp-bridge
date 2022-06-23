use crate::prelude::*;

#[derive(Debug)]
pub enum Command {
    ReadInputs1(config::Inverter),
    ReadInputs2(config::Inverter),
    ReadInputs3(config::Inverter),
    ReadHold(config::Inverter, u16, u16),
    ReadParam(config::Inverter, u16),
    SetHold(config::Inverter, u16, u16),
    ChargeRate(config::Inverter, u16),
    DischargeRate(config::Inverter, u16),
    AcCharge(config::Inverter, bool),
    ForcedDischarge(config::Inverter, bool),
    AcChargeRate(config::Inverter, u16),
    AcChargeSocLimit(config::Inverter, u16),
    DischargeCutoffSocLimit(config::Inverter, u16),
}

impl Command {
    pub fn to_result_topic(&self) -> String {
        use Command::*;

        let rest = match self {
            ReadInputs1(inverter) => format!("{}/read/inputs/1", inverter.datalog),
            ReadInputs2(inverter) => format!("{}/read/inputs/2", inverter.datalog),
            ReadInputs3(inverter) => format!("{}/read/inputs/3", inverter.datalog),
            ReadHold(inverter, register, _) => {
                format!("{}/read/hold/{}", inverter.datalog, register)
            }
            ReadParam(inverter, register) => {
                format!("{}/read/param/{}", inverter.datalog, register)
            }
            SetHold(inverter, register, _) => format!("{}/set/hold/{}", inverter.datalog, register),
            AcCharge(inverter, _) => format!("{}/set/ac_charge", inverter.datalog),
            ForcedDischarge(inverter, _) => format!("{}/set/forced_discharge", inverter.datalog),
            ChargeRate(inverter, _) => format!("{}/set/charge_rate_pct", inverter.datalog),
            DischargeRate(inverter, _) => format!("{}/set/discharge_rate_pct", inverter.datalog),
            AcChargeRate(inverter, _) => format!("{}/set/ac_charge_rate_pct", inverter.datalog),
            AcChargeSocLimit(inverter, _) => {
                format!("{}/set/ac_charge_soc_limit_pct", inverter.datalog)
            }
            DischargeCutoffSocLimit(inverter, _) => {
                format!("{}/set/discharge_cutoff_soc_limit_pct", inverter.datalog)
            }
        };

        format!("result/{}", rest)
    }
}
