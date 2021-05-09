use crate::prelude::*;

use nom_derive::{Nom, Parse};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;
use std::convert::TryFrom;

const HEADER_LENGTH: usize = 20;

fn le_u16_div10(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u16(input)?;
    Ok((input, num as f64 / 10.0))
}
fn le_u16_div100(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u16(input)?;
    Ok((input, num as f64 / 100.0))
}
fn le_u16_div1000(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u16(input)?;
    Ok((input, num as f64 / 1000.0))
}
fn le_u32_div10(input: &[u8]) -> nom::IResult<&[u8], f64> {
    let (input, num) = nom::number::complete::le_u32(input)?;
    Ok((input, num as f64 / 10.0))
}

pub struct Pair {
    pub register: u16,
    pub value: u16,
}

// {{{ ReadInput1
#[derive(Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInput1 {
    pub status: u16,
    #[nom(Ignore)]
    pub v_pv: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_pv_1: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_pv_2: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_pv_3: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_bat: f64,

    pub soc: u8,
    #[nom(SkipBefore(3))]
    #[nom(Ignore)]
    pub p_pv: u16,
    pub p_pv_1: u16,
    pub p_pv_2: u16,
    pub p_pv_3: u16,
    pub p_charge: u16,
    pub p_discharge: u16,

    #[nom(Parse = "le_u16_div10")]
    pub v_ac_r: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_ac_s: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_ac_t: f64,
    #[nom(Parse = "le_u16_div100")]
    pub f_ac: f64,

    pub p_inv: u16,
    pub p_rec: u16,

    #[nom(SkipBefore(2))]
    #[nom(Parse = "le_u16_div1000")]
    pub pf: f64,

    #[nom(Parse = "le_u16_div10")]
    pub v_eps_r: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_eps_s: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_eps_t: f64,
    #[nom(Parse = "le_u16_div100")]
    pub f_eps: f64,
    #[nom(SkipBefore(4))] // peps and seps
    pub p_to_grid: u16,
    pub p_to_user: u16,

    #[nom(Ignore)]
    pub e_pv_day: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_pv_day_1: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_pv_day_2: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_pv_day_3: f64,

    #[nom(Parse = "le_u16_div10")]
    pub e_inv_day: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_rec_day: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_chg_day: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_dischg_day: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_eps_day: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_to_grid_day: f64,
    #[nom(Parse = "le_u16_div10")]
    pub e_to_user_day: f64,

    #[nom(Parse = "le_u16_div10")]
    pub v_bus_1: f64,
    #[nom(Parse = "le_u16_div10")]
    pub v_bus_2: f64,
} // }}}

// {{{ ReadInput2
#[derive(Debug, Serialize, Nom)]
#[nom(Debug, LittleEndian)]
pub struct ReadInput2 {
    #[nom(Ignore)]
    pub e_pv_all: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_pv_all_1: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_pv_all_2: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_pv_all_3: f64,

    #[nom(Parse = "le_u32_div10")]
    pub e_inv_all: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_rec_all: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_chg_all: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_dischg_all: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_eps_all: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_to_grid_all: f64,
    #[nom(Parse = "le_u32_div10")]
    pub e_to_user_all: f64,

    #[nom(SkipBefore(8))] // 4 byte fault code, 4 byte warning code?
    pub t_inner: u16,
    pub t_rad_1: u16,
    pub t_rad_2: u16,
    pub t_bat: u16,

    #[nom(SkipBefore(2))] // no idea
    pub uptime: u32,
} // }}}

// {{{ ReadInput3
#[derive(Debug, Serialize, Nom)]
#[nom(LittleEndian)]
pub struct ReadInput3 {
    #[nom(SkipBefore(2))] // unsure, observed : 10)
    #[nom(Parse = "le_u16_div100")]
    pub max_chg_curr: f64,
    #[nom(Parse = "le_u16_div100")]
    pub max_dischg_curr: f64,
    #[nom(Parse = "le_u16_div10")]
    pub charge_volt_ref: f64,
    #[nom(Parse = "le_u16_div10")]
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
} // }}}

#[derive(PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum TcpFunction {
    Heartbeat = 193,
    TranslatedData = 194,
    ReadParam = 195,
    WriteParam = 196,
}

#[derive(PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum DeviceFunction {
    ReadHold = 3,
    ReadInput = 4,
    WriteSingle = 6,
    WriteMulti = 16,
}

#[derive(PartialEq)]
pub enum PacketType {
    Heartbeat,
    ReadHold,
    ReadInput1,
    ReadInput2,
    ReadInput3,
}

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

#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum RegisterBit {
    // Register 21
    AcChargeEnable = 1 << 7,
    ForcedDischargeEnable = 1 << 10,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub header: [u8; HEADER_LENGTH],
    pub data: Vec<u8>,
}

impl Packet {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut header = [0; HEADER_LENGTH];
        let data = vec![0; 16];

        header[0] = 161;
        header[1] = 26;
        header[6] = 1; // unsure, always seems to be 1

        let mut r = Self { header, data };

        r.set_protocol(1);

        // TODO: could do this in bytes()
        // header-6 + data + checksum?
        r.set_packet_length((14 + r.data.len() + 2) as u16);

        // TODO: update in bytes() ?
        r.set_data_length((r.data.len() + 2) as u8);

        r
    }

    pub fn bytes(&self) -> Vec<u8> {
        // header + data + checksum
        let len = HEADER_LENGTH + self.data.len() + 2;
        let mut r = Vec::with_capacity(len);

        r.extend_from_slice(&self.header);
        r.extend_from_slice(&self.data);
        r.extend_from_slice(&self.checksum());

        r
    }

    pub fn from_data(input: &[u8]) -> Result<Self> {
        if input[0..2] != [161, 26] {
            return Err(anyhow!("invalid packet"));
        }

        // used to think this was 20 bytes, but heartbeats only have 19?
        // so do all packets actually have a 19 byte header and always
        // have a nullbyte after?
        let mut header = [0; HEADER_LENGTH];
        header[0..19].copy_from_slice(&input[0..19]);

        let len = input.len();

        let data = if len > 19 {
            // header=19, null?byte, so data starts at 20.
            input[HEADER_LENGTH..len - 2].to_owned() // -2 to exclude checksum
        } else {
            // Heartbeat
            Vec::new()
        };

        let t = Self { header, data };

        if t.packet_type() != PacketType::Heartbeat {
            // last two bytes are checksum (but not for heartbeats)
            let checksum = &input[len - 2..];
            if t.checksum() != checksum {
                return Err(anyhow!(
                    "checksum mismatch - got {:?}, expected {:?}",
                    checksum,
                    t.checksum()
                ));
            }
        }

        Ok(t)
    }

    pub fn read_input1(&self) -> Result<ReadInput1> {
        match ReadInput1::parse(&self.values()) {
            Ok((_, mut r)) => {
                r.p_pv = r.p_pv_1 + r.p_pv_2 + r.p_pv_3;
                r.v_pv = r.v_pv_1 + r.v_pv_2 + r.v_pv_3;
                r.e_pv_day = r.e_pv_day_1 + r.e_pv_day_2 + r.e_pv_day_3;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    pub fn read_input2(&self) -> Result<ReadInput2> {
        match ReadInput2::parse(&self.values()) {
            Ok((_, mut r)) => {
                r.e_pv_all = r.e_pv_all_1 + r.e_pv_all_2 + r.e_pv_all_3;
                Ok(r)
            }
            Err(_) => Err(anyhow!("meh")),
        }
    }

    pub fn read_input3(&self) -> Result<ReadInput3> {
        match ReadInput3::parse(&self.values()) {
            Ok((_, r)) => Ok(r),
            Err(_) => Err(anyhow!("meh")),
        }
    }

    // Low-level Public Setters/Getters

    // HEADER

    pub fn packet_type(&self) -> PacketType {
        match self.tcp_function() {
            TcpFunction::Heartbeat => PacketType::Heartbeat,
            TcpFunction::TranslatedData => match self.device_function() {
                DeviceFunction::ReadHold => PacketType::ReadHold,
                DeviceFunction::ReadInput => match self.register() {
                    0 => PacketType::ReadInput1,
                    40 => PacketType::ReadInput2,
                    80 => PacketType::ReadInput3,
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn protocol(&self) -> u16 {
        Self::u16ify(&self.header, 2)
    }
    pub fn set_protocol(&mut self, protocol: u16) {
        self.header[2..4].copy_from_slice(&protocol.to_le_bytes())
    }

    #[allow(dead_code)]
    pub fn packet_length(&self) -> u16 {
        Self::u16ify(&self.header, 4)
    }
    pub fn set_packet_length(&mut self, packet_length: u16) {
        self.header[4..6].copy_from_slice(&packet_length.to_le_bytes())
    }

    pub fn tcp_function(&self) -> TcpFunction {
        TcpFunction::try_from(self.header[7]).unwrap()
    }
    pub fn set_tcp_function(&mut self, tcp_function: TcpFunction) {
        self.header[7] = tcp_function as u8
    }

    #[allow(dead_code)]
    pub fn datalog(&self) -> &str {
        std::str::from_utf8(&self.header[8..18]).unwrap()
    }
    pub fn set_datalog(&mut self, datalog: &str) {
        self.header[8..18].copy_from_slice(&datalog.as_bytes())
    }

    /* not quite sure if this is u8 or u16.
     *
     * heartbeats have a u8 zero in header[18], and stop there.
     * all other data packets I've seen have a zero in header[19].
     *
     * so what's 19 ever used for?
     */
    #[allow(dead_code)]
    pub fn data_length(&self) -> u8 {
        self.header[18]
    }
    pub fn set_data_length(&mut self, data_length: u8) {
        self.header[18] = data_length;
    }
    /*
    pub fn data_length(&self) -> u16 {
        Self::u16ify(&self.header, 18)
    }
    pub fn set_data_length(&mut self, data_length: u16) {
        self.header[18..20].copy_from_slice(&data_length.to_le_bytes())
    }
    */

    // DATA
    pub fn device_function(&self) -> DeviceFunction {
        DeviceFunction::try_from(self.data[1]).unwrap()
    }
    pub fn set_device_function(&mut self, device_function: DeviceFunction) {
        self.data[1] = device_function as u8
    }

    #[allow(dead_code)]
    pub fn serial(&self) -> &str {
        std::str::from_utf8(&self.data[2..12]).unwrap()
    }
    pub fn set_serial(&mut self, serial: &str) {
        self.data[2..12].copy_from_slice(&serial.as_bytes())
    }

    pub fn register(&self) -> u16 {
        Self::u16ify(&self.data, 12)
    }
    pub fn set_register(&mut self, register: u16) {
        self.data[12..14].copy_from_slice(&register.to_le_bytes())
    }

    pub fn has_value_length(&self) -> bool {
        self.protocol() == 2 && self.device_function() != DeviceFunction::WriteSingle
    }

    pub fn value_length(&self) -> u8 {
        if self.has_value_length() {
            //u16::from_le_bytes([self.data[13], self.data[14]])
            self.data[14]
        } else {
            2
        }
    }

    // value as u16, usually from ReadSingle
    pub fn value(&self) -> u16 {
        Self::u16ify(self.values(), 0)
    }
    pub fn set_value(&mut self, value: u16) {
        // TODO: only works for protocol 1!
        self.data[14..16].copy_from_slice(&value.to_le_bytes())
    }

    // slice of unprocessed u8 values
    pub fn values(&self) -> &[u8] {
        if self.has_value_length() {
            // protocol 2 normally has length at 14, then that many bytes of values
            let value_length = self.value_length() as usize;
            &self.data[15..15 + value_length]
        } else {
            // protocol 1 has value at 14 and 15
            &self.data[14..16]
        }
    }

    // Vec of register/value pairs in this packet
    pub fn pairs(&self) -> Vec<Pair> {
        self.values()
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| Pair {
                register: self.register() + pos as u16,
                value: Self::u16ify(value, 0),
            })
            .collect()
    }

    // Private
    fn checksum(&self) -> [u8; 2] {
        crc16::State::<crc16::MODBUS>::calculate(&self.data).to_le_bytes()
    }

    fn u16ify(array: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes([array[offset], array[offset + 1]])
    }
    /*
    fn u32ify(array: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([
            array[offset],
            array[offset + 1],
            array[offset + 2],
            array[offset + 3],
        ])
    }
    */
}
