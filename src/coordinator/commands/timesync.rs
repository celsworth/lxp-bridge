use crate::prelude::*;

use chrono::TimeZone;

use lxp::{
    inverter::WaitForReply,
    packet::{DeviceFunction, TranslatedData},
};

pub struct TimeSync {
    channels: Channels,
    inverter: config::Inverter,
}

impl TimeSync {
    pub fn new(channels: Channels, inverter: config::Inverter) -> Self {
        Self { channels, inverter }
    }

    pub async fn run(&self) -> Result<()> {
        let packet = Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog,
            device_function: DeviceFunction::ReadHold,
            inverter: self.inverter.serial,
            register: 12,
            values: vec![3, 0],
        });

        let mut receiver = self.channels.from_inverter.subscribe();

        self.channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;

        if let Packet::TranslatedData(td) = receiver.wait_for_reply(&packet).await? {
            let year = td.values[0] as u32;
            let month = td.values[1] as u32;
            let day = td.values[2] as u32;
            let hour = td.values[3] as u32;
            let minute = td.values[4] as u32;
            let second = td.values[5] as u32;

            let dt = chrono::Local
                .ymd(2000 + year as i32, month, day)
                .and_hms(hour, minute, second);

            let now = chrono::Local::now();

            debug!(
                "inverter {} time difference is {}",
                self.inverter.datalog,
                dt - now
            );

            let limit = chrono::Duration::seconds(120);

            if dt - now > limit || now - dt > limit {
                let packet = self.set_current_time_packet();
                self.channels
                    .to_inverter
                    .send(lxp::inverter::ChannelData::Packet(packet.clone()))?;
                if let Packet::TranslatedData(_) = receiver.wait_for_reply(&packet).await? {
                    debug!("time set ok");
                } else {
                    warn!("time set didn't get confirmation reply!");
                }
            }
        }

        Ok(())
    }

    fn set_current_time_packet(&self) -> Packet {
        use chrono::{Datelike, Timelike};

        let now = chrono::Local::now();

        Packet::TranslatedData(TranslatedData {
            datalog: self.inverter.datalog,
            device_function: DeviceFunction::WriteMulti,
            inverter: self.inverter.serial,
            register: 12,
            values: vec![
                (now.year() - 2000) as u8,
                now.month() as u8,
                now.day() as u8,
                now.hour() as u8,
                now.minute() as u8,
                now.second() as u8,
            ],
        })
    }
}
