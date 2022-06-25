use crate::prelude::*;

use enum_dispatch::*;
use nom_derive::{Nom, Parse};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

pub enum ReadInput {
    ReadInputAll(Box<ReadInputAll>),
    ReadInput1(ReadInput1),
    ReadInput2(ReadInput2),
    ReadInput3(ReadInput3),
}

// {{{ ReadInputAll
#[derive(PartialEq, Clone, Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInputAll {
    pub status: u16,
    #[nom(Ignore)]
    pub v_pv: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_2: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_3: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bat: f64,

    pub soc: u8,
    pub soh: u8,
    #[nom(SkipBefore(2))]
    #[nom(Ignore)]
    pub p_pv: u16,
    pub p_pv_1: u16,
    pub p_pv_2: u16,
    pub p_pv_3: u16,
    pub p_charge: u16,
    pub p_discharge: u16,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_r: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_s: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_t: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_ac: f64,

    pub p_inv: u16,
    pub p_rec: u16,

    #[nom(SkipBefore(2))]
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub pf: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_r: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_s: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_t: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_eps: f64,
    #[nom(SkipBefore(4))] // peps and seps
    pub p_to_grid: u16,
    pub p_to_user: u16,

    #[nom(Ignore)]
    pub e_pv_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_2: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_3: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_inv_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_rec_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_chg_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_dischg_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_eps_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_grid_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_user_day: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_2: f64,

    #[nom(Ignore)]
    pub e_pv_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_1: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_2: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_3: f64,

    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_inv_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_rec_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_chg_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_dischg_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_eps_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_grid_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_user_all: f64,

    #[nom(SkipBefore(8))] // 4 byte fault code, 4 byte warning code?
    pub t_inner: u16,
    pub t_rad_1: u16,
    pub t_rad_2: u16,
    pub t_bat: u16,
    #[nom(SkipBefore(2))] // reserved - radiator 3?
    pub runtime: u32,
    // 18 bytes of auto_test stuff here I'm not doing yet
    #[nom(SkipBefore(18))] // auto_test stuff, TODO..
    #[nom(SkipBefore(2))] // bat_brand, bat_com_type
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_chg_curr: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_dischg_curr: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub charge_volt_ref: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub dischg_cut_volt: f64,

    pub bat_status_0: u16,
    pub bat_status_1: u16,
    pub bat_status_2: u16,
    pub bat_status_3: u16,
    pub bat_status_4: u16,
    pub bat_status_5: u16,
    pub bat_status_6: u16,
    pub bat_status_7: u16,
    pub bat_status_8: u16,
    pub bat_status_9: u16,
    pub bat_status_inv: u16,

    pub bat_count: u16,
    pub bat_capacity: u16,

    #[nom(Parse = "Utils::le_i16_div100")]
    pub bat_current: f64,

    pub bms_event_1: u16,
    pub bms_event_2: u16,

    // TODO: probably floats but need non-zero sample data to check. just guessing at the div100.
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_cell_voltage: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub min_cell_voltage: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_cell_temp: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub min_cell_temp: f64,

    pub bms_fw_update_state: u16,

    pub cycle_count: u16,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub vbat_inv: f64,

    // 14 bytes I'm not sure what they are; possibly generator stuff
    #[nom(SkipBefore(14))]
    // following are for influx capability only
    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInput1
#[derive(Clone, Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInput1 {
    pub status: u16,
    #[nom(Ignore)]
    pub v_pv: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_2: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_pv_3: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bat: f64,

    pub soc: u8,
    pub soh: u8,
    #[nom(SkipBefore(2))]
    #[nom(Ignore)]
    pub p_pv: u16,
    pub p_pv_1: u16,
    pub p_pv_2: u16,
    pub p_pv_3: u16,
    pub p_charge: u16,
    pub p_discharge: u16,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_r: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_s: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_ac_t: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_ac: f64,

    pub p_inv: u16,
    pub p_rec: u16,

    #[nom(SkipBefore(2))]
    #[nom(Parse = "Utils::le_u16_div1000")]
    pub pf: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_r: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_s: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_eps_t: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub f_eps: f64,
    #[nom(SkipBefore(4))] // peps and seps
    pub p_to_grid: u16,
    pub p_to_user: u16,

    #[nom(Ignore)]
    pub e_pv_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_2: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_pv_day_3: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_inv_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_rec_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_chg_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_dischg_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_eps_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_grid_day: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub e_to_user_day: f64,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_1: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub v_bus_2: f64,

    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInput2
#[derive(Clone, Debug, Serialize, Nom)]
#[nom(Debug, LittleEndian)]
pub struct ReadInput2 {
    #[nom(Ignore)]
    pub e_pv_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_1: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_2: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_pv_all_3: f64,

    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_inv_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_rec_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_chg_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_dischg_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_eps_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_grid_all: f64,
    #[nom(Parse = "Utils::le_u32_div10")]
    pub e_to_user_all: f64,

    #[nom(SkipBefore(8))] // 4 byte fault code, 4 byte warning code?
    pub t_inner: u16,
    pub t_rad_1: u16,
    pub t_rad_2: u16,
    pub t_bat: u16,

    #[nom(SkipBefore(2))] // reserved
    pub runtime: u32,
    // 18 bytes of auto_test stuff here I'm not doing yet
    //
    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInput3
#[derive(Clone, Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInput3 {
    #[nom(SkipBefore(2))] // bat_brand, bat_com_type
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_chg_curr: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_dischg_curr: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub charge_volt_ref: f64,
    #[nom(Parse = "Utils::le_u16_div10")]
    pub dischg_cut_volt: f64,

    pub bat_status_0: u16,
    pub bat_status_1: u16,
    pub bat_status_2: u16,
    pub bat_status_3: u16,
    pub bat_status_4: u16,
    pub bat_status_5: u16,
    pub bat_status_6: u16,
    pub bat_status_7: u16,
    pub bat_status_8: u16,
    pub bat_status_9: u16,
    pub bat_status_inv: u16,

    pub bat_count: u16,
    pub bat_capacity: u16,

    #[nom(Parse = "Utils::le_i16_div100")]
    pub bat_current: f64,

    pub bms_event_1: u16,
    pub bms_event_2: u16,

    // TODO: probably floats but need non-zero sample data to check. just guessing at the div100.
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_cell_voltage: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub min_cell_voltage: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub max_cell_temp: f64,
    #[nom(Parse = "Utils::le_u16_div100")]
    pub min_cell_temp: f64,

    pub bms_fw_update_state: u16,

    pub cycle_count: u16,

    #[nom(Parse = "Utils::le_u16_div10")]
    pub vbat_inv: f64,

    // following are for influx capability only
    #[nom(Parse = "Utils::current_time_for_nom")]
    pub time: UnixTime,
    #[nom(Ignore)]
    pub datalog: Serial,
} // }}}

// {{{ ReadInputs
#[derive(Default, Clone, Debug)]
pub struct ReadInputs {
    read_input_1: Option<ReadInput1>,
    read_input_2: Option<ReadInput2>,
    read_input_3: Option<ReadInput3>,
}

impl ReadInputs {
    pub fn set_read_input_1(&mut self, i: ReadInput1) {
        self.read_input_1 = Some(i);
    }
    pub fn set_read_input_2(&mut self, i: ReadInput2) {
        self.read_input_2 = Some(i);
    }
    pub fn set_read_input_3(&mut self, i: ReadInput3) {
        self.read_input_3 = Some(i);
    }

    pub fn to_input_all(&self) -> Option<ReadInputAll> {
        match (
            self.read_input_1.as_ref(),
            self.read_input_2.as_ref(),
            self.read_input_3.as_ref(),
        ) {
            (Some(ri1), Some(ri2), Some(ri3)) => Some(ReadInputAll {
                status: ri1.status,
                v_pv: ri1.v_pv,
                v_pv_1: ri1.v_pv_1,
                v_pv_2: ri1.v_pv_2,
                v_pv_3: ri1.v_pv_3,
                v_bat: ri1.v_bat,
                soc: ri1.soc,
                soh: ri1.soh,
                p_pv: ri1.p_pv,
                p_pv_1: ri1.p_pv_1,
                p_pv_2: ri1.p_pv_2,
                p_pv_3: ri1.p_pv_3,
                p_charge: ri1.p_charge,
                p_discharge: ri1.p_discharge,
                v_ac_r: ri1.v_ac_r,
                v_ac_s: ri1.v_ac_s,
                v_ac_t: ri1.v_ac_t,
                f_ac: ri1.f_ac,
                p_inv: ri1.p_inv,
                p_rec: ri1.p_rec,
                pf: ri1.pf,
                v_eps_r: ri1.v_eps_r,
                v_eps_s: ri1.v_eps_s,
                v_eps_t: ri1.v_eps_t,
                f_eps: ri1.f_eps,
                p_to_grid: ri1.p_to_grid,
                p_to_user: ri1.p_to_user,
                e_pv_day: ri1.e_pv_day,
                e_pv_day_1: ri1.e_pv_day_1,
                e_pv_day_2: ri1.e_pv_day_2,
                e_pv_day_3: ri1.e_pv_day_3,
                e_inv_day: ri1.e_inv_day,
                e_rec_day: ri1.e_rec_day,
                e_chg_day: ri1.e_chg_day,
                e_dischg_day: ri1.e_dischg_day,
                e_eps_day: ri1.e_eps_day,
                e_to_grid_day: ri1.e_to_grid_day,
                e_to_user_day: ri1.e_to_user_day,
                v_bus_1: ri1.v_bus_1,
                v_bus_2: ri1.v_bus_2,
                e_pv_all: ri2.e_pv_all,
                e_pv_all_1: ri2.e_pv_all_1,
                e_pv_all_2: ri2.e_pv_all_2,
                e_pv_all_3: ri2.e_pv_all_3,
                e_inv_all: ri2.e_inv_all,
                e_rec_all: ri2.e_rec_all,
                e_chg_all: ri2.e_chg_all,
                e_dischg_all: ri2.e_dischg_all,
                e_eps_all: ri2.e_eps_all,
                e_to_grid_all: ri2.e_to_grid_all,
                e_to_user_all: ri2.e_to_user_all,
                t_inner: ri2.t_inner,
                t_rad_1: ri2.t_rad_1,
                t_rad_2: ri2.t_rad_2,
                t_bat: ri2.t_bat,
                runtime: ri2.runtime,
                max_chg_curr: ri3.max_chg_curr,
                max_dischg_curr: ri3.max_dischg_curr,
                charge_volt_ref: ri3.charge_volt_ref,
                dischg_cut_volt: ri3.dischg_cut_volt,
                bat_status_0: ri3.bat_status_0,
                bat_status_1: ri3.bat_status_1,
                bat_status_2: ri3.bat_status_2,
                bat_status_3: ri3.bat_status_3,
                bat_status_4: ri3.bat_status_4,
                bat_status_5: ri3.bat_status_5,
                bat_status_6: ri3.bat_status_6,
                bat_status_7: ri3.bat_status_7,
                bat_status_8: ri3.bat_status_8,
                bat_status_9: ri3.bat_status_9,
                bat_status_inv: ri3.bat_status_inv,
                bat_count: ri3.bat_count,
                bat_capacity: ri3.bat_capacity,
                bat_current: ri3.bat_current,
                bms_event_1: ri3.bms_event_1,
                bms_event_2: ri3.bms_event_2,
                max_cell_voltage: ri3.max_cell_voltage,
                min_cell_voltage: ri3.min_cell_voltage,
                max_cell_temp: ri3.max_cell_temp,
                min_cell_temp: ri3.min_cell_temp,
                bms_fw_update_state: ri3.bms_fw_update_state,
                cycle_count: ri3.cycle_count,
                vbat_inv: ri3.vbat_inv,
                datalog: ri1.datalog,
                time: ri1.time.clone(),
            }),
            _ => None,
        }
    }
} // }}}

// {{{ TcpFunction
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum TcpFunction {
    Heartbeat = 193,
    TranslatedData = 194,
    ReadParam = 195,
    WriteParam = 196,
} // }}}

// {{{ DeviceFunction
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum DeviceFunction {
    ReadHold = 3,
    ReadInput = 4,
    WriteSingle = 6,
    WriteMulti = 16,
    // UpdatePrepare = 33
    // UpdateSendData = 34
    // UpdateReset = 35
    // ReadHoldError = 131
    // ReadInputError = 132
    // WriteSingleError = 134
    // WriteMultiError = 144
} // }}}

#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register {
    Register21 = 21,            // not sure of a better name for this one..
    ChargePowerPercentCmd = 64, // System Charge Rate (%)
    DischgPowerPercentCmd = 65, // System Discharge Rate (%)
    AcChargePowerCmd = 66,      // Grid Charge Power Rate (%)
    AcChargeSocLimit = 67,      // AC Charge SOC Limit (%)
    DischgCutOffSocEod = 105,   // Discharge cut-off SOC (%)
}

#[derive(Clone, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum RegisterBit {
    // Register 21
    AcChargeEnable = 1 << 7,
    ForcedDischargeEnable = 1 << 10,
}

#[enum_dispatch]
pub trait PacketCommon {
    fn datalog(&self) -> Serial;
    fn set_datalog(&mut self, datalog: Serial);
    fn inverter(&self) -> Option<Serial>;
    fn set_inverter(&mut self, serial: Serial);
    fn protocol(&self) -> u16;
    fn tcp_function(&self) -> TcpFunction;
    fn bytes(&self) -> Vec<u8>;

    fn register(&self) -> u16 {
        unimplemented!("register() not implemented");
    }
    fn value(&self) -> u16 {
        unimplemented!("value() not implemented");
    }
}

pub struct TcpFrameFactory;
impl TcpFrameFactory {
    pub fn build(data: &Packet) -> Vec<u8> {
        let data_bytes = data.bytes();
        let data_length = data_bytes.len() as u8;
        let frame_length = (18 + data_length) as u16;

        // debug!("data_length={}, frame_length={}", data_length, frame_length);

        let mut r = vec![0; frame_length as usize];

        r[0] = 161;
        r[1] = 26;
        r[2..4].copy_from_slice(&data.protocol().to_le_bytes());
        r[4..6].copy_from_slice(&(frame_length - 6).to_le_bytes());
        r[6] = 1; // unsure what this is, always seems to be 1
        r[7] = data.tcp_function() as u8;

        r[8..18].copy_from_slice(&data.datalog().data());
        // WIP - trying to work out how to learn the inverter sn
        //r[8..18].copy_from_slice(&[0; 10]);

        r[18..].copy_from_slice(&data_bytes);

        r
    }
}

#[enum_dispatch(PacketCommon)]
#[derive(PartialEq, Debug, Clone)]
pub enum Packet {
    Heartbeat(Heartbeat),
    TranslatedData(TranslatedData),
    ReadParam(ReadParam),
}

#[derive(PartialEq)]
enum PacketSource {
    Inverter,
    Client,
}

/////////////
//
// HEARTBEATS
//
/////////////

#[derive(PartialEq, Clone, Debug)]
pub struct Heartbeat {
    pub datalog: Serial,
}
impl Heartbeat {
    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 19 {
            bail!("heartbeat packet too short");
        }

        // assert that the final byte is 0, meaning 0 data bytes follow it
        if input[18] != 0 {
            bail!("heartbeat with non-zero ({}) length byte?", input[18]);
        }

        let datalog = Serial::new(&input[8..18])?;

        Ok(Self { datalog })
    }
}

impl PacketCommon for Heartbeat {
    fn protocol(&self) -> u16 {
        2
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }
    fn set_datalog(&mut self, datalog: Serial) {
        self.datalog = datalog;
    }
    fn inverter(&self) -> Option<Serial> {
        None
    }
    fn set_inverter(&mut self, _datalog: Serial) {}

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::Heartbeat
    }

    fn bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

/////////////
//
// TRANSLATED DATA
//
/////////////

#[derive(PartialEq, Clone, Debug)]
pub struct TranslatedData {
    pub datalog: Serial,
    pub device_function: DeviceFunction, // ReadHold or ReadInput etc..
    pub inverter: Serial,                // inverter serial
    pub register: u16,                   // first register of values
    pub values: Vec<u8>,                 // undecoded, since can be u16 or u32s?
}
impl TranslatedData {
    pub fn pairs(&self) -> Vec<(u16, u16)> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    pub fn read_input(&self) -> Result<ReadInput> {
        // note len() is of Vec<u8>, so not register count
        match (self.register, self.values.len()) {
            (0, 254) => Ok(ReadInput::ReadInputAll(Box::new(self.read_input_all()?))),
            (0, 80) => Ok(ReadInput::ReadInput1(self.read_input1()?)),
            (40, 80) => Ok(ReadInput::ReadInput2(self.read_input2()?)),
            (80, 80) => Ok(ReadInput::ReadInput3(self.read_input3()?)),
            (r1, r2) => Err(anyhow!("unhandled ReadInput register={} len={}", r1, r2)),
        }
    }

    fn read_input_all(&self) -> Result<ReadInputAll> {
        match ReadInputAll::parse(&self.values) {
            Ok((_, mut r)) => {
                r.p_pv = r.p_pv_1 + r.p_pv_2 + r.p_pv_3;
                r.v_pv = r.v_pv_1 + r.v_pv_2 + r.v_pv_3;
                r.e_pv_day = r.e_pv_day_1 + r.e_pv_day_2 + r.e_pv_day_3;
                r.e_pv_all = r.e_pv_all_1 + r.e_pv_all_2 + r.e_pv_all_3;
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    fn read_input1(&self) -> Result<ReadInput1> {
        match ReadInput1::parse(&self.values) {
            Ok((_, mut r)) => {
                r.p_pv = r.p_pv_1 + r.p_pv_2 + r.p_pv_3;
                r.v_pv = r.v_pv_1 + r.v_pv_2 + r.v_pv_3;
                r.e_pv_day = r.e_pv_day_1 + r.e_pv_day_2 + r.e_pv_day_3;
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    fn read_input2(&self) -> Result<ReadInput2> {
        match ReadInput2::parse(&self.values) {
            Ok((_, mut r)) => {
                r.e_pv_all = r.e_pv_all_1 + r.e_pv_all_2 + r.e_pv_all_3;
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    fn read_input3(&self) -> Result<ReadInput3> {
        match ReadInput3::parse(&self.values) {
            Ok((_, mut r)) => {
                r.datalog = self.datalog;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 38 {
            bail!("packet too short");
        }

        let protocol = Utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[20..len - 2];

        let checksum = &input[len - 2..];
        if Self::checksum(data) != checksum {
            bail!(
                "TranslatedData::decode checksum mismatch - got {:?}, expected {:?}",
                checksum,
                Self::checksum(data)
            );
        }

        //let address = data[0]; // 0=client, 1=inverter?
        let device_function = DeviceFunction::try_from(data[1])?;
        let inverter = Serial::new(&data[2..12])?;
        let register = Utils::u16ify(data, 12);

        let mut value_len = 2;
        let mut value_offset = 14;

        if Self::has_value_length_byte(PacketSource::Inverter, protocol, device_function) {
            value_len = data[value_offset] as usize;
            value_offset += 1;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            bail!(
                "TranslatedData::decode mismatch: values.len()={}, value_length_byte={}",
                values.len(),
                value_len
            );
        }

        Ok(Self {
            datalog,
            device_function,
            inverter,
            register,
            values,
        })
    }

    fn has_value_length_byte(
        source: PacketSource,
        protocol: u16,
        device_function: DeviceFunction,
    ) -> bool {
        use DeviceFunction::*;

        let p1 = protocol == 1;
        let psi = source == PacketSource::Inverter;
        match device_function {
            ReadHold | ReadInput => !p1 && psi,
            WriteSingle => false,
            WriteMulti => !p1 && !psi,
        }
    }

    fn checksum(data: &[u8]) -> [u8; 2] {
        crc16::State::<crc16::MODBUS>::calculate(data).to_le_bytes()
    }
}

impl PacketCommon for TranslatedData {
    fn protocol(&self) -> u16 {
        if self.device_function == DeviceFunction::WriteMulti {
            2
        } else {
            1
        }
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }
    fn set_datalog(&mut self, datalog: Serial) {
        self.datalog = datalog;
    }

    fn inverter(&self) -> Option<Serial> {
        Some(self.inverter)
    }
    fn set_inverter(&mut self, serial: Serial) {
        self.inverter = serial;
    }

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::TranslatedData
    }

    fn bytes(&self) -> Vec<u8> {
        let mut data = vec![0; 16];

        // data[2] (address) is 0 when writing to inverter, 1 when reading from it
        data[3] = self.device_function as u8;
        data[4..14].copy_from_slice(&self.inverter.data());
        // WIP - trying to work out how to learn the datalog sn
        //data[2..12].copy_from_slice(&[0xFF; 10]);
        data[14..16].copy_from_slice(&self.register.to_le_bytes());

        if self.device_function == DeviceFunction::WriteMulti {
            let register_count = self.pairs().len() as u16;
            data.extend_from_slice(&register_count.to_le_bytes());
        }

        if Self::has_value_length_byte(PacketSource::Client, self.protocol(), self.device_function)
        {
            let len = self.values.len() as u8;
            data.extend_from_slice(&[len]);
        }

        let mut m = Vec::new();
        for i in &self.values {
            m.extend_from_slice(&i.to_le_bytes());
        }
        data.append(&mut m);

        // the first two bytes are the data length, excluding checksum which we'll add next
        let data_length = data.len() as u16;
        data[0..2].copy_from_slice(&data_length.to_le_bytes());

        // checksum does not include the first two bytes (data length)
        data.extend_from_slice(&Self::checksum(&data[2..]));

        data
    }

    fn register(&self) -> u16 {
        self.register
    }

    fn value(&self) -> u16 {
        Utils::u16ify(&self.values, 0)
    }
}

/////////////
//
// READ PARAM
//
/////////////

#[derive(PartialEq, Clone, Debug)]
pub struct ReadParam {
    pub datalog: Serial,
    pub register: u16,   // first register of values
    pub values: Vec<u8>, // undecoded, since can be u16 or u32s?
}
impl ReadParam {
    pub fn pairs(&self) -> Vec<(u16, u16)> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 24 {
            bail!("packet too short");
        }

        let protocol = Utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[18..];
        let register = Utils::u16ify(data, 0);

        let mut value_len = 2;
        let mut value_offset = 2;

        if Self::has_value_length_bytes(protocol) {
            value_len = Utils::u16ify(data, value_offset) as usize;
            value_offset += 2;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            bail!(
                "ReadParam::decode mismatch: values.len()={}, value_length_byte={}",
                values.len(),
                value_len
            );
        }

        Ok(Self {
            datalog,
            register,
            values,
        })
    }

    fn has_value_length_bytes(protocol: u16) -> bool {
        protocol == 2
    }
}

impl PacketCommon for ReadParam {
    fn protocol(&self) -> u16 {
        2
    }

    fn datalog(&self) -> Serial {
        self.datalog
    }
    fn set_datalog(&mut self, datalog: Serial) {
        self.datalog = datalog;
    }
    fn inverter(&self) -> Option<Serial> {
        None
    }
    fn set_inverter(&mut self, _datalog: Serial) {}

    fn tcp_function(&self) -> TcpFunction {
        TcpFunction::ReadParam
    }

    fn bytes(&self) -> Vec<u8> {
        let mut r = vec![0; 2];

        r[0..2].copy_from_slice(&self.register.to_le_bytes());

        r
    }

    fn register(&self) -> u16 {
        self.register
    }

    fn value(&self) -> u16 {
        Utils::u16ify(&self.values, 0)
    }
}

pub struct Parser;
impl Parser {
    pub fn parse(input: &[u8]) -> Result<Packet> {
        let input_len = input.len() as u8;
        if input_len < 18 {
            bail!("packet less than 18 bytes?");
        }

        if input[0..2] != [161, 26] {
            bail!("invalid packet prefix");
        }

        if input_len < input[4] - 6 {
            bail!(
                "Parser::parse mismatch: input.len()={},  frame_length={}",
                input_len,
                input[4] - 6
            );
        }

        match TcpFunction::try_from(input[7])? {
            TcpFunction::Heartbeat => Ok(Packet::Heartbeat(Heartbeat::decode(input)?)),
            TcpFunction::TranslatedData => {
                Ok(Packet::TranslatedData(TranslatedData::decode(input)?))
            }
            TcpFunction::ReadParam => Ok(Packet::ReadParam(ReadParam::decode(input)?)),
            _ => Err(anyhow!("not handled: tcp_function={}", input[7])),
        }
    }
}
