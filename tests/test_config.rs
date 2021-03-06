mod common;
use common::*;

pub fn example_serial() -> lxp::inverter::Serial {
    lxp::inverter::Serial::from_str("TESTSERIAL").unwrap()
}

#[test]
fn config_returns_err_on_nonexistent_file() {
    let config = Config::new("nonexistent".to_owned());

    assert!(config.is_err());
}

#[test]
fn config_returns_ok() {
    let config = Config::new("config.yaml.example".to_owned());

    assert!(config.is_ok());
}

#[test]
fn inverter_defaults() {
    let input =
        json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO" });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert!(inverter.enabled);
}

#[test]
fn database_defaults() {
    let input = json!({ "url": "url" });
    let database: config::Database = serde_json::from_value(input).unwrap();
    assert!(database.enabled);
}

#[test]
fn mqtt_defaults() {
    let input = json!({ "host": "host" });
    let mqtt: config::Mqtt = serde_json::from_value(input).unwrap();
    assert!(mqtt.enabled);
    assert_eq!(mqtt.port, 1883);
    assert_eq!(mqtt.namespace, "lxp");
}

#[test]
fn homeassistant_defaults() {
    let input = json!({});
    let ha: config::HomeAssistant = serde_json::from_value(input).unwrap();
    assert!(ha.enabled);
    assert_eq!(ha.prefix, "homeassistant");
    assert_eq!(ha.sensors, ["all"]);
}

#[test]
fn homeassistant_sensors_parsing() {
    let input = json!({ "sensors": "foo,bar" });
    let ha: config::HomeAssistant = serde_json::from_value(input).unwrap();
    assert_eq!(ha.sensors, ["foo", "bar"]);
}

#[test]
fn enabled_inverters() {
    let mut config = Factory::example_config();

    config.inverters = vec![
        config::Inverter {
            enabled: false,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
        },
        config::Inverter {
            enabled: true,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
        },
    ];

    let r: Vec<&config::Inverter> = config.enabled_inverters().collect();
    assert_eq!(r.len(), 1);
}

#[test]
fn inverters_for_message() {
    let mut config = Factory::example_config();

    config.inverters = vec![
        config::Inverter {
            enabled: true,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
        },
        config::Inverter {
            enabled: false,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
        },
    ];

    let message = mqtt::Message {
        topic: "cmd/all/foo".to_string(),
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 1);

    let message = mqtt::Message {
        topic: "cmd/MISMATCHED/foo".to_string(),
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 0);

    let message = mqtt::Message {
        topic: "cmd/TESTSERIAL/foo".to_string(),
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 1);
}

#[test]
fn enabled_databases() {
    let mut config = Factory::example_config();

    config.databases = vec![
        config::Database {
            enabled: false,
            url: "sqlite://test.db".to_owned(),
        },
        config::Database {
            enabled: true,
            url: "sqlite://test.db".to_owned(),
        },
    ];

    let r: Vec<&config::Database> = config.enabled_databases().collect();
    assert_eq!(r.len(), 1);
}
