use crate::prelude::*;

use influxdb::Client;

static INPUTS_MEASUREMENT: &str = "inputs";

pub struct Influx {
    config: Rc<Config>,
    from_inverter: lxp::inverter::PacketChannelSender,
}

impl Influx {
    pub fn new(config: Rc<Config>, from_inverter: lxp::inverter::PacketChannelSender) -> Self {
        Self {
            config,
            from_inverter,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let config = &self.config.influx;

        if !config.enabled {
            info!("influx disabled, skipping");
            return Ok(());
        }

        info!("initializing influx at {}", config.url);

        let mut client = Client::new(&config.url, &config.database);

        if let (Some(u), Some(p)) = (&config.username, &config.password) {
            client = client.with_auth(u, p);
        }

        match client.ping().await {
            Ok((b, v)) => {
                info!("influx responding ok: build {}, version {}", b, v);
            }
            Err(e) => return Err(anyhow!("influx error: {}", e)),
        }

        futures::try_join!(self.sender(client))?;

        Ok(())
    }

    async fn sender(&self, client: influxdb::Client) -> Result<()> {
        use lxp::packet::{DeviceFunction, ReadInput};

        let mut receiver = self.from_inverter.subscribe();

        loop {
            if let lxp::inverter::PacketChannelData::Packet(Packet::TranslatedData(td)) =
                receiver.recv().await?
            {
                if td.device_function == DeviceFunction::ReadInput {
                    let query = match td.read_input()? {
                        ReadInput::ReadInput1(r1) => r1.into_query(INPUTS_MEASUREMENT),
                        ReadInput::ReadInput2(r2) => r2.into_query(INPUTS_MEASUREMENT),
                        ReadInput::ReadInput3(r3) => r3.into_query(INPUTS_MEASUREMENT),
                    };

                    while let Err(err) = client.query(&query).await {
                        error!("push failed: {:?} - retrying in 10s", err);
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    }
                }
            }
        }
    }
}
