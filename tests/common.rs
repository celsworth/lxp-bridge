#![allow(dead_code)]

pub use lxp_bridge::prelude::*;

pub use {mockito::*, serde_json::json};

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

pub fn unwrap_inverter_channeldata_packet(
    packet: lxp::inverter::ChannelData,
) -> lxp::packet::Packet {
    if let lxp::inverter::ChannelData::Packet(packet) = packet {
        packet
    } else {
        todo!()
    }
}
