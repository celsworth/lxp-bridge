mod common;
use common::*;

#[tokio::test]
async fn for_param() {
    common_setup();

    let inverter = Factory::inverter();

    let packet = lxp::packet::ReadParam {
        datalog: inverter.datalog,
        register: 0,
        values: vec![1, 0],
    };

    assert_eq!(
        mqtt::Message::for_param(packet).unwrap(),
        vec![mqtt::Message {
            topic: "2222222222/param/0".to_owned(),
            payload: "1".to_owned()
        }]
    );
}

#[tokio::test]
async fn for_hold_single() {
    common_setup();

    let inverter = Factory::inverter();

    let packet = lxp::packet::TranslatedData {
        datalog: inverter.datalog,
        device_function: lxp::packet::DeviceFunction::ReadHold,
        inverter: inverter.serial,
        register: 0,
        values: vec![1, 0],
    };

    assert_eq!(
        mqtt::Message::for_hold(packet).unwrap(),
        vec![mqtt::Message {
            topic: "2222222222/hold/0".to_owned(),
            payload: "1".to_owned()
        }]
    );
}
#[tokio::test]
async fn for_hold_multi() {
    common_setup();

    let inverter = Factory::inverter();

    let packet = lxp::packet::TranslatedData {
        datalog: inverter.datalog,
        device_function: lxp::packet::DeviceFunction::ReadHold,
        inverter: inverter.serial,
        register: 12,
        values: vec![22, 6, 7, 8, 9, 0],
    };

    assert_eq!(
        mqtt::Message::for_hold(packet).unwrap(),
        vec![
            mqtt::Message {
                topic: "2222222222/hold/12".to_owned(),
                payload: "1558".to_owned()
            },
            mqtt::Message {
                topic: "2222222222/hold/13".to_owned(),
                payload: "2055".to_owned()
            },
            mqtt::Message {
                topic: "2222222222/hold/14".to_owned(),
                payload: "9".to_owned()
            },
        ]
    );
}
