use crate::prelude::*;

pub mod commands;

use std::sync::{Arc, Mutex};
use lxp::packet::{DeviceFunction, ReadInput, TranslatedData, Packet, TcpFunction};
use serde_json::json;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ChannelData {
    Shutdown,
    Packet(lxp::packet::Packet),
}

pub type InputsStore = std::collections::HashMap<Serial, lxp::packet::ReadInputs>;

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

#[derive(Clone)]
pub struct Coordinator {
    config: ConfigWrapper,
    channels: Channels,
    pub stats: Arc<Mutex<PacketStats>>,
}

impl Coordinator {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        Self { 
            config, 
            channels,
            stats: Arc::new(Mutex::new(PacketStats::default())),
        }
    }

    pub async fn start(&self) -> Result<()> {
        if self.config.mqtt().enabled() {
            futures::try_join!(self.inverter_receiver(), self.mqtt_receiver())?;
        } else {
            self.inverter_receiver().await?;
        }

        Ok(())
    }

    pub fn stop(&self) {
        // Send shutdown signals to channels
        let _ = self
            .channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Shutdown);

        if self.config.mqtt().enabled() {
            let _ = self.channels.from_mqtt.send(mqtt::ChannelData::Shutdown);
        }
    }

    async fn mqtt_receiver(&self) -> Result<()> {
        let mut receiver = self.channels.from_mqtt.subscribe();

        while let mqtt::ChannelData::Message(message) = receiver.recv().await? {
            let _ = self.process_message(message).await;
        }

        Ok(())
    }

    async fn process_message(&self, message: mqtt::Message) -> Result<()> {
        // If MQTT is disabled, don't process any messages
        if !self.config.mqtt().enabled() {
            return Ok(());
        }

        for inverter in self.config.inverters_for_message(&message)? {
            match message.to_command(inverter) {
                Ok(command) => {
                    info!("parsed command {:?}", command);
                    let result = self.process_command(command.clone()).await;
                    if result.is_err() {
                        let topic_reply = command.to_result_topic();
                        let reply = mqtt::ChannelData::Message(mqtt::Message {
                            topic: topic_reply,
                            retain: false,
                            payload: "FAIL".to_string(),
                        });
                        if self.channels.to_mqtt.send(reply).is_err() {
                            bail!("send(to_mqtt) failed - channel closed?");
                        }
                    }
                }
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        }

        Ok(())
    }

    fn increment_packets_sent(&self, packet: &Packet) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.packets_sent += 1;
            
            // Increment counter for specific sent packet type
            match packet {
                Packet::Heartbeat(_) => stats.heartbeat_packets_sent += 1,
                Packet::TranslatedData(_) => stats.translated_data_packets_sent += 1,
                Packet::ReadParam(_) => stats.read_param_packets_sent += 1,
                Packet::WriteParam(_) => stats.write_param_packets_sent += 1,
            }
        }
    }

    async fn process_command(&self, command: Command) -> Result<()> {
        use commands::time_register_ops::Action;
        use lxp::packet::{Register, RegisterBit};
        use Command::*;

        // Create a packet from the command for stats tracking
        let packet = match &command {
            ReadInputs(_, _) => Packet::TranslatedData(TranslatedData {
                datalog: Serial::default(),
                device_function: DeviceFunction::ReadInput,
                inverter: Serial::default(),
                register: 0,
                values: vec![],
            }),
            ReadInput(_, _, _) => Packet::TranslatedData(TranslatedData {
                datalog: Serial::default(),
                device_function: DeviceFunction::ReadInput,
                inverter: Serial::default(),
                register: 0,
                values: vec![],
            }),
            ReadHold(_, _, _) => Packet::TranslatedData(TranslatedData {
                datalog: Serial::default(),
                device_function: DeviceFunction::ReadHold,
                inverter: Serial::default(),
                register: 0,
                values: vec![],
            }),
            ReadParam(_, _) => Packet::TranslatedData(TranslatedData {
                datalog: Serial::default(),
                device_function: DeviceFunction::ReadHold, // Using ReadHold as device function
                inverter: Serial::default(),
                register: 0,
                values: vec![],
            }),
            WriteParam(_, _, _) => Packet::TranslatedData(TranslatedData {
                datalog: Serial::default(),
                device_function: DeviceFunction::WriteSingle, // Using WriteSingle as device function
                inverter: Serial::default(),
                register: 0,
                values: vec![],
            }),
            _ => Packet::TranslatedData(TranslatedData {
                datalog: Serial::default(),
                device_function: DeviceFunction::WriteSingle,
                inverter: Serial::default(),
                register: 0,
                values: vec![],
            }),
        };

        self.increment_packets_sent(&packet);

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
            self.config.clone(),
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
            self.config.clone(),
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

    async fn process_inverter_packet(
        &self,
        packet: Packet,
        inputs_store: &mut InputsStore,
    ) -> Result<()> {
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
            // Check if the inverter serial from packet matches any configured inverter
            let packet_serial = td.inverter;
            let packet_datalog = td.datalog;
            
            // Log if we see a mismatch between configured and actual serial
            if let Some(inverter) = self.config.enabled_inverter_with_datalog(packet_datalog) {
                if inverter.serial() != packet_serial {
                    warn!(
                        "Inverter serial mismatch - Config: {}, Actual: {} for datalog: {}",
                        inverter.serial(),
                        packet_serial,
                        packet_datalog
                    );
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.serial_mismatches += 1;
                    }
                }
            }

            // temporary special greppable logging for Param packets as I try to
            // work out what they do :)
            if td.tcp_function() == TcpFunction::ReadParam
                || td.tcp_function() == TcpFunction::WriteParam
            {
                warn!("got a Param packet! {:?}", td);
            }
        }

        // inputs_store handling. If we've received any ReadInput, update inputs_store
        // with the contents. If we got the third (of three) packets, send out the combined
        // MQTT message with all the data.
        match packet {
            Packet::Heartbeat(_) => Ok(()), // nothing to do
            Packet::TranslatedData(td) => {
                let entry = inputs_store
                    .entry(td.datalog)
                    .or_insert_with(lxp::packet::ReadInputs::default);

                match td.device_function {
                    DeviceFunction::ReadInput => match td.read_input() {
                        Ok(ReadInput::ReadInputAll(r_all)) => {
                            debug!("Received ReadInputAll, saving to InfluxDB");
                            self.save_input_all(r_all).await?;
                        }
                        Ok(ReadInput::ReadInput1(r1)) => {
                            debug!("Received ReadInput1");
                            entry.set_read_input_1(r1)
                        }
                        Ok(ReadInput::ReadInput2(r2)) => {
                            debug!("Received ReadInput2");
                            entry.set_read_input_2(r2)
                        }
                        Ok(ReadInput::ReadInput3(r3)) => {
                            debug!("Received ReadInput3");
                            entry.set_read_input_3(r3)
                        }
                        Ok(ReadInput::ReadInput4(r4)) => {
                            debug!("Received ReadInput4");
                            let datalog = r4.datalog;
                            entry.set_read_input_4(r4);

                            if let Some(input) = entry.to_input_all() {
                                info!("Assembled complete input set, saving to InfluxDB");
                                if self.config.mqtt().enabled() {
                                    match mqtt::Message::for_input_all(&input, datalog) {
                                        Ok(message) => {
                                            let channel_data = mqtt::ChannelData::Message(message);
                                            match self.channels.to_mqtt.send(channel_data) {
                                                Ok(_) => {
                                                    if let Ok(mut stats) = self.stats.lock() {
                                                        stats.mqtt_messages_sent += 1;
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Failed to send MQTT message: {}", e);
                                                    if let Ok(mut stats) = self.stats.lock() {
                                                        stats.mqtt_errors += 1;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to create MQTT message: {}", e);
                                            if let Ok(mut stats) = self.stats.lock() {
                                                stats.mqtt_errors += 1;
                                            }
                                        }
                                    }
                                }

                                self.save_input_all(Box::new(input)).await?;
                            } else {
                                debug!("Incomplete input set, waiting for more data");
                            }
                        }
                        Err(x) => warn!("ignoring {:?}", x),
                    },
                    DeviceFunction::ReadHold | DeviceFunction::WriteSingle => {
                        let channel_data = register_cache::ChannelData::RegisterData(td.register, td.value());
                        match self.channels.to_register_cache.send(channel_data) {
                            Ok(_) => {
                                if let Ok(mut stats) = self.stats.lock() {
                                    stats.register_cache_writes += 1;
                                }
                            }
                            Err(e) => {
                                error!("Failed to send to register cache: {}", e);
                                if let Ok(mut stats) = self.stats.lock() {
                                    stats.register_cache_errors += 1;
                                }
                            }
                        }

                        // Send the value message first
                        if self.config.mqtt().enabled() {
                            let value_message = mqtt::Message {
                                topic: format!("{}/hold/{}", td.datalog, td.register),
                                retain: true,
                                payload: td.value().to_string(),
                            };
                            if let Err(e) = self.channels.to_mqtt.send(mqtt::ChannelData::Message(value_message)) {
                                error!("Failed to send value MQTT message: {}", e);
                                if let Ok(mut stats) = self.stats.lock() {
                                    stats.mqtt_errors += 1;
                                }
                            } else if let Ok(mut stats) = self.stats.lock() {
                                stats.mqtt_messages_sent += 1;
                            }

                            // Then send the OK message
                            let ok_message = mqtt::Message {
                                topic: format!("result/{}/read/hold/{}", td.datalog, td.register),
                                retain: false,
                                payload: "OK".to_string(),
                            };
                            if let Err(e) = self.channels.to_mqtt.send(mqtt::ChannelData::Message(ok_message)) {
                                error!("Failed to send OK MQTT message: {}", e);
                                if let Ok(mut stats) = self.stats.lock() {
                                    stats.mqtt_errors += 1;
                                }
                            } else if let Ok(mut stats) = self.stats.lock() {
                                stats.mqtt_messages_sent += 1;
                            }
                        }

                        // Send to InfluxDB if enabled
                        if self.config.influx().enabled() {
                            debug!("InfluxDB is enabled, sending ReadHold data");
                            let mut json_data = serde_json::Map::new();
                            json_data.insert("time".to_string(), json!(chrono::Utc::now().timestamp()));
                            json_data.insert("datalog".to_string(), json!(td.datalog.to_string()));
                            json_data.insert(format!("hold_{}", td.register), json!(td.value()));
                            
                            let json = serde_json::Value::Object(json_data);
                            match self.channels.to_influx.send(influx::ChannelData::InputData(json)) {
                                Ok(_) => {
                                    debug!("Successfully sent ReadHold data to InfluxDB");
                                    if let Ok(mut stats) = self.stats.lock() {
                                        stats.influx_writes += 1;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to send ReadHold data to InfluxDB: {}", e);
                                    if let Ok(mut stats) = self.stats.lock() {
                                        stats.influx_errors += 1;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                if self.config.mqtt().enabled() {
                    // Process any individual input messages if enabled
                    if self.config.mqtt().publish_individual_input() {
                        match mqtt::Message::for_input(td, true) {
                            Ok(messages) => {
                                for message in messages {
                                    let channel_data = mqtt::ChannelData::Message(message);
                                    match self.channels.to_mqtt.send(channel_data) {
                                        Ok(_) => {
                                            if let Ok(mut stats) = self.stats.lock() {
                                                stats.mqtt_messages_sent += 1;
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to send individual MQTT message: {}", e);
                                            if let Ok(mut stats) = self.stats.lock() {
                                                stats.mqtt_errors += 1;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to create individual MQTT message: {}", e);
                                if let Ok(mut stats) = self.stats.lock() {
                                    stats.mqtt_errors += 1;
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn save_input_all(&self, input: Box<lxp::packet::ReadInputAll>) -> Result<()> {
        if self.config.influx().enabled() {
            debug!("InfluxDB is enabled, attempting to save data");
            let json = serde_json::to_value(&input)?;
            debug!("Serialized data for InfluxDB: {:?}", json);
            match self.channels.to_influx.send(influx::ChannelData::InputData(json)) {
                Ok(_) => {
                    debug!("Successfully sent data to InfluxDB channel");
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.influx_writes += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to send data to InfluxDB: {}", e);
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.influx_errors += 1;
                    }
                }
            }
        }

        if self.config.have_enabled_database() {
            let channel_data = database::ChannelData::ReadInputAll(input);
            match self.channels.to_database.send(channel_data) {
                Ok(_) => {
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.database_writes += 1;
                    }
                }
                Err(e) => {
                    // log error but avoid exiting loop as then we stop handling
                    // incoming packets. need better error handling here maybe?
                    error!("Failed to send to database: {}", e);
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.database_errors += 1;
                    }
                }
            }
        }

        Ok(())
    }

    async fn inverter_receiver(&self) -> Result<()> {
        use lxp::inverter::ChannelData::*;

        let mut receiver = self.channels.from_inverter.subscribe();

        let mut inputs_store = InputsStore::new();

        loop {
            match receiver.recv().await? {
                Packet(packet) => {
                    if let Err(e) = self.process_inverter_packet(packet, &mut inputs_store).await {
                        warn!("Failed to process packet: {}", e);
                    }
                }
                Connected(serial) => {
                    if let Err(e) = self.inverter_connected(serial).await {
                        error!("{}", e);
                    }
                }
                // Print statistics when an inverter disconnects
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

    async fn inverter_connected(&self, datalog: Serial) -> Result<()> {
        let inverter = match self.config.enabled_inverter_with_datalog(datalog) {
            Some(inverter) => inverter,
            None => {
                warn!("Unknown inverter datalog connected: {}, will continue processing its data", datalog);
                return Ok(());
            }
        };

        if !inverter.publish_holdings_on_connect() {
            return Ok(());
        }

        info!("Reading holding registers for inverter {}", datalog);

        // Add delay between read_hold requests to prevent overwhelming the inverter
        const DELAY_MS: u64 = 1; // 1ms delay between requests

        // Create a packet for stats tracking
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: Serial::default(),
            device_function: DeviceFunction::ReadHold,
            inverter: Serial::default(),
            register: 0,
            values: vec![],
        });

        // We can only read holding registers in blocks of 40. Provisionally,
        // there are 6 pages of 40 values.
        self.increment_packets_sent(&packet);
        self.read_hold(inverter.clone(), 0_u16, 40).await?;
//        tokio::time::sleep(std::time::Duration::from_millis(DELAY_MS)).await;
        
        self.increment_packets_sent(&packet);
        self.read_hold(inverter.clone(), 40_u16, 40).await?;
//        tokio::time::sleep(std::time::Duration::from_millis(DELAY_MS)).await;
        
        self.increment_packets_sent(&packet);
        self.read_hold(inverter.clone(), 80_u16, 40).await?;
//        tokio::time::sleep(std::time::Duration::from_millis(DELAY_MS)).await;
        
        self.increment_packets_sent(&packet);
        self.read_hold(inverter.clone(), 120_u16, 40).await?;
//        tokio::time::sleep(std::time::Duration::from_millis(DELAY_MS)).await;
        
        self.increment_packets_sent(&packet);
        self.read_hold(inverter.clone(), 160_u16, 40).await?;
//        tokio::time::sleep(std::time::Duration::from_millis(DELAY_MS)).await;
        
        self.increment_packets_sent(&packet);
        self.read_hold(inverter.clone(), 200_u16, 40).await?;

        // Also send any special interpretive topics which are derived from
        // the holding registers.
        //
        // FIXME: this is a further 12 round-trips to the inverter to read values
        // we have already taken, just above. We should be able to do better!
        for num in &[1, 2, 3] {
            self.increment_packets_sent(&packet);
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::AcCharge(*num),
            )
            .await?;
            self.increment_packets_sent(&packet);
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::ChargePriority(*num),
            )
            .await?;
            self.increment_packets_sent(&packet);
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::ForcedDischarge(*num),
            )
            .await?;
            self.increment_packets_sent(&packet);
            self.read_time_register(
                inverter.clone(),
                commands::time_register_ops::Action::AcFirst(*num),
            )
            .await?;
        }

        Ok(())
    }
}
