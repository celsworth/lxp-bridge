use crate::prelude::*;

use lxp::inverter::{PacketChannelData, WaitForReply};
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
    pub inverter: Inverter,
    pub mqtt: mqtt::Mqtt,
    pub influx: influx::Influx,
    from_inverter: lxp::inverter::PacketChannelSender,
    to_inverter: lxp::inverter::PacketChannelSender,
    from_mqtt: mqtt::MessageSender,
    to_mqtt: mqtt::MessageSender,
    to_influx: influx::ValueSender,
}

impl Coordinator {
    pub fn new(config: Config) -> Self {
        let from_inverter = Self::channel();
        let to_inverter = Self::channel();
        let from_mqtt = Self::channel();
        let to_mqtt = Self::channel();
        let to_influx = Self::channel();

        let config = Rc::new(config);

        // process messages from/to inverter, passing Packets
        let inverter = Inverter::new(
            Rc::clone(&config),
            to_inverter.clone(),
            from_inverter.clone(),
        );

        // process messages from/to MQTT, passing Messages
        let mqtt = mqtt::Mqtt::new(Rc::clone(&config), to_mqtt.clone(), from_mqtt.clone());

        // push messages to Influx
        let influx = influx::Influx::new(Rc::clone(&config), to_influx.clone());

        Self {
            config,
            inverter,
            mqtt,
            influx,
            from_inverter,
            to_inverter,
            from_mqtt,
            to_mqtt,
            to_influx,
        }
    }

    pub async fn start(&self) -> Result<((), ())> {
        let f1 = self.inverter_receiver();
        let f2 = self.mqtt_receiver();

        futures::try_join!(f1, f2)
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
        use lxp::packet::Register;
        use lxp::packet::RegisterBit;
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
        let register = register.into();

        let packet = Packet::TranslatedData(TranslatedData {
            datalog: inverter.datalog,
            device_function: DeviceFunction::ReadInput,
            inverter: inverter.serial,
            register,
            values: count.to_le_bytes().to_vec(),
        });

        let mut receiver = self.from_inverter.subscribe();

        self.to_inverter
            .send(PacketChannelData::Packet(packet.clone()))?;

        let _ = receiver.wait_for_reply(&packet).await?;

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
            .send(PacketChannelData::Packet(packet.clone()))?;

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
            .send(PacketChannelData::Packet(packet.clone()))?;

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
            .send(PacketChannelData::Packet(packet.clone()))?;

        let packet = receiver.wait_for_reply(&packet).await?;
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
            .send(PacketChannelData::Packet(packet.clone()))?;

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
            .send(PacketChannelData::Packet(packet.clone()))?;

        let packet = receiver.wait_for_reply(&packet).await?;
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

    async fn inverter_receiver(&self) -> Result<()> {
        let mut receiver = self.from_inverter.subscribe();

        let mut inputs_store = InputsStore::new();

        // this loop holds no state so doesn't care about inverter reconnects
        loop {
            if let PacketChannelData::Packet(packet) = receiver.recv().await? {
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
                use lxp::packet::ReadInput;

                let entry = inputs_store
                    .entry(td.datalog)
                    .or_insert_with(lxp::packet::ReadInputs::default);

                match td.read_input()? {
                    ReadInput::ReadInput1(r1) => entry.set_read_input_1(r1),
                    ReadInput::ReadInput2(r2) => entry.set_read_input_2(r2),
                    ReadInput::ReadInput3(r3) => {
                        let datalog = r3.datalog;

                        entry.set_read_input_3(r3);

                        // make a serde_json::value to send to influx
                        if self.config.influx.enabled {
                            let influx_data = serde_json::to_value(&entry)?;
                            self.to_influx.send(influx_data)?;
                        }

                        if self.config.mqtt.enabled {
                            for message in mqtt::Message::for_inputs(entry, datalog) {
                                self.to_mqtt.send(message)?;
                            }
                        }
                    }
                }
            }
        }

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
