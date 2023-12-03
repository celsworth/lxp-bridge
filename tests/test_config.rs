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
    assert!(inverter.enabled());
    assert_eq!(inverter.heartbeats(), false);
    assert_eq!(inverter.publish_holdings_on_connect(), false);
}

#[test]
fn inverter_heartbeats() {
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "heartbeats": false });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.heartbeats(), false);
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "heartbeats": true });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.heartbeats(), true);
}

#[test]
fn inverter_publish_holdings_on_connect() {
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "publish_holdings_on_connect": false });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.publish_holdings_on_connect(), false);
    let input = json!({ "host": "host", "port": 8000, "serial": "TESTSERIAL", "datalog": "TESTDATALO", "publish_holdings_on_connect": true });
    let inverter: config::Inverter = serde_json::from_value(input).unwrap();
    assert_eq!(inverter.publish_holdings_on_connect(), true);
}

#[test]
fn database_defaults() {
    let input = json!({ "url": "url" });
    let database: config::Database = serde_json::from_value(input).unwrap();
    assert!(database.enabled());
}

#[test]
fn mqtt_defaults() {
    let input = json!({ "host": "host" });
    let mqtt: config::Mqtt = serde_json::from_value(input).unwrap();
    assert!(mqtt.enabled());
    assert_eq!(mqtt.port(), 1883);
    assert_eq!(mqtt.namespace(), "lxp");
}

#[test]
fn homeassistant_defaults() {
    let input = json!({});
    let ha: config::HomeAssistant = serde_json::from_value(input).unwrap();
    assert!(ha.enabled());
    assert_eq!(ha.prefix(), "homeassistant");
}

#[test]
fn enabled_inverters() {
    let config = Factory::example_config_wrapped();

    config.set_inverters(vec![
        config::Inverter {
            enabled: false,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
        config::Inverter {
            enabled: true,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
    ]);

    assert_eq!(config.enabled_inverters().len(), 1);
}

#[test]
fn inverters_for_message() {
    let config = Factory::example_config_wrapped();

    config.set_inverters(vec![
        config::Inverter {
            enabled: true,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
        config::Inverter {
            enabled: false,
            datalog: example_serial(),
            host: "localhost".to_owned(),
            port: 8000,
            serial: example_serial(),
            heartbeats: None,
            publish_holdings_on_connect: None,
            read_timeout: None,
        },
    ]);

    let message = mqtt::Message {
        topic: "cmd/all/foo".to_string(),
        retain: false,
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 1);

    let message = mqtt::Message {
        topic: "cmd/MISMATCHED/foo".to_string(),
        retain: false,
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 0);

    let message = mqtt::Message {
        topic: "cmd/TESTSERIAL/foo".to_string(),
        retain: false,
        payload: "foo".to_string(),
    };

    let r = config.inverters_for_message(&message).unwrap();
    assert_eq!(r.len(), 1);
}

#[test]
fn enabled_databases() {
    let config = Factory::example_config_wrapped();

    config.set_databases(vec![
        config::Database {
            enabled: false,
            url: "sqlite://test.db".to_owned(),
        },
        config::Database {
            enabled: true,
            url: "sqlite://test.db".to_owned(),
        },
    ]);

    assert_eq!(config.enabled_databases().len(), 1);
}
