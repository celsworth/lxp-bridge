use crate::prelude::*;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::Decoder;

use lxp::packet::TcpFrameFactory;

pub type PacketSender = broadcast::Sender<Option<Packet>>;

pub struct Inverter {
    config: Rc<Config>,
    from_coordinator: PacketSender,
    to_coordinator: PacketSender,
}

impl Inverter {
    pub fn new(
        config: Rc<Config>,
        from_coordinator: PacketSender,
        to_coordinator: PacketSender,
    ) -> Self {
        Self {
            config,
            from_coordinator,
            to_coordinator,
        }
    }

    pub async fn start(&self) -> Result<()> {
        tokio::task::LocalSet::new()
            .run_until(async move {
                for inverter in &self.config.inverters {
                    let inverter = inverter.clone();
                    let from = self.from_coordinator.clone();
                    let to = self.to_coordinator.clone();

                    tokio::task::spawn_local(async move {
                        Self::foo(inverter, from, to).await.unwrap();
                    })
                    .await
                    .unwrap();
                }
            })
            .await;

        Ok(())
    }

    async fn foo(
        config: config::Inverter,
        from_coordinator: PacketSender,
        to_coordinator: PacketSender,
    ) -> Result<()> {
        info!("connecting to inverter at {}:{}", &config.host, config.port);
        loop {}
    }

    /*
    pub async fn start(&self) -> Result<()> {
        let config = &self.config.inverters[0];
        loop {
            match self.connect(config).await {
                Ok(_) => break,
                Err(e) => {
                    error!("connect: {}", e);
                    info!("attempting inverter reconnection in 5s");
                    self.to_coordinator.send(None)?; // kill any waiting readers
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            };
        }

        Ok(())
    }

    async fn connect(&self, config: &config::Inverter) -> Result<()> {
        info!("connecting to inverter at {}:{}", &config.host, config.port);

        let inverter_hp = (config.host.to_string(), config.port);
        let (reader, writer) = TcpStream::connect(inverter_hp).await?.into_split();

        info!("inverter connected!");

        futures::try_join!(self.sender(writer), self.receiver(reader))?;

        Ok(())
    }

    // inverter -> coordinator
    async fn receiver(&self, mut socket: tokio::net::tcp::OwnedReadHalf) -> Result<()> {
        let mut buf = BytesMut::new();
        let mut decoder = lxp::packet_decoder::PacketDecoder::new();

        loop {
            // read_buf appends to buf rather than overwrite existing data
            let len = socket.read_buf(&mut buf).await?;

            // TODO: reconnect if nothing for 5 minutes?
            // or maybe send our own heartbeats?

            if len == 0 {
                while let Some(packet) = decoder.decode_eof(&mut buf)? {
                    debug!("RX {:?}", packet);
                    self.to_coordinator.send(Some(packet))?;
                }
                break;
            }

            while let Some(packet) = decoder.decode(&mut buf)? {
                //debug!("RX ({} bytes): {:?}", packet.bytes().len(), packet);
                self.to_coordinator.send(Some(packet))?;
            }
        }

        Err(anyhow!("receiver exiting (inverter disconnect)"))
    }

    // coordinator -> inverter
    async fn sender(&self, mut socket: tokio::net::tcp::OwnedWriteHalf) -> Result<()> {
        let mut receiver = self.from_coordinator.subscribe();

        while let Some(packet) = receiver.recv().await? {
            debug!("TX {:?}", packet);

            let bytes = TcpFrameFactory::build(packet);
            //debug!("TX {:?}", bytes);

            socket.write_all(&bytes).await?
        }

        Err(anyhow!(
            "sender exiting due to receiving None from coordinator"
        ))
    }
    */
}
