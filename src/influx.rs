use crate::prelude::*;

use chrono::TimeZone;
use rinfluxdb::line_protocol::{r#async::Client, LineBuilder};

static INPUTS_MEASUREMENT: &str = "inputs";

pub struct Influx {
    config: Rc<Config>,
    from_coordinator: channel::MessageSender,
}

impl Influx {
    pub fn new(config: Rc<Config>, from_coordinator: channel::MessageSender) -> Self {
        Self {
            config,
            from_coordinator,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let config = &self.config.influx;

        if !config.enabled {
            info!("influx disabled, skipping");
            return Ok(());
        }

        info!("initializing influx at {}", config.url);

        let url = reqwest::Url::parse(&config.url)?;
        let credentials = match (&config.username, &config.password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        let client = Client::new(url, credentials)?;

        futures::try_join!(self.sender(client))?;

        Ok(())
    }

    async fn sender(&self, client: rinfluxdb::line_protocol::r#async::Client) -> Result<()> {
        let mut receiver = self.from_coordinator.subscribe();

        loop {
            let mut line = LineBuilder::new(INPUTS_MEASUREMENT);

            let channel::Message::JsonValue(data) = receiver.recv().await?;

            for (key, value) in data.as_object().unwrap() {
                let key = key.to_string();

                line = if key == "time" {
                    line.set_timestamp(chrono::Utc.timestamp(value.as_i64().unwrap(), 0))
                } else if key == "datalog" {
                    line.insert_tag(key, value.as_str().unwrap())
                } else if value.is_f64() {
                    line.insert_field(key, value.as_f64().unwrap())
                } else {
                    // can't be anything other than int
                    line.insert_field(key, value.as_u64().unwrap())
                }
            }

            let lines = vec![line.build()];

            while let Err(err) = client.send(&self.config.influx.database, &lines).await {
                error!("push failed: {:?} - retrying in 10s", err);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    }
}
