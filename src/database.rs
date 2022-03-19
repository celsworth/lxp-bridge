use crate::prelude::*;

pub type ValueSender = broadcast::Sender<serde_json::Value>;

pub struct Database {
    config: Rc<Config>,
    from_coordinator: ValueSender,
}

impl Database {
    pub fn new(config: Rc<Config>, from_coordinator: ValueSender) -> Self {
        Self {
            config,
            from_coordinator,
        }
    }

    pub async fn start(&self) -> Result<()> {
        futures::try_join!(self.inserter())?;
        Ok(())
    }

    async fn inserter(&self) -> Result<()> {
        use sqlx::Connection;

        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(10)
            .connect(&self.config.database.url)
            .await?;

        //let mut conn = sqlx::AnyConnection::connect(&self.config.database.url).await?;
        sqlx::migrate!().run(&pool).await?;

        let mut receiver = self.from_coordinator.subscribe();

        loop {
            let data = receiver.recv().await?;
            let object = data.as_object().unwrap();

            sqlx::query_as(
                r#"
                INSERT INTO inputs
                  ( p_pv )
                VALUES
                  ( $1 )
                RETURNING id
                "#,
            )
            .bind(&object["v_bat"].as_u64())
            .fetch_one(&pool)
            .await?;
        }
    }
}
