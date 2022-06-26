use crate::prelude::*;

pub mod commands;

use lxp::inverter::WaitForReply;
use lxp::packet::{DeviceFunction, ReadParam, TcpFunction};

#[derive(PartialEq, Debug, Clone)]
pub enum ChannelData {
    Shutdown,
}

pub type InputsStore = std::collections::HashMap<Serial, lxp::packet::ReadInputs>;

pub struct Coordinator {
    config: Rc<RefCell<Config>>,
    have_enabled_databases: bool,
    channels: Channels,
}

impl Coordinator {
    pub fn new(config: Rc<RefCell<Config>>, channels: Channels) -> Self {
        let have_enabled_databases = !config.borrow().enabled_databases().is_empty();

        Self {
            config,
            have_enabled_databases,
            channels,
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

        loop {
            match receiver.recv().await? {
                mqtt::ChannelData::Shutdown => break,
                mqtt::ChannelData::Message(message) => {
                    let _ = self.process_message(message).await;
                }
            }
        }

        Ok(())
    }

    async fn process_message(&self, message: mqtt::Message) -> Result<()> {
        let inverters = self.config.borrow().inverters_for_message(&message)?;
        for inverter in inverters {
            match message.to_command(inverter) {
                Ok(command) => {
                    debug!("parsed command {:?}", command);

                    let topic_reply = command.to_result_topic();
                    let result = self.process_command(command).await;

                    let reply = mqtt::ChannelData::Message(mqtt::Message {
                        topic: topic_reply,
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
        use lxp::packet::{Register, RegisterBit};
        use Command::*;

        match command {
            ReadInputs1(inverter) => self.read_inputs(inverter, 0_u16, 40).await,
            ReadInputs2(inverter) => self.read_inputs(inverter, 40_u16, 40).await,
            ReadInputs3(inverter) => self.read_inputs(inverter, 80_u16, 40).await,
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
        let register = register.into();
        let packet = Packet::ReadParam(ReadParam {
            datalog: inverter.datalog,
            register,
            values: vec![], // unused
        });

        let mut receiver = self.channels.from_inverter.subscribe();

        if self
            .channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))
            .is_err()
        {
            bail!("send(to_inverter) failed - channel closed?");
        }

        let _ = receiver.wait_for_reply(&packet).await?;

        Ok(())
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
                // this loop holds no state so doesn't care about inverter reconnects
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

                        if let Some(input) = entry.to_input_all() {
                            if self.config.borrow().mqtt.enabled {
                                let message = mqtt::Message::for_input_all(&input, datalog)?;
                                let channel_data = mqtt::ChannelData::Message(message);
                                if self.channels.to_mqtt.send(channel_data).is_err() {
                                    bail!("send(to_mqtt) failed - channel closed?");
                                }
                            }

                            self.save_input_all(Box::new(input)).await?;
                        }
                    }
                }
            }
        }

        if self.config.borrow().mqtt.enabled {
            // returns a Vec of messages to send. could be none;
            // not every packet produces an MQ message (eg, heartbeats),
            // and some produce >1 (multi-register ReadHold)
            match Self::packet_to_messages(packet) {
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

    async fn save_input_all(&self, input: Box<lxp::packet::ReadInputAll>) -> Result<()> {
        if self.config.borrow().influx.enabled {
            let channel_data = influx::ChannelData::InputData(serde_json::to_value(&input)?);
            if self.channels.to_influx.send(channel_data).is_err() {
                bail!("send(to_influx) failed - channel closed?");
            }
        }

        if self.have_enabled_databases {
            let channel_data = database::ChannelData::ReadInputAll(input);
            if self.channels.to_database.send(channel_data).is_err() {
                bail!("send(to_database) failed - channel closed?");
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
}
