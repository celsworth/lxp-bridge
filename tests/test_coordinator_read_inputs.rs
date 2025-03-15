mod common;
use common::*;
use lxp_bridge::prelude::*;
use lxp_bridge::coordinator::commands::read_inputs::ReadInputs;
use lxp_bridge::lxp;
use lxp_bridge::lxp::packet::{DeviceFunction, Packet, TranslatedData};

#[tokio::test]
#[cfg_attr(not(feature = "mocks"), ignore)]
async fn read_inputs_sends_packet() {
    let channels = Channels::new();
    let inverter = Factory::inverter();
    let read_inputs = ReadInputs::new(channels.clone(), inverter.clone(), 0_u16, 1_u16);

    let mut receiver = channels.to_inverter.subscribe();

    // Create a task to send the response after a delay
    let response = Packet::TranslatedData(TranslatedData {
        datalog: inverter.datalog(),
        device_function: DeviceFunction::ReadInput,
        inverter: inverter.serial(),
        register: 0,
        values: vec![0, 0],
    });
    let channels_clone = channels.clone();
    let response_clone = response.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        channels_clone.from_inverter.send(lxp::inverter::ChannelData::Packet(response_clone)).unwrap();
    });

    // Run the read_inputs command
    read_inputs.run().await.unwrap();

    // Check that the correct packet was sent
    match receiver.recv().await.unwrap() {
        lxp::inverter::ChannelData::Packet(packet) => {
            match packet {
                Packet::TranslatedData(td) => {
                    assert_eq!(td.datalog, inverter.datalog());
                    assert_eq!(td.device_function, DeviceFunction::ReadInput);
                    assert_eq!(td.inverter, inverter.serial());
                    assert_eq!(td.register, 0);
                    assert_eq!(td.values, vec![1, 0]);
                }
                _ => panic!("Expected TranslatedData packet"),
            }
        }
        _ => panic!("Expected Packet"),
    }
} 