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
        let timeout_duration = std::time::Duration::from_secs(Self::TIMEOUT);

        loop {
            if start.elapsed() >= timeout_duration {
                bail!("Timeout waiting for reply to {:?} after {} seconds", packet, Self::TIMEOUT);
            }

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
                (_, Ok(ChannelData::Packet(_))) => {} // Mismatched packet, continue waiting
                (_, Ok(ChannelData::Connected(_))) => {} // Connection status update, continue waiting
                (_, Ok(ChannelData::Disconnect(inverter_datalog))) => {
                    if inverter_datalog == packet.datalog() {
                        bail!("Inverter {} disconnected while waiting for reply", inverter_datalog);
                    }
                }
                (_, Ok(ChannelData::Shutdown)) => bail!("Channel shutdown received while waiting for reply"),
                (_, Err(broadcast::error::TryRecvError::Empty)) => {
                    // Channel empty, sleep briefly before retrying
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                }
                (_, Err(err)) => bail!("Channel error while waiting for reply: {:?}", err),
            }
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

#[derive(Clone)]
pub struct Inverter {
    config: ConfigWrapper,
    host: String,
    channels: Channels,
}

const READ_TIMEOUT_SECS: u64 = 1; // Multiplier for read_timeout from config
const WRITE_TIMEOUT_SECS: u64 = 5; // Timeout for write operations
const RECONNECT_DELAY_SECS: u64 = 5; // Delay before reconnection attempts
const TCP_KEEPALIVE_SECS: u64 = 60; // TCP keepalive interval

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
            info!("inverter {}: reconnecting in {}s", self.config().datalog(), RECONNECT_DELAY_SECS);
            self.channels
                .from_inverter
                .send(ChannelData::Disconnect(self.config().datalog()))?; // kill any waiting readers
            tokio::time::sleep(std::time::Duration::from_secs(RECONNECT_DELAY_SECS)).await;
        }

        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.channels.to_inverter.send(ChannelData::Shutdown);
    }

    async fn connect(&self) -> Result<()> {
        use net2::TcpStreamExt; // for set_keepalive

        let inverter_config = self.config();
        info!(
            "connecting to inverter {} at {}:{}",
            inverter_config.datalog(),
            inverter_config.host(),
            inverter_config.port()
        );

        let inverter_hp = (inverter_config.host().to_owned(), inverter_config.port());

        // Attempt TCP connection with timeout
        let stream = match tokio::time::timeout(
            std::time::Duration::from_secs(WRITE_TIMEOUT_SECS * 2),
            tokio::net::TcpStream::connect(inverter_hp)
        ).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => bail!("Failed to connect to inverter: {}", e),
            Err(_) => bail!("Connection timeout after {} seconds", WRITE_TIMEOUT_SECS * 2),
        };

        // Configure TCP socket
        let std_stream = stream.into_std()?;
        if let Err(e) = std_stream.set_keepalive(Some(std::time::Duration::new(TCP_KEEPALIVE_SECS, 0))) {
            warn!("Failed to set TCP keepalive: {}", e);
        }
        
        let stream = tokio::net::TcpStream::from_std(std_stream)?;
        
        // Set TCP_NODELAY to minimize latency
        if let Err(e) = stream.set_nodelay(true) {
            warn!("Failed to set TCP_NODELAY: {}", e);
        }

        let (reader, writer) = stream.into_split();

        info!("inverter {}: connected!", inverter_config.datalog());
        if let Err(e) = self.channels
            .from_inverter
            .send(ChannelData::Connected(inverter_config.datalog()))
        {
            bail!("Failed to send Connected message: {}", e);
        }

        // Run sender and receiver tasks
        match futures::try_join!(self.sender(writer), self.receiver(reader)) {
            Ok(_) => Ok(()),
            Err(e) => {
                // Ensure we send a disconnect message before returning error
                let _ = self.channels
                    .from_inverter
                    .send(ChannelData::Disconnect(inverter_config.datalog()));
                Err(e.into())
            }
        }
    }

    // inverter -> coordinator
    async fn receiver(&self, mut socket: tokio::net::tcp::OwnedReadHalf) -> Result<()> {
        use std::time::Duration;
        use tokio::time::timeout;
        use {bytes::BytesMut, tokio_util::codec::Decoder};

        const MAX_BUFFER_SIZE: usize = 16384; // 16KB max buffer size
        let mut buf = BytesMut::with_capacity(1024); // Start with 1KB
        let mut decoder = lxp::packet_decoder::PacketDecoder::new();
        let inverter_config = self.config();

        loop {
            // Check buffer capacity and prevent potential memory issues
            if buf.len() >= MAX_BUFFER_SIZE {
                bail!("Buffer overflow: received data exceeds maximum size of {} bytes", MAX_BUFFER_SIZE);
            }

            // read_buf appends to buf rather than overwrite existing data
            let future = socket.read_buf(&mut buf);
            let read_timeout = inverter_config.read_timeout();
            
            let len = if read_timeout > 0 {
                match timeout(
                    Duration::from_secs(read_timeout * READ_TIMEOUT_SECS),
                    future
                ).await {
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => bail!("Read error: {}", e),
                    Err(_) => bail!("No data received for {} seconds", read_timeout * READ_TIMEOUT_SECS),
                }
            } else {
                future.await?
            };

            if len == 0 {
                // Try to process any remaining data before disconnecting
                while let Some(packet) = decoder.decode_eof(&mut buf)? {
                    if let Err(e) = self.handle_incoming_packet(packet) {
                        warn!("Failed to handle final packet: {}", e);
                    }
                }
                bail!("Connection closed by peer");
            }

            // Process received data
            while let Some(packet) = decoder.decode(&mut buf)? {
                let packet_clone = packet.clone();
                
                // Validate and process the packet
                self.compare_datalog(packet.datalog());
                if let Packet::TranslatedData(td) = packet {
                    self.compare_inverter(td.inverter);
                }

                if let Err(e) = self.handle_incoming_packet(packet_clone) {
                    warn!("Failed to handle packet: {}", e);
                    // Continue processing other packets even if one fails
                    continue;
                }
            }

            // Clear the buffer if it's getting too large
            if buf.capacity() > MAX_BUFFER_SIZE / 2 {
                buf.clear();
                buf.reserve(1024);
            }
        }
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
        let inverter_config = self.config();

        loop {
            match receiver.recv().await {
                Ok(ChannelData::Shutdown) => {
                    info!("inverter {}: received shutdown signal", inverter_config.datalog());
                    break;
                }
                Ok(ChannelData::Connected(_)) | Ok(ChannelData::Disconnect(_)) => {
                    // These messages shouldn't be sent to this channel
                    warn!("Unexpected connection status message in sender channel");
                    continue;
                }
                Ok(ChannelData::Packet(packet)) => {
                    if packet.datalog() != inverter_config.datalog() {
                        debug!("Skipping packet for different inverter (expected {}, got {})",
                            inverter_config.datalog(), packet.datalog());
                        continue;
                    }

                    let bytes = lxp::packet::TcpFrameFactory::build(&packet);
                    if bytes.is_empty() {
                        warn!("Generated empty packet data for {:?}", packet);
                        continue;
                    }

                    debug!("inverter {}: TX {:?}", inverter_config.datalog(), bytes);
                    
                    // Use timeout for write operations
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        socket.write_all(&bytes)
                    ).await {
                        Ok(Ok(_)) => {
                            // Ensure data is actually sent
                            if let Err(e) = socket.flush().await {
                                bail!("Failed to flush socket: {}", e);
                            }
                        }
                        Ok(Err(e)) => bail!("Failed to write packet: {}", e),
                        Err(_) => bail!("Write operation timed out after 5 seconds"),
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    bail!("Channel closed");
                }
                Err(e) => {
                    warn!("Error receiving from channel: {}", e);
                    continue;
                }
            }
        }

        info!("inverter {}: sender exiting", inverter_config.datalog());
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
