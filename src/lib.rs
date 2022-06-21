pub mod channels;
pub mod command;
pub mod config;
pub mod coordinator;
pub mod database;
pub mod home_assistant;
pub mod influx;
pub mod lxp;
pub mod mqtt;
pub mod options;
pub mod prelude;
pub mod scheduler;
pub mod unixtime;
pub mod utils;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::prelude::*;

pub fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = app();

    if let Err(err) = rt.block_on(future) {
        error!("{:?}", err);
        std::process::exit(255);
    }
}

async fn app() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.level(),
                record.module_path().unwrap_or(""),
                record.args()
            )
        })
        .write_style(env_logger::WriteStyle::Never)
        .init();

    let options = Options::new()?;

    info!("lxp-bridge {} starting", CARGO_PKG_VERSION);

    let config = Rc::new(Config::new(options.config_file)?);

    let channels = Channels::new();

    let scheduler = Scheduler::new(Rc::clone(&config), channels.clone());
    let mqtt = Mqtt::new(Rc::clone(&config), channels.clone());
    let influx = Influx::new(Rc::clone(&config), channels.clone());
    let coordinator = Coordinator::new(Rc::clone(&config), channels.clone());

    let inverters = config
        .enabled_inverters()
        .cloned()
        .map(|inverter| {
            Inverter::new(
                inverter,
                channels.to_inverter.clone(),
                channels.from_inverter.clone(),
            )
        })
        .collect();

    let databases = config
        .enabled_databases()
        .cloned()
        .map(|database| Database::new(database, channels.to_database.clone()))
        .collect();

    futures::try_join!(
        start_databases(databases),
        start_inverters(inverters),
        scheduler.start(),
        mqtt.start(),
        influx.start(),
        coordinator.start()
    )?;

    Ok(())
}

async fn start_databases(databases: Vec<Database>) -> Result<()> {
    let futures = databases.iter().map(|d| d.start());

    futures::future::join_all(futures).await;

    Ok(())
}

async fn start_inverters(inverters: Vec<Inverter>) -> Result<()> {
    let futures = inverters.iter().map(|i| i.start());

    futures::future::join_all(futures).await;

    Ok(())
}
