use std::rc::Rc;
use diesel::prelude::*;
use bigdecimal::BigDecimal;

use crate::db::PgConnector;
use crate::models::{Pto, NewPto};
use crate::schema::ptos;

pub struct PtoRepo {
    pg_connector: Rc<PgConnector>,
}

impl PtoRepo {
    pub fn new(pg_connector: Rc<PgConnector>) -> Self {
        Self { pg_connector }
    }

    pub fn pg_connector(&self) -> Rc<PgConnector> {
        Rc::clone(&self.pg_connector)
    }

    pub fn find_all(&self) -> Vec<Pto> {
        let mut conn = self.pg_connector.get_connection();
        ptos::table
            .order(ptos::year.desc())
            .load::<Pto>(&mut *conn)
            .expect("Error loading PTOs")
    }

    pub fn find_by_id(&self, pto_id: i32) -> Option<Pto> {
        let mut conn = self.pg_connector.get_connection();
        ptos::table
            .find(pto_id)
            .first::<Pto>(&mut *conn)
            .ok()
    }

    pub fn create(&self, pto_year: i32, pto_available_hours: BigDecimal) -> Pto {
        let mut conn = self.pg_connector.get_connection();
        let new_pto = NewPto {
            year: pto_year,
            available_hours: pto_available_hours,
        };

        diesel::insert_into(ptos::table)
            .values(&new_pto)
            .returning(Pto::as_returning())
            .get_result(&mut *conn)
            .expect("Error saving new PTO")
    }

    pub fn update(&self, pto_id: i32, pto_year: i32, pto_available_hours: BigDecimal) -> Pto {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(ptos::table.filter(ptos::id.eq(pto_id)))
            .set((
                ptos::year.eq(pto_year),
                ptos::available_hours.eq(pto_available_hours),
            ))
            .returning(Pto::as_returning())
            .get_result(&mut *conn)
            .expect("Error updating PTO")
    }

    pub fn delete(&self, pto_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::delete(ptos::table.filter(ptos::id.eq(pto_id)))
            .execute(&mut *conn)
            .is_ok()
    }
}
