use crate::prelude::*;

use lxp::inverter::WaitForReply;

pub struct ReadParam {
    channels: Channels,
    inverter: config::Inverter,
    register: u16,
}

impl ReadParam {
    pub fn new<U>(channels: Channels, inverter: config::Inverter, register: U) -> Self
    where
        U: Into<u16>,
    {
        Self {
            channels,
            inverter,
            register: register.into(),
        }
    }

    pub async fn run(&self) -> Result<Packet> {
        let packet = Packet::ReadParam(lxp::packet::ReadParam {
            datalog: self.inverter.datalog(),
            register: self.register,
            values: vec![], // unused
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

        receiver.wait_for_reply(&packet).await
    }
}
