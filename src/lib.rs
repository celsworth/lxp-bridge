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
pub mod register_cache;
pub mod scheduler;
pub mod unixtime;
pub mod utils;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::prelude::*;

pub async fn app() -> Result<()> {
    let options = Options::new();

    let config = ConfigWrapper::new(options.config_file).unwrap_or_else(|err| {
        // no logging available yet, so eprintln! will have to do
        eprintln!("Error: {:?}", err);
        std::process::exit(255);
    });

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(config.loglevel()))
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

    info!("lxp-bridge {} starting", CARGO_PKG_VERSION);

    let channels = Channels::new();

    let scheduler = Scheduler::new(config.clone(), channels.clone());
    let mqtt = Mqtt::new(config.clone(), channels.clone());
    let influx = Influx::new(config.clone(), channels.clone());
    let register_cache = RegisterCache::new(channels.clone());
    let coordinator = Coordinator::new(config.clone(), channels.clone());

    let inverters = config
        .enabled_inverters()
        .into_iter()
        .map(|inverter| Inverter::new(config.clone(), &inverter, channels.clone()))
        .collect();

    let databases = config
        .enabled_databases()
        .into_iter()
        .map(|database| Database::new(database, channels.clone()))
        .collect();

    futures::try_join!(
        start_databases(databases),
        start_inverters(inverters),
        scheduler.start(),
        mqtt.start(),
        influx.start(),
        register_cache.start(),
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
