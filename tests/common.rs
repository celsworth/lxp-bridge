#![allow(dead_code)]

pub use lxp_bridge::prelude::*;

pub use {crate::broadcast::error::TryRecvError, mockito::*, serde_json::json};

pub struct Factory();
impl Factory {
    pub fn example_config() -> config::Config {
        Config::new("config.yaml.example".to_owned()).unwrap()
    }

    pub fn inverter() -> config::Inverter {
        config::Inverter {
            enabled: true,
            port: 8000,
            host: "localhost".to_owned(),
            datalog: Serial::from_str("2222222222").unwrap(),
            serial: Serial::from_str("5555555555").unwrap(),
        }
    }

    pub fn read_input_all() -> lxp::packet::ReadInputAll {
        lxp::packet::ReadInputAll {
            status: 16,
            v_pv: 0.0,
            v_pv_1: 0.0,
            v_pv_2: 0.0,
            v_pv_3: 0.0,
            v_bat: 49.2,
            soc: 50,
            soh: 0,
            p_pv: 0,
            p_pv_1: 0,
            p_pv_2: 0,
            p_pv_3: 0,
            p_charge: 0,
            p_discharge: 507,
            v_ac_r: 245.5,
            v_ac_s: 409.6,
            v_ac_t: 0.0,
            f_ac: 50.1,
            p_inv: 456,
            p_rec: 0,
            pf: 1.0,
            v_eps_r: 245.5,
            v_eps_s: 256.0,
            v_eps_t: 2875.2,
            f_eps: 50.1,
            p_to_grid: 0,
            p_to_user: 0,
            e_pv_day: 0.0,
            e_pv_day_1: 0.0,
            e_pv_day_2: 0.0,
            e_pv_day_3: 0.0,
            e_inv_day: 6.6,
            e_rec_day: 0.0,
            e_chg_day: 0.0,
            e_dischg_day: 8.6,
            e_eps_day: 0.0,
            e_to_grid_day: 0.1,
            e_to_user_day: 1.0,
            v_bus_1: 371.7,
            v_bus_2: 296.8,
            e_pv_all: 4215.8,
            e_pv_all_1: 4215.8,
            e_pv_all_2: 0.0,
            e_pv_all_3: 0.0,
            e_inv_all: 3226.0,
            e_rec_all: 3891.0,
            e_chg_all: 4363.3,
            e_dischg_all: 4063.4,
            e_eps_all: 0.0,
            e_to_grid_all: 979.0,
            e_to_user_all: 5850.6,
            t_inner: 48,
            t_rad_1: 35,
            t_rad_2: 35,
            t_bat: 0,
            runtime: 67344843,
            max_chg_curr: 150.0,
            max_dischg_curr: 150.0,
            charge_volt_ref: 53.2,
            dischg_cut_volt: 40.0,
            bat_status_0: 0,
            bat_status_1: 0,
            bat_status_2: 0,
            bat_status_3: 0,
            bat_status_4: 0,
            bat_status_5: 192,
            bat_status_6: 0,
            bat_status_7: 0,
            bat_status_8: 0,
            bat_status_9: 0,
            bat_status_inv: 3,
            bat_count: 6,
            bat_capacity: 0,
            bat_current: 0.0,
            bms_event_1: 0,
            bms_event_2: 0,
            max_cell_voltage: 0.0,
            min_cell_voltage: 0.0,
            max_cell_temp: 0.0,
            min_cell_temp: 0.0,
            bms_fw_update_state: 0,
            cycle_count: 0,
            vbat_inv: 0.0,
            time: UnixTime::now(),
            datalog: Serial::from_str("1234567890").unwrap(),
        }
    }
}

pub fn common_setup() {
    let _ = env_logger::try_init();
}

pub fn unwrap_inverter_channeldata_packet(i: lxp::inverter::ChannelData) -> lxp::packet::Packet {
    if let lxp::inverter::ChannelData::Packet(i) = i {
        return i;
    }
    panic!()
}

pub fn unwrap_influx_channeldata_input_data(i: influx::ChannelData) -> serde_json::Value {
    if let influx::ChannelData::InputData(i) = i {
        return i;
    }
    panic!()
}

pub fn unwrap_database_channeldata_read_input_all(
    i: database::ChannelData,
) -> lxp::packet::ReadInputAll {
    if let database::ChannelData::ReadInputAll(i) = i {
        return *i;
    }
    panic!()
}

pub fn type_of<T>(_: &T) -> &str {
    std::any::type_name::<T>()
}
