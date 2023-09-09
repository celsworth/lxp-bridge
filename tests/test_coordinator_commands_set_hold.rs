mod common;
use common::*;

#[tokio::test]
async fn happy_path() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let register = 5 as u16;
    let value = 10 as u16;

    let subject = coordinator::commands::set_hold::SetHold::new(
        channels.clone(),
        inverter.clone(),
        register,
        value,
    );

    let reply = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: inverter.datalog(),
        device_function: lxp::packet::DeviceFunction::WriteSingle,
        inverter: inverter.serial(),
        register: 5,
        values: vec![10, 0],
    });

    let sf = async {
        let result = subject.run().await;
        assert_eq!(result?, reply.clone());
        Ok(())
    };

    let tf = async {
        channels.to_inverter.subscribe().recv().await?;
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(reply.clone()))?;
        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, sf).unwrap();
}

#[tokio::test]
async fn bad_reply() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let register = 5 as u16;
    let value = 10 as u16;

    let subject = coordinator::commands::set_hold::SetHold::new(
        channels.clone(),
        inverter.clone(),
        register,
        value,
    );

    let reply = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: inverter.datalog(),
        device_function: lxp::packet::DeviceFunction::WriteSingle,
        inverter: inverter.serial(),
        register: 5,
        values: vec![200, 0], // reply has wrong value
    });

    let sf = async {
        let result = subject.run().await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "failed to set register 5, got back value 200 (wanted 10)"
        );
        Ok::<(), anyhow::Error>(())
    };

    let tf = async {
        channels.to_inverter.subscribe().recv().await?;
        channels
            .from_inverter
            .send(lxp::inverter::ChannelData::Packet(reply.clone()))?;
        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, sf).unwrap();
}

#[tokio::test]
async fn no_reply() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let register = 5 as u16;
    let value = 10 as u16;

    let subject = coordinator::commands::set_hold::SetHold::new(
        channels.clone(),
        inverter.clone(),
        register,
        value,
    );

    let sf = async {
        let result = subject.run().await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "wait_for_reply TranslatedData(TranslatedData { datalog: 2222222222, device_function: WriteSingle, inverter: 5555555555, register: 5, values: [10, 0] }) - timeout"
        );
        Ok(())
    };

    let tf = async {
        channels.to_inverter.subscribe().recv().await?;
        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, sf).unwrap();
}

#[tokio::test]
async fn inverter_not_receiving() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let register = 5 as u16;
    let value = 10 as u16;

    let subject = coordinator::commands::set_hold::SetHold::new(
        channels.clone(),
        inverter.clone(),
        register,
        value,
    );

    let sf = async {
        let result = subject.run().await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "send(to_inverter) failed - channel closed?"
        );
        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(sf).unwrap();
}
