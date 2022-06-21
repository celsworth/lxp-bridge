use crate::prelude::*;

use cron_parser::parse;

pub struct Scheduler {
    config: Rc<Config>,
    channels: Channels,
}

impl Scheduler {
    pub fn new(config: Rc<Config>, channels: Channels) -> Self {
        Self { config, channels }
    }

    pub async fn start(&self) -> Result<()> {
        if let Some(config) = &self.config.scheduler {
            if !config.enabled {
                info!("scheduler disabled, skipping");
                return Ok(());
            }

            info!("scheduler starting");

            if config.timesync.enabled {
                let timesync_config = &config.timesync;

                while let Ok(next) = parse(&timesync_config.cron, &Utils::localtime()) {
                    let sleep = next - Utils::localtime();
                    info!("next timesync at {}, sleeping for {}", next, sleep);
                    tokio::time::sleep(sleep.to_std()?).await;
                    self.timesync().await?;
                }
            }

            info!("scheduler exiting");
        }

        Ok(())
    }

    async fn timesync(&self) -> Result<()> {
        info!("timesync starting");

        let inverters = self.config.enabled_inverters().cloned();
        for inverter in inverters {
            coordinator::commands::timesync::TimeSync::new(self.channels.clone(), inverter)
                .run()
                .await?;
        }

        info!("timesync complete");

        Ok(())
    }
}
