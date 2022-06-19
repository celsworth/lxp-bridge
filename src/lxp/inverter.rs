use crate::prelude::*;

use async_trait::async_trait;
use bytes::BytesMut;
use net2::TcpStreamExt; // for set_keepalive
use serde::{Serialize, Serializer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::Decoder;

#[derive(Debug, Clone)]
pub enum ChannelData {
    Disconnect(Serial), // strictly speaking, only ever goes inverter->coordinator, but eh.
    Packet(Packet),     // this one goes both ways through the channel.
}
pub type Sender = broadcast::Sender<ChannelData>;
pub type Receiver = broadcast::Receiver<ChannelData>;

#[async_trait]
pub trait WaitForReply {
    async fn wait_for_reply(&mut self, packet: &Packet) -> Result<Packet>;
}
#[async_trait]
impl WaitForReply for Receiver {
    async fn wait_for_reply(&mut self, packet: &Packet) -> Result<Packet> {
        let start = std::time::Instant::now();

        loop {
            match (packet, self.try_recv()) {
                (
                    Packet::TranslatedData(td),
                    Ok(ChannelData::Packet(Packet::TranslatedData(reply))),
                ) => {
                    if td.datalog == reply.datalog
                        && td.register == reply.register
                        && td.device_function == reply.device_function
                    {
                        return Ok(Packet::TranslatedData(reply));
                    }
                }
                (_, Ok(ChannelData::Packet(_))) => {} // TODO ReadParam and WriteParam
                (_, Ok(ChannelData::Disconnect(inverter_datalog))) => {
                    if inverter_datalog == packet.datalog() {
                        return Err(anyhow!("inverter disconnect?"));
                    }
                }
                (_, Err(broadcast::error::TryRecvError::Empty)) => {} // ignore and loop
                (_, Err(err)) => return Err(anyhow!("try_recv error: {:?}", err)),
            }
            if start.elapsed().as_secs() > 5 {
                return Err(anyhow!("wait_for_reply {:?} - timeout", packet));
            }

            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }
}

impl ChannelData {}

// Serial {{{
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl Serialize for Serial {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
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
    config: config::Inverter,
    from_coordinator: Sender,
    to_coordinator: Sender,
}

impl Inverter {
    pub fn new(config: config::Inverter, from_coordinator: Sender, to_coordinator: Sender) -> Self {
        Self {
            config,
            from_coordinator,
            to_coordinator,
        }
    }

    pub async fn start(&self) -> Result<()> {
        loop {
            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!("inverter {}: {}", self.config.datalog, e);
                    info!("inverter {}: reconnecting in 5s", self.config.datalog);
                    self.to_coordinator
                        .send(ChannelData::Disconnect(self.config.datalog))?; // kill any waiting readers
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect(&self) -> Result<()> {
        info!(
            "connecting to inverter {} at {}:{}",
            &self.config.datalog, &self.config.host, self.config.port
        );

        let inverter_hp = (self.config.host.to_string(), self.config.port);

        let stream = tokio::net::TcpStream::connect(inverter_hp).await?;
        let std_stream = stream.into_std()?;
        std_stream.set_keepalive(Some(std::time::Duration::new(60, 0)))?;
        let (reader, writer) = tokio::net::TcpStream::from_std(std_stream)?.into_split();

        info!("inverter {}: connected!", self.config.datalog);

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
                    debug!("inverter {}: RX {:?}", self.config.datalog, packet);
                    self.to_coordinator.send(ChannelData::Packet(packet))?;
                }
                break;
            }

            while let Some(packet) = decoder.decode(&mut buf)? {
                debug!("inverter {}: RX {:?}", self.config.datalog, packet);
                self.to_coordinator.send(ChannelData::Packet(packet))?;
            }
        }

        Err(anyhow!("lost connection"))
    }

    // coordinator -> inverter
    async fn sender(&self, mut socket: tokio::net::tcp::OwnedWriteHalf) -> Result<()> {
        let mut receiver = self.from_coordinator.subscribe();

        while let ChannelData::Packet(packet) = receiver.recv().await? {
            if packet.datalog() == self.config.datalog {
                //debug!("inverter {}: TX {:?}", self.config.datalog, packet);
                let bytes = lxp::packet::TcpFrameFactory::build(&packet);
                //debug!("inverter {}: TX {:?}", self.config.datalog, bytes);
                socket.write_all(&bytes).await?
            }
        }

        // this doesn't actually happen yet; Disconnect is never sent to this channel
        Err(anyhow!("sender exiting due to ChannelData::Disconnect"))
    }
}
