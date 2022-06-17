use crate::prelude::*;

pub mod commands;

use lxp::inverter::WaitForReply;
use lxp::packet::{DeviceFunction, ReadParam, TcpFunction, TranslatedData};

pub type InputsStore = std::collections::HashMap<Serial, lxp::packet::ReadInputs>;

// the coordinator takes messages from both MQ and the inverter and decides
// what to do with them.
//
// usually this will just be relaying directly out the other side, but some
// messages need a bit of state storing, eg "enable ac charge" is actually
// two inverter messages.

pub struct Coordinator {
    config: Rc<Config>,
    inverters: Vec<Inverter>,
    pub mqtt: mqtt::Mqtt,
    pub influx: influx::Influx,
    databases: Vec<Database>,
    have_enabled_databases: bool,
    from_inverter: lxp::inverter::Sender,
    to_inverter: lxp::inverter::Sender,
    from_mqtt: mqtt::MessageSender,
    to_mqtt: mqtt::MessageSender,
    to_influx: influx::Sender,
    to_database: database::Sender,
}

impl Coordinator {
    pub fn new(config: Config) -> Self {
        let from_inverter = Self::channel();
        let to_inverter = Self::channel();
        let from_mqtt = Self::channel();
        let to_mqtt = Self::channel();
        let to_influx = Self::channel();
        let to_database = Self::channel();

        let config = Rc::new(config);

        // process messages from/to inverter, passing Packets
        let inverters = config
            .enabled_inverters()
            .cloned()
            .map(|inverter| Inverter::new(inverter, to_inverter.clone(), from_inverter.clone()))
            .collect();

        // process messages from/to MQTT, passing Messages
        let mqtt = mqtt::Mqtt::new(Rc::clone(&config), to_mqtt.clone(), from_mqtt.clone());

        // push messages to Influx
        let influx = influx::Influx::new(Rc::clone(&config), to_influx.clone());

        // push messages to databases
        let databases = config
            .enabled_databases()
            .cloned()
            .map(|database| Database::new(database, to_database.clone()))
            .collect();
        let have_enabled_databases = config.enabled_databases().count() > 0;

        Self {
            config,
            inverters,
            mqtt,
            influx,
            databases,
            have_enabled_databases,
            from_inverter,
            to_inverter,
            from_mqtt,
            to_mqtt,
            to_influx,
            to_database,
        }
    }

    pub async fn start(&self) -> Result<()> {
        futures::try_join!(
            self.inverter_receiver(),
            self.mqtt_receiver(),
            self.start_inverters(),
            self.mqtt.start(),
            self.influx.start(),
            self.start_databases()
        )?;

        Ok(())
    }

    async fn start_inverters(&self) -> Result<()> {
        let futures = self.inverters.iter().map(|i| i.start());

        futures::future::join_all(futures).await;

        Ok(())
    }

    async fn start_databases(&self) -> Result<()> {
        let futures = self.databases.iter().map(|d| d.start());

        futures::future::join_all(futures).await;

        Ok(())
    }

    async fn mqtt_receiver(&self) -> Result<()> {
        let mut receiver = self.from_mqtt.subscribe();

        loop {
            let message = receiver.recv().await?;

            let _ = self.process_message(message).await;
        }
    }

    async fn process_message(&self, message: mqtt::Message) -> Result<()> {
        for inverter in self.config.inverters_for_message(&message)? {
            match message.to_command(inverter) {
                Ok(command) => {
                    debug!("parsed command {:?}", command);

                    let topic_reply = command.to_result_topic();
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

        Ok(())
    }

    async fn process_command(&self, command: Command) -> Result<()> {
        use lxp::packet::{Register, RegisterBit};
        use Command::*;

        match command {
            ReadInputs(inverter, register, count) => {
                self.read_inputs(inverter, register, count).await
            }
            ReadHold(inverter, register, count) => self.read_hold(inverter, register, count).await,
            ReadParam(inverter, register) => self.read_param(inverter, register).await,
            SetHold(inverter, register, value) => self.set_hold(inverter, register, value).await,
            AcCharge(inverter, enable) => {
                self.update_hold(
                    inverter,
                    Register::Register21,
                    RegisterBit::AcChargeEnable,
                    enable,
                )
                .await
            }
            ForcedDischarge(inverter, enable) => {
                self.update_hold(
                    inverter,
                    Register::Register21,
                    RegisterBit::ForcedDischargeEnable,
                    enable,
                )
                .await
            }
            ChargeRate(inverter, pct) => {
                self.set_hold(inverter, Register::ChargePowerPercentCmd, pct)
                    .await
            }
            DischargeRate(inverter, pct) => {
                self.set_hold(inverter, Register::DischgPowerPercentCmd, pct)
                    .await
            }

            AcChargeRate(inverter, pct) => {
                self.set_hold(inverter, Register::AcChargePowerCmd, pct)
                    .await
            }

            AcChargeSocLimit(inverter, pct) => {
                self.set_hold(inverter, Register::AcChargeSocLimit, pct)
                    .await
            }

            DischargeCutoffSocLimit(inverter, pct) => {
                self.set_hold(inverter, Register::DischgCutOffSocEod, pct)
                    .await
            }
        }
    }

    async fn read_inputs<U>(
        &self,
        inverter: config::Inverter,
        register: U,
        count: u16,
    ) -> Result<()>
    where
        U: Into<u16>,
    {
        commands::read_inputs::ReadInputs::new(
            self.from_inverter.clone(),
            self.to_inverter.clone(),
            inverter.clone(),
            register,
            count,
        )
        .run()
        .await?;

        Ok(())
    }

    async fn read_hold<U>(&self, inverter: config::Inverter, register: U, count: u16) -> Result<()>
    where
        U: Into<u16>,
    {
        let register = register.into();

        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::ReadHold,
            inverter: inverter.serial,
            register,
            values: count.to_le_bytes().to_vec(),
        });

        let mut receiver = self.from_inverter.subscribe();

        self.to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        let _ = receiver.wait_for_reply(&packet).await?;

        Ok(())
    }

    async fn read_param<U>(&self, inverter: config::Inverter, register: U) -> Result<()>
    where
        U: Into<u16>,
    {
        let register = register.into();
        let packet = Packet::ReadParam(ReadParam {
            datalog: inverter.datalog,
            register,
            values: vec![], // unused
        });

        let mut receiver = self.from_inverter.subscribe();

        self.to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        let _ = receiver.wait_for_reply(&packet).await?;

        Ok(())
    }

    async fn set_hold<U>(&self, inverter: config::Inverter, register: U, value: u16) -> Result<()>
    where
        U: Into<u16>,
    {
        let mut receiver = self.from_inverter.subscribe();
        let register = register.into();

        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::WriteSingle,
            inverter: inverter.serial,
            register,
            values: value.to_le_bytes().to_vec(),
        });

        self.to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        let packet = receiver.wait_for_reply(&packet).await?;
        if packet.value() != value {
            bail!(
                "failed to set register {}, got back value {} (wanted {})",
                register,
                packet.value(),
                value
            );
        }

        Ok(())
    }

    async fn update_hold<U>(
        &self,
        inverter: config::Inverter,
        register: U,
        bit: lxp::packet::RegisterBit,
        enable: bool,
    ) -> Result<()>
    where
        U: Into<u16>,
    {
        let mut receiver = self.from_inverter.subscribe();
        let register = register.into();

        // get register from inverter
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::ReadHold,
            inverter: inverter.serial,
            register,
            values: vec![1, 0],
        });

        self.to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        let packet = receiver.wait_for_reply(&packet).await?;
        let value = if enable {
            packet.value() | u16::from(bit)
        } else {
            packet.value() & !u16::from(bit)
        };

        // new packet to set register with a new value
        let values = value.to_le_bytes().to_vec();
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::WriteSingle,
            inverter: inverter.serial,
            register,
            values,
        });
        self.to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        let packet = receiver.wait_for_reply(&packet).await?;
        if packet.value() != value {
            bail!(
                "failed to update register {:?}, got back value {} (wanted {})",
                register,
                packet.value(),
                value
            );
        }

        Ok(())
    }

    async fn inverter_receiver(&self) -> Result<()> {
        let mut receiver = self.from_inverter.subscribe();

        let mut inputs_store = InputsStore::new();

        // this loop holds no state so doesn't care about inverter reconnects
        loop {
            if let lxp::inverter::ChannelData::Packet(packet) = receiver.recv().await? {
                self.process_inverter_packet(packet, &mut inputs_store)
                    .await?;
            }
        }
    }

    async fn process_inverter_packet(
        &self,
        packet: lxp::packet::Packet,
        inputs_store: &mut InputsStore,
    ) -> Result<()> {
        debug!("RX: {:?}", packet);

        if let Packet::TranslatedData(td) = &packet {
            // temporary special greppable logging for Param packets as I try to
            // work out what they do :)
            if td.tcp_function() == TcpFunction::ReadParam
                || td.tcp_function() == TcpFunction::WriteParam
            {
                warn!("got a Param packet! {:?}", td);
            }

            // inputs_store handling. If we've received any ReadInput, update inputs_store
            // with the contents. If we got the third (of three) packets, send out the combined
            // MQTT message with all the data.
            if td.device_function == DeviceFunction::ReadInput {
                use lxp::packet::{ReadInput, ReadInputs};

                let entry = inputs_store
                    .entry(td.datalog)
                    .or_insert_with(ReadInputs::default);

                match td.read_input()? {
                    ReadInput::ReadInputAll(r_all) => {
                        // no need for MQTT here, done below
                        self.save_input_all(r_all).await?
                    }

                    ReadInput::ReadInput1(r1) => entry.set_read_input_1(r1),
                    ReadInput::ReadInput2(r2) => entry.set_read_input_2(r2),
                    ReadInput::ReadInput3(r3) => {
                        let datalog = r3.datalog;

                        entry.set_read_input_3(r3);

                        let input = entry.to_input_all();

                        if self.config.mqtt.enabled {
                            for message in mqtt::Message::for_input_all(&input, datalog) {
                                self.to_mqtt.send(message)?;
                            }
                        }

                        self.save_input_all(Box::new(input)).await?;
                    }
                }
            }
        }

        if self.config.mqtt.enabled {
            // returns a Vec of messages to send. could be none;
            // not every packet produces an MQ message (eg, heartbeats),
            // and some produce >1 (multi-register ReadHold)
            match Self::packet_to_messages(packet) {
                Ok(messages) => {
                    for message in messages {
                        self.to_mqtt.send(message)?;
                    }
                }
                Err(e) => {
                    // log error but avoid exiting loop as then we stop handling
                    // incoming packets. need better error handling here maybe?
                    error!("{}", e);
                }
            }
        }

        Ok(())
    }

    async fn save_input_all(&self, input: Box<lxp::packet::ReadInputAll>) -> Result<()> {
        if self.config.influx.enabled {
            let channel_data = influx::ChannelData::InputData(serde_json::to_value(&input)?);
            self.to_influx.send(channel_data)?;
        }

        if self.have_enabled_databases {
            let channel_data = database::ChannelData::ReadInputAll(input);
            self.to_database.send(channel_data)?;
        }

        Ok(())
    }

    fn packet_to_messages(packet: Packet) -> Result<Vec<mqtt::Message>> {
        match packet {
            Packet::Heartbeat(_) => Ok(Vec::new()), // always no message
            Packet::TranslatedData(td) => match td.device_function {
                DeviceFunction::ReadHold => mqtt::Message::for_hold(td),
                DeviceFunction::ReadInput => mqtt::Message::for_input(td),
                DeviceFunction::WriteSingle => mqtt::Message::for_hold(td),
                DeviceFunction::WriteMulti => Ok(Vec::new()), // TODO, for_hold might just work
            },
            Packet::ReadParam(rp) => mqtt::Message::for_param(rp),
        }
    }

    fn channel<T: Clone>() -> broadcast::Sender<T> {
        broadcast::channel(512).0 // we only need tx half
    }
}
