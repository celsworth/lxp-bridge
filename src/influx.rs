use crate::prelude::*;

use chrono::TimeZone;
use rinfluxdb::line_protocol::{r#async::Client, LineBuilder};

static INPUTS_MEASUREMENT: &str = "inputs";

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ChannelData {
    InputData(serde_json::Value),
    Shutdown,
}

pub type Receiver = broadcast::Receiver<ChannelData>;
pub type Sender = broadcast::Sender<ChannelData>;

pub struct Influx {
    config: ConfigWrapper,
    channels: Channels,
    receiver: Cell<Option<Receiver>>,
}

impl Influx {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        // Create the receiver at new() time, rather than start(), so things
        // wanting to send to queue messages, rather than failing with an error
        // about closed channels
        let receiver = Cell::new(Some(channels.to_influx.subscribe()));

        Self {
            config,
            channels,
            receiver,
        }
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

        let mut receiver = self.receiver.take().expect("should only be called once");

        loop {
            let mut line = LineBuilder::new(INPUTS_MEASUREMENT);

            match receiver.recv().await? {
                Shutdown => break,
                InputData(data) => {
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
