use crate::prelude::*;

use bytes::{Buf, BytesMut};
use std::io::{Error, ErrorKind};
use tokio_util::codec::Decoder;

// Maximum allowed packet size to prevent excessive memory allocation
const MAX_PACKET_SIZE: usize = 1024; // Adjust this value based on protocol specifications
// Magic header bytes that identify a valid LXP packet
const HEADER_BYTES: [u8; 2] = [161, 26];

pub struct PacketDecoder(());

impl PacketDecoder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }

    // Verify checksum for TranslatedData packets
    fn verify_checksum(data: &[u8], tcp_function: u8) -> Result<(), Error> {
        // Only TranslatedData packets (194) have checksums
        if tcp_function == 194 {
            let len = data.len();
            if len < 22 { // Minimum length for a TranslatedData packet with checksum
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Packet too short for TranslatedData checksum verification"
                ));
            }
            
            // The checksum is calculated over the data portion, excluding header and checksum itself
            let data_start = 20; // Skip header
            let data_end = len - 2; // Exclude checksum bytes
            
            if data_end <= data_start {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Invalid packet length for checksum calculation"
                ));
            }

            let payload = &data[data_start..data_end];
            let received_checksum = &data[data_end..];
            let calculated_checksum = crc16::State::<crc16::MODBUS>::calculate(payload).to_le_bytes();
            
            if calculated_checksum != received_checksum[..2] {
                debug!(
                    "Checksum mismatch - received: {:02x?}, calculated: {:02x?}",
                    received_checksum,
                    calculated_checksum
                );
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Checksum mismatch - received: {:02x?}, calculated: {:02x?}",
                        received_checksum,
                        calculated_checksum
                    ),
                ));
            }
        }
        Ok(())
    }
}

impl Decoder for PacketDecoder {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let src_len = src.len();

        if src_len < 6 {
            // Not enough data to read header (2 bytes) + protocol (2 bytes) + length (2 bytes)
            trace!("Waiting for more data, current length: {}", src_len);
            return Ok(None);
        }

        // Verify packet header
        if src[0..2] != HEADER_BYTES {
            debug!("Invalid packet header: {:02x?}, expected: {:02x?}", &src[0..2], HEADER_BYTES);
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid packet header: {:02x?}, expected: {:02x?}", &src[0..2], HEADER_BYTES),
            ));
        }

        // Read packet length (little-endian)
        let packet_len = usize::from(u16::from_le_bytes([src[4], src[5]]));
        
        // Check against maximum allowed size
        if packet_len > MAX_PACKET_SIZE {
            debug!("Packet size {} exceeds maximum allowed size {}", packet_len, MAX_PACKET_SIZE);
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Packet size {} exceeds maximum allowed size {}", packet_len, MAX_PACKET_SIZE),
            ));
        }

        // Total frame length includes 6-byte header
        let frame_len = 6 + packet_len;

        if src_len < frame_len {
            // Partial frame - reserve space for the remaining bytes
            trace!("Waiting for complete frame: have {}, need {}", src_len, frame_len);
            src.reserve(frame_len - src_len);
            return Ok(None);
        }

        // Get TCP function for checksum verification
        let tcp_function = src[7];
        debug!("Processing packet: len={}, tcp_function={}", frame_len, tcp_function);

        // Verify checksum if applicable
        if let Err(e) = Self::verify_checksum(&src[..frame_len], tcp_function) {
            debug!("Checksum verification failed: {}", e);
            return Err(e);
        }

        // Create a reference to the frame data instead of copying
        let data = src.split_to(frame_len);

        debug!("Successfully decoded packet: {} bytes", data.len());
        trace!("Packet data: {:02x?}", data);

        // Parse the packet using the LXP parser
        match lxp::packet::Parser::parse(&data) {
            Ok(packet) => {
                debug!("Successfully parsed packet: {:?}", packet);
                Ok(Some(packet))
            }
            Err(e) => {
                debug!("Failed to parse packet: {}", e);
                Err(Error::new(ErrorKind::InvalidData, e))
            }
        }
    }
}
