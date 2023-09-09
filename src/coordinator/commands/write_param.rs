use crate::prelude::*;

use lxp::inverter::WaitForReply;

pub struct WriteParam {
    channels: Channels,
    inverter: config::Inverter,
    register: u16,
    value: u16,
}

impl WriteParam {
    pub fn new<U>(channels: Channels, inverter: config::Inverter, register: U, value: u16) -> Self
    where
        U: Into<u16>,
    {
        Self {
            channels,
            inverter,
            register: register.into(),
            value,
        }
    }

    pub async fn run(&self) -> Result<Packet> {
        let packet = Packet::WriteParam(lxp::packet::WriteParam {
            datalog: self.inverter.datalog(),
            register: self.register,
            values: self.value.to_le_bytes().to_vec(),
        });

        let mut receiver = self.channels.from_inverter.subscribe();

        if self
            .channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))
            .is_err()
        {
            bail!("send(to_inverter) failed - channel closed?");
        }

        let packet = receiver.wait_for_reply(&packet).await?;
        // WriteParam packets seem to reply with 0 on success, very odd
        if packet.value() != 0 {
            bail!("failed to set register {}", self.register);
        }

        Ok(packet)
    }
}
