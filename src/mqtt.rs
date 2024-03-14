use crate::prelude::*;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, LastWill, MqttOptions, Publish, QoS};

// Message {{{
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Message {
    pub topic: String,
    pub retain: bool,
    pub payload: String,
}

pub enum TargetInverter {
    Serial(Serial),
    All,
}

impl Message {
    pub fn to_command(&self, inverter: config::Inverter) -> Result<Command> {
        use Command::*;

        let (_datalog, parts) = self.split_cmd_topic()?;

        let r = match parts[..] {
            ["read", "inputs", "1"] => ReadInputs(inverter, 1),
            ["read", "inputs", "2"] => ReadInputs(inverter, 2),
            ["read", "inputs", "3"] => ReadInputs(inverter, 3),
            ["read", "inputs", "4"] => ReadInputs(inverter, 4),
            ["read", "input", register] => {
                ReadInput(inverter, register.parse()?, self.payload_int_or_1()?)
            }
            ["read", "hold", register] => {
                ReadHold(inverter, register.parse()?, self.payload_int_or_1()?)
            }
            ["read", "param", register] => ReadParam(inverter, register.parse()?),
            ["read", "ac_charge", num] => ReadAcChargeTime(inverter, num.parse()?),
            ["read", "ac_first", num] => ReadAcFirstTime(inverter, num.parse()?),
            ["read", "charge_priority", num] => ReadChargePriorityTime(inverter, num.parse()?),
            ["read", "forced_discharge", num] => ReadForcedDischargeTime(inverter, num.parse()?),
            ["set", "hold", register] => SetHold(inverter, register.parse()?, self.payload_int()?),
            ["set", "param", register] => {
                WriteParam(inverter, register.parse()?, self.payload_int()?)
            }
            ["set", "ac_charge"] => AcCharge(inverter, self.payload_bool()),
            ["set", "ac_charge", num] => {
                SetAcChargeTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
            ["set", "ac_first", num] => {
                SetAcFirstTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
            ["set", "charge_priority"] => ChargePriority(inverter, self.payload_bool()),
            ["set", "charge_priority", num] => {
                SetChargePriorityTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
            ["set", "forced_discharge"] => ForcedDischarge(inverter, self.payload_bool()),
            ["set", "forced_discharge", num] => {
                SetForcedDischargeTime(inverter, num.parse()?, self.payload_start_end_time()?)
            }
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
            let serial = Serial::from_str(datalog)?;
            Ok((TargetInverter::Serial(serial), rest))
        }
    }

    // not entirely happy with this return type but it avoids needing to expose a struct for now
    fn payload_start_end_time(&self) -> Result<[u8; 4]> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct StartEndTime {
            start: String,
            end: String,
        }

        // {"start":"20:00", "end":"21:00"} -> [20, 0, 21, 0]
        let t = serde_json::from_str::<StartEndTime>(&self.payload)?;
        // split on : then make u8s to return in an array
        let start: Vec<&str> = t.start.split(':').collect();
        let end: Vec<&str> = t.end.split(':').collect();
        if start.len() != 2 || end.len() != 2 {
            bail!("badly formatted time, use HH:MM")
        }
        Ok([
            start[0].parse()?,
            start[1].parse()?,
            end[0].parse()?,
            end[1].parse()?,
        ])
    }

    fn payload_int_or_1(&self) -> Result<u16> {
        self.payload_int().or(Ok(1))
    }

    fn payload_int(&self) -> Result<u16> {
        self.payload
            .parse()
            .map_err(|err| anyhow!("payload_int: {}", err))
    }

    fn payload_bool(&self) -> bool {
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

        let will = LastWill {
            topic: self.lwt_topic(),
            message: bytes::Bytes::from("offline"),
            qos: QoS::AtLeastOnce,
            retain: true,
        };
        options.set_last_will(will);

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
            .publish(self.lwt_topic(), QoS::AtLeastOnce, true, "online")
            .await?;

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
                let ha = home_assistant::Config::new(&inverter, &self.config.mqtt());
                for msg in ha.all()?.into_iter() {
                    let _ = client
                        .publish(&msg.topic, QoS::AtLeastOnce, msg.retain, msg.payload)
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
            retain: publish.retain,
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
                        .publish(&topic, QoS::AtLeastOnce, message.retain, message.payload)
                        .await
                        .map_err(|err| error!("publish {} failed: {:?} .. skipping", topic, err));
                }
            }
        }

        info!("sender loop exiting");

        Ok(())
    }

    fn lwt_topic(&self) -> String {
        format!("{}/LWT", self.config.mqtt().namespace())
    }
}
