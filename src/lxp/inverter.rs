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
        let futures = self.config.inverters.iter().cloned().map(|inverter| {
            Self::run_for_inverter(inverter, &self.from_coordinator, &self.to_coordinator)
        });

        futures::future::join_all(futures).await;

        Ok(())
    }

    async fn run_for_inverter(
        config: config::Inverter,
        from_coordinator: &PacketSender,
        to_coordinator: &PacketSender,
    ) -> Result<()> {
        loop {
            match Self::connect(&config, from_coordinator, to_coordinator).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!("connect: {}", e);
                    info!("attempting inverter reconnection in 5s");
                    to_coordinator.send(None)?; // kill any waiting readers
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect(
        config: &config::Inverter,
        from_coordinator: &PacketSender,
        to_coordinator: &PacketSender,
    ) -> Result<()> {
        info!("connecting to inverter at {}:{}", &config.host, config.port);

        let inverter_hp = (config.host.to_string(), config.port);
        let (reader, writer) = TcpStream::connect(inverter_hp).await?.into_split();

        info!("inverter connected!");

        futures::try_join!(
            Self::sender(from_coordinator, writer),
            Self::receiver(to_coordinator, reader)
        )?;

        Ok(())
    }

    // inverter -> coordinator
    async fn receiver(
        to_coordinator: &PacketSender,
        mut socket: tokio::net::tcp::OwnedReadHalf,
    ) -> Result<()> {
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
                    to_coordinator.send(Some(packet))?;
                }
                break;
            }

            while let Some(packet) = decoder.decode(&mut buf)? {
                //debug!("RX ({} bytes): {:?}", packet.bytes().len(), packet);
                to_coordinator.send(Some(packet))?;
            }
        }

        Err(anyhow!("receiver exiting (inverter disconnect)"))
    }

    // coordinator -> inverter
    async fn sender(
        from_coordinator: &PacketSender,
        mut socket: tokio::net::tcp::OwnedWriteHalf,
    ) -> Result<()> {
        let mut receiver = from_coordinator.subscribe();

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
}
