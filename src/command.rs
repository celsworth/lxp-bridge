use crate::prelude::*;

#[derive(Debug)]
pub enum Command {
    ReadInputs(config::Inverter, u16),
    ReadInput(config::Inverter, u16, u16),
    ReadHold(config::Inverter, u16, u16),
    ReadParam(config::Inverter, u16),
    ReadAcChargeTime(config::Inverter, u16),
    ReadAcFirstTime(config::Inverter, u16),
    ReadChargePriorityTime(config::Inverter, u16),
    ReadForcedDischargeTime(config::Inverter, u16),
    SetHold(config::Inverter, u16, u16),
    WriteParam(config::Inverter, u16, u16),
    SetAcChargeTime(config::Inverter, u16, [u8; 4]),
    SetAcFirstTime(config::Inverter, u16, [u8; 4]),
    SetChargePriorityTime(config::Inverter, u16, [u8; 4]),
    SetForcedDischargeTime(config::Inverter, u16, [u8; 4]),
    ChargeRate(config::Inverter, u16),
    DischargeRate(config::Inverter, u16),
    AcCharge(config::Inverter, bool),
    ChargePriority(config::Inverter, bool),
    ForcedDischarge(config::Inverter, bool),
    AcChargeRate(config::Inverter, u16),
    AcChargeSocLimit(config::Inverter, u16),
    DischargeCutoffSocLimit(config::Inverter, u16),
}

impl Command {
    pub fn to_result_topic(&self) -> String {
        use Command::*;

        let rest = match self {
            ReadInputs(inverter, c) => format!("{}/read/inputs/{}", inverter.datalog(), c),
            ReadInput(inverter, register, _) => {
                format!("{}/read/input/{}", inverter.datalog(), register)
            }
            ReadHold(inverter, register, _) => {
                format!("{}/read/hold/{}", inverter.datalog(), register)
            }
            ReadParam(inverter, register) => {
                format!("{}/read/param/{}", inverter.datalog(), register)
            }
            ReadAcChargeTime(inverter, num) => {
                format!("{}/read/ac_charge/{}", inverter.datalog(), num)
            }
            ReadAcFirstTime(inverter, num) => {
                format!("{}/read/ac_first/{}", inverter.datalog(), num)
            }
            ReadChargePriorityTime(inverter, num) => {
                format!("{}/read/charge_priority/{}", inverter.datalog(), num)
            }
            ReadForcedDischargeTime(inverter, num) => {
                format!("{}/read/forced_discharge/{}", inverter.datalog(), num)
            }
            SetHold(inverter, register, _) => {
                format!("{}/set/hold/{}", inverter.datalog(), register)
            }
            WriteParam(inverter, register, _) => {
                format!("{}/set/param/{}", inverter.datalog(), register)
            }
            SetAcChargeTime(inverter, num, _) => {
                format!("{}/set/ac_charge/{}", inverter.datalog(), num)
            }
            SetAcFirstTime(inverter, num, _) => {
                format!("{}/set/ac_first/{}", inverter.datalog(), num)
            }
            SetChargePriorityTime(inverter, num, _) => {
                format!("{}/set/charge_priority/{}", inverter.datalog(), num)
            }
            SetForcedDischargeTime(inverter, num, _) => {
                format!("{}/set/forced_discharge/{}", inverter.datalog(), num)
            }
            AcCharge(inverter, _) => format!("{}/set/ac_charge", inverter.datalog()),
            ChargePriority(inverter, _) => format!("{}/set/charge_priority", inverter.datalog()),
            ForcedDischarge(inverter, _) => format!("{}/set/forced_discharge", inverter.datalog()),
            ChargeRate(inverter, _) => format!("{}/set/charge_rate_pct", inverter.datalog()),
            DischargeRate(inverter, _) => format!("{}/set/discharge_rate_pct", inverter.datalog()),
            AcChargeRate(inverter, _) => format!("{}/set/ac_charge_rate_pct", inverter.datalog()),
            AcChargeSocLimit(inverter, _) => {
                format!("{}/set/ac_charge_soc_limit_pct", inverter.datalog())
            }
            DischargeCutoffSocLimit(inverter, _) => {
                format!("{}/set/discharge_cutoff_soc_limit_pct", inverter.datalog())
            }
        };

        format!("result/{}", rest)
    }
}
