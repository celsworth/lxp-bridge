use crate::prelude::*;

pub mod commands;

use lxp::packet::{DeviceFunction, TcpFunction};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Shutdown,
}

pub type InputsStore = std::collections::HashMap<Serial, lxp::packet::ReadInputs>;

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

        let mut inputs_store = InputsStore::new();

        loop {
            match receiver.recv().await? {
                Packet(packet) => {
                    self.process_inverter_packet(packet, &mut inputs_store)
                        .await?;
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

                match td.read_input() {
                    Ok(ReadInput::ReadInputAll(r_all)) => {
                        // no need for MQTT here, done below
                        self.save_input_all(r_all).await?
                    }

                    Ok(ReadInput::ReadInput1(r1)) => entry.set_read_input_1(r1),
                    Ok(ReadInput::ReadInput2(r2)) => entry.set_read_input_2(r2),
                    Ok(ReadInput::ReadInput3(r3)) => entry.set_read_input_3(r3),
                    Ok(ReadInput::ReadInput4(r4)) => {
                        let datalog = r4.datalog;

                        entry.set_read_input_4(r4);

                        if let Some(input) = entry.to_input_all() {
                            if self.config.mqtt().enabled() {
                                let message = mqtt::Message::for_input_all(&input, datalog)?;
                                let channel_data = mqtt::ChannelData::Message(message);
                                if self.channels.to_mqtt.send(channel_data).is_err() {
                                    bail!("send(to_mqtt) failed - channel closed?");
                                }
                            }

                            self.save_input_all(Box::new(input)).await?;
                        }
                    }
                    Err(x) => warn!("ignoring {:?}", x),
                }
            } else if td.device_function == DeviceFunction::ReadHold
                || td.device_function == DeviceFunction::WriteSingle
            {
                let channel_data =
                    register_cache::ChannelData::RegisterData(td.register, td.value());
                if self.channels.to_register_cache.send(channel_data).is_err() {
                    bail!("send(to_register_cache) failed - channel closed?");
                }
            }
        }

        if self.config.mqtt().enabled() {
            // returns a Vec of messages to send. could be none;
            // not every packet produces an MQ message (eg, heartbeats),
            // and some produce >1 (multi-register ReadHold)
            match Self::packet_to_messages(packet, self.config.mqtt().publish_individual_input()) {
                Ok(messages) => {
                    for message in messages {
                        let message = mqtt::ChannelData::Message(message);
                        if self.channels.to_mqtt.send(message).is_err() {
                            bail!("send(to_mqtt) failed - channel closed?");
                        }
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

    fn packet_to_messages(
        packet: Packet,
        publish_individual_input: bool,
    ) -> Result<Vec<mqtt::Message>> {
        match packet {
            Packet::Heartbeat(_) => Ok(Vec::new()), // always no message
            Packet::TranslatedData(td) => match td.device_function {
                DeviceFunction::ReadHold => mqtt::Message::for_hold(td),
                DeviceFunction::ReadInput => mqtt::Message::for_input(td, publish_individual_input),
                DeviceFunction::WriteSingle => mqtt::Message::for_hold(td),
                DeviceFunction::WriteMulti => Ok(Vec::new()), // TODO, for_hold might just work
            },
            Packet::ReadParam(rp) => mqtt::Message::for_param(rp),
            Packet::WriteParam(_) => Ok(Vec::new()), // ignoring for now
        }
    }
}
