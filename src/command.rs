#[derive(Debug)]
pub enum Command {
    ReadHold(u16),
    ReadParam(u16),
    SetHold(u16, u16),
    ChargeRate(u16),
    DischargeRate(u16),
    AcCharge(bool),
    ForcedDischarge(bool),
    AcChargeRate(u16),
    AcChargeSocLimit(u16),
    DischargeCutoffSocLimit(u16),
}
