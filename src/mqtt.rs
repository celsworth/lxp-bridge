use crate::prelude::*;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, Publish, QoS};

// Message {{{
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub enum TargetInverter {
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

    pub fn for_input_all(
        inputs: &lxp::packet::ReadInputAll,
        datalog: lxp::inverter::Serial,
    ) -> Result<Message> {
        Ok(mqtt::Message {
            topic: format!("{}/inputs/all", datalog),
            payload: serde_json::to_string(&inputs)?,
        })
    }

    pub fn for_input(
        td: lxp::packet::TranslatedData,
        publish_individual: bool,
    ) -> Result<Vec<Message>> {
        use lxp::packet::ReadInput;

        let mut r = Vec::new();

        if publish_individual {
            for (register, value) in td.pairs() {
                r.push(mqtt::Message {
                    topic: format!("{}/input/{}", td.datalog, register),
                    payload: serde_json::to_string(&value)?,
                });
            }
        }

        match td.read_input() {
            Ok(ReadInput::ReadInputAll(r_all)) => r.push(mqtt::Message {
                topic: format!("{}/inputs/all", td.datalog),
                payload: serde_json::to_string(&r_all)?,
            }),
            Ok(ReadInput::ReadInput1(r1)) => r.push(mqtt::Message {
                topic: format!("{}/inputs/1", td.datalog),
                payload: serde_json::to_string(&r1)?,
            }),
            Ok(ReadInput::ReadInput2(r2)) => r.push(mqtt::Message {
                topic: format!("{}/inputs/2", td.datalog),
                payload: serde_json::to_string(&r2)?,
            }),
            Ok(ReadInput::ReadInput3(r3)) => r.push(mqtt::Message {
                topic: format!("{}/inputs/3", td.datalog),
                payload: serde_json::to_string(&r3)?,
            }),
            Err(x) => warn!("ignoring {:?}", x),
        }

        Ok(r)
    }

    pub fn to_command(&self, inverter: config::Inverter) -> Result<Command> {
        use Command::*;

        let (_datalog, parts) = self.split_cmd_topic()?;

        let r = match parts[..] {
            ["read", "inputs", "1"] => ReadInputs1(inverter),
            ["read", "inputs", "2"] => ReadInputs2(inverter),
            ["read", "inputs", "3"] => ReadInputs3(inverter),
            ["read", "input", register] => {
                ReadInput(inverter, register.parse()?, self.payload_int_or_1()?)
            }
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

            [..] => bail!("unhandled: {:?}", self),
        };

        Ok(r)
    }

    // given a cmd Message, return the datalog it is intended for.
    //
    // eg cmd/AB12345678/set/ac_charge => (AB12345678, ['set', 'ac_charge'])
    pub fn split_cmd_topic(&self) -> Result<(TargetInverter, Vec<&str>)> {
        let parts: Vec<&str> = self.topic.split('/').collect();

        // bail if the topic is too short to handle.
        // this *shouldn't* happen as our subscribe is for lxp/cmd/{datalog}/#
        if parts.len() < 2 {
            bail!("ignoring badly formed MQTT topic: {}", self.topic);
        }

        // parts[0] should be cmd
        let datalog = parts[1];
        let rest = parts[2..].to_vec();

        if datalog == "all" {
            Ok((TargetInverter::All, rest))
        } else {
            let serial = Serial::from_str(&datalog)?;
            Ok((TargetInverter::Serial(serial), rest))
        }
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

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Message(Message),
    Shutdown,
}

pub type Sender = broadcast::Sender<ChannelData>;

pub struct Mqtt {
    config: ConfigWrapper,
    shutdown: bool,
    channels: Channels,
}

impl Mqtt {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self {
            config,
            channels,
            shutdown: false,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let c = &self.config;

        if !c.mqtt().enabled() {
            info!("mqtt disabled, skipping");
            return Ok(());
        }

        let mut options = MqttOptions::new("lxp-bridge", c.mqtt().host(), c.mqtt().port());

        options.set_keep_alive(std::time::Duration::from_secs(60));
        if let (Some(u), Some(p)) = (c.mqtt().username(), c.mqtt().password()) {
            options.set_credentials(u, p);
        }

        info!(
            "initializing mqtt at {}:{}",
            c.mqtt().host(),
            c.mqtt().port()
        );

        let (client, eventloop) = AsyncClient::new(options, 10);

        futures::try_join!(
            self.setup(client.clone()),
            self.receiver(eventloop),
            self.sender(client)
        )?;

        Ok(())
    }

    pub fn stop(&mut self) {
        self.shutdown = true;

        let _ = self.channels.from_mqtt.send(ChannelData::Shutdown);
    }

    async fn setup(&self, client: AsyncClient) -> Result<()> {
        client
            .subscribe(
                format!("{}/cmd/all/#", self.config.mqtt().namespace()),
                QoS::AtMostOnce,
            )
            .await?;

        for inverter in self.config.enabled_inverters() {
            client
                .subscribe(
                    format!(
                        "{}/cmd/{}/#",
                        self.config.mqtt().namespace(),
                        inverter.datalog()
                    ),
                    QoS::AtMostOnce,
                )
                .await?;

            if self.config.mqtt().homeassistant().enabled() {
                let msgs = home_assistant::Config::all(&inverter, &self.config.mqtt())?;
                for msg in msgs.into_iter() {
                    let _ = client
                        .publish(&msg.topic, QoS::AtLeastOnce, true, msg.payload)
                        .await;
                }
            }
        }

        Ok(())
    }

    // mqtt -> coordinator
    async fn receiver(&self, mut eventloop: EventLoop) -> Result<()> {
        loop {
            if self.shutdown {
                break;
            }

            if let Ok(event) =
                tokio::time::timeout(std::time::Duration::from_secs(1), eventloop.poll()).await
            {
                match event {
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

        info!("receiver loop exiting");

        Ok(())
    }

    fn handle_message(&self, publish: Publish) -> Result<()> {
        // remove the namespace, including the first /
        // doing it this way means we don't break if namespace happens to contain a /
        let topic = publish.topic[self.config.mqtt().namespace().len() + 1..].to_owned();

        let message = Message {
            topic,
            payload: String::from_utf8(publish.payload.to_vec())?,
        };
        debug!("RX: {:?}", message);
        if self
            .channels
            .from_mqtt
            .send(ChannelData::Message(message))
            .is_err()
        {
            bail!("send(from_mqtt) failed - channel closed?");
        }

        Ok(())
    }

    // coordinator -> mqtt
    async fn sender(&self, client: AsyncClient) -> Result<()> {
        use ChannelData::*;

        let mut receiver = self.channels.to_mqtt.subscribe();

        loop {
            match receiver.recv().await? {
                Shutdown => break,
                Message(message) => {
                    let topic = format!("{}/{}", self.config.mqtt().namespace(), message.topic);
                    info!("publishing: {} = {}", topic, message.payload);
                    let _ = client
                        .publish(&topic, QoS::AtLeastOnce, false, message.payload)
                        .await
                        .map_err(|err| error!("publish {} failed: {:?} .. skipping", topic, err));
                }
            }
        }

        info!("sender loop exiting");

        Ok(())
    }
}
