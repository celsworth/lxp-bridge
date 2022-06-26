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

    let mut config = Factory::example_config();
    config.influx.url = mockito::server_url();
    let channels = Channels::new();

    let influx = Influx::new(Rc::new(RefCell::new(config)), channels.clone());

    let tf = async {
        let json = json!({ "time": 1, "soc": 100, "v_bat": 52.4 });
        channels
            .to_influx
            .send(influx::ChannelData::InputData(json))?;
        channels.to_influx.send(influx::ChannelData::Shutdown)?;
        Ok(())
    };

    let mock = mock_influxdb()
        .match_body("inputs soc=100i,v_bat=52.4 1000000000")
        .create();

    futures::try_join!(influx.start(), tf).unwrap();

    mock.assert();
}
