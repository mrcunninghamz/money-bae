use std::rc::Rc;
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use crate::db::PgConnector;
use crate::models::{Bill, NewBill};
use crate::schema::bills::dsl::*;

pub struct BillRepo {
    pg_connector: Rc<PgConnector>,
}

impl BillRepo {
    pub fn new(pg_connector: Rc<PgConnector>) -> Self {
        Self { pg_connector }
    }

    pub fn find_all(&self) -> Vec<Bill> {
        let mut conn = self.pg_connector.get_connection();
        bills
            .load::<Bill>(&mut *conn)
            .expect("Error loading bills")
    }

    pub fn find_by_id(&self, bill_id: i32) -> Option<Bill> {
        let mut conn = self.pg_connector.get_connection();
        bills
            .filter(id.eq(bill_id))
            .first::<Bill>(&mut *conn)
            .ok()
    }

    pub fn create(&self, bill_name: String, bill_amount: BigDecimal, bill_due_day: Option<NaiveDate>, bill_is_auto_pay: bool, bill_notes: Option<String>) -> Bill {
        let mut conn = self.pg_connector.get_connection();
        let new_bill = NewBill {
            name: bill_name,
            amount: bill_amount,
            due_day: bill_due_day,
            is_auto_pay: bill_is_auto_pay,
            notes: bill_notes,
        };

        diesel::insert_into(bills)
            .values(&new_bill)
            .returning(Bill::as_returning())
            .get_result(&mut *conn)
            .expect("Error saving new bill")
    }

    pub fn update(&self, bill_id: i32, bill_name: String, bill_amount: BigDecimal, bill_due_day: Option<NaiveDate>, bill_is_auto_pay: bool, bill_notes: Option<String>) -> Bill {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(bills.filter(id.eq(bill_id)))
            .set((
                name.eq(bill_name),
                amount.eq(bill_amount),
                due_day.eq(bill_due_day),
                is_auto_pay.eq(bill_is_auto_pay),
                notes.eq(bill_notes),
            ))
            .returning(Bill::as_returning())
            .get_result(&mut *conn)
            .expect("Error updating bill")
    }

    pub fn delete(&self, bill_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::delete(bills.filter(id.eq(bill_id)))
            .execute(&mut *conn)
            .is_ok()
    }
}
