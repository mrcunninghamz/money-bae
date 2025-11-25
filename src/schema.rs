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
    }
}

diesel::joinable!(incomes -> ledgers (ledger_id));
diesel::joinable!(ledger_bills -> bills (bill_id));
diesel::joinable!(ledger_bills -> ledgers (ledger_id));

diesel::allow_tables_to_appear_in_same_query!(bills, incomes, ledger_bills, ledgers,);
