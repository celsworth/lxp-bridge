use crate::prelude::*;

pub mod commands;

use lxp::packet::{DeviceFunction, TcpFunction};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Shutdown,
}

pub struct Coordinator {
    config: ConfigWrapper,
    channels: Channels,
}

impl Coordinator {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self { config, channels }
    }

    pub async fn start(&self) -> Result<()> {
        futures::try_join!(self.inverter_receiver(), self.mqtt_receiver())?;

        Ok(())
    }

    pub fn stop(&self) {
        let _ = self
            .channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Shutdown);

        let _ = self.channels.from_mqtt.send(mqtt::ChannelData::Shutdown);
    }

    async fn mqtt_receiver(&self) -> Result<()> {
        let mut receiver = self.channels.from_mqtt.subscribe();

        while let mqtt::ChannelData::Message(message) = receiver.recv().await? {
            let _ = self.process_message(message).await;
        }

        Ok(())
    }

    async fn process_message(&self, message: mqtt::Message) -> Result<()> {
        for inverter in self.config.inverters_for_message(&message)? {
            match message.to_command(inverter) {
                Ok(command) => {
                    debug!("parsed command {:?}", command);

                    let topic_reply = command.to_result_topic();
                    let result = self.process_command(command).await;

                    let reply = mqtt::ChannelData::Message(mqtt::Message {
                        topic: topic_reply,
                        retain: false,
                        payload: if result.is_ok() { "OK" } else { "FAIL" }.to_string(),
                    });
                    if self.channels.to_mqtt.send(reply).is_err() {
                        bail!("send(to_mqtt) failed - channel closed?");
                    }
                }
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }

        Ok(())
    }

    async fn process_command(&self, command: Command) -> Result<()> {
        use commands::time_register_ops::Action;
        use lxp::packet::{Register, RegisterBit};
        use Command::*;

        match command {
            ReadInputs(inverter, 1) => self.read_inputs(inverter, 0_u16, 40).await,
            ReadInputs(inverter, 2) => self.read_inputs(inverter, 40_u16, 40).await,
            ReadInputs(inverter, 3) => self.read_inputs(inverter, 80_u16, 40).await,
            ReadInputs(inverter, 4) => self.read_inputs(inverter, 120_u16, 40).await,
            ReadInputs(_, _) => unreachable!(),
            ReadInput(inverter, register, count) => {
                self.read_inputs(inverter, register, count).await
            }
            ReadHold(inverter, register, count) => self.read_hold(inverter, register, count).await,
            ReadParam(inverter, register) => self.read_param(inverter, register).await,
            ReadAcChargeTime(inverter, num) => {
                self.read_time_register(inverter, Action::AcCharge(num))
                    .await
            }
            ReadAcFirstTime(inverter, num) => {
                self.read_time_register(inverter, Action::AcFirst(num))
                    .await
            }
            ReadChargePriorityTime(inverter, num) => {
                self.read_time_register(inverter, Action::ChargePriority(num))
                    .await
            }
            ReadForcedDischargeTime(inverter, num) => {
                self.read_time_register(inverter, Action::ForcedDischarge(num))
                    .await
            }
            SetHold(inverter, register, value) => self.set_hold(inverter, register, value).await,
            WriteParam(inverter, register, value) => {
                self.write_param(inverter, register, value).await
            }
            SetAcChargeTime(inverter, num, values) => {
                self.set_time_register(inverter, Action::AcCharge(num), values)
                    .await
            }
            SetAcFirstTime(inverter, num, values) => {
                self.set_time_register(inverter, Action::AcFirst(num), values)
                    .await
            }
            SetChargePriorityTime(inverter, num, values) => {
                self.set_time_register(inverter, Action::ChargePriority(num), values)
                    .await
            }
            SetForcedDischargeTime(inverter, num, values) => {
                self.set_time_register(inverter, Action::ForcedDischarge(num), values)
                    .await
            }
            AcCharge(inverter, enable) => {
                self.update_hold(
                    inverter,
                    Register::Register21,
                    RegisterBit::AcChargeEnable,
                    enable,
                )
                .await
            }
            ChargePriority(inverter, enable) => {
                self.update_hold(
                    inverter,
                    Register::Register21,
                    RegisterBit::ChargePriorityEnable,
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
            self.channels.clone(),
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
        commands::read_hold::ReadHold::new(
            self.channels.clone(),
            inverter.clone(),
            register,
            count,
        )
        .run()
        .await?;

        Ok(())
    }

    async fn read_param<U>(&self, inverter: config::Inverter, register: U) -> Result<()>
    where
        U: Into<u16>,
    {
        commands::read_param::ReadParam::new(self.channels.clone(), inverter.clone(), register)
            .run()
            .await?;

        Ok(())
    }

    async fn read_time_register(
        &self,
        inverter: config::Inverter,
        action: commands::time_register_ops::Action,
    ) -> Result<()> {
        commands::time_register_ops::ReadTimeRegister::new(
            self.channels.clone(),
            inverter.clone(),
            action,
        )
        .run()
        .await
    }

    async fn write_param<U>(
        &self,
        inverter: config::Inverter,
        register: U,
        value: u16,
    ) -> Result<()>
    where
        U: Into<u16>,
    {
        commands::write_param::WriteParam::new(
            self.channels.clone(),
            inverter.clone(),
            register,
            value,
        )
        .run()
        .await?;

        Ok(())
    }

    async fn set_time_register(
        &self,
        inverter: config::Inverter,
        action: commands::time_register_ops::Action,
        values: [u8; 4],
    ) -> Result<()> {
        commands::time_register_ops::SetTimeRegister::new(
            self.channels.clone(),
            inverter.clone(),
            action,
            values,
        )
        .run()
        .await
    }

    async fn set_hold<U>(&self, inverter: config::Inverter, register: U, value: u16) -> Result<()>
    where
        U: Into<u16>,
    {
        commands::set_hold::SetHold::new(self.channels.clone(), inverter.clone(), register, value)
            .run()
            .await?;

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
        commands::update_hold::UpdateHold::new(
            self.channels.clone(),
            inverter.clone(),
            register,
            bit,
            enable,
        )
        .run()
        .await?;

        Ok(())
    }

    async fn inverter_receiver(&self) -> Result<()> {
        use lxp::inverter::ChannelData::*;

        let mut receiver = self.channels.from_inverter.subscribe();

        loop {
            match receiver.recv().await? {
                Packet(packet) => {
                    self.process_inverter_packet(packet).await?;
                }
                Connected(serial) => {
                    if let Err(e) = self.inverter_connected(serial).await {
                        error!("{}", e);
                    }
                }
                // this loop holds no state so doesn't care about inverter disconnects
                Disconnect(_) => {}
                Shutdown => break,
            }
        }

        Ok(())
    }

    async fn process_inverter_packet(&self, packet: lxp::packet::Packet) -> Result<()> {
        debug!("RX: {:?}", packet);

        if let Packet::TranslatedData(td) = &packet {
            // temporary special greppable logging for Param packets as I try to
            // work out what they do :)
            if td.tcp_function() == TcpFunction::ReadParam
                || td.tcp_function() == TcpFunction::WriteParam
            {
                warn!("got a Param packet! {:?}", td);
            }

            match td.device_function {
                DeviceFunction::WriteMulti => {}
                DeviceFunction::ReadInput => {
                    let parser = lxp::register_parser::Parser::from_pairs(td.pairs());
                    let parsed_inputs = parser.parse_inputs()?;
                    //debug!("{}", serde_json::to_string(&parsed_inputs)?);

                    if self.config.mqtt().enabled() {
                        // individual message publishing, raw and parsed
                        self.publish_raw_input_messages(&td)?;
                        self.publish_parsed_input_messages(&td, &parsed_inputs)?;

                        // inputs/1/2/3/4
                        if let Some(topic_fragment) = parser.guess_legacy_inputs_topic() {
                            let m = mqtt::Message {
                                topic: format!("{}/inputs/{}", td.datalog, topic_fragment),
                                retain: false,
                                payload: serde_json::to_string(&parsed_inputs)?,
                            };
                            let channel_data = mqtt::ChannelData::Message(m);
                            if self.channels.to_mqtt.send(channel_data).is_err() {
                                bail!("send(to_mqtt) failed - channel closed?");
                            }
                        };
                    }

                    for (register, value) in td.pairs() {
                        self.cache_register(register_cache::Register::Input(register), value)?;
                    }

                    // add new config option to say when to broadcast "all".
                    //   defaults to 80 (so 1/2/3 -> all) suggest 120 for 4
                    //
                    // if parser.contains_register(option) then feed contents of cache_register
                    // into parser; which should get us "all". feed that to mqtt+influx
                    //   and then clear cache to start over
                    // let parser = lxp::register_parser::ParseInputs::from_hashmap(register_cache);
                }
                DeviceFunction::ReadHold | DeviceFunction::WriteSingle => {
                    let parser = lxp::register_parser::Parser::from_pairs(td.pairs());
                    let parsed_holds = parser.parse_holds()?;

                    // TODO: broadcast parsed
                    self.publish_raw_hold_messages(&td)?;
                    self.publish_parsed_hold_messages(&td, &parsed_holds)?;

                    for (register, value) in td.pairs() {
                        self.cache_register(register_cache::Register::Hold(register), value)?;
                    }
                }
            }
        }

        Ok(())
    }

    // Unlike input registers, holding registers are not broadcast by inverters,
    // but they are interesting nevertheless. Publishing the holding registers
    // when we connect to an inverter makes it easy for configuration data to be
    // tracked, which is particularly useful in conjunction with HomeAssistant.
    async fn inverter_connected(&self, datalog: Serial) -> Result<()> {
        let inverter = match self.config.enabled_inverter_with_datalog(datalog) {
            Some(inverter) => inverter,
            None => bail!("Unknown inverter connected: {}", datalog),
        };

        if !inverter.publish_holdings_on_connect() {
            return Ok(());
        }

        info!("Reading holding registers for inverter {}", datalog);

        // We can only read holding registers in blocks of 40. Provisionally,
        // there are 6 pages of 40 values.
        self.read_hold(inverter.clone(), 0_u16, 40).await?;
        self.read_hold(inverter.clone(), 40_u16, 40).await?;
        self.read_hold(inverter.clone(), 80_u16, 40).await?;
        self.read_hold(inverter.clone(), 120_u16, 40).await?;
        self.read_hold(inverter.clone(), 160_u16, 40).await?;
        self.read_hold(inverter.clone(), 200_u16, 40).await?;

        // Also send any special interpretive topics which are derived from
        // the holding registers.
        //
        // FIXME: this is a further 12 round-trips to the inverter to read values
        // we have already taken, just above. We should be able to do better!
        for num in &[1, 2, 3] {
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::AcCharge(*num),
            )
            .await?;
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::ChargePriority(*num),
            )
            .await?;
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::ForcedDischarge(*num),
            )
            .await?;
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::AcFirst(*num),
            )
            .await?;
        }

        Ok(())
    }

    fn publish_message(&self, topic: String, payload: String) -> Result<()> {
        let m = mqtt::Message {
            topic,
            payload,
            retain: false,
        };
        let channel_data = mqtt::ChannelData::Message(m);
        if self.channels.to_mqtt.send(channel_data).is_err() {
            bail!("send(to_mqtt) failed - channel closed?");
        }
        Ok(())
    }

    fn publish_raw_hold_messages(&self, td: &lxp::packet::TranslatedData) -> Result<()> {
        for (register, value) in td.pairs() {
            self.publish_message(
                format!("{}/hold/{}", td.datalog, register),
                serde_json::to_string(&value)?,
            )?;
        }

        Ok(())
    }

    fn publish_raw_input_messages(&self, td: &lxp::packet::TranslatedData) -> Result<()> {
        for (register, value) in td.pairs() {
            self.publish_message(
                format!("{}/input/{}", td.datalog, register),
                serde_json::to_string(&value)?,
            )?;
        }

        Ok(())
    }

    fn publish_parsed_input_messages(
        &self,
        td: &lxp::packet::TranslatedData,
        parsed_inputs: &lxp::register_parser::ParsedData,
    ) -> Result<()> {
        for (key, parsed_value) in parsed_inputs.clone() {
            self.publish_message(
                format!("{}/input/{}/parsed", td.datalog, key),
                parsed_value.to_string(),
            )?;
        }

        Ok(())
    }

    fn publish_parsed_hold_messages(
        &self,
        td: &lxp::packet::TranslatedData,
        parsed_inputs: &lxp::register_parser::ParsedData,
    ) -> Result<()> {
        for (key, parsed_value) in parsed_inputs.clone() {
            self.publish_message(
                format!("{}/hold/{}", td.datalog, key),
                parsed_value.to_string(),
            )?;
        }

        Ok(())
    }

    async fn save_input_all(&self, input: Box<lxp::packet::ReadInputAll>) -> Result<()> {
        if self.config.influx().enabled() {
            let channel_data = influx::ChannelData::InputData(serde_json::to_value(&input)?);
            if self.channels.to_influx.send(channel_data).is_err() {
                bail!("send(to_influx) failed - channel closed?");
            }
        }

        if self.config.have_enabled_database() {
            let channel_data = database::ChannelData::ReadInputAll(input);
            if self.channels.to_database.send(channel_data).is_err() {
                bail!("send(to_database) failed - channel closed?");
            }
        }

        Ok(())
    }

    fn cache_register(&self, register: register_cache::Register, value: u16) -> Result<()> {
        let channel_data = register_cache::ChannelData::RegisterData(register, value);

        if self.channels.register_cache.send(channel_data).is_err() {
            bail!("send(to_register_cache) failed - channel closed?");
        }

        Ok(())
    }
}
