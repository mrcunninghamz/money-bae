// @generated automatically by Diesel CLI.

diesel::table! {
    bills (id) {
        id -> Int4,
        name -> Varchar,
        amount -> Numeric,
        due_day -> Nullable<Date>,
        is_auto_pay -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    holiday_hours (id) {
        id -> Int4,
        pto_id -> Int4,
        date -> Date,
        name -> Varchar,
        hours -> Numeric,
        created_at -> Timestamp,
    }
}

diesel::table! {
    incomes (id) {
        id -> Int4,
        date -> Date,
        amount -> Numeric,
        created_at -> Timestamp,
        ledger_id -> Nullable<Int4>,
    }
}

diesel::table! {
    ledger_bills (id) {
        id -> Int4,
        ledger_id -> Int4,
        bill_id -> Int4,
        amount -> Numeric,
        due_day -> Nullable<Date>,
        is_payed -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    ledgers (id) {
        id -> Int4,
        date -> Date,
        bank_balance -> Numeric,
        income -> Numeric,
        expenses -> Numeric,
        net -> Nullable<Numeric>,
        created_at -> Timestamp,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        total -> Nullable<Numeric>,
    }
}

diesel::table! {
    pto_plan (id) {
        id -> Int4,
        pto_id -> Int4,
        start_date -> Date,
        end_date -> Date,
        name -> Varchar,
        description -> Nullable<Text>,
        hours -> Numeric,
        status -> Varchar,
        custom_hours -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    ptos (id) {
        id -> Int4,
        year -> Int4,
        prev_year_hours -> Numeric,
        available_hours -> Numeric,
        hours_planned -> Numeric,
        hours_used -> Numeric,
        hours_remaining -> Numeric,
        rollover_hours -> Bool,
        created_at -> Timestamp,
    }
}

diesel::joinable!(holiday_hours -> ptos (pto_id));
diesel::joinable!(incomes -> ledgers (ledger_id));
diesel::joinable!(ledger_bills -> bills (bill_id));
diesel::joinable!(ledger_bills -> ledgers (ledger_id));
diesel::joinable!(pto_plan -> ptos (pto_id));

diesel::allow_tables_to_appear_in_same_query!(
    bills,
    holiday_hours,
    incomes,
    ledger_bills,
    ledgers,
    pto_plan,
    ptos,
);
