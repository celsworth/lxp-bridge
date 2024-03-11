use crate::prelude::*;

use {
    async_trait::async_trait,
    serde::{Serialize, Serializer},
    tokio::io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Connected(Serial),  // strictly speaking, these two only ever go
    Disconnect(Serial), // inverter->coordinator, but eh.
    Packet(Packet),     // this one goes both ways through the channel.
    Shutdown,
}
pub type Sender = broadcast::Sender<ChannelData>;
pub type Receiver = broadcast::Receiver<ChannelData>;

// WaitForReply {{{
#[async_trait]
pub trait WaitForReply {
    #[cfg(not(feature = "mocks"))]
    const TIMEOUT: u64 = 10;

    #[cfg(feature = "mocks")]
    const TIMEOUT: u64 = 0; // fail immediately in tests

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
                (Packet::ReadParam(rp), Ok(ChannelData::Packet(Packet::ReadParam(reply)))) => {
                    if rp.datalog == reply.datalog && rp.register == reply.register {
                        return Ok(Packet::ReadParam(reply));
                    }
                }
                (Packet::WriteParam(wp), Ok(ChannelData::Packet(Packet::WriteParam(reply)))) => {
                    if wp.datalog == reply.datalog && wp.register == reply.register {
                        return Ok(Packet::WriteParam(reply));
                    }
                }
                (_, Ok(ChannelData::Packet(_))) => {} // TODO ReadParam and WriteParam
                (_, Ok(ChannelData::Connected(_))) => {} // Breaks on channel overflow, but we have a timeout
                (_, Ok(ChannelData::Disconnect(inverter_datalog))) => {
                    if inverter_datalog == packet.datalog() {
                        bail!("inverter disconnect?");
                    }
                }
                (_, Ok(ChannelData::Shutdown)) => bail!("shutting down"),
                (_, Err(broadcast::error::TryRecvError::Empty)) => {} // ignore and loop
                (_, Err(err)) => bail!("try_recv error: {:?}", err),
            }
            if start.elapsed().as_secs() > Self::TIMEOUT {
                bail!("wait_for_reply {:?} - timeout", packet);
            }

            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }
} // }}}

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
    config: ConfigWrapper,
    host: String,
    channels: Channels,
}

impl Inverter {
    pub fn new(config: ConfigWrapper, inverter: &config::Inverter, channels: Channels) -> Self {
        // remember which inverter this instance is for
        let host = inverter.host().to_string();

        Self {
            config,
            host,
            channels,
        }
    }

    pub fn config(&self) -> config::Inverter {
        self.config
            .inverter_with_host(&self.host)
            .expect("can't find my inverter")
    }

    pub async fn start(&self) -> Result<()> {
        while let Err(e) = self.connect().await {
            error!("inverter {}: {}", self.config().datalog(), e);
            info!("inverter {}: reconnecting in 5s", self.config().datalog());
            self.channels
                .from_inverter
                .send(ChannelData::Disconnect(self.config().datalog()))?; // kill any waiting readers
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.channels.to_inverter.send(ChannelData::Shutdown);
    }

    async fn connect(&self) -> Result<()> {
        use net2::TcpStreamExt; // for set_keepalive

        info!(
            "connecting to inverter {} at {}:{}",
            self.config().datalog(),
            self.config().host(),
            self.config().port()
        );

        let inverter_hp = (self.config().host().to_owned(), self.config().port());

        let stream = tokio::net::TcpStream::connect(inverter_hp).await?;
        let std_stream = stream.into_std()?;
        std_stream.set_keepalive(Some(std::time::Duration::new(60, 0)))?;
        let (reader, writer) = tokio::net::TcpStream::from_std(std_stream)?.into_split();

        info!("inverter {}: connected!", self.config().datalog());
        self.channels
            .from_inverter
            .send(ChannelData::Connected(self.config().datalog()))?;

        futures::try_join!(self.sender(writer), self.receiver(reader))?;

        Ok(())
    }

    // inverter -> coordinator
    async fn receiver(&self, mut socket: tokio::net::tcp::OwnedReadHalf) -> Result<()> {
        use std::time::Duration;
        use tokio::time::timeout;
        use {bytes::BytesMut, tokio_util::codec::Decoder};

        let mut buf = BytesMut::new();
        let mut decoder = lxp::packet_decoder::PacketDecoder::new();

        loop {
            // read_buf appends to buf rather than overwrite existing data
            let future = socket.read_buf(&mut buf);
            let read_timeout = self.config().read_timeout();
            let len = if read_timeout > 0 {
                match timeout(Duration::from_millis(read_timeout * 1000), future).await {
                    Ok(r) => r,
                    Err(_) => bail!("no data for {} seconds", read_timeout),
                }
            } else {
                future.await
            }?;

            if len == 0 {
                while let Some(packet) = decoder.decode_eof(&mut buf)? {
                    self.handle_incoming_packet(packet)?;
                }
                break;
            }

            while let Some(packet) = decoder.decode(&mut buf)? {
                self.handle_incoming_packet(packet.clone())?;

                self.compare_datalog(packet.datalog()); // all packets have datalog serial
                if let Packet::TranslatedData(td) = packet {
                    // only TranslatedData has inverter serial
                    self.compare_inverter(td.inverter);
                };
            }
        }

        Err(anyhow!("lost connection"))
    }

    fn handle_incoming_packet(&self, packet: Packet) -> Result<()> {
        // bytes received are logged in packet_decoder, no need here
        //debug!("inverter {}: RX {:?}", self.config.datalog, packet);

        if self.config().heartbeats()
            && packet.tcp_function() == lxp::packet::TcpFunction::Heartbeat
        {
            self.channels
                .to_inverter
                .send(ChannelData::Packet(packet.clone()))?;
        }

        self.channels
            .from_inverter
            .send(ChannelData::Packet(packet))?;

        Ok(())
    }

    // coordinator -> inverter
    async fn sender(&self, mut socket: tokio::net::tcp::OwnedWriteHalf) -> Result<()> {
        let mut receiver = self.channels.to_inverter.subscribe();

        use ChannelData::*;

        loop {
            match receiver.recv().await? {
                Shutdown => break,
                // this doesn't actually happen yet; (Dis)connect is never sent to this channel
                Connected(_) => {}
                Disconnect(_) => bail!("sender exiting due to ChannelData::Disconnect"),
                Packet(packet) => {
                    // this works, but needs more thought. because we only fix it here, immediately
                    // before transmission, calls to wait_for_reply with the original serials will
                    // never complete. ideally we need to pass the fixed packet back?
                    //self.fix_outgoing_packet_serials(&mut packet);

                    if packet.datalog() == self.config().datalog() {
                        //debug!("inverter {}: TX {:?}", self.config.datalog, packet);
                        let bytes = lxp::packet::TcpFrameFactory::build(&packet);
                        debug!("inverter {}: TX {:?}", self.config().datalog(), bytes);
                        socket.write_all(&bytes).await?
                    }
                }
            }
        }

        info!("inverter {}: sender exiting", self.config().datalog());

        Ok(())
    }

    /* TODO. need to solve wait_for_reply hanging when we fix the serials.. */
    /*
    #[allow(dead_code)]
    fn fix_outgoing_packet_serials(&self, packet: &mut Packet) {
        let ob = self.serials.borrow();
        if packet.datalog() != ob.datalog {
            warn!(
                "fixing datalog in outgoing packet from {} to {}",
                packet.datalog(),
                ob.datalog
            );
            packet.set_datalog(ob.datalog);
        }

        if let Some(inverter) = packet.inverter() {
            if inverter != ob.inverter {
                warn!(
                    "fixing serial in outgoing packet from {} to {}",
                    inverter, ob.inverter
                );
                packet.set_inverter(ob.inverter);
            }
        }
    }
    */

    fn compare_datalog(&self, packet: Serial) {
        if packet != self.config().datalog() {
            warn!(
                "datalog serial mismatch found; packet={}, config={} - please check config!",
                packet,
                self.config().datalog()
            );
            // uncomment this when I fix serials in outgoing packets?
            //self.config.datalog = packet;
        }
    }

    fn compare_inverter(&self, packet: Serial) {
        if packet != self.config().serial() {
            warn!(
                "inverter serial mismatch found; packet={}, config={} - please check config!",
                packet,
                self.config().serial()
            );
            // uncomment this when I fix serials in outgoing packets?
            //self.config.serial = packet;
        }
    }
}
