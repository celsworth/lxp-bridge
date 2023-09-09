use crate::prelude::*;

use lxp::{
    inverter::WaitForReply,
    packet::{DeviceFunction, TranslatedData},
};

pub struct UpdateHold {
    channels: Channels,
    inverter: config::Inverter,
    register: u16,
    bit: lxp::packet::RegisterBit,
    enable: bool,
}

impl UpdateHold {
    pub fn new<U>(
        channels: Channels,
        inverter: config::Inverter,
        register: U,
        bit: lxp::packet::RegisterBit,
        enable: bool,
    ) -> Self
    where
        U: Into<u16>,
    {
        Self {
            channels,
            inverter,
            register: register.into(),
            bit,
            enable,
        }
    }

    pub async fn run(&self) -> Result<Packet> {
        let mut receiver = self.channels.from_inverter.subscribe();

        // get register from inverter
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog(),
            device_function: DeviceFunction::ReadHold,
            inverter: self.inverter.serial(),
            register: self.register,
            values: vec![1, 0],
        });

        if self
            .channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))
            .is_err()
        {
            bail!("send(to_inverter) failed - channel closed?");
        }

        let packet = receiver.wait_for_reply(&packet).await?;
        let bit = u16::from(self.bit.clone());
        let value = if self.enable {
            packet.value() | (bit as u16)
        } else {
            packet.value() & !(bit as u16)
        };

        // new packet to set register with a new value
        let values = value.to_le_bytes().to_vec();
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog(),
            device_function: DeviceFunction::WriteSingle,
            inverter: self.inverter.serial(),
            register: self.register,
            values,
        });

        if self
            .channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))
            .is_err()
        {
            bail!("send(to_inverter) failed - channel closed?");
        }

        let packet = receiver.wait_for_reply(&packet).await?;
        if packet.value() != value {
            bail!(
                "failed to update register {:?}, got back value {} (wanted {})",
                self.register,
                packet.value(),
                value
            );
        }

        Ok(packet)
    }
}
