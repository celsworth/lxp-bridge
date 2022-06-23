mod common;
use common::*;

use {futures::TryStreamExt, sqlx::Row};

#[tokio::test]
async fn sqlite_insertion() {
    common_setup();

    let config = config::Database {
        enabled: true,
        url: "sqlite::memory:".to_string(),
    };
    let channels = Channels::new();

    let database = Database::new(config, channels.clone());

    let tf = async {
        let channel_data = database::ChannelData::ReadInputAll(Box::new(Factory::read_input_all()));

        let mut retries = 0;
        // wait up to 5 seconds for database to be ready and accepting messages
        while channels.to_database.send(channel_data.clone()).is_err() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            retries = retries + 1;
            if retries > 50 {
                panic!("database not ready for messages");
            }
        }

        database.stop();

        let mut conn = database.connection().await?;

        let mut retries = 0;
        loop {
            let mut rows = sqlx::query("SELECT * FROM inputs").fetch(&mut conn);
            if let Some(row) = rows.try_next().await? {
                let datalog: &str = row.try_get("datalog")?;
                assert_eq!(datalog, "1234567890");
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            retries = retries + 1;

            if retries > 50 {
                panic!("row not inserted");
            }
        }

        Ok::<(), anyhow::Error>(())
    };

    futures::try_join!(database.start(), tf).unwrap();
}
