mod common;
use common::*;

fn datalog() -> Serial {
    Serial::from_str("2222222222").unwrap()
}

fn serial() -> Serial {
    Serial::from_str("5555555555").unwrap()
}

#[test]
fn parse_heartbeat() {
    let input = [
        161, 26, 2, 0, 13, 0, 1, 193, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 0,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::Heartbeat(lxp::packet::Heartbeat { datalog: datalog() })
    );
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
fn parse_read_hold_reply() {
    let input = [
        161, 26, 2, 0, 37, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 23, 0, 1, 3, 53, 53,
        53, 53, 53, 53, 53, 53, 53, 53, 12, 0, 6, 22, 6, 20, 5, 16, 57, 93, 135,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: datalog(),
            device_function: lxp::packet::DeviceFunction::ReadHold,
            inverter: serial(),
            register: 12,
            values: vec![22, 6, 20, 5, 16, 57],
        })
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
fn parse_read_inputs_reply() {
    let input = [
        161, 26, 2, 0, 111, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 97, 0, 1, 4, 53, 53,
        53, 53, 53, 53, 53, 53, 53, 53, 0, 0, 80, 32, 0, 0, 0, 0, 0, 0, 0, 250, 1, 77, 0, 0, 53, 0,
        0, 0, 0, 0, 0, 128, 13, 0, 0, 114, 9, 0, 16, 132, 0, 142, 19, 0, 0, 198, 13, 202, 5, 232,
        3, 114, 9, 0, 10, 80, 112, 142, 19, 0, 0, 0, 0, 0, 0, 36, 15, 0, 0, 0, 0, 0, 0, 91, 0, 83,
        0, 87, 0, 114, 0, 0, 0, 1, 0, 102, 0, 174, 14, 183, 12, 71, 187,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: datalog(),
            device_function: lxp::packet::DeviceFunction::ReadInput,
            inverter: serial(),
            register: 0,
            values: vec![
                32, 0, 0, 0, 0, 0, 0, 0, 250, 1, 77, 0, 0, 53, 0, 0, 0, 0, 0, 0, 128, 13, 0, 0,
                114, 9, 0, 16, 132, 0, 142, 19, 0, 0, 198, 13, 202, 5, 232, 3, 114, 9, 0, 10, 80,
                112, 142, 19, 0, 0, 0, 0, 0, 0, 36, 15, 0, 0, 0, 0, 0, 0, 91, 0, 83, 0, 87, 0, 114,
                0, 0, 0, 1, 0, 102, 0, 174, 14, 183, 12
            ]
        })
    );
}

#[test]
fn build_write_single() {
    let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
        datalog: datalog(),
        device_function: lxp::packet::DeviceFunction::WriteSingle,
        inverter: serial(),
        register: 66,
        values: vec![100, 0],
    });

    assert_eq!(
        lxp::packet::TcpFrameFactory::build(&packet),
        vec![
            161, 26, 1, 0, 32, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 18, 0, 0, 6, 53,
            53, 53, 53, 53, 53, 53, 53, 53, 53, 66, 0, 100, 0, 136, 61
        ]
    );
}

#[test]
fn parse_write_single_reply() {
    let input = [
        161, 26, 2, 0, 32, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 18, 0, 1, 6, 53, 53,
        53, 53, 53, 53, 53, 53, 53, 53, 66, 0, 100, 0, 73, 173,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: datalog(),
            device_function: lxp::packet::DeviceFunction::WriteSingle,
            inverter: serial(),
            register: 66,
            values: vec![100, 0]
        })
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

#[test]
fn parse_write_multi_reply() {
    let input = [
        161, 26, 2, 0, 32, 0, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 18, 0, 1, 16, 53, 53,
        53, 53, 53, 53, 53, 53, 53, 53, 12, 0, 3, 0, 226, 187,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: datalog(),
            device_function: lxp::packet::DeviceFunction::WriteMulti,
            inverter: serial(),
            register: 12,
            values: vec![3, 0]
        })
    );
}
