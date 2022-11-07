mod common;
use common::*;

#[tokio::test]
async fn happy_path() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let register = 0 as u16;
    let count = 40 as u16;

    let subject = coordinator::commands::read_hold::ReadHold::new(
        channels.clone(),
        inverter.clone(),
        register,
        count,
    );

    let reply = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: inverter.datalog(),
        device_function: lxp::packet::DeviceFunction::ReadHold,
        inverter: inverter.serial(),
        register: 0,
        values: vec![0, 0],
    });

    let sf = async {
        let result = subject.run().await;
        assert_eq!(result?, reply.clone());
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

    let register = 0 as u16;
    let count = 40 as u16;

    let subject = coordinator::commands::read_hold::ReadHold::new(
        channels.clone(),
        inverter.clone(),
        register,
        count,
    );

    let sf = async {
        let result = subject.run().await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "wait_for_reply TranslatedData(TranslatedData { datalog: 2222222222, device_function: ReadHold, inverter: 5555555555, register: 0, values: [40, 0] }) - timeout"
        );
        Ok::<(), anyhow::Error>(())
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

    let register = 0 as u16;
    let count = 40 as u16;

    let subject = coordinator::commands::read_hold::ReadHold::new(
        channels.clone(),
        inverter.clone(),
        register,
        count,
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
