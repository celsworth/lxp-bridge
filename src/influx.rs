use crate::prelude::*;

use influxdb::Client;

pub struct Influx {
    config: Rc<Config>,
    from_inverter: lxp::inverter::PacketSender,
}

impl Influx {
    pub fn new(config: Rc<Config>, from_inverter: lxp::inverter::PacketSender) -> Self {
        Self {
            config,
            from_inverter,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let client = Client::new("http://nas:8086", &self.config.influx.database);

        futures::try_join!(self.sender(client))?;

        Ok(())
    }

    async fn sender(&self, client: influxdb::Client) -> Result<()> {
        use lxp::packet::ReadInput;

        let mut receiver = self.from_inverter.subscribe();

        loop {
            if let (_datalog, Some(packet)) = receiver.recv().await? {
                debug!("RX: {:?}", packet);

                if let Packet::TranslatedData(td) = packet {
                    let query = match td.read_input()? {
                        ReadInput::ReadInput1(r1) => r1.into_query(&self.config.influx.measurement),
                        ReadInput::ReadInput2(r2) => r2.into_query(&self.config.influx.measurement),
                        ReadInput::ReadInput3(r3) => r3.into_query(&self.config.influx.measurement),
                    };

                    client.query(&query).await?;
                }
            }
        }
    }
}
