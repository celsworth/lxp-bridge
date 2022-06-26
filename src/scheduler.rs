use crate::prelude::*;

use cron_parser::parse;

pub struct Scheduler {
    config: Rc<RefCell<Config>>,
    channels: Channels,
}

impl Scheduler {
    pub fn new(config: Rc<RefCell<Config>>, channels: Channels) -> Self {
        Self { config, channels }
    }

    fn config(&self) -> config::Scheduler {
        self.config.borrow().scheduler.as_ref().unwrap().clone()
    }

    pub async fn start(&self) -> Result<()> {
        if self.config.borrow().scheduler.is_none() {
            return Ok(());
        }

        // now we can unwrap the config safely, its always Some

        if !self.config().enabled {
            info!("scheduler disabled, skipping");
            return Ok(());
        }

        info!("scheduler starting");

        if self.config().timesync.enabled {
            let cron = self.config().timesync.cron;

            while let Ok(next) = parse(&cron, &Utils::localtime()) {
                let sleep = next - Utils::localtime();
                info!("next timesync at {}, sleeping for {}", next, sleep);
                tokio::time::sleep(sleep.to_std()?).await;
                self.timesync().await?;
            }
        }

        info!("scheduler exiting");

        Ok(())
    }

    async fn timesync(&self) -> Result<()> {
        info!("timesync starting");

        let inverters = self.config.borrow().enabled_inverters();
        for inverter in inverters {
            coordinator::commands::timesync::TimeSync::new(self.channels.clone(), inverter)
                .run()
                .await?;
        }

        info!("timesync complete");

        Ok(())
    }
}
