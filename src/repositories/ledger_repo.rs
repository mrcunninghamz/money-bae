use std::rc::Rc;
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;

use crate::db::PgConnector;
use crate::models::{Ledger, NewLedger, LedgerBill, NewLedgerBill, Bill, Income};
use crate::schema;

pub struct LedgerRepo {
    pg_connector: Rc<PgConnector>,
}

impl LedgerRepo {
    pub fn new(pg_connector: Rc<PgConnector>) -> Self {
        Self { pg_connector }
    }

    pub fn pg_connector(&self) -> Rc<PgConnector> {
        Rc::clone(&self.pg_connector)
    }

    pub fn find_all(&self) -> Vec<Ledger> {
        let mut conn = self.pg_connector.get_connection();
        schema::ledgers::table
            .load::<Ledger>(&mut *conn)
            .expect("Error loading ledgers")
    }

    pub fn find_by_id(&self, ledger_id: i32) -> Option<Ledger> {
        let mut conn = self.pg_connector.get_connection();
        schema::ledgers::table
            .find(ledger_id)
            .first::<Ledger>(&mut *conn)
            .ok()
    }

    pub fn create(&self, ledger_date: NaiveDate, ledger_name: String, ledger_bank_balance: BigDecimal, ledger_notes: Option<String>) -> Ledger {
        let mut conn = self.pg_connector.get_connection();
        let new_ledger = NewLedger {
            date: ledger_date,
            name: ledger_name,
            bank_balance: ledger_bank_balance,
            notes: ledger_notes,
        };

        diesel::insert_into(schema::ledgers::table)
            .values(&new_ledger)
            .returning(Ledger::as_returning())
            .get_result(&mut *conn)
            .expect("Error saving new ledger")
    }

    pub fn update(&self, ledger_id: i32, ledger_date: NaiveDate, ledger_name: String, ledger_bank_balance: BigDecimal, ledger_notes: Option<String>) -> Ledger {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(schema::ledgers::table.filter(schema::ledgers::id.eq(ledger_id)))
            .set((
                schema::ledgers::date.eq(ledger_date),
                schema::ledgers::name.eq(ledger_name),
                schema::ledgers::bank_balance.eq(ledger_bank_balance),
                schema::ledgers::notes.eq(ledger_notes),
            ))
            .returning(Ledger::as_returning())
            .get_result(&mut *conn)
            .expect("Error updating ledger")
    }

    pub fn delete(&self, ledger_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::delete(schema::ledgers::table.filter(schema::ledgers::id.eq(ledger_id)))
            .execute(&mut *conn)
            .is_ok()
    }

    // Ledger detail queries
    pub fn find_ledger_bills_with_bill_names(&self, ledger_id: i32) -> Vec<(LedgerBill, Bill)> {
        let mut conn = self.pg_connector.get_connection();
        schema::ledger_bills::table
            .filter(schema::ledger_bills::ledger_id.eq(ledger_id))
            .inner_join(schema::bills::table)
            .load(&mut *conn)
            .unwrap_or_default()
    }

    pub fn find_incomes_by_ledger(&self, ledger_id: i32) -> Vec<Income> {
        let mut conn = self.pg_connector.get_connection();
        schema::incomes::table
            .filter(schema::incomes::ledger_id.eq(ledger_id))
            .load(&mut *conn)
            .unwrap_or_default()
    }

    pub fn create_ledger_bill(&self, ledger_id: i32, bill_id: i32, bill_amount: BigDecimal, bill_due_day: Option<NaiveDate>, bill_is_payed: bool, bill_notes: Option<String>) -> LedgerBill {
        let mut conn = self.pg_connector.get_connection();
        let new_ledger_bill = NewLedgerBill {
            ledger_id,
            bill_id,
            amount: bill_amount,
            due_day: bill_due_day,
            is_payed: bill_is_payed,
            notes: bill_notes,
        };

        diesel::insert_into(schema::ledger_bills::table)
            .values(&new_ledger_bill)
            .returning(LedgerBill::as_returning())
            .get_result(&mut *conn)
            .expect("Error saving new ledger bill")
    }

    pub fn update_ledger_bill(&self, ledger_bill_id: i32, bill_amount: BigDecimal, bill_due_day: Option<NaiveDate>, bill_is_payed: bool, bill_notes: Option<String>) -> LedgerBill {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(schema::ledger_bills::table.filter(schema::ledger_bills::id.eq(ledger_bill_id)))
            .set((
                schema::ledger_bills::amount.eq(bill_amount),
                schema::ledger_bills::due_day.eq(bill_due_day),
                schema::ledger_bills::is_payed.eq(bill_is_payed),
                schema::ledger_bills::notes.eq(bill_notes),
            ))
            .returning(LedgerBill::as_returning())
            .get_result(&mut *conn)
            .expect("Error updating ledger bill")
    }

    pub fn delete_ledger_bill(&self, ledger_bill_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::delete(schema::ledger_bills::table.filter(schema::ledger_bills::id.eq(ledger_bill_id)))
            .execute(&mut *conn)
            .is_ok()
    }

    pub fn unassign_income_from_ledger(&self, income_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(schema::incomes::table.filter(schema::incomes::id.eq(income_id)))
            .set(schema::incomes::ledger_id.eq(None::<i32>))
            .execute(&mut *conn)
            .is_ok()
    }
}
