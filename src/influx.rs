use crate::prelude::*;

use chrono::TimeZone;
use rinfluxdb::line_protocol::{r#async::Client, LineBuilder};

static INPUTS_MEASUREMENT: &str = "inputs";

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ChannelData {
    InputData(serde_json::Value),
    Shutdown,
}

#[derive(Clone)]
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
        info!("InfluxDB sender started");

        loop {
            let mut line = LineBuilder::new(INPUTS_MEASUREMENT);

            match receiver.recv().await? {
                Shutdown => {
                    info!("InfluxDB sender received shutdown signal");
                    break;
                }
                InputData(data) => {
                    debug!("InfluxDB processing input data: {:?}", data);
                    for (key, value) in data.as_object().ok_or_else(|| anyhow!("Invalid data format"))? {
                        let key = key.to_string();
                        debug!("Processing field: {} = {:?}", key, value);

                        line = if key == "time" {
                            let value = value.as_i64().unwrap_or_else(|| {
                                panic!("cannot represent {value} as i64 for {key}")
                            });
                            line.set_timestamp(chrono::Utc.timestamp_opt(value, 0)
                                .single()
                                .ok_or_else(|| anyhow!("Invalid timestamp: {}", value))?)
                        } else if key == "datalog" || key == "inverter" {
                            let value = value.as_str().unwrap_or_else(|| {
                                panic!("cannot represent {value} as str for {key}")
                            });
                            line.insert_tag(key, value)
                        } else if value.is_f64() {
                            let value = value.as_f64().unwrap_or_else(|| {
                                panic!("cannot represent {value} as f64 for {key}")
                            });
                            line.insert_field(key, value)
                        } else {
                            // can't be anything other than int
                            let value = value.as_i64().unwrap_or_else(|| {
                                panic!("cannot represent {value} as i64 for {key}")
                            });
                            line.insert_field(key, value)
                        }
                    }

                    let lines = vec![line.build()];
                    debug!("Sending to InfluxDB: {:?}", lines);

                    let mut retry_count = 0;
                    while retry_count < 3 {
                        match client.send(&self.database(), &lines).await {
                            Ok(_) => {
                                debug!("Successfully sent data to InfluxDB");
                                break;
                            }
                            Err(err) => {
                                error!("InfluxDB push failed: {:?} - retrying in 10s (attempt {}/3)", err, retry_count + 1);
                                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                                retry_count += 1;
                            }
                        }
                    }
                    if retry_count == 3 {
                        error!("Failed to send data to InfluxDB after 3 attempts");
                    }
                }
            }
        }

        info!("InfluxDB sender loop exiting");

        Ok(())
    }

    fn database(&self) -> String {
        self.config.influx().database().to_string()
    }
}
