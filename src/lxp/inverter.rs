use crate::prelude::*;

use bytes::BytesMut;
use net2::TcpStreamExt; // for set_keepalive
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::Decoder;

#[derive(Debug, Clone)]
pub enum ChannelContent {
    Disconnect(Serial), // strictly speaking, only ever goes inverter->coordinator, but eh.
    Packet(Packet),     // this one goes both ways through the channel.
}
pub type PacketSender = broadcast::Sender<ChannelContent>;

// Serial {{{
#[derive(Clone, Copy, PartialEq)]
pub struct Serial([u8; 10]);

impl Serial {
    pub fn new(input: &[u8]) -> Result<Self> {
        Ok(Self(input.try_into()?))
    }

    pub fn default() -> Self {
        Self([0; 10])
    }

    pub fn data(&self) -> [u8; 10] {
        self.0
    }
}

impl From<Serial> for influxdb::Type {
    fn from(b: Serial) -> Self {
        influxdb::Type::Text(b.to_string())
    }
}

impl std::str::FromStr for Serial {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 10 {
            return Err(anyhow!("{} must be exactly 10 characters", s));
        }

        let mut r: [u8; 10] = Default::default();
        r.copy_from_slice(s.as_bytes());
        Ok(Self(r))
    }
}

impl std::fmt::Display for Serial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

impl std::fmt::Debug for Serial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
} // }}}

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
        let futures = self.config.enabled_inverters().cloned().map(|inverter| {
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
                    error!("inverter {}: {}", config.datalog, e);
                    info!("inverter {}: reconnecting in 5s", config.datalog);
                    to_coordinator.send(ChannelContent::Disconnect(config.datalog))?; // kill any waiting readers
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
        info!(
            "connecting to inverter {} at {}:{}",
            &config.datalog, &config.host, config.port
        );

        let inverter_hp = (config.host.to_string(), config.port);

        let stream = tokio::net::TcpStream::connect(inverter_hp).await?;
        let std_stream = stream.into_std()?;
        std_stream.set_keepalive(Some(std::time::Duration::new(60, 0)))?;
        let (reader, writer) = tokio::net::TcpStream::from_std(std_stream)?.into_split();

        info!("inverter {}: connected!", config.datalog);

        futures::try_join!(
            Self::sender(from_coordinator, writer, config.datalog),
            Self::receiver(to_coordinator, reader, config.datalog)
        )?;

        Ok(())
    }

    // inverter -> coordinator
    async fn receiver(
        to_coordinator: &PacketSender,
        mut socket: tokio::net::tcp::OwnedReadHalf,
        datalog: Serial,
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
                    debug!("inverter {}: RX {:?}", datalog, packet);
                    to_coordinator.send(ChannelContent::Packet(packet))?;
                }
                break;
            }

            while let Some(packet) = decoder.decode(&mut buf)? {
                debug!("inverter {}: RX {:?}", datalog, packet);
                to_coordinator.send(ChannelContent::Packet(packet))?;
            }
        }

        Err(anyhow!("lost connection"))
    }

    // coordinator -> inverter
    async fn sender(
        from_coordinator: &PacketSender,
        mut socket: tokio::net::tcp::OwnedWriteHalf,
        datalog: Serial,
    ) -> Result<()> {
        let mut receiver = from_coordinator.subscribe();

        while let ChannelContent::Packet(packet) = receiver.recv().await? {
            if packet.datalog() == datalog {
                // debug!("inverter {}: TX {:?}", datalog, packet);
                let bytes = lxp::packet::TcpFrameFactory::build(packet);
                socket.write_all(&bytes).await?
            }
        }

        // this doesn't actually happen yet; None is never sent to this channel
        Err(anyhow!(
            "sender exiting due to receiving None from coordinator"
        ))
    }
}
