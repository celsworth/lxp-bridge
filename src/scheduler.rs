use crate::prelude::*;

use cron_parser::parse;

use chrono::{DateTime, Local};

pub struct Scheduler {
    config: ConfigWrapper,
    channels: Channels,
}

impl Scheduler {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self { config, channels }
    }

    pub async fn start(&self) -> Result<()> {
        let scheduler = self.config.scheduler().clone();
        if scheduler.is_none() {
            info!("scheduler config not found, skipping");
            return Ok(());
        }

        let scheduler = scheduler.unwrap();
        if !scheduler.enabled() {
            info!("scheduler disabled, skipping");
            return Ok(());
        }

        info!("scheduler starting");

        if let Some(timesync_cron) = scheduler.timesync_cron() {
            // sticking to Utc here avoids some "invalid date" panics around DST changes
            while let Ok(next) = parse(timesync_cron, &Utils::utc()) {
                let sleep = next - Utils::utc();

                // localtime is only used for display
                let local_next: DateTime<Local> = DateTime::from(next);
                info!("next timesync at {}, sleeping for {}", local_next, sleep);

                tokio::time::sleep(sleep.to_std()?).await;
                self.timesync().await?;
            }
        } else {
            info!("timesync_cron config not found, skipping");
        }

        info!("scheduler exiting");

        Ok(())
    }

    async fn timesync(&self) -> Result<()> {
        info!("timesync starting");

        for inverter in self.config.enabled_inverters() {
            coordinator::commands::timesync::TimeSync::new(self.channels.clone(), inverter)
                .run()
                .await?;
        }

        info!("timesync complete");

        Ok(())
    }
}
