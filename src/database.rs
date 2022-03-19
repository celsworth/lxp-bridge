use crate::prelude::*;

pub type InputsSender = broadcast::Sender<lxp::packet::ReadInputs>;

pub struct Database {
    config: Rc<Config>,
    from_coordinator: InputsSender,
}

impl Database {
    pub fn new(config: Rc<Config>, from_coordinator: InputsSender) -> Self {
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
        let mut receiver = self.from_coordinator.subscribe();

        use sqlx::Connection;
        let mut conn = sqlx::AnyConnection::connect(&self.config.database.url).await?;
        sqlx::migrate!().run(&mut conn).await?;

        loop {
            let data = receiver.recv().await?;

            let ri1 = data.read_input_1.unwrap();
            let ri2 = data.read_input_2.unwrap();
            let ri3 = data.read_input_3.unwrap();

            sqlx::query::<sqlx::Any>(
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
                VALUES
                  ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                    $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28,
                    $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42,
                    $43, $44, $45, $46, $47, $48, $49, $50, $51, $52, $53, $54, $55, $56,
                    $57, $58, $59, $60, $61, $62, $63, $64, $65, $66, $67, $68, $69, $70,
                    $71, $72, $73, $74, $75
                  )
                "#,
            )
            .bind(ri1.status as i32)
            .bind(ri1.v_pv)
            .bind(ri1.v_pv_1)
            .bind(ri1.v_pv_2)
            .bind(ri1.v_pv_3)
            .bind(ri1.v_bat)
            .bind(ri1.soc as i32)
            .bind(ri1.soh as i32)
            .bind(ri1.p_pv as i32)
            .bind(ri1.p_pv_1 as i32)
            .bind(ri1.p_pv_2 as i32)
            .bind(ri1.p_pv_3 as i32)
            .bind(ri1.p_charge as i32)
            .bind(ri1.p_discharge as i32)
            .bind(ri1.v_ac_r)
            .bind(ri1.v_ac_s)
            .bind(ri1.v_ac_t)
            .bind(ri1.f_ac)
            .bind(ri1.p_inv as i32)
            .bind(ri1.p_rec as i32)
            .bind(ri1.pf)
            .bind(ri1.v_eps_r)
            .bind(ri1.v_eps_s)
            .bind(ri1.v_eps_t)
            .bind(ri1.f_eps)
            .bind(ri1.p_to_grid as i32)
            .bind(ri1.p_to_user as i32)
            .bind(ri1.e_pv_day)
            .bind(ri1.e_pv_day_1)
            .bind(ri1.e_pv_day_2)
            .bind(ri1.e_pv_day_3)
            .bind(ri1.e_inv_day)
            .bind(ri1.e_rec_day)
            .bind(ri1.e_chg_day)
            .bind(ri1.e_dischg_day)
            .bind(ri1.e_eps_day)
            .bind(ri1.e_to_grid_day)
            .bind(ri1.e_to_user_day)
            .bind(ri1.v_bus_1)
            .bind(ri1.v_bus_2)
            .bind(ri2.e_pv_all)
            .bind(ri2.e_pv_all_1)
            .bind(ri2.e_pv_all_2)
            .bind(ri2.e_pv_all_3)
            .bind(ri2.e_inv_all)
            .bind(ri2.e_rec_all)
            .bind(ri2.e_chg_all)
            .bind(ri2.e_dischg_all)
            .bind(ri2.e_eps_all)
            .bind(ri2.e_to_grid_all)
            .bind(ri2.e_to_user_all)
            .bind(ri2.t_inner as i32)
            .bind(ri2.t_rad_1 as i32)
            .bind(ri2.t_rad_2 as i32)
            .bind(ri2.t_bat as i32)
            .bind(ri2.runtime as i32)
            .bind(ri3.max_chg_curr)
            .bind(ri3.max_dischg_curr)
            .bind(ri3.charge_volt_ref)
            .bind(ri3.dischg_cut_volt)
            .bind(ri3.bat_status_0 as i32)
            .bind(ri3.bat_status_1 as i32)
            .bind(ri3.bat_status_2 as i32)
            .bind(ri3.bat_status_3 as i32)
            .bind(ri3.bat_status_4 as i32)
            .bind(ri3.bat_status_5 as i32)
            .bind(ri3.bat_status_6 as i32)
            .bind(ri3.bat_status_7 as i32)
            .bind(ri3.bat_status_8 as i32)
            .bind(ri3.bat_status_9 as i32)
            .bind(ri3.bat_status_inv as i32)
            .bind(ri3.bat_count as i32)
            .bind(ri3.bat_capacity as i32)
            .bind(ri1.datalog.to_string())
            .bind(ri1.time.0)
            .persistent(true)
            .fetch_one(&mut conn)
            .await?;
        }
    }
}
