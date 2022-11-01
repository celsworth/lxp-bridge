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
fn build_heartbeat() {
    let packet = Packet::Heartbeat(lxp::packet::Heartbeat { datalog: datalog() });

    assert_eq!(
        lxp::packet::TcpFrameFactory::build(&packet),
        vec![161, 26, 2, 0, 13, 0, 1, 193, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 0]
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
fn parse_read_inputs_all_protocol5_reply() {
    let input = [
        161, 26, 5, 0, 29, 1, 1, 194, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 15, 1, 1, 4, 53, 53,
        53, 53, 53, 53, 53, 53, 53, 53, 0, 0, 254, 16, 0, 0, 0, 0, 0, 121, 15, 247, 1, 100, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74, 9, 0, 0, 0, 0, 141, 19, 0, 0, 0, 0, 120, 0, 0, 0, 73,
        9, 212, 1, 193, 124, 140, 19, 0, 0, 0, 0, 0, 0, 8, 2, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 8, 0,
        1, 0, 0, 0, 0, 0, 55, 0, 118, 15, 232, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        232, 1, 0, 0, 177, 1, 0, 0, 61, 0, 0, 0, 41, 0, 0, 0, 0, 0, 0, 0, 217, 10, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 24, 0, 25, 0, 39, 0, 24, 0, 0, 0, 75, 52, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 0, 0, 0, 136, 19, 23, 2, 0, 0, 0, 0, 1, 8, 0, 16, 16, 220,
        255, 42, 48, 18, 255, 171, 14, 131, 240, 96, 157, 16, 2, 0, 1, 0, 50, 0, 76, 255, 0, 0,
        245, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 16, 246, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 0,
        0, 50, 48, 52, 51, 48, 50, 50, 52, 48, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 189,
        20,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: datalog(),
            device_function: lxp::packet::DeviceFunction::ReadInput,
            inverter: serial(),
            register: 0,
            values: vec![
                16, 0, 0, 0, 0, 0, 121, 15, 247, 1, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74,
                9, 0, 0, 0, 0, 141, 19, 0, 0, 0, 0, 120, 0, 0, 0, 73, 9, 212, 1, 193, 124, 140, 19,
                0, 0, 0, 0, 0, 0, 8, 2, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 8, 0, 1, 0, 0, 0, 0, 0, 55,
                0, 118, 15, 232, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 232, 1, 0, 0,
                177, 1, 0, 0, 61, 0, 0, 0, 41, 0, 0, 0, 0, 0, 0, 0, 217, 10, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 24, 0, 25, 0, 39, 0, 24, 0, 0, 0, 75, 52, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 34, 0, 0, 0, 136, 19, 23, 2, 0, 0, 0, 0, 1, 8, 0, 16,
                16, 220, 255, 42, 48, 18, 255, 171, 14, 131, 240, 96, 157, 16, 2, 0, 1, 0, 50, 0,
                76, 255, 0, 0, 245, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 16, 246, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 4, 1, 0, 0, 50, 48, 52, 51, 48, 50, 50, 52, 48, 49, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0
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

#[test]
fn build_read_param() {
    let packet = Packet::ReadParam(lxp::packet::ReadParam {
        datalog: datalog(),
        register: 7,
        values: vec![0, 0], // not used?
    });

    assert_eq!(
        lxp::packet::TcpFrameFactory::build(&packet),
        vec![161, 26, 2, 0, 14, 0, 1, 195, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 7, 0]
    );
}

#[test]
fn parse_read_param_reply() {
    let input = [
        161, 26, 2, 0, 18, 0, 1, 195, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 0, 0, 2, 0, 44, 1,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::ReadParam(lxp::packet::ReadParam {
            datalog: datalog(),
            register: 0,
            values: vec![44, 1]
        })
    );
}

#[test]
fn build_write_param() {
    // this isn't hooked up yet anyway, no way to make a WriteParam packet from MQTT
    // not even sure if this is right.. 2 bytes of register, 2 bytes len, then x bytes of values?
    let packet = Packet::WriteParam(lxp::packet::WriteParam {
        datalog: datalog(),
        register: 7,
        values: vec![0, 3],
    });

    assert_eq!(
        lxp::packet::TcpFrameFactory::build(&packet),
        vec![
            161, 26, 2, 0, 18, 0, 1, 196, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 7, 0, 2, 0, 0, 3
        ]
    );
}

#[test]
fn parse_write_param_reply() {
    // I guess this means that the inverter wrote 3 registers starting at 7?
    // not sure if it's register 7, 0 then 3,   or register 7, then 0, 3
    let input = [
        161, 26, 2, 0, 15, 0, 1, 196, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 7, 0, 3,
    ];

    assert_eq!(
        lxp::packet::Parser::parse(&input).unwrap(),
        Packet::WriteParam(lxp::packet::WriteParam {
            datalog: datalog(),
            register: 7,
            values: vec![0, 3]
        })
    );
}
