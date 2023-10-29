use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Channels {
    pub from_inverter: broadcast::Sender<lxp::inverter::ChannelData>,
    pub to_inverter: broadcast::Sender<lxp::inverter::ChannelData>,
    pub from_mqtt: broadcast::Sender<mqtt::ChannelData>,
    pub to_mqtt: broadcast::Sender<mqtt::ChannelData>,
    pub to_influx: broadcast::Sender<influx::ChannelData>,
    pub to_database: broadcast::Sender<database::ChannelData>,
    pub read_register_cache: broadcast::Sender<register_cache::ChannelData>,
    pub to_register_cache: broadcast::Sender<register_cache::ChannelData>,
}

impl Default for Channels {
    fn default() -> Self {
        Self::new()
    }
}

impl Channels {
    pub fn new() -> Self {
        Self {
            from_inverter: Self::channel(),
            to_inverter: Self::channel(),
            from_mqtt: Self::channel(),
            to_mqtt: Self::channel(),
            to_influx: Self::channel(),
            to_database: Self::channel(),
            read_register_cache: Self::channel(),
            to_register_cache: Self::channel(),
        }
    }

    fn channel<T: Clone>() -> broadcast::Sender<T> {
        broadcast::channel(2048).0 // we only need tx half
    }
}
