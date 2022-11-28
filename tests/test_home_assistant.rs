mod common;
use common::*;

#[tokio::test]
async fn all_empty_with_no_sensors() {
    common_setup();

    let mut config = Factory::example_config();
    config.mqtt.homeassistant.sensors = vec![];

    let r = home_assistant::Config::all(&config.inverters[0], &config.mqtt);

    assert!(r.is_ok());
    assert_eq!(r.unwrap(), vec![]);
}

#[tokio::test]
async fn all_has_soc() {
    common_setup();

    let mut config = Factory::example_config();
    config.mqtt.homeassistant.sensors = vec!["soc".to_owned()];

    let r = home_assistant::Config::all(&config.inverters[0], &config.mqtt);

    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        vec![
            mqtt::Message {
                topic: "homeassistant/sensor/lxp_2222222222/soc/config".to_string(),
                payload: "{\"device_class\":\"battery\",\"name\":\"Battery Percentage\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"measurement\",\"value_template\":\"{{ value_json.soc }}\",\"unit_of_measurement\":\"%\",\"unique_id\":\"lxp_2222222222_soc\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]}}".to_string()
            }
        ]
    );
}

#[tokio::test]
async fn all_has_v_pv() {
    common_setup();

    let mut config = Factory::example_config();
    config.mqtt.homeassistant.sensors = vec!["v_pv".to_owned()];

    let r = home_assistant::Config::all(&config.inverters[0], &config.mqtt);

    assert!(r.is_ok());
    assert_eq!(r.unwrap(), vec![
        mqtt::Message {
            topic: "homeassistant/sensor/lxp_2222222222/v_pv/config".to_string(),
            payload: "{\"device_class\":\"voltage\",\"name\":\"Voltage (PV Array)\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"measurement\",\"value_template\":\"{{ value_json.v_pv }}\",\"unit_of_measurement\":\"V\",\"unique_id\":\"lxp_2222222222_v_pv\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]}}".to_string()
        }
    ]);
}

#[tokio::test]
async fn all_has_p_pv() {
    common_setup();

    let mut config = Factory::example_config();
    config.mqtt.homeassistant.sensors = vec!["p_pv".to_owned()];

    let r = home_assistant::Config::all(&config.inverters[0], &config.mqtt);

    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        vec![
            mqtt::Message {
                topic: "homeassistant/sensor/lxp_2222222222/p_pv/config".to_string(),
                payload: "{\"device_class\":\"power\",\"name\":\"Power (PV Array)\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"measurement\",\"value_template\":\"{{ value_json.p_pv }}\",\"unit_of_measurement\":\"W\",\"unique_id\":\"lxp_2222222222_p_pv\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]}}".to_string()
            }
        ]
    );
}

#[tokio::test]
async fn all_has_e_pv_all() {
    common_setup();

    let mut config = Factory::example_config();
    config.mqtt.homeassistant.sensors = vec!["e_pv_all".to_owned()];

    let r = home_assistant::Config::all(&config.inverters[0], &config.mqtt);

    assert!(r.is_ok());
    assert_eq!(r.unwrap(), vec![
        mqtt::Message {
            topic: "homeassistant/sensor/lxp_2222222222/e_pv_all/config".to_string(),
            payload: "{\"device_class\":\"energy\",\"name\":\"PV Generation (All time)\",\"state_topic\":\"lxp/2222222222/inputs/all\",\"state_class\":\"total_increasing\",\"value_template\":\"{{ value_json.e_pv_all }}\",\"unit_of_measurement\":\"kWh\",\"unique_id\":\"lxp_2222222222_e_pv_all\",\"device\":{\"manufacturer\":\"LuxPower\",\"name\":\"lxp_2222222222\",\"identifiers\":[\"lxp_2222222222\"]}}".to_string() }
    ]);
}
