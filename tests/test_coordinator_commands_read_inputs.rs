mod common;
use common::*;

#[tokio::test]
async fn stuff() {
    common_setup();

    let config = example_config();
    let inverter = config.inverters[0].clone();

    let from_inverter = sender();
    let to_inverter = sender();
    let register = 0 as u16;
    let count = 40 as u16;

    let subject = coordinator::commands::read_inputs::ReadInputs::new(
        from_inverter.clone(),
        to_inverter.clone(),
        inverter.clone(),
        register,
        count,
    );

    let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: inverter.datalog,
        device_function: lxp::packet::DeviceFunction::ReadInput,
        inverter: inverter.serial,
        register: 0,
        values: vec![0, 0],
    });

    let sf = async {
        let reply = subject.run().await?;
        assert_eq!(reply, packet.clone());
        Ok(())
    };

    let tf = async {
        to_inverter.subscribe().recv().await?;
        from_inverter.send(lxp::inverter::ChannelData::Packet(packet.clone()))?;
        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, sf).unwrap();
}
