mod common;
use common::*;

#[tokio::test]
async fn happy_path() {
    common_setup();

    let inverter = Factory::inverter();
    let channels = Channels::new();

    let register = 0 as u16;

    let subject = coordinator::commands::read_param::ReadParam::new(
        channels.clone(),
        inverter.clone(),
        register,
    );

    let reply = Packet::ReadParam(lxp::packet::ReadParam {
        datalog: inverter.datalog,
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

    let subject = coordinator::commands::read_param::ReadParam::new(
        channels.clone(),
        inverter.clone(),
        register,
    );

    let sf = async {
        let result = subject.run().await;
        assert_eq!(
            result.unwrap_err().to_string(),
            "wait_for_reply ReadParam(ReadParam { datalog: 2222222222, register: 0, values: [] }) - timeout"
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

    let subject = coordinator::commands::read_param::ReadParam::new(
        channels.clone(),
        inverter.clone(),
        register,
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
