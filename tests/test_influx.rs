mod common;
use common::*;

fn mock_influxdb() -> Mock {
    mock("POST", "/write")
        .match_query(Matcher::UrlEncoded("db".to_owned(), "lxp".to_owned()))
        .with_status(204)
}

#[tokio::test]
async fn sends_http_request() {
    common_setup();

    let config = Factory::example_config_wrapped();
    //config.set_influx_url(mockito::server_url());
    config.influx_mut().url = mockito::server_url();
    let channels = Channels::new();

    let influx = Influx::new(config, channels.clone());

    let tf = async {
        let json =
            json!({ "time": 1, "datalog": "BA12345678", "soc": 100, "p_pv": 250, "v_bat": 52.4 });
        channels
            .to_influx
            .send(influx::ChannelData::InputData(json))?;
        channels.to_influx.send(influx::ChannelData::Shutdown)?;
        Ok(())
    };

    let mock = mock_influxdb()
        .match_body("inputs,datalog=BA12345678 p_pv=250i,soc=100i,v_bat=52.4 1000000000")
        .create();

    futures::try_join!(influx.start(), tf).unwrap();

    mock.assert();
}
