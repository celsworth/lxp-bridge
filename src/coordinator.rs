use crate::prelude::*;

use lxp::packet::{DeviceFunction, ReadParam, TcpFunction, TranslatedData};

// the coordinator takes messages from both MQ and the inverter and decides
// what to do with them.
//
// usually this will just be relaying directly out the other side, but some
// messages need a bit of state storing, eg "enable ac charge" is actually
// two inverter messages.

pub struct Coordinator {
    config: Rc<Config>,
    pub inverter: Inverter,
    pub mqtt: mqtt::Mqtt,
    from_inverter: lxp::inverter::PacketSender,
    to_inverter: lxp::inverter::PacketSender,
    from_mqtt: mqtt::MessageSender,
    to_mqtt: mqtt::MessageSender,
}

impl Coordinator {
    pub fn new(config: Config) -> Self {
        let from_inverter = Self::channel();
        let to_inverter = Self::channel();
        let from_mqtt = Self::channel();
        let to_mqtt = Self::channel();

        let config = Rc::new(config);

        // process messages from/to inverter, passing Packets
        let inverter = Inverter::new(
            Rc::clone(&config),
            to_inverter.clone(),
            from_inverter.clone(),
        );

        // process messages from/to MQTT, passing Messages
        let mqtt = mqtt::Mqtt::new(Rc::clone(&config), to_mqtt.clone(), from_mqtt.clone());

        Self {
            config,
            inverter,
            mqtt,
            from_inverter,
            to_inverter,
            from_mqtt,
            to_mqtt,
        }
    }

    pub async fn start(&self) -> Result<()> {
        loop {
            let f1 = self.inverter_receiver();
            let f2 = self.mqtt_receiver();

            let _ = futures::try_join!(f1, f2); // ignore result, just re-loop and restart
        }
    }

    async fn mqtt_receiver(&self) -> Result<()> {
        let mut receiver = self.from_mqtt.subscribe();

        loop {
            let message = receiver.recv().await?;

            match self.message_to_command(message) {
                Ok(command) => {
                    debug!("parsed command {:?}", command);

                    let result = self.process_command(&command).await;

                    let reply = mqtt::Message {
                        topic: self.command_to_mqtt_topic(&command),
                        payload: if result.is_ok() { "OK" } else { "FAIL" }.to_string(),
                    };
                    self.to_mqtt.send(reply)?;

                    if let Err(err) = result {
                        error!("{:?}: {:?}", command, err);
                    }
                }
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }
    }

    async fn process_command(&self, command: &Command) -> Result<()> {
        use lxp::packet::Register;
        use lxp::packet::RegisterBit;
        use Command::*;

        match *command {
            ReadHold(register) => self.read_register(register).await,
            ReadParam(register) => self.read_param(register).await,
            SetHold(register, value) => self.set_register(register, value).await,
            AcCharge(enable) => {
                self.update_register(
                    Register::Register21.into(),
                    RegisterBit::AcChargeEnable,
                    enable,
                )
                .await
            }
            ForcedDischarge(enable) => {
                self.update_register(
                    Register::Register21.into(),
                    RegisterBit::ForcedDischargeEnable,
                    enable,
                )
                .await
            }
            ChargeRate(pct) => {
                self.set_register(Register::ChargePowerPercentCmd.into(), pct)
                    .await
            }
            DischargeRate(pct) => {
                self.set_register(Register::DischgPowerPercentCmd.into(), pct)
                    .await
            }

            AcChargeRate(pct) => {
                self.set_register(Register::AcChargePowerCmd.into(), pct)
                    .await
            }

            AcChargeSocLimit(pct) => {
                self.set_register(Register::AcChargeSocLimit.into(), pct)
                    .await
            }

            DischargeCutoffSocLimit(pct) => {
                self.set_register(Register::DischgCutOffSocEod.into(), pct)
                    .await
            }
        }
    }

    async fn read_register(&self, register: u16) -> Result<()> {
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.config.inverter.datalog.to_owned(),
            device_function: DeviceFunction::ReadHold,
            inverter: self.config.inverter.serial.to_owned(),
            register,
            values: vec![1],
        });

        self.to_inverter.send(Some(packet))?;

        // note that we don't have to wait for a reply and send over MQTT here.
        // inverter_receiver will do it for us!

        Ok(())
    }

    async fn read_param(&self, register: u16) -> Result<()> {
        let packet = Packet::ReadParam(ReadParam {
            datalog: self.config.inverter.datalog.to_owned(),
            register,
            values: vec![1],
        });

        self.to_inverter.send(Some(packet))?;

        // note that we don't have to wait for a reply and send over MQTT here.
        // inverter_receiver will do it for us!

        Ok(())
    }

    async fn set_register(&self, register: u16, value: u16) -> Result<()> {
        let mut receiver = self.from_inverter.subscribe();

        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.config.inverter.datalog.to_owned(),
            device_function: DeviceFunction::WriteSingle,
            inverter: self.config.inverter.serial.to_owned(),
            register,
            values: value.to_le_bytes().to_vec(),
        });
        self.to_inverter.send(Some(packet))?;

        let packet =
            Self::wait_for_packet(&mut receiver, DeviceFunction::WriteSingle, register).await?;
        if packet.value() != value {
            return Err(anyhow!(
                "failed to set register {}, got back value {} (wanted {})",
                register,
                packet.value(),
                value
            ));
        }

        Ok(())
    }

    async fn update_register(
        &self,
        register: u16,
        bit: lxp::packet::RegisterBit,
        enable: bool,
    ) -> Result<()> {
        let mut receiver = self.from_inverter.subscribe();

        // get register from inverter
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.config.inverter.datalog.to_owned(),
            device_function: DeviceFunction::ReadHold,
            inverter: self.config.inverter.serial.to_owned(),
            register,
            values: vec![1],
        });
        self.to_inverter.send(Some(packet))?;

        let packet =
            Self::wait_for_packet(&mut receiver, DeviceFunction::ReadHold, register).await?;
        let value = if enable {
            packet.value() | u16::from(bit)
        } else {
            packet.value() & !u16::from(bit)
        };

        // new packet to set register with a new value
        let values = value.to_le_bytes().to_vec();
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.config.inverter.datalog.to_owned(),
            device_function: DeviceFunction::WriteSingle,
            inverter: self.config.inverter.serial.to_owned(),
            register,
            values,
        });
        self.to_inverter.send(Some(packet))?;

        let packet =
            Self::wait_for_packet(&mut receiver, DeviceFunction::WriteSingle, register).await?;
        if packet.value() != value {
            return Err(anyhow!(
                "failed to update register {:?}, got back value {} (wanted {})",
                register,
                packet.value(),
                value
            ));
        }

        Ok(())
    }

    async fn wait_for_packet(
        receiver: &mut broadcast::Receiver<Option<Packet>>,
        function: DeviceFunction,
        register: u16,
    ) -> Result<Packet> {
        let start = std::time::Instant::now();

        loop {
            match receiver.try_recv() {
                Ok(Some(Packet::TranslatedData(td))) => {
                    if td.register == register && td.device_function == function {
                        return Ok(Packet::TranslatedData(td));
                    }
                }
                Ok(Some(_)) => {} // TODO ReadParam and WriteParam

                Ok(None) => return Err(anyhow!("inverter disconnect?")),
                Err(broadcast::error::TryRecvError::Empty) => {} // ignore and loop
                Err(err) => return Err(anyhow!("try_recv error: {:?}", err)),
            }

            if start.elapsed().as_secs() > 5 {
                return Err(anyhow!("wait_for_packet register={:?} - timeout", register));
            }

            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }

    async fn inverter_receiver(&self) -> Result<()> {
        let mut receiver = self.from_inverter.subscribe();

        // this loop holds no state so doesn't care about inverter reconnects
        loop {
            if let Some(packet) = receiver.recv().await? {
                debug!("RX: {:?}", packet);

                if let Packet::TranslatedData(td) = &packet {
                    // temporary special greppable logging for Param packets as I try to
                    // work out what they do :)
                    if td.tcp_function() == TcpFunction::ReadParam
                        || td.tcp_function() == TcpFunction::WriteParam
                    {
                        warn!("got a Param packet! {:?}", td);
                    }
                }

                // returns a Vec of messages to send. could be none;
                // not every packet produces an MQ message (eg, heartbeats),
                // and some produce >1 (multi-register ReadHold)
                for message in mqtt::Message::from_packet(packet)? {
                    self.to_mqtt.send(message)?;
                }
            }
        }
    }

    fn channel<T: Clone>() -> broadcast::Sender<T> {
        let (tx, _) = broadcast::channel(128);
        tx
    }

    // borrow a Command and return the MQTT topic we should send our result on
    fn command_to_mqtt_topic(&self, command: &Command) -> String {
        use Command::*;

        let r = match command {
            ReadHold(register) => format!("read/hold/{}", register),
            ReadParam(register) => format!("read/param/{}", register),
            SetHold(register, _) => format!("set/hold/{}", register),
            AcCharge(_) => "set/ac_charge".to_string(),
            ForcedDischarge(_) => "set/forced_discharge".to_string(),
            ChargeRate(_) => "set/charge_rate_pct".to_string(),
            DischargeRate(_) => "set/discharge_rate_pct".to_string(),
            AcChargeRate(_) => "set/ac_charge_rate_pct".to_string(),
            AcChargeSocLimit(_) => "set/ac_charge_soc_limit_pct".to_string(),
            DischargeCutoffSocLimit(_) => "set/discharge_cutoff_soc_limit_pct".to_string(),
        };

        format!("lxp/result/{}/{}", self.config.inverter.datalog, r)
    }

    // consume an incoming mqtt message (from lxp/cmd/..) and return a populated Command
    fn message_to_command(&self, message: mqtt::Message) -> Result<Command> {
        use Command::*;

        let parts: Vec<&str> = message.topic.split('/').collect();

        // bail if the topic is too short to handle
        if parts.len() < 3 {
            return Err(anyhow!(
                "ignoring badly formed MQTT topic: {}",
                message.topic
            ));
        }

        // bail if next part isn't our inverter's datalog
        if parts[2] != self.config.inverter.datalog {
            warn!("ignoring message for another datalog");
            return Err(anyhow!("ignoring"));
        }

        let parts = &parts[3..]; // drop lxp/cmd/{datalog}

        match parts {
            // TODO: read input
            ["read", "hold", register] => Ok(ReadHold(register.parse()?)),
            ["read", "param", register] => Ok(ReadParam(register.parse()?)),
            ["set", "hold", register] => Ok(SetHold(register.parse()?, message.payload_int()?)),
            ["set", "ac_charge"] => Ok(AcCharge(message.payload_bool())),

            ["set", "forced_discharge"] => Ok(ForcedDischarge(message.payload_bool())),

            ["set", "charge_rate_pct"] => Ok(ChargeRate(message.payload_int()?)),
            ["set", "discharge_rate_pct"] => Ok(DischargeRate(message.payload_int()?)),
            ["set", "ac_charge_rate_pct"] => Ok(AcChargeRate(message.payload_int()?)),

            ["set", "ac_charge_soc_limit_pct"] => Ok(AcChargeSocLimit(message.payload_int()?)),

            ["set", "discharge_cutoff_soc_limit_pct"] => {
                Ok(DischargeCutoffSocLimit(message.payload_int()?))
            }

            [..] => Err(anyhow!("unhandled: {:?}", parts)),
        }
    }
}
