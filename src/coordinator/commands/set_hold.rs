use crate::prelude::*;

use lxp::{
    inverter::WaitForReply,
    packet::{DeviceFunction, TranslatedData},
};

pub struct SetHold {
    channels: Channels,
    inverter: config::Inverter,
    register: u16,
    value: u16,
}

impl SetHold {
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
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog(),
            device_function: DeviceFunction::WriteSingle,
            inverter: self.inverter.serial(),
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
        if packet.value() != self.value {
            bail!(
                "failed to set register {}, got back value {} (wanted {})",
                self.register,
                packet.value(),
                self.value
            );
        }

        Ok(packet)
    }
}
