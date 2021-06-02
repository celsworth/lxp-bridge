use crate::prelude::*;

use bytes::{Buf, BytesMut};
use std::io::{Error, ErrorKind};
use tokio_util::codec::Decoder;

pub struct PacketDecoder(());

impl PacketDecoder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}

impl Decoder for PacketDecoder {
    type Item = Packet;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let src_len = src.len();

        if src_len < 6 {
            // not enough data to read packet length
            return Ok(None);
        }

        if src[0..2] != [161, 26] {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "161, 26 header not found",
            ));
        }

        // protocol is in src[2..4], not used here yet

        let packet_len = usize::from(u16::from_le_bytes([src[4], src[5]]));

        // packet_len excludes the first 6 bytes, re-add those to make maths easier
        let frame_len = 6 + packet_len;

        if src_len < frame_len {
            // partial frame
            src.reserve(frame_len - src_len);
            return Ok(None);
        }

        let data = &src[..frame_len].to_owned();
        src.advance(frame_len);

        debug!("{} bytes in: {:?}", data.len(), data);

        match lxp::packet::Parser::parse(data) {
            Ok(packet) => Ok(Some(packet)),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
        }
    }
}
