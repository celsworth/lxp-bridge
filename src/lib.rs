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

    let config = Config::new(options.config_file)?;

    let coordinator = Coordinator::new(config);

    futures::try_join!(coordinator.start())?;

    Ok(())
}
