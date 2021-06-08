use crate::prelude::*;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, Publish, QoS};

// Message {{{
#[derive(Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub struct MessageTopicParts {
    pub datalog: Datalog,
    pub parts: Vec<String>,
}

impl Message {
    pub fn to_command(&self, inverter: config::Inverter) -> Result<Command> {
        use Command::*;

        let parts: Vec<&str> = self.topic.split('/').collect();
        let _datalog = parts[1];
        let parts = &parts[2..];

        let r = match parts {
            // TODO: read input
            ["read", "hold", register] => ReadHold(inverter, register.parse()?),
            ["read", "param", register] => ReadParam(inverter, register.parse()?),
            ["set", "hold", register] => SetHold(inverter, register.parse()?, self.payload_int()?),
            ["set", "ac_charge"] => AcCharge(inverter, self.payload_bool()),

            ["set", "forced_discharge"] => ForcedDischarge(inverter, self.payload_bool()),
            ["set", "charge_rate_pct"] => ChargeRate(inverter, self.payload_int()?),
            ["set", "discharge_rate_pct"] => DischargeRate(inverter, self.payload_int()?),
            ["set", "ac_charge_rate_pct"] => AcChargeRate(inverter, self.payload_int()?),

            ["set", "ac_charge_soc_limit_pct"] => AcChargeSocLimit(inverter, self.payload_int()?),

            ["set", "discharge_cutoff_soc_limit_pct"] => {
                DischargeCutoffSocLimit(inverter, self.payload_int()?)
            }

            [..] => return Err(anyhow!("unhandled: {:?}", self)),
        };

        Ok(r)
    }
    // given a cmd Message, return the datalog it is intended for.
    //
    // eg cmd/AB12345678/ac_charge => AB12345678
    pub fn split_cmd_topic(&self) -> Result<MessageTopicParts> {
        // this starts at cmd/{datalog}/..
        let parts: Vec<String> = self.topic.split('/').map(|x| x.to_string()).collect();

        // bail if the topic is too short to handle.
        // this *shouldn't* happen as our subscribe is for lxp/cmd/{datalog}/#
        if parts.len() < 2 {
            return Err(anyhow!("ignoring badly formed MQTT topic: {}", self.topic));
        }

        Ok(MessageTopicParts {
            datalog: Datalog::from_str(&parts[1]),
            parts: parts[2..].to_vec(),
        })
    }

    pub fn payload_int(&self) -> Result<u16> {
        self.payload
            .parse()
            .map_err(|err| anyhow!("payload_int: {}", err))
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

        for inverter in self.config.inverters.iter() {
            client
                .subscribe(
                    format!("{}/cmd/{}/#", self.config.mqtt.namespace, inverter.datalog),
                    QoS::AtMostOnce,
                )
                .await?;
        }

        futures::try_join!(self.receiver(eventloop), self.sender(client))?;

        Ok(())
    }

    // mqtt -> coordinator
    async fn receiver(&self, mut eventloop: EventLoop) -> Result<()> {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(publish))) => {
                    self.handle_message(publish)?;
                }
                Err(e) => {
                    // should automatically reconnect on next poll()..
                    error!("{}", e);
                }
                _ => {} // keepalives etc
            }
        }
    }

    fn handle_message(&self, publish: Publish) -> Result<()> {
        // remove the namespace, including the first /
        // doing it this way means we don't break if namespace happens to contain a /
        let topic = publish.topic[self.config.mqtt.namespace.len() + 1..].to_owned();

        let message = Message {
            topic,
            payload: String::from_utf8(publish.payload.to_vec())?,
        };
        debug!("RX: {:?}", message);
        self.to_coordinator.send(message)?;

        Ok(())
    }

    // coordinator -> mqtt
    async fn sender(&self, client: AsyncClient) -> Result<()> {
        let mut receiver = self.from_coordinator.subscribe();
        loop {
            let message = receiver.recv().await?;

            // this is the only place that cares about the mqtt namespace; add it here
            let topic = format!("{}/{}", self.config.mqtt.namespace, message.topic);
            debug!("publishing: {} = {}", topic, message.payload);
            client
                .publish(topic, QoS::AtLeastOnce, false, message.payload)
                .await?;
        }
    }
}
