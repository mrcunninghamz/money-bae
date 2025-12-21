use std::cmp::Ordering;
use std::rc::Rc;
use bigdecimal::BigDecimal;
use chrono::{Datelike, Local, NaiveDate, ParseResult};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, HideableView, LinearLayout, ListView, Panel, TextArea};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;

use crate::models;
use crate::repositories::ledger_repo::LedgerRepo;
use crate::db::PgConnector;
use crate::ui_helpers::toggle_buttons_visible;

// Button name constants
const LEDGER_VIEW_BUTTON: &str = "ledger_table_view_button";
const LEDGER_DUPLICATE_BUTTON: &str = "ledger_table_duplicate_button";
const LEDGER_DELETE_BUTTON: &str = "ledger_table_delete_button";
const TOGGLE_BUTTONS: &[&str] = &[LEDGER_VIEW_BUTTON, LEDGER_DUPLICATE_BUTTON, LEDGER_DELETE_BUTTON];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Date,
    Name,
    BankBalance,
    Income,
    Total,
    Expenses,
    Net,
}

#[derive(Clone, Debug)]
struct LedgerDisplay {
    id: i32,
    date: NaiveDate,
    name: String,
    bank_balance: BigDecimal,
    income: BigDecimal,
    total: BigDecimal,
    expenses: BigDecimal,
    net: BigDecimal,
    notes: Option<String>,
}

impl From<models::Ledger> for LedgerDisplay {
    fn from(ledger: models::Ledger) -> Self {
        LedgerDisplay {
            id: ledger.id,
            date: ledger.date,
            name: ledger.name.unwrap_or_default(),
            bank_balance: ledger.bank_balance,
            income: ledger.income,
            total: ledger.total.unwrap_or(BigDecimal::from(0)),
            expenses: ledger.expenses,
            net: ledger.net.unwrap_or(BigDecimal::from(0)),
            notes: ledger.notes,
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
            BasicColumn::Total => format!("${}", self.total),
            BasicColumn::Name => self.name.clone(),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized
    {
        match column {
            BasicColumn::Date => self.date.cmp(&other.date).reverse(),
            BasicColumn::BankBalance => self.bank_balance.cmp(&other.bank_balance),
            BasicColumn::Income => self.income.cmp(&other.income),
            BasicColumn::Expenses => self.expenses.cmp(&other.expenses),
            BasicColumn::Net => self.net.cmp(&other.net),
            BasicColumn::Total => self.total.cmp(&other.total),
            BasicColumn::Name => self.name.cmp(&other.name),
        }
    }
}

pub struct LedgerTableView {
    table: TableView<LedgerDisplay, BasicColumn>,
    ledger_repo: Rc<LedgerRepo>,
}

impl LedgerTableView {
    pub fn new(ledger_repo: Rc<LedgerRepo>) -> Self {
        let results = ledger_repo.find_all();

        let ledger_displays: Vec<LedgerDisplay> = results
            .into_iter()
            .map(|l| l.into())
            .collect();

        Self {
            table: TableView::<LedgerDisplay, BasicColumn>::new()
                .column(BasicColumn::Date, "Date", |c| c.width_percent(20))
                .column(BasicColumn::Name, "Name", |c| c.width_percent(20))
                .column(BasicColumn::Total, "Available Funds", |c| c.width_percent(20))
                .column(BasicColumn::Expenses, "Expenses", |c| c.width_percent(20))
                .column(BasicColumn::Net, "Net", |c| c.width_percent(20))
                .items(ledger_displays),
            ledger_repo,
        }
    }

    pub fn add_table(self, siv: &mut Cursive) {
        siv.pop_layer();

        let repo_add = Rc::clone(&self.ledger_repo);
        let repo_duplicate = Rc::clone(&self.ledger_repo);
        let repo_delete = Rc::clone(&self.ledger_repo);
        let repo_view = Rc::clone(&self.ledger_repo);

        let buttons = LinearLayout::horizontal()
            .child(Button::new("Add", move |s| add_ledger_dialog(s, None, &repo_add)))
            .child(HideableView::new(Button::new("View", move |s| view_ledger_detail(s, &repo_view))).with_name(LEDGER_VIEW_BUTTON))
            .child(HideableView::new(Button::new("Duplicate", move |s| {
                let selected = s.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
                    v.borrow_item(v.item().unwrap()).cloned()
                }).flatten();

                if let Some(ledger) = selected {
                    add_ledger_dialog(s, Some(ledger), &repo_duplicate);
                }
            })).with_name(LEDGER_DUPLICATE_BUTTON))
            .child(HideableView::new(Button::new("Delete", move |s| delete_ledger(s, &repo_delete))).with_name(LEDGER_DELETE_BUTTON));

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

fn add_ledger_dialog(siv: &mut Cursive, existing: Option<LedgerDisplay>, ledger_repo: &Rc<LedgerRepo>) {
    let is_duplicating = existing.is_some();

    let title = if is_duplicating { "Duplicate Ledger" } else { "Add Ledger" };

    let ledger_date = if is_duplicating {
        existing.as_ref().map(|l| l.date.clone()).unwrap_or_default().format("%d/%m/%Y").to_string()
    } else {
        Local::now().format("%d/%m/%Y").to_string()
    };
    
    let notes_value = existing
        .as_ref()
        .and_then(|l| l.notes.clone())
        .unwrap_or_default();

    let button_label = if is_duplicating { "Duplicate" } else { "Ok" };
    
    let repo = Rc::clone(ledger_repo);

    siv.add_layer(
        Dialog::new()
            .title(title)
            .button(button_label, move |s| {
                if is_duplicating {
                    duplicate_ledger(s, existing.clone(), &repo);
                }
                else {
                    add_ledger(s, &repo);
                }

            })
            .button("Cancel", |s| { s.pop_layer(); })
            .content(
                ListView::new()
                    .child("Date (DD/MM/YYYY)", EditView::new().content(ledger_date).with_name("date_input").fixed_width(20))
                    .child("Name", EditView::new().with_name("ledger_name").fixed_width(20))
                    .child("Notes", TextArea::new().content(notes_value).with_name("notes_input").min_size((40, 3)))
            )
    );
}

fn view_ledger_detail(siv: &mut Cursive, ledger_repo: &Rc<LedgerRepo>) {
    let selected = siv.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(ledger) = selected {
        // ledger_detail still uses PgConnector - will be migrated in p3.4
        crate::ledger_detail::show_ledger_detail(siv, ledger.id, ledger_repo);
    }
}

fn get_form_values(s: &mut Cursive) -> (ParseResult<NaiveDate>, String, String) {
    let date_str = s.call_on_name("date_input", |v: &mut EditView| {
        v.get_content()
    }).unwrap();

    let parsed_date = NaiveDate::parse_from_str(&date_str, "%d/%m/%Y");

    let ledger_name = s.call_on_name("ledger_name", |v: &mut EditView| {
        v.get_content()
    }).unwrap();
    
    let notes_str = s.call_on_name("notes_input", |v: &mut TextArea| {
        v.get_content().to_string()
    }).unwrap();

    (parsed_date, ledger_name.to_string(), notes_str.to_string())
}
fn add_ledger(s: &mut Cursive, ledger_repo: &Rc<LedgerRepo>) {
    let (parsed_date, ledger_name, notes_str) = get_form_values(s);

    if parsed_date.is_err() {
        s.add_layer(Dialog::info("Invalid date format. Use DD/MM/YYYY"));
        return;
    }

    ledger_repo.create(
        parsed_date.unwrap(),
        ledger_name.to_string(),
        BigDecimal::from(0),
        if notes_str.is_empty() { None } else { Some(notes_str) }
    );

    // Reload table
    let ledger_displays = ledger_repo.find_all()
        .into_iter()
        .map(|l| l.into())
        .collect::<Vec<LedgerDisplay>>();
    let ledger_count = ledger_displays.len();

    s.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
        v.set_items(ledger_displays);
    });
    s.pop_layer();

    toggle_buttons_visible(s, ledger_count, TOGGLE_BUTTONS);
}

fn duplicate_ledger(s: &mut Cursive, selected: Option<LedgerDisplay>, ledger_repo: &Rc<LedgerRepo>) {

    if let Some(ledger) = selected {
        let (parsed_date, ledger_name, notes_str) = get_form_values(s);

        if parsed_date.is_err() {
            s.add_layer(Dialog::info("Invalid date format. Use DD/MM/YYYY"));
            return;
        }

        if ledger_name.trim().is_empty() {
            s.add_layer(Dialog::info("Ledger name cannot be empty or whitespace."));
            return;
        }

        let new_ledger_record = ledger_repo.create(
            parsed_date.unwrap(),
            ledger_name,
            BigDecimal::from(0),
            if notes_str.is_empty() { None } else { Some(notes_str) }
        );

        // Duplicate ledger bills using raw queries
        let pg_connector = ledger_repo.pg_connector();
        let old_ledger_bills = {
            use crate::schema::ledger_bills;
            let mut conn = pg_connector.get_connection();
            ledger_bills::table
                .filter(ledger_bills::ledger_id.eq(ledger.id))
                .load::<models::LedgerBill>(&mut *conn)
                .expect("Error loading ledger bills")
        };

        {
            use crate::schema::ledger_bills;
            use crate::schema::bills;
            let mut conn = pg_connector.get_connection();

            for old_bill in old_ledger_bills {
                // Get the bill to check is_auto_pay
                let bill = bills::table
                    .find(old_bill.bill_id)
                    .first::<models::Bill>(&mut *conn)
                    .expect("Error loading bill");

                // Update due_day to new ledger's month if it exists
                let updated_due_day = old_bill.due_day.map(|old_date| {
                    new_ledger_record.date.with_day(old_date.day()).unwrap_or(new_ledger_record.date)
                });

                let new_ledger_bill = models::NewLedgerBill {
                    ledger_id: new_ledger_record.id,
                    bill_id: old_bill.bill_id,
                    amount: old_bill.amount,
                    due_day: updated_due_day,
                    is_payed: bill.is_auto_pay,
                    notes: None,
                };

                diesel::insert_into(ledger_bills::table)
                    .values(&new_ledger_bill)
                    .execute(&mut *conn)
                    .expect("Error duplicating ledger bill");
            }
        }

        // Reload table
        let ledger_displays = ledger_repo.find_all()
            .into_iter()
            .map(|l| l.into())
            .collect::<Vec<LedgerDisplay>>();
        let ledger_count = ledger_displays.len();

        s.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
            v.set_items(ledger_displays);
        });

        s.pop_layer();

        toggle_buttons_visible(s, ledger_count, TOGGLE_BUTTONS);
    }
}

fn delete_ledger(siv: &mut Cursive, ledger_repo: &Rc<LedgerRepo>) {
    let selected = siv.call_on_name("ledger_table", |v: &mut TableView<LedgerDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(ledger) = selected {
        let repo = Rc::clone(ledger_repo);
        siv.add_layer(
            Dialog::text("Delete this ledger?")
                .button("Yes", move |s| {
                    repo.delete(ledger.id);

                    // Reload table
                    let ledger_displays = repo.find_all()
                        .into_iter()
                        .map(|l| l.into())
                        .collect::<Vec<LedgerDisplay>>();
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
