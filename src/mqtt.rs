use crate::prelude::*;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, Publish, QoS};

// Message {{{
#[derive(Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

impl Message {
    pub fn command_result(final_part: &str, success: bool) -> Self {
        let mut topic = "lxp/result/".to_owned();
        topic.push_str(&final_part);

        let payload = match success {
            true => "OK",
            false => "FAIL",
        }
        .to_string();

        Self { topic, payload }
    }

    pub fn from_packet(packet: Packet) -> Result<Vec<Self>> {
        use lxp::packet::PacketType;

        let mut r = Vec::new();

        match packet.packet_type() {
            PacketType::Heartbeat => {}
            PacketType::WriteSingle => {} // ignore for now..?
            PacketType::WriteMulti => {}  // ignore for now..?
            PacketType::ReadHold => {
                for pair in packet.pairs() {
                    r.push(Self {
                        topic: format!("lxp/hold/{}", pair.register),
                        payload: serde_json::to_string(&pair.value)?,
                    });
                }
            }
            PacketType::ReadInput1 => r.push(Self {
                topic: "lxp/inputs/1".to_owned(),
                payload: serde_json::to_string(&packet.read_input1()?)?,
            }),
            PacketType::ReadInput2 => r.push(Self {
                topic: "lxp/inputs/2".to_owned(),
                payload: serde_json::to_string(&packet.read_input2()?)?,
            }),
            PacketType::ReadInput3 => r.push(Self {
                topic: "lxp/inputs/3".to_owned(),
                payload: serde_json::to_string(&packet.read_input3()?)?,
            }),
        };

        Ok(r)
    }

    pub fn payload_percent(&self) -> Result<u16> {
        match self.payload.parse() {
            Ok(i) if i <= 100 => Ok(i),
            Ok(i) => Err(anyhow!("{} is too high (max 100)", i)),
            Err(err) => Err(anyhow!("payload_percent: {}", err)),
        }
    }

    pub fn payload_bool(&self) -> bool {
        matches!(
            self.payload.to_ascii_lowercase().as_str(),
            "1" | "t" | "true" | "on" | "y" | "yes"
        )
    }
} // }}}

pub type MessageSender = broadcast::Sender<Message>;

pub struct Mqtt {
    config: Rc<Config>,
    from_coordinator: MessageSender,
    to_coordinator: MessageSender,
}

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
