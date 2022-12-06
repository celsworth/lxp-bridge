use crate::prelude::*;

// use cron_parser::parse;

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

        /* this doesn't do anything yet */

        info!("scheduler exiting");

        Ok(())
    }
}
