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

            let inverter = self.inverter_for_message(&message).unwrap();
            match message.to_command(inverter) {
                Ok(command) => {
                    debug!("parsed command {:?}", command);

                    let topic_reply = self.command_to_mqtt_topic(&command);
                    let result = self.process_command(command).await;

                    let reply = mqtt::Message {
                        topic: topic_reply,
                        payload: if result.is_ok() { "OK" } else { "FAIL" }.to_string(),
                    };
                    self.to_mqtt.send(reply)?;

                    //if let Err(err) = result {
                    //    error!("{:?}: {:?}", command, err);
                    //}
                }
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }
    }

    async fn process_command(&self, command: Command) -> Result<()> {
        use lxp::packet::Register;
        use lxp::packet::RegisterBit;
        use Command::*;

        match command {
            ReadHold(inverter, register) => self.read_register(inverter, register).await,
            ReadParam(inverter, register) => self.read_param(inverter, register).await,
            SetHold(inverter, register, value) => {
                self.set_register(inverter, register, value).await
            }
            AcCharge(inverter, enable) => {
                self.update_register(
                    inverter,
                    Register::Register21.into(),
                    RegisterBit::AcChargeEnable,
                    enable,
                )
                .await
            }
            ForcedDischarge(inverter, enable) => {
                self.update_register(
                    inverter,
                    Register::Register21.into(),
                    RegisterBit::ForcedDischargeEnable,
                    enable,
                )
                .await
            }
            ChargeRate(inverter, pct) => {
                self.set_register(inverter, Register::ChargePowerPercentCmd.into(), pct)
                    .await
            }
            DischargeRate(inverter, pct) => {
                self.set_register(inverter, Register::DischgPowerPercentCmd.into(), pct)
                    .await
            }

            AcChargeRate(inverter, pct) => {
                self.set_register(inverter, Register::AcChargePowerCmd.into(), pct)
                    .await
            }

            AcChargeSocLimit(inverter, pct) => {
                self.set_register(inverter, Register::AcChargeSocLimit.into(), pct)
                    .await
            }

            DischargeCutoffSocLimit(inverter, pct) => {
                self.set_register(inverter, Register::DischgCutOffSocEod.into(), pct)
                    .await
            }
        }
    }

    async fn read_register(&self, inverter: config::Inverter, register: u16) -> Result<()> {
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::ReadHold,
            inverter: inverter.serial,
            register,
            values: vec![1, 0],
        });

        self.to_inverter.send((inverter.datalog, Some(packet)))?;

        // note that we don't have to wait for a reply and send over MQTT here.
        // inverter_receiver will do it for us!

        Ok(())
    }

    async fn read_param(&self, inverter: config::Inverter, register: u16) -> Result<()> {
        let packet = Packet::ReadParam(ReadParam {
            datalog: inverter.datalog,
            register,
            values: vec![], // unused
        });

        self.to_inverter.send((inverter.datalog, Some(packet)))?;

        // note that we don't have to wait for a reply and send over MQTT here.
        // inverter_receiver will do it for us!

        Ok(())
    }

    async fn set_register(
        &self,
        inverter: config::Inverter,
        register: u16,
        value: u16,
    ) -> Result<()> {
        let mut receiver = self.from_inverter.subscribe();

        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::WriteSingle,
            inverter: inverter.serial.to_owned(),
            register,
            values: value.to_le_bytes().to_vec(),
        });
        self.to_inverter
            .send((inverter.datalog.to_owned(), Some(packet)))?;

        let packet = Self::wait_for_packet(
            inverter.datalog,
            &mut receiver,
            DeviceFunction::WriteSingle,
            register,
        )
        .await?;
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
        inverter: config::Inverter,
        register: u16,
        bit: lxp::packet::RegisterBit,
        enable: bool,
    ) -> Result<()> {
        let mut receiver = self.from_inverter.subscribe();

        // get register from inverter
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::ReadHold,
            inverter: inverter.serial.to_owned(),
            register,
            values: vec![1, 0],
        });
        self.to_inverter
            .send((inverter.datalog.to_owned(), Some(packet)))?;

        let packet = Self::wait_for_packet(
            inverter.datalog,
            &mut receiver,
            DeviceFunction::ReadHold,
            register,
        )
        .await?;
        let value = if enable {
            packet.value() | u16::from(bit)
        } else {
            packet.value() & !u16::from(bit)
        };

        // new packet to set register with a new value
        let values = value.to_le_bytes().to_vec();
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog.to_owned(),
            device_function: DeviceFunction::WriteSingle,
            inverter: inverter.serial.to_owned(),
            register,
            values,
        });
        self.to_inverter
            .send((inverter.datalog.to_owned(), Some(packet)))?;

        let packet = Self::wait_for_packet(
            inverter.datalog,
            &mut receiver,
            DeviceFunction::WriteSingle,
            register,
        )
        .await?;
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
        datalog: Datalog,
        receiver: &mut broadcast::Receiver<lxp::inverter::ChannelContent>,
        function: DeviceFunction,
        register: u16,
    ) -> Result<Packet> {
        let start = std::time::Instant::now();

        loop {
            match receiver.try_recv() {
                Ok((inverter_datalog, Some(Packet::TranslatedData(td)))) => {
                    if inverter_datalog == datalog
                        && td.register == register
                        && td.device_function == function
                    {
                        return Ok(Packet::TranslatedData(td));
                    }
                }
                Ok((_inverter_datalog, Some(_))) => {} // TODO ReadParam and WriteParam

                Ok((inverter_datalog, None)) => {
                    if inverter_datalog == datalog {
                        return Err(anyhow!("inverter disconnect?"));
                    }
                }

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
            if let (_datalog, Some(packet)) = receiver.recv().await? {
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
                for message in self.packet_to_messages(packet)? {
                    self.to_mqtt.send(message)?;
                }
            }
        }
    }

    fn packet_to_messages(&self, packet: Packet) -> Result<Vec<mqtt::Message>> {
        let mut r = Vec::new();

        let prefix = packet.datalog().to_string();

        match packet {
            Packet::Heartbeat(_) => {}
            Packet::TranslatedData(t) => match t.device_function {
                DeviceFunction::ReadHold => {
                    for (register, value) in t.pairs() {
                        r.push(mqtt::Message {
                            topic: format!("{}/hold/{}", prefix, register),
                            payload: serde_json::to_string(&value)?,
                        });
                    }
                }
                DeviceFunction::ReadInput => match t.register {
                    0 => r.push(mqtt::Message {
                        topic: format!("{}/inputs/1", prefix),
                        payload: serde_json::to_string(&t.read_input1()?)?,
                    }),
                    40 => r.push(mqtt::Message {
                        topic: format!("{}/inputs/2", prefix),
                        payload: serde_json::to_string(&t.read_input2()?)?,
                    }),
                    80 => r.push(mqtt::Message {
                        topic: format!("{}/inputs/3", prefix),
                        payload: serde_json::to_string(&t.read_input3()?)?,
                    }),
                    _ => {
                        warn!("unhandled ReadInput register={}", t.register);
                    }
                },
                DeviceFunction::WriteSingle => {}
                DeviceFunction::WriteMulti => {}
            },
            Packet::ReadParam(rp) => {
                for (register, value) in rp.pairs() {
                    r.push(mqtt::Message {
                        topic: format!("{}/param/{}", prefix, register),
                        payload: serde_json::to_string(&value)?,
                    });
                }
            }
        };

        Ok(r)
    }

    fn channel<T: Clone>() -> broadcast::Sender<T> {
        let (tx, _) = broadcast::channel(128);
        tx
    }

    // borrow a Command and return the MQTT topic we should send our result on
    fn command_to_mqtt_topic(&self, command: &Command) -> String {
        use Command::*;

        let rest = match command {
            ReadHold(inverter, register) => format!("{}/read/hold/{}", inverter.datalog, register),
            ReadParam(inverter, register) => {
                format!("{}/read/param/{}", inverter.datalog, register)
            }
            SetHold(inverter, register, _) => format!("{}/set/hold/{}", inverter.datalog, register),
            AcCharge(inverter, _) => format!("{}/set/ac_charge", inverter.datalog),
            ForcedDischarge(inverter, _) => format!("{}/set/forced_discharge", inverter.datalog),
            ChargeRate(inverter, _) => format!("{}/set/charge_rate_pct", inverter.datalog),
            DischargeRate(inverter, _) => format!("{}/set/discharge_rate_pct", inverter.datalog),
            AcChargeRate(inverter, _) => format!("{}/set/ac_charge_rate_pct", inverter.datalog),
            AcChargeSocLimit(inverter, _) => {
                format!("{}/set/ac_charge_soc_limit_pct", inverter.datalog)
            }
            DischargeCutoffSocLimit(inverter, _) => {
                format!("{}/set/discharge_cutoff_soc_limit_pct", inverter.datalog)
            }
        };

        format!("result/{}", rest)
    }

    // find the inverter in our config for the given message.
    // this can then be put into a Command
    fn inverter_for_message(&self, message: &mqtt::Message) -> Option<config::Inverter> {
        // TODO is this ok()? sufficient? might be throwing away an error
        let r = message.split_cmd_topic().ok()?;

        // search for inverter datalog in our config
        self.config
            .inverters
            .iter()
            .cloned()
            .find(|i| i.datalog == r.datalog)
    }
}
