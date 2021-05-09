pub mod command;
pub mod config;
pub mod coordinator;
pub mod lxp;
pub mod mqtt;
pub mod options;
pub mod prelude;

use crate::prelude::*;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = app();

    if let Err(err) = rt.block_on(future) {
        error!("{:?}", err);
        std::process::exit(255);
    }
}

async fn app() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format_timestamp_millis()
        .write_style(env_logger::WriteStyle::Never)
        .init();

    let options = Options::new()?;

    let config = Rc::new(Config::new(options.config_file)?);

    /*
    let mut packet = Packet::new();
    packet.set_tcp_function(TcpFunction::TranslatedData);
    packet.set_device_function(DeviceFunction::ReadHold);
    packet.set_datalog(&config.inverter.datalog);
    packet.set_serial(&config.inverter.serial);
    packet.set_register(105);
    packet.set_value(1);
    */

    let coordinator = Coordinator::new(Rc::clone(&config));

    futures::try_join!(
        coordinator.start(),
        coordinator.inverter.start(),
        coordinator.mqtt.start()
    )?;

    Ok(())
}
