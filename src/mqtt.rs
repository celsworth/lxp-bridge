use crate::prelude::*;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, Publish, QoS};

// Message {{{
#[derive(Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub struct MessageTopicParts {
    pub datalog: Serial,
    pub parts: Vec<String>,
}

pub enum SerialOrAll {
    Serial(Serial),
    All,
}

impl Message {
    pub fn for_param(rp: lxp::packet::ReadParam) -> Result<Vec<Message>> {
        let mut r = Vec::new();

        for (register, value) in rp.pairs() {
            r.push(mqtt::Message {
                topic: format!("{}/param/{}", rp.datalog, register),
                payload: serde_json::to_string(&value)?,
            });
        }

        Ok(r)
    }

    pub fn for_hold(td: lxp::packet::TranslatedData) -> Result<Vec<Message>> {
        let mut r = Vec::new();

        for (register, value) in td.pairs() {
            r.push(mqtt::Message {
                topic: format!("{}/hold/{}", td.datalog, register),
                payload: serde_json::to_string(&value)?,
            });
        }

        Ok(r)
    }

    pub fn for_inputs(
        inputs: &lxp::packet::ReadInputs,
        datalog: lxp::inverter::Serial,
    ) -> Vec<Message> {
        let payload = serde_json::to_string(&inputs).unwrap();

        vec![mqtt::Message {
            topic: format!("{}/inputs/all", datalog),
            payload,
        }]
    }

    pub fn for_input(td: lxp::packet::TranslatedData) -> Result<Vec<Message>> {
        use lxp::packet::ReadInput;

        let mut r = Vec::new();

        match td.read_input()? {
            ReadInput::ReadInput1(r1) => r.push(mqtt::Message {
                topic: format!("{}/inputs/1", td.datalog),
                payload: serde_json::to_string(&r1)?,
            }),
            ReadInput::ReadInput2(r2) => r.push(mqtt::Message {
                topic: format!("{}/inputs/2", td.datalog),
                payload: serde_json::to_string(&r2)?,
            }),
            ReadInput::ReadInput3(r3) => r.push(mqtt::Message {
                topic: format!("{}/inputs/3", td.datalog),
                payload: serde_json::to_string(&r3)?,
            }),
        }

        Ok(r)
    }

    pub fn to_command(&self, inverter: config::Inverter) -> Result<Command> {
        use Command::*;

        let parts: Vec<&str> = self.topic.split('/').collect();
        let _datalog = parts[1];
        let parts = &parts[2..];

        let r = match parts {
            ["read", "inputs", "1"] => ReadInputs(inverter, 0, 40),
            ["read", "inputs", "2"] => ReadInputs(inverter, 40, 40),
            ["read", "inputs", "3"] => ReadInputs(inverter, 80, 40),
            ["read", "hold", register] => {
                ReadHold(inverter, register.parse()?, self.payload_int_or_1()?)
            }
            ["read", "param", register] => ReadParam(inverter, register.parse()?),
            ["set", "hold", register] => SetHold(inverter, register.parse()?, self.payload_int()?),
            // TODO: set param
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
    pub fn split_cmd_topic(&self) -> Result<SerialOrAll> {
        // this starts at cmd/{datalog}/..
        let parts: Vec<String> = self.topic.split('/').map(|x| x.to_string()).collect();

        // bail if the topic is too short to handle.
        // this *shouldn't* happen as our subscribe is for lxp/cmd/{datalog}/#
        if parts.len() < 2 {
            return Err(anyhow!("ignoring badly formed MQTT topic: {}", self.topic));
        }

        if parts[1] == "all" {
            return Ok(SerialOrAll::All);
        }

        let serial = Serial::from_str(&parts[1])?;
        Ok(SerialOrAll::Serial(serial))
    }

    pub fn payload_int_or_1(&self) -> Result<u16> {
        self.payload_int().or(Ok(1))
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

        if !m.enabled {
            info!("mqtt disabled, skipping");
            return Ok(());
        }

        let mut options = MqttOptions::new("lxp-bridge", &m.host, m.port);

        options.set_keep_alive(std::time::Duration::from_secs(60));
        if let (Some(u), Some(p)) = (&m.username, &m.password) {
            options.set_credentials(u, p);
        }

        info!("initializing mqtt at {}:{}", &m.host, m.port);

        let (client, eventloop) = AsyncClient::new(options, 10);

        futures::try_join!(
            self.setup(client.clone()),
            self.receiver(eventloop),
            self.sender(client)
        )?;

        Ok(())
    }

    async fn setup(&self, client: AsyncClient) -> Result<()> {
        client
            .subscribe(
                format!("{}/cmd/all/#", self.config.mqtt.namespace),
                QoS::AtMostOnce,
            )
            .await?;

        for inverter in self.config.enabled_inverters() {
            client
                .subscribe(
                    format!("{}/cmd/{}/#", self.config.mqtt.namespace, inverter.datalog),
                    QoS::AtMostOnce,
                )
                .await?;

            if self.config.mqtt.homeassistant.enabled {
                let msgs = home_assistant::Config::all(inverter, &self.config.mqtt)?;
                for msg in msgs.into_iter() {
                    let _ = client
                        .publish(&msg.topic, QoS::AtLeastOnce, false, msg.payload)
                        .await;
                }
            }
        }

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
                    info!("reconnecting in 5s");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
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

            let topic = format!("{}/{}", self.config.mqtt.namespace, message.topic);
            debug!("publishing: {} = {}", topic, message.payload);
            let _ = client
                .publish(&topic, QoS::AtLeastOnce, false, message.payload)
                .await
                .map_err(|err| error!("publish {} failed: {:?} .. skipping", topic, err));
        }
    }
}
