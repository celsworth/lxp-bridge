use crate::prelude::*;

pub type ValueSender = broadcast::Sender<serde_json::Value>;

enum DatabaseConnection {
    Sqlite(sqlx::SqliteConnection),
}

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
        let conn = self.connect().await?;
        futures::try_join!(self.inserter(conn))?;
        Ok(())
    }

    async fn connect(&self) -> Result<DatabaseConnection> {
        use sqlx::Connection;

        match self.config.database.r#type.as_str() {
            "sqlite" => {
                let url = "sqlite://foo.db";
                let mut conn = sqlx::SqliteConnection::connect(&url).await?;
                sqlx::migrate!().run(&mut conn).await?;
                Ok(DatabaseConnection::Sqlite(conn))
            }
            _ => return Err(anyhow!("unsupported database type")),
        }
    }

    async fn inserter(&self, mut conn: DatabaseConnection) -> Result<()> {
        let mut receiver = self.from_coordinator.subscribe();

        loop {
            let data = receiver.recv().await?;
            let object = data.as_object().unwrap();

            let query = sqlx::query(
                r#"
                INSERT INTO inputs
                  ( v_bat )
                VALUES
                  ( $1 )
                RETURNING id
                "#,
            )
            .bind(&object["v_bat"]);

            match conn {
                DatabaseConnection::Sqlite(ref mut c) => query.fetch_one(c).await?,
            };
        }
    }
}
