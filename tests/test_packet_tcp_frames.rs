mod common;
use common::*;

fn datalog() -> Serial {
    Serial::from_str("2222222222").unwrap()
}

fn serial() -> Serial {
    Serial::from_str("5555555555").unwrap()
}

#[test]
fn build_read_hold() {
    let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: datalog(),
        device_function: lxp::packet::DeviceFunction::ReadHold,
        inverter: serial(),
        register: 12,
        values: vec![3, 0],
    });

    assert_eq!(
        lxp::packet::TcpFrameFactory::build(&packet),
        vec![
            161, 26, 1, 0, 32, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 18, 0, 0, 3, 53,
            53, 53, 53, 53, 53, 53, 53, 53, 53, 12, 0, 3, 0, 112, 38
        ]
    );
}

#[test]
fn build_read_inputs() {
    let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: datalog(),
        device_function: lxp::packet::DeviceFunction::ReadInput,
        inverter: serial(),
        register: 0,
        values: vec![40, 0],
    });

    assert_eq!(
        lxp::packet::TcpFrameFactory::build(&packet),
        vec![
            161, 26, 1, 0, 32, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 18, 0, 0, 4, 53,
            53, 53, 53, 53, 53, 53, 53, 53, 53, 0, 0, 40, 0, 42, 132
        ]
    );
}

#[test]
fn build_write_multi() {
    let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: datalog(),
        device_function: lxp::packet::DeviceFunction::WriteMulti,
        inverter: serial(),
        register: 12,
        values: vec![22, 6, 19, 20, 23, 33],
    });

    assert_eq!(
        lxp::packet::TcpFrameFactory::build(&packet),
        vec![
            161, 26, 2, 0, 39, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 25, 0, 0, 16, 53,
            53, 53, 53, 53, 53, 53, 53, 53, 53, 12, 0, 3, 0, 6, 22, 6, 19, 20, 23, 33, 115, 71
        ]
    );
}
