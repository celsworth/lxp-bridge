use crate::prelude::*;

use lxp::{
    inverter::WaitForReply,
    packet::{DeviceFunction, TranslatedData},
};

pub struct ReadInputs {
    from_inverter: lxp::inverter::Sender,
    to_inverter: lxp::inverter::Sender,
    inverter: config::Inverter,
    register: u16,
    count: u16,
}

impl ReadInputs {
    pub fn new<U>(
        from_inverter: lxp::inverter::Sender,
        to_inverter: lxp::inverter::Sender,
        inverter: config::Inverter,
        register: U,
        count: u16,
    ) -> Self
    where
        U: Into<u16>,
    {
        Self {
            from_inverter,
            to_inverter,
            inverter,
            register: register.into(),
            count,
        }
    }

    pub async fn run(&self) -> Result<Packet> {
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog,
            device_function: DeviceFunction::ReadInput,
            inverter: self.inverter.serial,
            register: self.register,
            values: self.count.to_le_bytes().to_vec(),
        });

        let mut receiver = self.from_inverter.subscribe();

        self.to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        receiver.wait_for_reply(&packet).await
    }
}
