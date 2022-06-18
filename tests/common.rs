pub use lxp_bridge::prelude::*;

pub use {mockito::*, serde_json::json};

pub fn common_setup() {
    let _ = env_logger::try_init();
}

pub fn example_config() -> config::Config {
    Config::new("config.yaml.example".to_owned()).unwrap()
}

pub fn example_serial() -> lxp::inverter::Serial {
    lxp::inverter::Serial::from_str("TESTSERIAL").unwrap()
}

pub fn sender<T: Clone>() -> broadcast::Sender<T> {
    broadcast::channel(512).0
}
