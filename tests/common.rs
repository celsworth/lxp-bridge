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
}

pub fn common_setup() {
    let _ = env_logger::try_init();
}

pub fn sender<T: Clone>() -> broadcast::Sender<T> {
    broadcast::channel(512).0
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
