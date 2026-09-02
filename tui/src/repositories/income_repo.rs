use std::rc::Rc;
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use crate::db::PgConnector;
use crate::models::{Income, NewIncome};
use crate::schema::incomes::dsl::*;

pub struct IncomeRepo {
    pg_connector: Rc<PgConnector>,
}

impl IncomeRepo {
    pub fn new(pg_connector: Rc<PgConnector>) -> Self {
        Self { pg_connector }
    }

    pub fn pg_connector(&self) -> Rc<PgConnector> {
        Rc::clone(&self.pg_connector)
    }

    pub fn find_all(&self) -> Vec<Income> {
        let mut conn = self.pg_connector.get_connection();
        incomes
            .load::<Income>(&mut *conn)
            .expect("Error loading incomes")
    }

    pub fn find_by_id(&self, income_id: i32) -> Option<Income> {
        let mut conn = self.pg_connector.get_connection();
        incomes
            .filter(id.eq(income_id))
            .first::<Income>(&mut *conn)
            .ok()
    }

    pub fn create(&self, income_date: NaiveDate, income_amount: BigDecimal, income_notes: Option<String>) -> Income {
        let mut conn = self.pg_connector.get_connection();
        let new_income = NewIncome {
            date: income_date,
            amount: income_amount,
            notes: income_notes,
        };

        diesel::insert_into(incomes)
            .values(&new_income)
            .returning(Income::as_returning())
            .get_result(&mut *conn)
            .expect("Error saving new income")
    }

    pub fn update(&self, income_id: i32, income_date: NaiveDate, income_amount: BigDecimal, income_notes: Option<String>) -> Income {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(incomes.filter(id.eq(income_id)))
            .set((
                date.eq(income_date),
                amount.eq(income_amount),
                notes.eq(income_notes),
            ))
            .returning(Income::as_returning())
            .get_result(&mut *conn)
            .expect("Error updating income")
    }

    pub fn delete(&self, income_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::delete(incomes.filter(id.eq(income_id)))
            .execute(&mut *conn)
            .is_ok()
    }
}
