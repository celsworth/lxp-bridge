use crate::prelude::*;

#[derive(Debug)]
pub enum Command {
    ReadHold(config::Inverter, u16),
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
