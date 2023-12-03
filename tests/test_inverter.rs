mod common;
use common::*;

// these tests are shonky, I need to work on how to test the inverter code reliably

#[allow(dead_code)]
//#[tokio::test]
async fn serials_fixed_in_outgoing_packets() {
    // in this test, we configure an inverter with serials of 0000000000, but make the fake
    // inverter talk to us using a serial of XXXXXXXXXX. We expect the final packet to be
    // sent to the inverter using the corrected serials.

    common_setup();

    let config = Factory::example_config_wrapped();
    let inverter = config::Inverter {
        enabled: true,
        host: "localhost".to_owned(),
        port: 1235,
        datalog: Serial::from_str("0000000000").unwrap(),
        serial: Serial::from_str("0000000000").unwrap(),
        heartbeats: None,
        publish_holdings_on_connect: None,
        read_timeout: None,
    };
    let channels = Channels::new();
    let inverter = lxp::inverter::Inverter::new(config, &inverter, channels.clone());

    let mut from_inverter = channels.from_inverter.subscribe();

    let tf = async {
        // pretend to be an inverter
        let listener = tokio::net::TcpListener::bind("localhost:1235")
            .await
            .unwrap();

        let (socket, _) = listener.accept().await?;

        // send a packet from inverter with the correct serials
        socket.writable().await?;
        socket
            .try_write(&[
                161, 26, 2, 0, 111, 0, 1, 194, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 97, 0, 1, 4,
                88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 0, 0, 80, 16, 0, 0, 0, 0, 0, 0, 0, 233, 1,
                44, 0, 0, 47, 0, 0, 0, 0, 0, 0, 0, 0, 89, 9, 124, 9, 0, 16, 0, 0, 134, 19, 105, 8,
                0, 0, 124, 3, 232, 3, 124, 9, 0, 10, 80, 112, 134, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 75, 0, 3, 0, 3, 0, 95, 0, 0, 0, 1, 0, 26, 0, 98, 14, 33, 11, 151,
                62,
            ])
            .unwrap();

        assert_eq!(
            unwrap_inverter_channeldata_packet(from_inverter.recv().await.unwrap()),
            Packet::TranslatedData(lxp::packet::TranslatedData {
                datalog: Serial::from_str("XXXXXXXXXX").unwrap(),
                device_function: lxp::packet::DeviceFunction::ReadInput,
                inverter: Serial::from_str("XXXXXXXXXX").unwrap(),
                register: 0,
                values: vec![
                    16, 0, 0, 0, 0, 0, 0, 0, 233, 1, 44, 0, 0, 47, 0, 0, 0, 0, 0, 0, 0, 0, 89, 9,
                    124, 9, 0, 16, 0, 0, 134, 19, 105, 8, 0, 0, 124, 3, 232, 3, 124, 9, 0, 10, 80,
                    112, 134, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 75, 0, 3, 0, 3, 0, 95,
                    0, 0, 0, 1, 0, 26, 0, 98, 14, 33, 11,
                ]
            })
        );

        let packet = Packet::TranslatedData(lxp::packet::TranslatedData {
            datalog: Serial::from_str("0000000000").unwrap(),
            device_function: lxp::packet::DeviceFunction::ReadInput,
            inverter: Serial::from_str("0000000000").unwrap(),
            register: 12,
            values: vec![1, 0],
        });
        channels
            .to_inverter
            .send(lxp::inverter::ChannelData::Packet(packet))
            .unwrap();

        // wait for inverter to receive the ReadHold and verify the serials have been replaced
        socket.readable().await?;
        let mut buf = [0; 38];
        assert_eq!(38, socket.try_read(&mut buf).unwrap());
        assert_eq!(
            buf.to_vec(),
            &[
                161, 26, 1, 0, 32, 0, 1, 194, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 18, 0, 0, 4,
                88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 12, 0, 1, 0, 220, 47
            ]
        );

        inverter.stop();

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, inverter.start()).unwrap();
}

#[allow(dead_code)]
//#[tokio::test]
async fn test_replies_to_heartbeats() {
    common_setup();

    let config = Factory::example_config_wrapped();
    let inverter = config::Inverter {
        enabled: true,
        host: "localhost".to_owned(),
        port: 1235,
        datalog: Serial::from_str("XXXXXXXXXX").unwrap(),
        serial: Serial::from_str("0000000000").unwrap(),
        heartbeats: Some(true),
        publish_holdings_on_connect: None,
        read_timeout: None,
    };
    let channels = Channels::new();
    let inverter = lxp::inverter::Inverter::new(config, &inverter, channels.clone());

    let from_inverter = channels.from_inverter.subscribe();

    let tf = async {
        // pretend to be an inverter
        let listener = tokio::net::TcpListener::bind("localhost:1235")
            .await
            .unwrap();

        let (socket, _) = listener.accept().await?;

        socket.writable().await?;
        socket
            .try_write(&[
                161, 26, 2, 0, 13, 0, 1, 193, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 0,
            ])
            .unwrap();

        // wait for inverter to receive the ReadHold and verify the serials have been replaced
        socket.readable().await?;
        let mut buf = [0; 19];
        assert_eq!(19, socket.try_read(&mut buf).unwrap());
        assert_eq!(
            buf.to_vec(),
            &[161, 26, 2, 0, 13, 0, 1, 193, 88, 88, 88, 88, 88, 88, 88, 88, 88, 88, 0,]
        );

        debug!("stop");
        inverter.stop();

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(tf, inverter.start()).unwrap();
}
