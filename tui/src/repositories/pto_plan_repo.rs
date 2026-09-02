use std::rc::Rc;
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use crate::db::PgConnector;
use crate::models::{PtoPlan, NewPtoPlan};
use crate::schema::pto_plan;

pub struct PtoPlanRepo {
    pg_connector: Rc<PgConnector>,
}

impl PtoPlanRepo {
    pub fn new(pg_connector: Rc<PgConnector>) -> Self {
        Self { pg_connector }
    }

    pub fn pg_connector(&self) -> Rc<PgConnector> {
        Rc::clone(&self.pg_connector)
    }

    pub fn find_by_pto_id(&self, pto_id: i32) -> Vec<PtoPlan> {
        let mut conn = self.pg_connector.get_connection();
        pto_plan::table
            .filter(pto_plan::pto_id.eq(pto_id))
            .order(pto_plan::start_date.asc())
            .load::<PtoPlan>(&mut *conn)
            .expect("Error loading PTO plans")
    }

    pub fn find_by_id(&self, plan_id: i32) -> Option<PtoPlan> {
        let mut conn = self.pg_connector.get_connection();
        pto_plan::table
            .find(plan_id)
            .first::<PtoPlan>(&mut *conn)
            .ok()
    }

    pub fn create(
        &self,
        plan_pto_id: i32,
        plan_start_date: NaiveDate,
        plan_end_date: NaiveDate,
        plan_name: String,
        plan_description: Option<String>,
        plan_hours: BigDecimal,
        plan_status: String,
        plan_custom_hours: bool,
    ) -> PtoPlan {
        let mut conn = self.pg_connector.get_connection();
        let new_plan = NewPtoPlan {
            pto_id: plan_pto_id,
            start_date: plan_start_date,
            end_date: plan_end_date,
            name: plan_name,
            description: plan_description,
            hours: plan_hours,
            status: plan_status,
            custom_hours: plan_custom_hours,
        };

        diesel::insert_into(pto_plan::table)
            .values(&new_plan)
            .returning(PtoPlan::as_returning())
            .get_result(&mut *conn)
            .expect("Error saving new PTO plan")
    }

    pub fn update(
        &self,
        plan_id: i32,
        plan_start_date: NaiveDate,
        plan_end_date: NaiveDate,
        plan_name: String,
        plan_description: Option<String>,
        plan_hours: BigDecimal,
        plan_status: String,
        plan_custom_hours: bool,
    ) -> PtoPlan {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(pto_plan::table.filter(pto_plan::id.eq(plan_id)))
            .set((
                pto_plan::start_date.eq(plan_start_date),
                pto_plan::end_date.eq(plan_end_date),
                pto_plan::name.eq(plan_name),
                pto_plan::description.eq(plan_description),
                pto_plan::hours.eq(plan_hours),
                pto_plan::status.eq(plan_status),
                pto_plan::custom_hours.eq(plan_custom_hours),
            ))
            .returning(PtoPlan::as_returning())
            .get_result(&mut *conn)
            .expect("Error updating PTO plan")
    }

    pub fn delete(&self, plan_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::delete(pto_plan::table.filter(pto_plan::id.eq(plan_id)))
            .execute(&mut *conn)
            .is_ok()
    }
}
