use crate::prelude::*;

use lxp::{
    inverter::WaitForReply,
    packet::{DeviceFunction, TranslatedData},
};

use serde::Serialize;

pub struct ReadTimeRegister {
    channels: Channels,
    inverter: config::Inverter,
    action: Action,
}

#[derive(Debug, Serialize)]
struct MqttReplyPayload {
    start: String,
    end: String,
}

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
            _ => bail!("unsupported command"),
        }
    }

    fn mqtt_reply_topic(&self, datalog: Serial) -> String {
        use Action::*;
        // no need to be defensive about n here, we checked it already in register()
        match self {
            AcCharge(n) => format!("{}/ac_charge/{}", datalog, n),
            AcFirst(n) => format!("{}/ac_first/{}", datalog, n),
            ChargePriority(n) => format!("{}/charge_priority/{}", datalog, n),
            ForcedDischarge(n) => format!("{}/forced_discharge/{}", datalog, n),
        }
    }
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
            let payload = MqttReplyPayload {
                start: format!("{:02}:{:02}", td.values[0], td.values[1]),
                end: format!("{:02}:{:02}", td.values[2], td.values[3]),
            };
            let message = mqtt::Message {
                topic: self.action.mqtt_reply_topic(td.datalog),
                retain: true,
                payload: serde_json::to_string(&payload)?,
            };
            let channel_data = mqtt::ChannelData::Message(message);

            if self.channels.to_mqtt.send(channel_data).is_err() {
                bail!("send(to_mqtt) failed - channel closed?");
            }

            Ok(())
        } else {
            bail!("didn't get expected reply from inverter");
        }
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

        // FIXME: If we only update one of the two registers, we should probably
        // still output the change we did manage to make here.
        let payload = MqttReplyPayload {
            start: format!("{:02}:{:02}", self.values[0], self.values[1]),
            end: format!("{:02}:{:02}", self.values[2], self.values[3]),
        };
        let message = mqtt::Message {
            topic: self.action.mqtt_reply_topic(self.inverter.datalog),
            retain: true,
            payload: serde_json::to_string(&payload)?,
        };
        let channel_data = mqtt::ChannelData::Message(message);

        if self.channels.to_mqtt.send(channel_data).is_err() {
            bail!("send(to_mqtt) failed - channel closed?");
        }

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
