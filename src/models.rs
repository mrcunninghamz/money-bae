use chrono::NaiveDate;
use diesel::prelude::*;
use bigdecimal::BigDecimal;

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::incomes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Income {
    pub id: i32,
    pub date: NaiveDate,
    pub amount: BigDecimal,
    pub created_at: chrono::NaiveDateTime,
    pub ledger_id: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::incomes)]
pub struct NewIncome {
    pub date: NaiveDate,
    pub amount: BigDecimal,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::bills)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Bill {
    pub id: i32,
    pub name: String,
    pub amount: BigDecimal,
    pub due_day: NaiveDate,
    pub is_auto_pay: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::bills)]
pub struct NewBill {
    pub name: String,
    pub amount: BigDecimal,
    pub due_day: NaiveDate,
    pub is_auto_pay: bool,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::ledgers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Ledger {
    pub id: i32,
    pub date: NaiveDate,
    pub bank_balance: BigDecimal,
    pub income: BigDecimal,
    pub expenses: BigDecimal,
    pub net: Option<BigDecimal>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ledgers)]
pub struct NewLedger {
    pub date: NaiveDate,
    pub bank_balance: BigDecimal,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::ledger_bills)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LedgerBill {
    pub id: i32,
    pub ledger_id: i32,
    pub bill_id: i32,
    pub amount: BigDecimal,
    pub due_day: NaiveDate,
    pub is_payed: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ledger_bills)]
pub struct NewLedgerBill {
    pub ledger_id: i32,
    pub bill_id: i32,
    pub amount: BigDecimal,
    pub due_day: NaiveDate,
    pub is_payed: bool,
}
