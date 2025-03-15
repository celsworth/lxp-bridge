use crate::prelude::*;

pub mod commands;

use lxp::packet::{DeviceFunction, TcpFunction};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Shutdown,
}

#[derive(Default)]
pub struct PacketStats {
    packets_received: u64,
    packets_sent: u64,
    // Received packet counters
    heartbeat_packets_received: u64,
    translated_data_packets_received: u64,
    read_param_packets_received: u64,
    write_param_packets_received: u64,
    // Sent packet counters
    heartbeat_packets_sent: u64,
    translated_data_packets_sent: u64,
    read_param_packets_sent: u64,
    write_param_packets_sent: u64,
    // Other stats
    mqtt_messages_sent: u64,
    mqtt_errors: u64,
    influx_writes: u64,
    influx_errors: u64,
    database_writes: u64,
    database_errors: u64,
    register_cache_writes: u64,
    register_cache_errors: u64,
    // Connection stats
    inverter_disconnections: std::collections::HashMap<Serial, u64>,
    serial_mismatches: u64,
    // Last message received per inverter
    last_messages: std::collections::HashMap<Serial, String>,
}

impl PacketStats {
    pub fn print_summary(&self) {
        info!("Packet Statistics:");
        info!("  Total packets received: {}", self.packets_received);
        info!("  Total packets sent: {}", self.packets_sent);
        info!("  Received Packet Types:");
        info!("    Heartbeat packets: {}", self.heartbeat_packets_received);
        info!("    TranslatedData packets: {}", self.translated_data_packets_received);
        info!("    ReadParam packets: {}", self.read_param_packets_received);
        info!("    WriteParam packets: {}", self.write_param_packets_received);
        info!("  Sent Packet Types:");
        info!("    Heartbeat packets: {}", self.heartbeat_packets_sent);
        info!("    TranslatedData packets: {}", self.translated_data_packets_sent);
        info!("    ReadParam packets: {}", self.read_param_packets_sent);
        info!("    WriteParam packets: {}", self.write_param_packets_sent);
        info!("  MQTT:");
        info!("    Messages sent: {}", self.mqtt_messages_sent);
        info!("    Errors: {}", self.mqtt_errors);
        info!("  InfluxDB:");
        info!("    Writes: {}", self.influx_writes);
        info!("    Errors: {}", self.influx_errors);
        info!("  Database:");
        info!("    Writes: {}", self.database_writes);
        info!("    Errors: {}", self.database_errors);
        info!("  Register Cache:");
        info!("    Writes: {}", self.register_cache_writes);
        info!("    Errors: {}", self.register_cache_errors);
        info!("  Connection Stats:");
        info!("    Serial number mismatches: {}", self.serial_mismatches);
        info!("    Inverter disconnections by serial:");
        for (serial, count) in &self.inverter_disconnections {
            info!("      {}: {}", serial, count);
            if let Some(last_msg) = self.last_messages.get(serial) {
                info!("      Last message: {}", last_msg);
            }
        }
    }
}

pub struct Coordinator {
    config: ConfigWrapper,
    channels: Channels,
    stats: std::sync::Arc<std::sync::Mutex<PacketStats>>,
}

impl Coordinator {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self { 
            config, 
            channels,
            stats: std::sync::Arc::new(std::sync::Mutex::new(PacketStats::default())),
        }
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
            action.clone(),
            values,
        )
        .run()
        .await?;

        // after setting, issue a ReadTimeRegister so we can send a new message out
        self.read_time_register(inverter, action).await
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
                Disconnect(serial) => {
                    info!("Inverter {} disconnected, printing statistics:", serial);
                    if let Ok(mut stats) = self.stats.lock() {
                        *stats.inverter_disconnections.entry(serial).or_insert(0) += 1;
                        stats.print_summary();
                    }
                }
                Shutdown => {
                    info!("Received shutdown signal, printing final statistics:");
                    if let Ok(stats) = self.stats.lock() {
                        stats.print_summary();
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_inverter_packet(&self, packet: lxp::packet::Packet) -> Result<()> {
        debug!("RX: {:?}", packet);

        // Update packet stats first
        if let Ok(mut stats) = self.stats.lock() {
            stats.packets_received += 1;
            
            // Store last message for the inverter
            if let Packet::TranslatedData(td) = &packet {
                stats.last_messages.insert(td.datalog, format!("{:?}", packet));
            }
            
            // Increment counter for specific received packet type
            match &packet {
                Packet::Heartbeat(_) => stats.heartbeat_packets_received += 1,
                Packet::TranslatedData(_) => stats.translated_data_packets_received += 1,
                Packet::ReadParam(_) => stats.read_param_packets_received += 1,
                Packet::WriteParam(_) => stats.write_param_packets_received += 1,
            }
        }

        if let Packet::TranslatedData(td) = &packet {
            // temporary special greppable logging for Param packets as I try to
            // work out what they do :)
            if td.tcp_function() == TcpFunction::ReadParam
                || td.tcp_function() == TcpFunction::WriteParam
            {
                warn!("got a Param packet! {:?}", td);
            }

            match td.device_function {
                DeviceFunction::ReadInput => {
                    let register_map = td.register_map();
                    let parser = lxp::register_parser::Parser::new(register_map.clone());
                    let parsed_inputs = parser.parse_inputs()?;

                    if self.config.mqtt().enabled() {
                        // individual message publishing, raw and parsed
                        self.publish_raw_input_messages(td)?;
                        self.publish_parsed_input_messages(td, &parsed_inputs)?;

                        // inputs/1/2/3/4
                        if let Some(topic_fragment) = parser.guess_legacy_inputs_topic() {
                            self.publish_combined_parsed_input_message(
                                td,
                                &parsed_inputs,
                                topic_fragment,
                            )?;
                        };
                    }

                    for (register, value) in register_map {
                        self.cache_register(register_cache::Register::Input(register), value)?;
                    }

                    // if we've seen the triggering register in config.publish_inputs_all_trigger
                    // then feed contents of cache_register into a new parser;
                    // which should get us "all". feed that to mqtt+influx
                    if parser.contains_register(self.config.publish_inputs_all_trigger()) {
                        let cache = register_cache::RegisterCache::dump(
                            &self.channels,
                            register_cache::AllRegisters::Input,
                        )
                        .await;
                        let all_parser = lxp::register_parser::Parser::new(cache);
                        if all_parser.guess_legacy_inputs_topic() == Some("all") {
                            let all_parsed_inputs = all_parser.parse_inputs()?;
                            if self.config.mqtt().enabled() {
                                // inputs/all
                                self.publish_combined_parsed_input_message(
                                    td,
                                    &all_parsed_inputs,
                                    "all",
                                )?;
                            }
                            if self.config.influx().enabled() {
                                let channel_data = influx::ChannelData::InputData(
                                    td.datalog(),
                                    all_parsed_inputs.clone(),
                                );
                                if self.channels.to_influx.send(channel_data).is_err() {
                                    bail!("send(to_influx) failed - channel closed?");
                                }
                                if let Ok(mut stats) = self.stats.lock() {
                                    stats.influx_writes += 1;
                                }
                            }
                        };

                        // clear the cache so we start over next time
                        register_cache::RegisterCache::clear(&self.channels).await;
                    }
                }
                DeviceFunction::ReadHold | DeviceFunction::WriteSingle => {
                    let register_map = td.register_map();

                    let parser = lxp::register_parser::Parser::new(register_map.clone());
                    let parsed_holds = parser.parse_holds()?;

                    if self.config.mqtt().enabled() {
                        self.publish_raw_hold_messages(td)?;
                        self.publish_parsed_hold_messages(td, &parsed_holds)?;
                    }

                    if td.device_function == DeviceFunction::WriteSingle {
                        let inverter = self.inverter_config_for_datalog(td.datalog)?;
                        // if register_map contains an interesting register that's
                        // part of a multi-register setup (like AC Charge times) then
                        // issue a ReadHold request to get the other parts so register_parser
                        // can construct an MQTT message to send out with current data
                        self.maybe_send_read_holds(register_map, inverter).await?;
                    }

                    /* not used yet
                    for (register, value) in register_map {
                      self.cache_register(register_cache::Register::Hold(register), value)?;
                    }
                    */
                }
                DeviceFunction::WriteMulti => {}
            }
        }

        Ok(())
    }

    async fn maybe_send_read_holds(
        &self,
        register_map: RegisterMap,
        inverter: config::Inverter,
    ) -> Result<()> {
        // ^ is true if one key is present, but false if none or both are

        if register_map.contains_key(&68) ^ register_map.contains_key(&69) {
            self.read_hold(inverter.clone(), 84_u16, 2).await?; // ac_charge/1
        }
        if register_map.contains_key(&70) ^ register_map.contains_key(&71) {
            self.read_hold(inverter.clone(), 70_u16, 2).await?; // ac_charge/2
        }
        if register_map.contains_key(&72) ^ register_map.contains_key(&73) {
            self.read_hold(inverter.clone(), 72_u16, 2).await?; // ac_charge/3
        }
        if register_map.contains_key(&76) ^ register_map.contains_key(&77) {
            self.read_hold(inverter.clone(), 76_u16, 2).await?; // charge_priority/1
        }
        if register_map.contains_key(&78) ^ register_map.contains_key(&79) {
            self.read_hold(inverter.clone(), 78_u16, 2).await?; // charge_priority/2
        }
        if register_map.contains_key(&80) ^ register_map.contains_key(&81) {
            self.read_hold(inverter.clone(), 80_u16, 2).await?; // charge_priority/3
        }
        if register_map.contains_key(&84) ^ register_map.contains_key(&85) {
            self.read_hold(inverter.clone(), 84_u16, 2).await?; // forced_discharge/1
        }
        if register_map.contains_key(&86) ^ register_map.contains_key(&87) {
            self.read_hold(inverter.clone(), 86_u16, 2).await?; // forced_discharge/2
        }
        if register_map.contains_key(&88) ^ register_map.contains_key(&89) {
            self.read_hold(inverter.clone(), 88_u16, 2).await?; // forced_discharge/3
        }
        if register_map.contains_key(&152) ^ register_map.contains_key(&153) {
            self.read_hold(inverter.clone(), 152_u16, 2).await?; // ac_first/1
        }
        if register_map.contains_key(&154) ^ register_map.contains_key(&155) {
            self.read_hold(inverter.clone(), 154_u16, 2).await?; // ac_first/2
        }
        if register_map.contains_key(&156) ^ register_map.contains_key(&157) {
            self.read_hold(inverter.clone(), 156_u16, 2).await?; // ac_first/3
        }

        Ok(())
    }

    fn inverter_config_for_datalog(&self, datalog: Serial) -> Result<config::Inverter> {
        self.config
            .enabled_inverter_with_datalog(datalog)
            .ok_or(anyhow!("Unknown inverter connected: {}", datalog))
    }

    // Unlike input registers, holding registers are not broadcast by inverters,
    // but they are interesting nevertheless. Publishing the holding registers
    // when we connect to an inverter makes it easy for configuration data to be
    // tracked, which is particularly useful in conjunction with HomeAssistant.
    async fn inverter_connected(&self, datalog: Serial) -> Result<()> {
        let inverter = self.inverter_config_for_datalog(datalog)?;

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

    fn publish_message(&self, topic: String, payload: String, retain: bool) -> Result<()> {
        let m = mqtt::Message {
            topic,
            payload,
            retain,
        };
        let channel_data = mqtt::ChannelData::Message(m);
        if self.channels.to_mqtt.send(channel_data).is_err() {
            if let Ok(mut stats) = self.stats.lock() {
                stats.mqtt_errors += 1;
            }
            bail!("send(to_mqtt) failed - channel closed?");
        }
        if let Ok(mut stats) = self.stats.lock() {
            stats.mqtt_messages_sent += 1;
        }
        Ok(())
    }

    fn publish_raw_hold_messages(&self, td: &lxp::packet::TranslatedData) -> Result<()> {
        for (register, value) in td.register_map() {
            self.publish_message(
                format!("{}/hold/{}", td.datalog, register),
                serde_json::to_string(&value)?,
                true,
            )?;
        }

        Ok(())
    }

    fn publish_raw_input_messages(&self, td: &lxp::packet::TranslatedData) -> Result<()> {
        for (register, value) in td.register_map() {
            self.publish_message(
                format!("{}/input/{}", td.datalog, register),
                serde_json::to_string(&value)?,
                false,
            )?;
        }

        Ok(())
    }

    fn publish_combined_parsed_input_message(
        &self,
        td: &lxp::packet::TranslatedData,
        parsed_inputs: &lxp::register_parser::ParsedData,
        topic_fragment: &str,
    ) -> Result<()> {
        self.publish_message(
            format!("{}/inputs/{}", td.datalog, topic_fragment),
            serde_json::to_string(&parsed_inputs)?,
            false,
        )?;

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
                false,
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
                // no "hold" here, it's done in parse_holds if required!
                // (some don't, like ac_charge/1)
                format!("{}/{}", td.datalog, key),
                parsed_value.to_string(),
                true,
            )?;
        }

        Ok(())
    }

    /*
    async fn save_input_all(&self, input: Box<lxp::packet::ReadInputAll>) -> Result<()> {
        if self.config.have_enabled_database() {
            let channel_data = database::ChannelData::ReadInputAll(input);
            if self.channels.to_database.send(channel_data).is_err() {
                bail!("send(to_database) failed - channel closed?");
            }
        }

        Ok(())
    }
    */

    fn cache_register(&self, register: register_cache::Register, value: u16) -> Result<()> {
        let channel_data = register_cache::ChannelData::RegisterData(register, value);

        if self.channels.register_cache.send(channel_data).is_err() {
            if let Ok(mut stats) = self.stats.lock() {
                stats.register_cache_errors += 1;
            }
            bail!("send(to_register_cache) failed - channel closed?");
        }
        if let Ok(mut stats) = self.stats.lock() {
            stats.register_cache_writes += 1;
        }

        Ok(())
    }
}
