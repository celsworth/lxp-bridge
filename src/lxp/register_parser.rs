use crate::prelude::*;
use serde::Serialize;

pub type ParsedData = HashMap<&'static str, Value>;

#[derive(PartialEq, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Value {
    Integer(i64),
    Float(f64),
    // for strings (status, fault_code, warning_code),
    // we also return the raw i64 value for inserting into InfluxDB.
    // time registers are strings too but we don't need it there so just use 0 for now
    String(i64, &'static str),
    StringOwned(i64, String),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::String(_, s) => s.to_string(),
            Self::StringOwned(_, s) => s.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
struct StartEndTimePayload {
    start: String,
    end: String,
}
#[derive(Debug, Clone)]
pub struct Parser {
    registers: HashMap<u16, u16>,
}

impl Parser {
    pub fn new(registers: HashMap<u16, u16>) -> Self {
        Self { registers }
    }

    // this bodge is to support sending inputs/1 inputs/2 etc with the correct keys in.
    // we look at which registers are present and make a guess ;)
    pub fn guess_legacy_inputs_topic(&self) -> Option<&'static str> {
        let has_register_0 = self.contains_register(0);
        let has_register_40 = self.contains_register(40);
        let has_register_80 = self.contains_register(80);
        let has_register_120 = self.contains_register(120);

        if has_register_0 && has_register_40 && has_register_80 {
            // TODO: this should cope with 120 being present too?
            // but that may need to be configurable (default off)
            Some("all")
        } else if has_register_0 && !has_register_40 && !has_register_80 && !has_register_120 {
            Some("1")
        } else if !has_register_0 && has_register_40 && !has_register_80 && !has_register_120 {
            Some("2")
        } else if !has_register_0 && !has_register_40 && has_register_80 && !has_register_120 {
            Some("3")
        } else if !has_register_0 && !has_register_40 && !has_register_80 && has_register_120 {
            Some("4")
        } else {
            None
        }
    }

    pub fn contains_register(&self, register: u16) -> bool {
        self.v_for(register).is_ok()
    }

    // given a set of raw input registers from td.registers(), decode what we can
    // and return a list of Key -> Value
    //
    // this will Err if you pass it a subset of registers it doesn't expect, for example
    // if you pass in registers that contain 0-7, then it will try to work out p_pv because it
    // has seen 7. but p_pv requires 7 + 8 + 9, which aren't present.
    //
    // I think this is fine because generally we get these registers in lumps of 40, ie 0-39
    // etc. It would be unusual to get a single input register.
    //
    pub fn parse_inputs(&self) -> Result<ParsedData> {
        let mut ret = HashMap::new();

        for (r, v) in self.registers.clone() {
            let e = match r {
                0 => vec![("status", self.parse_status(v))],
                1 => vec![("v_pv_1", self.parse_f64_1(v, 10))],
                2 => vec![("v_pv_2", self.parse_f64_1(v, 10))],
                3 => vec![("v_pv_3", self.parse_f64_1(v, 10))],
                4 => vec![("v_bat", self.parse_f64_1(v, 10))],
                5 => vec![("soc", self.parse_i64_l(v)), ("soh", self.parse_i64_h(v))],
                6 => vec![], // reserved
                7 => vec![("p_pv_1", self.parse_i64_1(v)), ("p_pv", self.p_pv()?)],
                8 => vec![("p_pv_2", self.parse_i64_1(v))],
                9 => vec![("p_pv_3", self.parse_i64_1(v))],
                10 => vec![
                    ("p_charge", self.parse_i64_1(v)),
                    ("p_battery", self.p_battery()?), // homebrew net power flow field
                ],
                11 => vec![("p_discharge", self.parse_i64_1(v))],
                12 => vec![("v_ac_r", self.parse_f64_1(v, 10))],
                13 => vec![("v_ac_s", self.parse_f64_1(v, 10))],
                14 => vec![("v_ac_t", self.parse_f64_1(v, 10))],
                15 => vec![("f_ac", self.parse_f64_1(v, 100))],
                16 => vec![("p_inv", self.parse_i64_1(v))],
                17 => vec![("p_rec", self.parse_i64_1(v))],
                18 => vec![], // IinvRMS, 0.01A
                19 => vec![("pf", self.parse_f64_1(v, 1000))],
                20 => vec![("v_eps_r", self.parse_f64_1(v, 10))],
                21 => vec![("v_eps_s", self.parse_f64_1(v, 10))],
                22 => vec![("v_eps_t", self.parse_f64_1(v, 10))],
                23 => vec![("f_eps", self.parse_f64_1(v, 100))],
                24 => vec![("p_eps", self.parse_i64_1(v))],
                25 => vec![("s_eps", self.parse_i64_1(v))],
                26 => vec![
                    ("p_to_grid", self.parse_i64_1(v)),
                    ("p_grid", self.p_grid()?),
                ],
                27 => vec![("p_to_user", self.parse_i64_1(v))],
                28 => vec![("e_pv_day_1", self.parse_f64_1(v, 10))],
                29 => vec![("e_pv_day_2", self.parse_f64_1(v, 10))],
                30 => vec![("e_pv_day_3", self.parse_f64_1(v, 10))],
                31 => vec![("e_inv_day", self.parse_f64_1(v, 10))],
                32 => vec![("e_rec_day", self.parse_f64_1(v, 10))],
                33 => vec![("e_chg_day", self.parse_f64_1(v, 10))],
                34 => vec![("e_dischg_day", self.parse_f64_1(v, 10))],
                35 => vec![("e_eps_day", self.parse_f64_1(v, 10))],
                36 => vec![("e_to_grid_day", self.parse_f64_1(v, 10))],
                37 => vec![("e_to_user_day", self.parse_f64_1(v, 10))],
                38 => vec![("v_bus_1", self.parse_f64_1(v, 10))],
                39 => vec![("v_bus_2", self.parse_f64_1(v, 10))],

                40 => vec![("e_pv_all_1", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                41 => vec![], // done in 40
                42 => vec![("e_pv_all_2", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                43 => vec![], // done in 42
                44 => vec![("e_pv_all_3", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                45 => vec![], // done in 44
                46 => vec![("e_inv_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                47 => vec![], // done in 46
                48 => vec![("e_rec_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                49 => vec![], // done in 48
                50 => vec![("e_chg_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                51 => vec![], // done in 40
                52 => vec![("e_dischg_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                53 => vec![], // done in 52
                54 => vec![("e_eps_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                55 => vec![], // done in 54
                56 => vec![("e_to_grid_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                57 => vec![], // done in 56
                58 => vec![("e_to_user_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                59 => vec![], // done in 58
                60 => vec![("fault_code", self.parse_fault(v, self.v_for(r + 1)?))],
                61 => vec![], // done in 60
                62 => vec![("warning_code", self.parse_warning(v, self.v_for(r + 1)?))],
                63 => vec![], // done in 62
                64 => vec![("t_inner", self.parse_i64_1(v))],
                65 => vec![("t_rad_1", self.parse_i64_1(v))],
                66 => vec![("t_rad_2", self.parse_i64_1(v))],
                67 => vec![("t_bat", self.parse_i64_1(v))],
                68 => vec![], // reserved
                69 => vec![("runtime", self.parse_i64_2(v, self.v_for(70)?))],
                70 => vec![],      // done in 69
                71..=79 => vec![], // TODO, rest of ReadInput2

                80 => vec![], // bat_brand & bat_com_type
                81 => vec![("max_chg_curr", self.parse_f64_1(v, 100))],
                82 => vec![("max_dischg_curr", self.parse_f64_1(v, 100))],
                83 => vec![("charge_volt_ref", self.parse_f64_1(v, 10))],
                84 => vec![("dischg_cut_volt", self.parse_f64_1(v, 10))],
                85..=95 => vec![], // bat_status_*, not yet parsed
                96 => vec![("bat_count", self.parse_i64_1(v))],
                97 => vec![("bat_capacity", self.parse_i64_1(v))],
                98 => vec![("bat_current", self.parse_f64_1(v, 100))],
                99 => vec![],  // bms_event_1
                100 => vec![], // bms_event_2
                101 => vec![("max_cell_voltage", self.parse_f64_1(v, 1000))],
                102 => vec![("min_cell_voltage", self.parse_f64_1(v, 1000))],
                103 => vec![("max_cell_temp", self.parse_f64_1(v, 10))],
                104 => vec![("min_cell_temp", self.parse_f64_1(v, 10))],
                105 => vec![], // bms_fw_update_state
                106 => vec![("cycle_count", self.parse_i64_1(v))],
                107 => vec![("vbat_inv", self.parse_f64_1(v, 10))],
                108..=119 => vec![], // TODO, rest of ReadInput3

                120 => vec![], // half bus voltage?
                121 => vec![("v_gen", self.parse_f64_1(v, 10))],
                122 => vec![("f_gen", self.parse_f64_1(v, 100))],
                123 => vec![("p_gen", self.parse_i64_1(v))],
                124 => vec![("e_gen_day", self.parse_f64_1(v, 10))],
                125 => vec![("e_gen_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                126 => vec![], // done in 125
                127 => vec![("v_eps_l1", self.parse_f64_1(v, 10))],
                128 => vec![("v_eps_l2", self.parse_f64_1(v, 10))],
                129 => vec![("p_eps_l1", self.parse_i64_1(v))],
                130 => vec![("p_eps_l2", self.parse_i64_1(v))],
                131 => vec![("s_eps_l1", self.parse_i64_1(v))],
                132 => vec![("s_eps_l2", self.parse_i64_1(v))],
                133 => vec![("e_eps_l1_day", self.parse_f64_1(v, 10))],
                134 => vec![("e_eps_l2_day", self.parse_f64_1(v, 10))],
                135 => vec![("e_eps_l1_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                136 => vec![], // done in 135
                137 => vec![("e_eps_l2_all", self.parse_f64_2(v, self.v_for(r + 1)?, 10))],
                138 => vec![], // done in 137

                ..=255 => vec![], // ignore everything else for now

                _ => bail!("unhandled input register {}", r),
            };

            ret.extend(e);
        }

        // debug!("{:?}", ret);

        Ok(ret)
    }
    // given a set of raw hold registers from td.registers(), decode what we can
    // and return a list of Key -> Value
    //
    // unlike parse_inputs, this one does not Err if passed unknown registers. We just silently
    // return an empty array. This is because Hold registers are far more likely to be sent
    // to us individually, ie a single ReadHold command. Also most hold registers aren't parsed
    // and we don't care if we're passed some that aren't implemented.
    pub fn parse_holds(&self) -> Result<ParsedData> {
        let mut ret = HashMap::new();

        for (r, v) in self.registers.clone() {
            let e = match r {
                // only place single-register matches in here. If any are missing (which
                // is much more likely in ReadHold messages), multi-register operations
                // will break!
                //
                // these should have hold/ prefixed if appropriate!
                //
                21 => vec![("hold/21/bits", self.parse_21_bits(v)?)],
                110 => vec![("hold/110/bits", self.parse_110_bits(v)?)],

                // ignore any unknown registers, do not return Err
                _ => vec![],
            };
            ret.extend(e);
        }

        /* self.start_end_tuple hides away a lot of logic to check if the registers
         * we pass in are present, and if so, return a Vec suitable for putting into our
         * returned HashMap. If any are missing, it does nothing (no error)
         * */
        ret.extend(self.start_end_tuple("ac_charge/1", [70, 71]));
        ret.extend(self.start_end_tuple("ac_charge/2", [72, 73]));
        ret.extend(self.start_end_tuple("ac_charge/3", [74, 75]));

        ret.extend(self.start_end_tuple("charge_priority/1", [76, 77]));
        ret.extend(self.start_end_tuple("charge_priority/2", [78, 79]));
        ret.extend(self.start_end_tuple("charge_priority/3", [80, 81]));

        ret.extend(self.start_end_tuple("forced_discharge/1", [84, 85]));
        ret.extend(self.start_end_tuple("forced_discharge/2", [86, 87]));
        ret.extend(self.start_end_tuple("forced_discharge/3", [88, 89]));

        ret.extend(self.start_end_tuple("ac_first/1", [152, 153]));
        ret.extend(self.start_end_tuple("ac_first/2", [154, 155]));
        ret.extend(self.start_end_tuple("ac_first/3", [156, 157]));

        // debug!("{:?}", ret);

        Ok(ret)
    }

    fn start_end_tuple(
        &self,
        key: &'static str,
        registers: [u16; 2],
    ) -> Vec<(&'static str, Value)> {
        if self.all_registers_present(&registers) {
            vec![(key, self.start_end(registers[0], registers[1]).unwrap())]
        } else {
            vec![]
        }
    }

    fn start_end(&self, r1: u16, r2: u16) -> Result<Value> {
        let start = self.v_for(r1)?.to_le_bytes();
        let end = self.v_for(r2)?.to_le_bytes();

        let payload = StartEndTimePayload {
            start: format!("{:02}:{:02}", start[0], start[1]),
            end: format!("{:02}:{:02}", end[0], end[1]),
        };

        Ok(Value::StringOwned(
            0, // raw value unused, holds are not inserted to Influx anyway
            serde_json::to_string(&payload)?,
        ))
    }

    fn parse_21_bits(&self, value: u16) -> Result<Value> {
        let bits = lxp::packet::Register21Bits::new(value);

        Ok(Value::StringOwned(
            value as i64,
            serde_json::to_string(&bits)?,
        ))
    }

    fn parse_110_bits(&self, value: u16) -> Result<Value> {
        let bits = lxp::packet::Register110Bits::new(value);

        Ok(Value::StringOwned(
            value as i64,
            serde_json::to_string(&bits)?,
        ))
    }

    fn p_pv(&self) -> Result<Value> {
        let p_pv_1 = self.v_for(7)? as i64;
        let p_pv_2 = self.v_for(8)? as i64;
        let p_pv_3 = self.v_for(9)? as i64;

        Ok(Value::Integer(p_pv_1 + p_pv_2 + p_pv_3))
    }

    fn p_battery(&self) -> Result<Value> {
        // special case - use p_charge and p_discharge to return a signed net power flow
        let p_charge = self.v_for(10)? as i64;
        let p_discharge = self.v_for(11)? as i64;

        Ok(Value::Integer(p_charge - p_discharge))
    }

    fn p_grid(&self) -> Result<Value> {
        // special case - use p_charge and p_discharge to return a signed net power flow
        let p_to_grid = self.v_for(26)? as i64;
        let p_to_user = self.v_for(27)? as i64;

        Ok(Value::Integer(p_to_user - p_to_grid))
    }

    fn parse_status(&self, value: u16) -> Value {
        Value::String(value as i64, StatusString::from_value(value))
    }

    fn parse_fault(&self, v1: u16, v2: u16) -> Value {
        let value: i64 = (v1 as i64) | (v2 as i64) << 16;
        Value::String(value, FaultCodeString::from_value(value))
    }

    fn parse_warning(&self, v1: u16, v2: u16) -> Value {
        let value: i64 = (v1 as i64) | (v2 as i64) << 16;
        Value::String(value, WarningCodeString::from_value(value))
    }

    // one register input, using low half, i64 output
    fn parse_i64_l(&self, v1: u16) -> Value {
        let r: i64 = (v1 & 0xff) as i64;
        Value::Integer(r)
    }

    // one register input, using high half, i64 output
    fn parse_i64_h(&self, v1: u16) -> Value {
        let r: i64 = (v1 >> 8) as i64;
        Value::Integer(r)
    }

    // one register input, i64 output
    fn parse_i64_1(&self, v1: u16) -> Value {
        let r: i64 = v1 as i64;
        Value::Integer(r)
    }

    // two register input, i64 output
    fn parse_i64_2(&self, v1: u16, v2: u16) -> Value {
        let r: i64 = (v1 as i64) | (v2 as i64) << 16;
        Value::Integer(r)
    }

    // one register input, f64 output
    fn parse_f64_1(&self, v1: u16, divider: i64) -> Value {
        let r: i64 = v1 as i64;
        Value::Float(r as f64 / divider as f64)
    }

    // two register input, f64 output
    fn parse_f64_2(&self, v1: u16, v2: u16, divider: i64) -> Value {
        let r: i64 = (v1 as i64) | (v2 as i64) << 16;
        Value::Float(r as f64 / divider as f64)
    }

    // get the value for a given register, or Err
    fn v_for(&self, register: u16) -> Result<u16> {
        self.registers
            .get(&register)
            .ok_or(anyhow!("no value found for register {}", register))
            .cloned()
    }

    // return true if we have a value for ALL the registers requested
    fn all_registers_present(&self, registers: &[u16]) -> bool {
        registers
            .into_iter()
            .all(|register| self.registers.contains_key(register))
    }
}

struct StatusString;
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

struct WarningCodeString;
impl WarningCodeString {
    pub fn from_value(value: i64) -> &'static str {
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

struct FaultCodeString;
impl FaultCodeString {
    pub fn from_value(value: i64) -> &'static str {
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
