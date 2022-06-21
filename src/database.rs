use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum ChannelData {
    ReadInputAll(Box<lxp::packet::ReadInputAll>),
    Shutdown,
}

pub type Sender = broadcast::Sender<ChannelData>;

enum DatabaseType {
    MySQL,
    Postgres,
    SQLite,
}

pub struct Database {
    config: config::Database,
    from_coordinator: Sender,
}

impl Database {
    pub fn new(config: config::Database, from_coordinator: Sender) -> Self {
        Self {
            config,
            from_coordinator,
        }
    }

    pub async fn start(&self) -> Result<()> {
        // TODO: could log the url but would need to redact password
        info!("initializing database");

        futures::try_join!(self.inserter())?;

        info!("database loop exiting");

        Ok(())
    }

    fn database(&self) -> Result<DatabaseType> {
        let prefix: Vec<&str> = self.config.url.splitn(2, ':').collect();
        match prefix[0] {
            "sqlite" => Ok(DatabaseType::SQLite),
            "mysql" => Ok(DatabaseType::MySQL),
            "postgres" => Ok(DatabaseType::Postgres),
            _ => Err(anyhow!("unsupported database {}", self.config.url)),
        }
    }

    async fn connect(&self) -> Result<sqlx::any::AnyConnection> {
        use sqlx::ConnectOptions;
        sqlx::any::AnyConnectOptions::from_str(&self.config.url)?
            .disable_statement_logging()
            .connect()
            .await
            .map_err(|err| anyhow!("database connection error: {}", err))
    }

    async fn migrate(&self, conn: &mut sqlx::AnyConnection) -> Result<()> {
        use DatabaseType::*;

        // work out migration directory to use based on database url
        match self.database()? {
            SQLite => sqlx::migrate!("db/migrations/sqlite").run(conn).await?,
            MySQL => sqlx::migrate!("db/migrations/mysql").run(conn).await?,
            Postgres => sqlx::migrate!("db/migrations/postgres").run(conn).await?,
        }

        Ok(())
    }

    async fn inserter(&self) -> Result<()> {
        let mut conn = self.connect().await?;
        info!("database connected");
        self.migrate(&mut conn).await?;

        let mut receiver = self.from_coordinator.subscribe();

        let values = match self.database()? {
            DatabaseType::MySQL => Self::values_for_mysql(),
            _ => Self::values_for_not_mysql(),
        };

        let query = format!(
            r#"
            INSERT INTO inputs
              ( status,
                v_pv, v_pv_1, v_pv_2, v_pv_3, v_bat,
                soc, soh,
                p_pv, p_pv_1, p_pv_2, p_pv_3,
                p_charge, p_discharge,
                v_ac_r, v_ac_s, v_ac_t, f_ac,
                p_inv, p_rec,
                pf,
                v_eps_r, v_eps_s, v_eps_t, f_eps,
                p_to_grid, p_to_user,
                e_pv_day, e_pv_day_1, e_pv_day_2, e_pv_day_3,
                e_inv_day, e_rec_day, e_chg_day, e_dischg_day,
                e_eps_day, e_to_grid_day, e_to_user_day,
                v_bus_1, v_bus_2,

                e_pv_all, e_pv_all_1, e_pv_all_2, e_pv_all_3,
                e_inv_all, e_rec_all, e_chg_all, e_dischg_all,
                e_eps_all, e_to_grid_all, e_to_user_all,
                t_inner, t_rad_1, t_rad_2, t_bat,
                runtime,

                max_chg_curr, max_dischg_curr, charge_volt_ref, dischg_cut_volt,
                bat_status_0, bat_status_1, bat_status_2, bat_status_3, bat_status_4,
                bat_status_5, bat_status_6, bat_status_7, bat_status_8, bat_status_9,
                bat_status_inv,
                bat_count, bat_capacity,

                datalog, created_at
              )
            VALUES {} "#,
            values
        );

        loop {
            use ChannelData::*;

            match receiver.recv().await? {
                Shutdown => break,
                ReadInputAll(data) => {
                    while let Err(err) = self.insert(&mut conn, &query, &data).await {
                        error!("INSERT failed: {:?} - reconnecting in 10s", err);
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                        conn = self.connect().await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn insert(
        &self,
        conn: &mut sqlx::AnyConnection,
        query: &str,
        data: &lxp::packet::ReadInputAll,
    ) -> Result<()> {
        sqlx::query::<sqlx::Any>(query)
            .bind(data.status as i32)
            .bind(data.v_pv)
            .bind(data.v_pv_1)
            .bind(data.v_pv_2)
            .bind(data.v_pv_3)
            .bind(data.v_bat)
            .bind(data.soc as i32)
            .bind(data.soh as i32)
            .bind(data.p_pv as i32)
            .bind(data.p_pv_1 as i32)
            .bind(data.p_pv_2 as i32)
            .bind(data.p_pv_3 as i32)
            .bind(data.p_charge as i32)
            .bind(data.p_discharge as i32)
            .bind(data.v_ac_r)
            .bind(data.v_ac_s)
            .bind(data.v_ac_t)
            .bind(data.f_ac)
            .bind(data.p_inv as i32)
            .bind(data.p_rec as i32)
            .bind(data.pf)
            .bind(data.v_eps_r)
            .bind(data.v_eps_s)
            .bind(data.v_eps_t)
            .bind(data.f_eps)
            .bind(data.p_to_grid as i32)
            .bind(data.p_to_user as i32)
            .bind(data.e_pv_day)
            .bind(data.e_pv_day_1)
            .bind(data.e_pv_day_2)
            .bind(data.e_pv_day_3)
            .bind(data.e_inv_day)
            .bind(data.e_rec_day)
            .bind(data.e_chg_day)
            .bind(data.e_dischg_day)
            .bind(data.e_eps_day)
            .bind(data.e_to_grid_day)
            .bind(data.e_to_user_day)
            .bind(data.v_bus_1)
            .bind(data.v_bus_2)
            .bind(data.e_pv_all)
            .bind(data.e_pv_all_1)
            .bind(data.e_pv_all_2)
            .bind(data.e_pv_all_3)
            .bind(data.e_inv_all)
            .bind(data.e_rec_all)
            .bind(data.e_chg_all)
            .bind(data.e_dischg_all)
            .bind(data.e_eps_all)
            .bind(data.e_to_grid_all)
            .bind(data.e_to_user_all)
            .bind(data.t_inner as i32)
            .bind(data.t_rad_1 as i32)
            .bind(data.t_rad_2 as i32)
            .bind(data.t_bat as i32)
            .bind(data.runtime as i32)
            .bind(data.max_chg_curr)
            .bind(data.max_dischg_curr)
            .bind(data.charge_volt_ref)
            .bind(data.dischg_cut_volt)
            .bind(data.bat_status_0 as i32)
            .bind(data.bat_status_1 as i32)
            .bind(data.bat_status_2 as i32)
            .bind(data.bat_status_3 as i32)
            .bind(data.bat_status_4 as i32)
            .bind(data.bat_status_5 as i32)
            .bind(data.bat_status_6 as i32)
            .bind(data.bat_status_7 as i32)
            .bind(data.bat_status_8 as i32)
            .bind(data.bat_status_9 as i32)
            .bind(data.bat_status_inv as i32)
            .bind(data.bat_count as i32)
            .bind(data.bat_capacity as i32)
            .bind(data.datalog.to_string())
            .bind(data.time.0)
            .persistent(true)
            .fetch_optional(conn)
            .await?;

        Ok(())
    }

    fn values_for_mysql() -> &'static str {
        r#"(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
    }

    fn values_for_not_mysql() -> &'static str {
        r#"($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
            $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28,
            $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42,
            $43, $44, $45, $46, $47, $48, $49, $50, $51, $52, $53, $54, $55, $56,
            $57, $58, $59, $60, $61, $62, $63, $64, $65, $66, $67, $68, $69, $70,
            $71, $72, $73, $74, $75)"#
    }
}
