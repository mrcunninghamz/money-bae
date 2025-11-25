use std::cmp::Ordering;
use bigdecimal::BigDecimal;
use chrono::{Local, NaiveDate};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, HideableView, LinearLayout, ListView, Panel};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;

use crate::models;
use crate::schema::ledgers::dsl::*;
use crate::db::establish_connection;
use crate::ui_helpers::toggle_buttons_visible;

// Button name constants
const LEDGER_VIEW_BUTTON: &str = "ledger_table_view_button";
const LEDGER_DUPLICATE_BUTTON: &str = "ledger_table_duplicate_button";
const LEDGER_DELETE_BUTTON: &str = "ledger_table_delete_button";
const TOGGLE_BUTTONS: &[&str] = &[LEDGER_VIEW_BUTTON, LEDGER_DUPLICATE_BUTTON, LEDGER_DELETE_BUTTON];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Date,
    BankBalance,
    Income,
    Expenses,
    Net,
}

#[derive(Clone, Debug)]
struct LedgerDisplay {
    id: i32,
    date: NaiveDate,
    bank_balance: BigDecimal,
    income: BigDecimal,
    expenses: BigDecimal,
    net: BigDecimal,
}

impl From<models::Ledger> for LedgerDisplay {
    fn from(ledger: models::Ledger) -> Self {
        LedgerDisplay {
            id: ledger.id,
            date: ledger.date,
            bank_balance: ledger.bank_balance,
            income: ledger.income,
            expenses: ledger.expenses,
            net: ledger.net.unwrap_or(BigDecimal::from(0)),
        }
    }
}

impl TableViewItem<BasicColumn> for LedgerDisplay {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Date => self.date.format("%d/%m/%Y").to_string(),
            BasicColumn::BankBalance => format!("${}", self.bank_balance),
            BasicColumn::Income => format!("${}", self.income),
            BasicColumn::Expenses => format!("${}", self.expenses),
            BasicColumn::Net => format!("${}", self.net),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized
    {
        match column {
            BasicColumn::Date => self.date.cmp(&other.date),
            BasicColumn::BankBalance => self.bank_balance.cmp(&other.bank_balance),
            BasicColumn::Income => self.income.cmp(&other.income),
            BasicColumn::Expenses => self.expenses.cmp(&other.expenses),
            BasicColumn::Net => self.net.cmp(&other.net),
        }
    }
}

pub struct LedgerTableView {
    table: TableView<LedgerDisplay, BasicColumn>,
}

impl LedgerTableView {
    pub fn new() -> Self {
        let mut conn = establish_connection();
        let results = ledgers
            .load::<models::Ledger>(&mut conn)
            .expect("Error loading ledgers");

        let ledger_displays: Vec<LedgerDisplay> = results
            .into_iter()
            .map(|l| l.into())
            .collect();

        Self {
            table: TableView::<LedgerDisplay, BasicColumn>::new()
                .column(BasicColumn::Date, "Date", |c| c.width_percent(20))
                .column(BasicColumn::BankBalance, "Balance", |c| c.width_percent(20))
                .column(BasicColumn::Income, "Income", |c| c.width_percent(20))
                .column(BasicColumn::Expenses, "Expenses", |c| c.width_percent(20))
                .column(BasicColumn::Net, "Net", |c| c.width_percent(20))
                .items(ledger_displays)
        }
    }

    pub fn add_table(self, siv: &mut Cursive) {
        siv.pop_layer();

        let buttons = LinearLayout::horizontal()
            .child(Button::new("Add", |s| add_ledger_dialog(s)))
            .child(HideableView::new(Button::new("View", |s| view_ledger_detail(s))).with_name(LEDGER_VIEW_BUTTON))
            .child(HideableView::new(Button::new("Duplicate", |s| duplicate_ledger(s))).with_name(LEDGER_DUPLICATE_BUTTON))
            .child(HideableView::new(Button::new("Delete", |s| delete_ledger(s))).with_name(LEDGER_DELETE_BUTTON));

        let ledger_count = self.table.len();
        let content = LinearLayout::vertical()
            .child(Panel::new(
                self.table
                    .with_name("ledger_table")
                    .min_size((70, 20))
            ).full_screen())
            .child(buttons);

        let screen = crate::common_layout::create_screen(
            "Ledger Table",
            content,
            &crate::common_layout::view_footer()
        );

        siv.add_layer(screen);

        toggle_buttons_visible(siv, ledger_count, TOGGLE_BUTTONS);
    }
}

fn add_ledger_dialog(siv: &mut Cursive) {
    let today = Local::now().format("%d/%m/%Y").to_string();

    siv.add_layer(
        Dialog::new()
            .title("Add Ledger")
            .button("Ok", |s| {
                let date_str = s.call_on_name("date_input", |v: &mut EditView| {
                    v.get_content()
                }).unwrap();

                let parsed_date = NaiveDate::parse_from_str(&date_str, "%d/%m/%Y");

                if parsed_date.is_err() {
                    s.add_layer(Dialog::info("Invalid date format. Use DD/MM/YYYY"));
                    return;
                }

                let mut conn = establish_connection();
                let new_ledger = models::NewLedger {
                    date: parsed_date.unwrap(),
                    bank_balance: BigDecimal::from(0),
                };

                diesel::insert_into(ledgers)
                    .values(&new_ledger)
                    .execute(&mut conn)
                    .expect("Error saving ledger");

                // Reload table
                let results = ledgers
                    .load::<models::Ledger>(&mut conn)
                    .expect("Error loading ledgers");

                let ledger_displays: Vec<LedgerDisplay> = results
                    .into_iter()
                    .map(|l| l.into())
                    .collect();
                let ledger_count = ledger_displays.len();

                s.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
                    v.set_items(ledger_displays);
                });
                s.pop_layer();

                toggle_buttons_visible(s, ledger_count, TOGGLE_BUTTONS);
            })
            .button("Cancel", |s| { s.pop_layer(); })
            .content(
                ListView::new()
                    .child("Date (DD/MM/YYYY)", EditView::new().content(today).with_name("date_input").fixed_width(20))
            )
    );
}

fn view_ledger_detail(siv: &mut Cursive) {
    let selected = siv.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(ledger) = selected {
        crate::ledger_detail::show_ledger_detail(siv, ledger.id);
    }
}

fn duplicate_ledger(siv: &mut Cursive) {
    let selected = siv.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(ledger) = selected {
        let mut conn = establish_connection();

        // Create new ledger with today's date
        let new_ledger = models::NewLedger {
            date: Local::now().date_naive(),
            bank_balance: ledger.bank_balance,
        };

        let new_ledger_record: models::Ledger = diesel::insert_into(ledgers)
            .values(&new_ledger)
            .get_result(&mut conn)
            .expect("Error duplicating ledger");

        // Duplicate ledger bills
        use crate::schema::ledger_bills;
        use crate::schema::bills;

        let old_ledger_bills = ledger_bills::table
            .filter(ledger_bills::ledger_id.eq(ledger.id))
            .load::<models::LedgerBill>(&mut conn)
            .expect("Error loading ledger bills");

        for old_bill in old_ledger_bills {
            // Get the bill to check is_auto_pay
            let bill = bills::table
                .find(old_bill.bill_id)
                .first::<models::Bill>(&mut conn)
                .expect("Error loading bill");

            let new_ledger_bill = models::NewLedgerBill {
                ledger_id: new_ledger_record.id,
                bill_id: old_bill.bill_id,
                amount: old_bill.amount,
                due_day: old_bill.due_day,
                is_payed: bill.is_auto_pay,
            };

            diesel::insert_into(ledger_bills::table)
                .values(&new_ledger_bill)
                .execute(&mut conn)
                .expect("Error duplicating ledger bill");
        }

        // Reload table
        let results = ledgers
            .load::<models::Ledger>(&mut conn)
            .expect("Error loading ledgers");

        let ledger_displays: Vec<LedgerDisplay> = results
            .into_iter()
            .map(|l| l.into())
            .collect();

        let ledger_count = ledger_displays.len();

        siv.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
            v.set_items(ledger_displays);
        });

        toggle_buttons_visible(siv, ledger_count, TOGGLE_BUTTONS);
    }
}

fn delete_ledger(siv: &mut Cursive) {
    let selected = siv.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(ledger) = selected {
        siv.add_layer(
            Dialog::text("Delete this ledger?")
                .button("Yes", move |s| {
                    let mut conn = establish_connection();

                    diesel::delete(ledgers.find(ledger.id))
                        .execute(&mut conn)
                        .expect("Error deleting ledger");

                    // Reload table
                    let results = ledgers
                        .load::<models::Ledger>(&mut conn)
                        .expect("Error loading ledgers");

                    let ledger_displays: Vec<LedgerDisplay> = results
                        .into_iter()
                        .map(|l| l.into())
                        .collect();
                    let ledger_count = ledger_displays.len();

                    s.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
                        v.set_items(ledger_displays);
                    });

                    toggle_buttons_visible(s, ledger_count, TOGGLE_BUTTONS);

                    s.pop_layer();
                })
                .button("No", |s| { s.pop_layer(); })
        );
    }
}
