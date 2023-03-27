mod common;
use common::*;

#[tokio::test]
#[cfg_attr(not(feature = "mocks"), ignore)]
async fn update_time() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let subject =
        coordinator::commands::timesync::TimeSync::new(channels.clone(), inverter.clone());

    let sf = async {
        subject.run().await?;
        Ok(())
    };

    let tf = async {
        // wait for packet to request time from inverter and verify it is correct
        assert_eq!(
            unwrap_inverter_channeldata_packet(channels.to_inverter.subscribe().recv().await?),
            Packet::TranslatedData(lxp::packet::TranslatedData {
                datalog: inverter.datalog(),
                device_function: lxp::packet::DeviceFunction::ReadHold,
                inverter: inverter.serial(),
                register: 12,
                values: vec![3, 0]
            })
        );

        // send reply with time of 2022-06-18 21:03:10
        let inverter_time_packet = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: inverter.datalog(),
            device_function: lxp::packet::DeviceFunction::ReadHold,
            inverter: inverter.serial(),
            register: 12,
            values: vec![22, 6, 18, 21, 03, 10],
        });
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(inverter_time_packet))?;

        // wait for packet to set time
        assert_eq!(
            unwrap_inverter_channeldata_packet(channels.to_inverter.subscribe().recv().await?),
            Packet::TranslatedData(lxp::packet::TranslatedData {
                datalog: inverter.datalog(),
                device_function: lxp::packet::DeviceFunction::WriteMulti,
                inverter: inverter.serial(),
                register: 12,
                values: vec![22, 3, 4, 5, 6, 7] // hardcoded test time
            })
        );

        // send reply that we set the time
        let inverter_ok_packet = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: inverter.datalog(),
            device_function: lxp::packet::DeviceFunction::WriteMulti,
            inverter: inverter.serial(),
            register: 12,
            values: vec![3, 0],
        });
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(inverter_ok_packet))?;

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, sf).unwrap();
}

#[tokio::test]
#[cfg_attr(not(feature = "mocks"), ignore)]
async fn time_already_correct() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let subject =
        coordinator::commands::timesync::TimeSync::new(channels.clone(), inverter.clone());

    let sf = async {
        subject.run().await?;
        Ok(())
    };

    let tf = async {
        // wait for packet to request time from inverter and verify it is correct
        assert_eq!(
            unwrap_inverter_channeldata_packet(channels.to_inverter.subscribe().recv().await?),
            Packet::TranslatedData(lxp::packet::TranslatedData {
                datalog: inverter.datalog(),
                device_function: lxp::packet::DeviceFunction::ReadHold,
                inverter: inverter.serial(),
                register: 12,
                values: vec![3, 0]
            })
        );

        // send reply with hardcoded test time
        let inverter_time_packet = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: inverter.datalog(),
            device_function: lxp::packet::DeviceFunction::ReadHold,
            inverter: inverter.serial(),
            register: 12,
            values: vec![22, 3, 4, 5, 6, 7], // hardcoded test time
        });
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(inverter_time_packet))?;

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, sf).unwrap();
}
