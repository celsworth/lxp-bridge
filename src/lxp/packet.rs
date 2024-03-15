use crate::prelude::*;

use enum_dispatch::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

// {{{ TcpFunction
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum TcpFunction {
    Heartbeat = 193,
    TranslatedData = 194,
    ReadParam = 195,
    WriteParam = 196,
} // }}}

// {{{ DeviceFunction
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Register {
    Register21 = 21,             // not sure of a better name for this one..
    ChargePowerPercentCmd = 64,  // System Charge Rate (%)
    DischgPowerPercentCmd = 65,  // System Discharge Rate (%)
    AcChargePowerCmd = 66,       // Grid Charge Power Rate (%)
    AcChargeSocLimit = 67,       // AC Charge SOC Limit (%)
    ChargePriorityPowerCmd = 74, // Charge Priority Charge Rate (%)
    ChargePrioritySocLimit = 75, // Charge Priority SOC Limit (%)
    ForcedDischgSocLimit = 83,   // Forced Discharge SOC Limit (%)
    DischgCutOffSocEod = 105,    // Discharge cut-off SOC (%)
    EpsDischgCutoffSocEod = 125, // EPS Discharge cut-off SOC (%)
    AcChargeStartSocLimit = 160, // SOC at which AC charging will begin (%)
    AcChargeEndSocLimit = 161,   // SOC at which AC charging will end (%)
}

#[derive(Clone, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum RegisterBit {
    // Register 21
    AcChargeEnable = 1 << 7,
    ForcedDischargeEnable = 1 << 10,
    ChargePriorityEnable = 1 << 11,
}

// Register21Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register21Bits {
    pub eps_en: String,
    pub ovf_load_derate_en: String,
    pub drms_en: String,
    pub lvrt_en: String,
    pub anti_island_en: String,
    pub neutral_detect_en: String,
    pub grid_on_power_ss_en: String,
    pub ac_charge_en: String,
    pub sw_seamless_en: String,
    pub set_to_standby: String,
    pub forced_discharge_en: String,
    pub charge_priority_en: String,
    pub iso_en: String,
    pub gfci_en: String,
    pub dci_en: String,
    pub feed_in_grid_en: String,
}

impl Register21Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            eps_en: Self::is_bit_set(data, 1 << 0),
            ovf_load_derate_en: Self::is_bit_set(data, 1 << 1),
            drms_en: Self::is_bit_set(data, 1 << 2),
            lvrt_en: Self::is_bit_set(data, 1 << 3),
            anti_island_en: Self::is_bit_set(data, 1 << 4),
            neutral_detect_en: Self::is_bit_set(data, 1 << 5),
            grid_on_power_ss_en: Self::is_bit_set(data, 1 << 6),
            ac_charge_en: Self::is_bit_set(data, 1 << 7),
            sw_seamless_en: Self::is_bit_set(data, 1 << 8),
            set_to_standby: Self::is_bit_set(data, 1 << 9),
            forced_discharge_en: Self::is_bit_set(data, 1 << 10),
            charge_priority_en: Self::is_bit_set(data, 1 << 11),
            iso_en: Self::is_bit_set(data, 1 << 12),
            gfci_en: Self::is_bit_set(data, 1 << 13),
            dci_en: Self::is_bit_set(data, 1 << 14),
            feed_in_grid_en: Self::is_bit_set(data, 1 << 15),
        }
    }
} // }}}

// Register110Bits {{{
#[derive(Clone, Debug, Serialize)]
pub struct Register110Bits {
    pub ub_pv_grid_off_en: String,
    pub ub_run_without_grid: String,
    pub ub_micro_grid_en: String,
}
impl Register110Bits {
    fn is_bit_set(data: u16, bit: u16) -> String {
        if (data & bit) == bit {
            "ON".to_string()
        } else {
            "OFF".to_string()
        }
    }

    pub fn new(data: u16) -> Self {
        Self {
            ub_pv_grid_off_en: Self::is_bit_set(data, 1 << 0),
            ub_run_without_grid: Self::is_bit_set(data, 1 << 1),
            ub_micro_grid_en: Self::is_bit_set(data, 1 << 2),
        }
    }
} // }}}

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
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Packet {
    Heartbeat(Heartbeat),
    TranslatedData(TranslatedData),
    ReadParam(ReadParam),
    WriteParam(WriteParam),
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

#[derive(Eq, PartialEq, Clone, Debug)]
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
        vec![0]
    }
}

/////////////
//
// TRANSLATED DATA
//
/////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct TranslatedData {
    pub datalog: Serial,
    pub device_function: DeviceFunction, // ReadHold or ReadInput etc..
    pub inverter: Serial,                // inverter serial
    pub register: u16,                   // first register of values
    pub values: Vec<u8>,                 // undecoded, since can be u16 or u32s?
}
impl TranslatedData {
    pub fn pairs(&self) -> HashMap<u16, u16> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 38 {
            bail!("TranslatedData::decode packet too short");
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

        // experimental: looks like maybe you don't need to fill this in..
        data[4..14].copy_from_slice(&self.inverter.data());
        //data[4..14].copy_from_slice(&[0; 10]);

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

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ReadParam {
    pub datalog: Serial,
    pub register: u16,   // first register of values
    pub values: Vec<u8>, // undecoded, since can be u16 or i32s?
}
impl ReadParam {
    pub fn pairs(&self) -> HashMap<u16, u16> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 24 {
            bail!("ReadParam::decode packet too short");
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
        vec![self.register() as u8, 0]
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
// WRITE PARAM
//
/////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct WriteParam {
    pub datalog: Serial,
    pub register: u16,   // first register of values
    pub values: Vec<u8>, // undecoded, since can be u16 or i32s?
}
impl WriteParam {
    pub fn pairs(&self) -> HashMap<u16, u16> {
        self.values
            .chunks(2)
            .enumerate()
            .map(|(pos, value)| (self.register + pos as u16, Utils::u16ify(value, 0)))
            .collect()
    }

    fn decode(input: &[u8]) -> Result<Self> {
        let len = input.len();
        if len < 21 {
            bail!("WriteParam::decode packet too short");
        }

        let protocol = Utils::u16ify(input, 2);
        let datalog = Serial::new(&input[8..18])?;

        let data = &input[18..];
        let register = u16::from(data[0]);

        let mut value_len = 2;
        let mut value_offset = 1;

        if Self::has_value_length_bytes(protocol) {
            value_len = Utils::u16ify(data, value_offset) as usize;
            value_offset += 2;
        }

        let values = data[value_offset..].to_vec();

        if values.len() != value_len {
            bail!(
                "WriteParam::decode mismatch: values.len()={}, value_length_byte={}",
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

    fn has_value_length_bytes(_protocol: u16) -> bool {
        false
    }
}

impl PacketCommon for WriteParam {
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
        TcpFunction::WriteParam
    }

    fn bytes(&self) -> Vec<u8> {
        let mut data = vec![0; 2];

        data[0..2].copy_from_slice(&self.register.to_le_bytes());

        let len = self.values.len() as u16;
        data.extend_from_slice(&len.to_le_bytes());

        let mut m = Vec::new();
        for i in &self.values {
            m.extend_from_slice(&i.to_le_bytes());
        }
        data.append(&mut m);

        data
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

        let r = match TcpFunction::try_from(input[7])? {
            TcpFunction::Heartbeat => Packet::Heartbeat(Heartbeat::decode(input)?),
            TcpFunction::TranslatedData => Packet::TranslatedData(TranslatedData::decode(input)?),
            TcpFunction::ReadParam => Packet::ReadParam(ReadParam::decode(input)?),
            TcpFunction::WriteParam => Packet::WriteParam(WriteParam::decode(input)?),
            //_ => bail!("unhandled: tcp_function={} input={:?}", input[7], input),
        };

        Ok(r)
    }
}

pub struct StatusString;
impl StatusString {
    pub fn from_value(status: u16) -> &'static str {
        match status {
            0x00 => "Standby",
            0x02 => "FW Updating",
            0x04 => "PV On-grid",
            0x08 => "PV Charge",
            0x0C => "PV Charge On-grid",
            0x10 => "Battery On-grid",
            0x11 => "Bypass",
            0x14 => "PV & Battery On-grid",
            0x19 => "PV Charge + Bypass",
            0x20 => "AC Charge",
            0x28 => "PV & AC Charge",
            0x40 => "Battery Off-grid",
            0x80 => "PV Off-grid",
            0xC0 => "PV & Battery Off-grid",
            0x88 => "PV Charge Off-grid",

            _ => "Unknown",
        }
    }
}

pub struct WarningCodeString;
impl WarningCodeString {
    pub fn from_value(value: u32) -> &'static str {
        if value == 0 {
            return "OK";
        }

        (0..=31)
            .find(|i| value & (1 << i) > 0)
            .map(Self::from_bit)
            .unwrap()
    }

    fn from_bit(bit: usize) -> &'static str {
        match bit {
            0 => "W000: Battery communication failure",
            1 => "W001: AFCI communication failure",
            2 => "W002: AFCI high",
            3 => "W003: Meter communication failure",
            4 => "W004: Both charge and discharge forbidden by battery",
            5 => "W005: Auto test failed",
            6 => "W006: Reserved",
            7 => "W007: LCD communication failure",
            8 => "W008: FW version mismatch",
            9 => "W009: Fan stuck",
            10 => "W010: Reserved",
            11 => "W011: Parallel number out of range",
            12 => "W012: Bat On Mos",
            13 => "W013: Overtemperature (NTC reading is too high)",
            14 => "W014: Reserved",
            15 => "W015: Battery reverse connection",
            16 => "W016: Grid power outage",
            17 => "W017: Grid voltage out of range",
            18 => "W018: Grid frequency out of range",
            19 => "W019: Reserved",
            20 => "W020: PV insulation low",
            21 => "W021: Leakage current high",
            22 => "W022: DCI high",
            23 => "W023: PV short",
            24 => "W024: Reserved",
            25 => "W025: Battery voltage high",
            26 => "W026: Battery voltage low",
            27 => "W027: Battery open circuit",
            28 => "W028: EPS overload",
            29 => "W029: EPS voltage high",
            30 => "W030: Meter reverse connection",
            31 => "W031: DCV high",

            _ => todo!("Unknown Warning"),
        }
    }
}

pub struct FaultCodeString;
impl FaultCodeString {
    pub fn from_value(value: u32) -> &'static str {
        if value == 0 {
            return "OK";
        }

        (0..=31)
            .find(|i| value & (1 << i) > 0)
            .map(Self::from_bit)
            .unwrap()
    }

    fn from_bit(bit: usize) -> &'static str {
        match bit {
            0 => "E000: Internal communication fault 1",
            1 => "E001: Model fault",
            2 => "E002: BatOnMosFail",
            3 => "E003: CT Fail",
            4 => "E004: Reserved",
            5 => "E005: Reserved",
            6 => "E006: Reserved",
            7 => "E007: Reserved",
            8 => "E008: CAN communication error in parallel system",
            9 => "E009: master lost in parallel system",
            10 => "E010: multiple master units in parallel system",
            11 => "E011: AC input inconsistent in parallel system",
            12 => "E012: UPS short",
            13 => "E013: Reverse current on UPS output",
            14 => "E014: Bus short",
            15 => "E015: Phase error in three phase system",
            16 => "E016: Relay check fault",
            17 => "E017: Internal communication fault 2",
            18 => "E018: Internal communication fault 3",
            19 => "E019: Bus voltage high",
            20 => "E020: EPS connection fault",
            21 => "E021: PV voltage high",
            22 => "E022: Over current protection",
            23 => "E023: Neutral fault",
            24 => "E024: PV short",
            25 => "E025: Radiator temperature over range",
            26 => "E026: Internal fault",
            27 => "E027: Sample inconsistent between Main CPU and redundant CPU",
            28 => "E028: Reserved",
            29 => "E029: Reserved",
            30 => "E030: Reserved",
            31 => "E031: Internal communication fault 4",
            _ => todo!("Unknown Fault"),
        }
    }
}
