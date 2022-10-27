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

    async fn start_timesync(&self, o_config: &Option<config::Crontab>) -> Result<()> {
        if let Some(config) = o_config {
            if config.enabled {
                while let Ok(next) = parse(&config.cron, &Utils::localtime()) {
                    let sleep = next - Utils::localtime();
                    info!("next timesync at {}, sleeping for {}", next, sleep);
                    tokio::time::sleep(sleep.to_std()?).await;
                    self.timesync().await?;
                }
            } else {
                info!("timesync disabled, skipping");
            }
        }

        Ok(())
    }

    async fn start_read_inputs(&self, o_config: &Option<config::Crontab>) -> Result<()> {
        if let Some(config) = o_config {
            if config.enabled {
                while let Ok(next) = parse(&config.cron, &Utils::localtime()) {
                    let sleep = next - Utils::localtime();
                    info!("next read_inputs at {}, sleeping for {}", next, sleep);
                    tokio::time::sleep(sleep.to_std()?).await;
                    self.read_inputs().await?;
                }
            } else {
                info!("read_inputs disabled, skipping");
            }
        }

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        if let Some(config) = &self.config.scheduler {
            if !config.enabled {
                info!("scheduler disabled, skipping");
                return Ok(());
            }

            info!("scheduler starting");

            let timesync_future = self.start_timesync(&config.timesync);
            let read_inputs_future = self.start_read_inputs(&config.read_inputs);

            futures::try_join!(timesync_future, read_inputs_future)?;

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

    async fn read_inputs(&self) -> Result<()> {
        info!("read_inputs starting");

        let inverters = self.config.enabled_inverters().cloned();
        let pairs = [(0, 40), (40, 40), (80, 40)];

        for inverter in inverters {
            for (register, count) in pairs {
                coordinator::commands::read_inputs::ReadInputs::new(
                    self.channels.clone(),
                    inverter.clone(),
                    register as u16,
                    count,
                )
                .run()
                .await?;
            }
        }

        info!("read_inputs complete");

        Ok(())
    }
}
