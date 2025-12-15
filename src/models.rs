use chrono::NaiveDate;
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtoStatus {
    Planned,
    Requested,
    Approved,
    Completed,
}

impl PtoStatus {
    pub fn all() -> Vec<PtoStatus> {
        vec![
            PtoStatus::Planned,
            PtoStatus::Requested,
            PtoStatus::Approved,
            PtoStatus::Completed,
        ]
    }
}

impl fmt::Display for PtoStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PtoStatus::Planned => write!(f, "Planned"),
            PtoStatus::Requested => write!(f, "Requested"),
            PtoStatus::Approved => write!(f, "Approved"),
            PtoStatus::Completed => write!(f, "Completed"),
        }
    }
}

impl From<String> for PtoStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Requested" => PtoStatus::Requested,
            "Approved" => PtoStatus::Approved,
            "Completed" => PtoStatus::Completed,
            _ => PtoStatus::Planned,
        }
    }
}

impl From<PtoStatus> for String {
    fn from(status: PtoStatus) -> Self {
        status.to_string()
    }
}


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
    pub due_day: Option<NaiveDate>,
    pub is_auto_pay: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::bills)]
pub struct NewBill {
    pub name: String,
    pub amount: BigDecimal,
    pub due_day: Option<NaiveDate>,
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
    pub name: Option<String>,
    pub total: Option<BigDecimal>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ledgers)]
pub struct NewLedger {
    pub date: NaiveDate,
    pub name: String,
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
    pub due_day: Option<NaiveDate>,
    pub is_payed: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ledger_bills)]
pub struct NewLedgerBill {
    pub ledger_id: i32,
    pub bill_id: i32,
    pub amount: BigDecimal,
    pub due_day: Option<NaiveDate>,
    pub is_payed: bool,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::ptos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Pto {
    pub id: i32,
    pub year: i32,
    pub prev_year_hours: BigDecimal,
    pub available_hours: BigDecimal,
    pub hours_planned: BigDecimal,
    pub hours_used: BigDecimal,
    pub hours_remaining: BigDecimal,
    pub rollover_hours: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ptos)]
pub struct NewPto {
    pub year: i32,
    pub available_hours: BigDecimal,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::pto_plan)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PtoPlan {
    pub id: i32,
    pub pto_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub name: String,
    pub description: Option<String>,
    pub hours: BigDecimal,
    pub status: String,
    pub custom_hours: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::pto_plan)]
pub struct NewPtoPlan {
    pub pto_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub name: String,
    pub description: Option<String>,
    pub hours: BigDecimal,
    pub status: String,
    pub custom_hours: bool,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::holiday_hours)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HolidayHours {
    pub id: i32,
    pub pto_id: i32,
    pub date: NaiveDate,
    pub name: String,
    pub hours: BigDecimal,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::holiday_hours)]
pub struct NewHolidayHours {
    pub pto_id: i32,
    pub date: NaiveDate,
    pub name: String,
    pub hours: BigDecimal,
}
