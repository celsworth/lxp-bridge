mod common;
use common::*;

#[tokio::test]
async fn all_has_soc() {
    common_setup();

    let config = Factory::example_config();
    let r = home_assistant::Config::new(&config.inverters[0], &config.mqtt).all();

    assert!(r.is_ok());
    assert!(r.unwrap().contains(&mqtt::Message {
        topic: "homeassistant/sensor/lxp_2222222222/soc/config".to_string(),
        retain: true,
        payload: "{\"device_class\":\"battery\",\"name\":\"Battery Percentage\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"measurement\",\"value_template\":\"{{ value_json.soc }}\",\"unit_of_measurement\":\"%\",\"unique_id\":\"lxp_2222222222_soc\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]},\"availability\":{\"topic\":\"lxp/LWT\"}}".to_string()
    }));
}

#[tokio::test]
async fn all_has_v_pv_1() {
    common_setup();

    let config = Factory::example_config();
    let r = home_assistant::Config::new(&config.inverters[0], &config.mqtt).all();

    assert!(r.is_ok());
    assert!(r.unwrap().contains(&mqtt::Message {
        topic: "homeassistant/sensor/lxp_2222222222/v_pv_1/config".to_string(),
        retain: true,
        payload: "{\"device_class\":\"voltage\",\"name\":\"Voltage (PV String 1)\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"measurement\",\"value_template\":\"{{ value_json.v_pv_1 }}\",\"unit_of_measurement\":\"V\",\"unique_id\":\"lxp_2222222222_v_pv_1\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]},\"availability\":{\"topic\":\"lxp/LWT\"}}".to_string()
    }));
}

#[tokio::test]
async fn all_has_p_pv() {
    common_setup();

    let config = Factory::example_config();
    let r = home_assistant::Config::new(&config.inverters[0], &config.mqtt).all();

    assert!(r.is_ok());
    assert!(r.unwrap().contains(&mqtt::Message {
        topic: "homeassistant/sensor/lxp_2222222222/p_pv/config".to_string(),
        retain: true,
        payload: "{\"device_class\":\"power\",\"name\":\"Power (PV Array)\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"measurement\",\"value_template\":\"{{ value_json.p_pv }}\",\"unit_of_measurement\":\"W\",\"unique_id\":\"lxp_2222222222_p_pv\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]},\"availability\":{\"topic\":\"lxp/LWT\"}}".to_string()
    }));
}

#[tokio::test]
async fn all_has_e_pv_all() {
    common_setup();

    let config = Factory::example_config();
    let r = home_assistant::Config::new(&config.inverters[0], &config.mqtt).all();

    assert!(r.is_ok());
    assert!(r.unwrap().contains(&mqtt::Message {
        topic: "homeassistant/sensor/lxp_2222222222/e_pv_all/config".to_string(),
        retain: true,
        payload: "{\"device_class\":\"energy\",\"name\":\"PV Generation (All time)\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"total_increasing\",\"value_template\":\"{{ value_json.e_pv_all }}\",\"unit_of_measurement\":\"kWh\",\"unique_id\":\"lxp_2222222222_e_pv_all\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]},\"availability\":{\"topic\":\"lxp/LWT\"}}".to_string()
    }));
}

#[tokio::test]
async fn all_has_switch_ac_charge() {
    common_setup();

    let config = Factory::example_config();
    let r = home_assistant::Config::new(&config.inverters[0], &config.mqtt).all();

    assert!(r.is_ok());
    assert!(r.unwrap().contains(&mqtt::Message {
        topic: "homeassistant/switch/lxp_2222222222/ac_charge/config".to_string(),
        retain: true,
        payload: "{\"name\":\"AC Charge\",\"state_topic\":\"lxp/2222222222/hold/21/bits\",\"command_topic\":\"lxp/cmd/2222222222/set/ac_charge\",\"value_template\":\"{{ value_json.ac_charge_en }}\",\"unique_id\":\"lxp_2222222222_ac_charge\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]},\"availability\":{\"topic\":\"lxp/LWT\"}}".to_string() 
    }));
}

#[tokio::test]
async fn all_has_number_ac_charge_soc_limit_pct() {
    common_setup();

    let config = Factory::example_config();
    let r = home_assistant::Config::new(&config.inverters[0], &config.mqtt).all();

    assert!(r.is_ok());
    assert!(r.unwrap().contains(&mqtt::Message {
        topic: "homeassistant/number/lxp_2222222222/AcChargeSocLimit/config".to_string(),
        retain: true,
        payload: "{\"name\":\"AC Charge Limit %\",\"state_topic\":\"lxp/2222222222/hold/67\",\"command_topic\":\"lxp/cmd/2222222222/set/hold/67\",\"value_template\":\"{{ float(value) }}\",\"unique_id\":\"lxp_2222222222_number_AcChargeSocLimit\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]},\"availability\":{\"topic\":\"lxp/LWT\"},\"min\":0.0,\"max\":100.0,\"step\":1.0,\"unit_of_measurement\":\"%\"}".to_string()
    }));
}

#[tokio::test]
async fn all_has_time_range_ac_charge_1() {
    common_setup();

    let config = Factory::example_config();
    let r = home_assistant::Config::new(&config.inverters[0], &config.mqtt).all();

    assert!(r.is_ok());
    assert!(r.unwrap().contains(&mqtt::Message {
        topic: "homeassistant/text/lxp_2222222222/ac_charge_1/config".to_string(),
        retain: true,
        payload: r#"{"name":"AC Charge Timeslot 1","state_topic":"lxp/2222222222/ac_charge/1","command_topic":"lxp/cmd/2222222222/set/ac_charge/1","command_template":"{% set parts = value.split(\"-\") %}{\"start\":\"{{ parts[0] }}\", \"end\":\"{{ parts[1] }}\"}","value_template":"{{ value_json[\"start\"] }}-{{ value_json[\"end\"] }}","unique_id":"lxp_2222222222_text_ac_charge/1","device":{"manufacturer":"LuxPower","name":"lxp_2222222222","identifiers":["lxp_2222222222"]},"availability":{"topic":"lxp/LWT"},"pattern":"([01]?[0-9]|2[0-3]):[0-5][0-9]-([01]?[0-9]|2[0-3]):[0-5][0-9]"}"#.to_string()
    }));
}
