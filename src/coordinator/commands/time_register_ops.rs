use crate::prelude::*;

use lxp::{
    inverter::WaitForReply,
    packet::{DeviceFunction, TranslatedData},
};

#[derive(Clone)]
pub enum Action {
    AcCharge(u16),
    AcFirst(u16),
    ChargePriority(u16),
    ForcedDischarge(u16),
}

impl Action {
    fn register(&self) -> Result<u16> {
        use Action::*;
        match self {
            AcCharge(1) => Ok(68),
            AcCharge(2) => Ok(70),
            AcCharge(3) => Ok(72),
            AcFirst(1) => Ok(152),
            AcFirst(2) => Ok(154),
            AcFirst(3) => Ok(156),
            ChargePriority(1) => Ok(76),
            ChargePriority(2) => Ok(78),
            ChargePriority(3) => Ok(80),
            ForcedDischarge(1) => Ok(84),
            ForcedDischarge(2) => Ok(86),
            ForcedDischarge(3) => Ok(88),
            _ => Err(anyhow!("unsupported command")),
        }
    }
}

pub struct ReadTimeRegister {
    channels: Channels,
    inverter: config::Inverter,
    action: Action,
}

impl ReadTimeRegister {
    pub fn new(channels: Channels, inverter: config::Inverter, action: Action) -> Self {
        Self {
            channels,
            inverter,
            action,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog(),
            device_function: DeviceFunction::ReadHold,
            inverter: self.inverter.serial(),
            register: self.action.register()?,
            values: vec![2, 0],
        });

        if self
            .channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))
            .is_err()
        {
            bail!("send(to_inverter) failed - channel closed?");
        }

        Ok(())
    }
}

pub struct SetTimeRegister {
    channels: Channels,
    inverter: config::Inverter,
    action: Action,
    values: [u8; 4],
}

impl SetTimeRegister {
    pub fn new(
        channels: Channels,
        inverter: config::Inverter,
        action: Action,
        values: [u8; 4],
    ) -> Self {
        Self {
            channels,
            inverter,
            action,
            values,
        }
    }

    pub async fn run(&self) -> Result<()> {
        self.set_register(self.action.register()?, &self.values[0..2])
            .await?;
        self.set_register(self.action.register()? + 1, &self.values[2..4])
            .await?;

        Ok(())
    }

    async fn set_register(&self, register: u16, values: &[u8]) -> Result<()> {
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog(),
            device_function: DeviceFunction::WriteSingle,
            inverter: self.inverter.serial(),
            values: values.to_vec(),
            register,
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

        let reply = receiver.wait_for_reply(&packet).await?;
        if let Packet::TranslatedData(td) = reply {
            if td.values != values {
                bail!(
                    "failed to set register {}, got back value {:?} (wanted {:?})",
                    register,
                    td.values,
                    values
                );
            }
        } else {
            bail!("didn't get expected reply from inverter");
        }

        Ok(())
    }
}
