mod common;
use common::*;

#[tokio::test]
async fn sends_http_request() {
    common_setup();

    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/write")
        .match_query(Matcher::UrlEncoded("db".to_owned(), "lxp".to_owned()))
        .with_status(204)
        .match_body("inputs,datalog=BA12345678 p_pv=250i,soc=100i,v_bat=52.4 1000000000")
        .create();

    let config = Factory::example_config_wrapped();
    //config.set_influx_url(server.url());
    config.influx_mut().url = server.url();
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

    futures::try_join!(influx.start(), tf).unwrap();

    mock.assert();
}
