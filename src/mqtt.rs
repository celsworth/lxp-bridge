use crate::prelude::*;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, Publish, QoS};

// Message {{{
#[derive(Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

impl Message {
    pub fn from_command_result(command: &Command, success: bool) -> Self {
        let mut topic = "lxp/result/".to_owned();
        topic.push_str(command.mqtt_topic());

        let payload = match success {
            true => "OK",
            false => "FAIL",
        }
        .to_string();

        Message { topic, payload }
    }

    pub fn from_packet(packet: Packet) -> Result<Option<Self>> {
        let message = match packet.packet_type() {
            PacketType::Heartbeat => None,
            PacketType::ReadHold => Some(Message {
                topic: format!("lxp/hold/{}", packet.register()),
                payload: serde_json::to_string(&packet.value())?,
            }),
            PacketType::ReadInput1 => Some(Message {
                topic: "lxp/inputs/1".to_owned(),
                payload: serde_json::to_string(&packet.read_input1()?)?,
            }),
            PacketType::ReadInput2 => Some(Message {
                topic: "lxp/inputs/2".to_owned(),
                payload: serde_json::to_string(&packet.read_input2()?)?,
            }),
            PacketType::ReadInput3 => Some(Message {
                topic: "lxp/inputs/3".to_owned(),
                payload: serde_json::to_string(&packet.read_input3()?)?,
            }),
        };

        Ok(message)
    }

    pub fn to_command(&self) -> Result<Command> {
        let parts: Vec<&str> = self.topic.split('/').collect();
        // drop lxp/cmd
        match parts[2..] {
            ["read_hold", register] => {
                let r: u16 = register.parse()?;
                Ok(Command::ReadHold(r))
            }

            ["ac_charge"] => Ok(Command::AcCharge(self.payload_bool())),

            ["forced_discharge"] => Ok(Command::ForcedDischarge(self.payload_bool())),

            ["charge_pct"] | ["charge_rate_pct"] => Ok(Command::ChargeRate(self.payload_percent())),
            ["discharge_pct"] | ["discharge_rate_pct"] => {
                Ok(Command::DischargeRate(self.payload_percent()))
            }

            ["ac_charge_rate_pct"] => Ok(Command::AcChargeRate(self.payload_percent())),

            ["charge_amount_pct"] | ["ac_charge_soc_limit_pct"] => {
                Ok(Command::AcChargeSocLimit(self.payload_percent()))
            }

            ["discharge_cutoff_soc_limit_pct"] => {
                Ok(Command::DischargeCutoffSocLimit(self.payload_percent()))
            }

            [..] => Err(anyhow!("unhandled: {:?}", parts)),
        }
    }

    pub fn payload_percent(&self) -> u16 {
        // TODO cap at 0-100, return Result?
        self.payload.parse::<u16>().unwrap_or(100)
    }

    pub fn payload_bool(&self) -> bool {
        matches!(
            self.payload.to_ascii_lowercase().as_str(),
            "1" | "t" | "true" | "on" | "y" | "yes"
        )
    }
} // }}}

pub struct Mqtt {
    config: Rc<Config>,
    from_coordinator: MessageSender,
    to_coordinator: MessageSender,
}

pub type MessageSender = tokio::sync::broadcast::Sender<Message>;

impl Mqtt {
    pub fn new(
        config: Rc<Config>,
        from_coordinator: MessageSender,
        to_coordinator: MessageSender,
    ) -> Self {
        Self {
            config,
            from_coordinator,
            to_coordinator,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let m = &self.config.mqtt;

        let mut options = MqttOptions::new("lxp-bridge", &m.host, m.port);

        options.set_keep_alive(60);
        if let (Some(u), Some(p)) = (&m.username, &m.password) {
            options.set_credentials(u, p);
        }

        info!("connecting to mqtt at {}:{}", &m.host, m.port);

        let (client, eventloop) = AsyncClient::new(options, 10);

        info!("mqtt connected!");

        client.subscribe("lxp/cmd/+", QoS::AtMostOnce).await?;
        client
            .subscribe("lxp/cmd/read_hold/+", QoS::AtMostOnce)
            .await?;

        futures::try_join!(self.receiver(eventloop), self.sender(client))?;

        Ok(())
    }

    // mqtt -> coordinator
    async fn receiver(&self, mut eventloop: EventLoop) -> Result<()> {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(publish))) => {
                    let message = Self::parse_message(publish)?;
                    debug!("RX: {:?}", message);
                    self.to_coordinator.send(message)?;
                }
                Err(e) => {
                    // should automatically reconnect on next poll()..
                    error!("{}", e);
                }
                _ => {} // keepalives etc
            }
        }
    }

    fn parse_message(publish: Publish) -> Result<Message> {
        Ok(Message {
            topic: publish.topic,
            payload: String::from_utf8(publish.payload.to_vec())?,
        })
    }

    // coordinator -> mqtt
    async fn sender(&self, client: AsyncClient) -> Result<()> {
        let mut receiver = self.from_coordinator.subscribe();
        loop {
            let message = receiver.recv().await?;
            debug!("MQTT publishing: {:?}", message);
            client
                .publish(message.topic, QoS::AtLeastOnce, false, message.payload)
                .await?;
        }
    }
}
