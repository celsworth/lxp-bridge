use crate::prelude::*;

use chrono::TimeZone;
use rinfluxdb::line_protocol::{r#async::Client, LineBuilder};

static INPUTS_MEASUREMENT: &str = "inputs";

#[derive(PartialEq, Clone, Debug)]
pub enum ChannelData {
    InputData(Serial, lxp::register_parser::ParsedData),
    Shutdown,
}

pub struct Influx {
    config: ConfigWrapper,
    channels: Channels,
}

impl Influx {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self { config, channels }
    }

    pub async fn start(&self) -> Result<()> {
        if !self.config.influx().enabled() {
            info!("influx disabled, skipping");
            return Ok(());
        }

        info!("initializing influx at {}", self.config.influx().url());

        let client = {
            let config = self.config.influx();
            let url = reqwest::Url::parse(config.url())?;
            let credentials = match (config.username(), config.password()) {
                (Some(u), Some(p)) => Some((u, p)),
                _ => None,
            };

            Client::new(url, credentials)?
        };

        futures::try_join!(self.sender(client))?;

        info!("influx loop exiting");

        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.channels.to_influx.send(ChannelData::Shutdown);
    }

    async fn sender(&self, client: Client) -> Result<()> {
        use ChannelData::*;

        let mut receiver = self.channels.to_influx.subscribe();

        loop {
            use lxp::register_parser::ParsedValue;

            let mut line = LineBuilder::new(INPUTS_MEASUREMENT);

            match receiver.recv().await? {
                Shutdown => break,
                InputData(datalog, data) => {
                    for (key, value) in data {
                        line = match (key, value) {
                            ("time", _) => line.set_timestamp(Utils::utc()),
                            (_, ParsedValue::String(v)) => line.insert_field(key, v),
                            (_, ParsedValue::StringOwned(v)) => line.insert_field(key, v),
                            (_, ParsedValue::Integer(v)) => line.insert_field(key, v),
                            (_, ParsedValue::Float(v)) => line.insert_field(key, v),
                        };
                    }
                    line = line.insert_tag("datalog", datalog.to_string());

                    let lines = vec![line.build()];

                    debug!("{:?}", lines);

                    while let Err(err) = client.send(&self.database(), &lines).await {
                        error!("push failed: {:?} - retrying in 10s", err);
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    }
                }
            }
        }

        info!("sender loop exiting");

        Ok(())
    }

    fn database(&self) -> String {
        self.config.influx().database().to_string()
    }
}
