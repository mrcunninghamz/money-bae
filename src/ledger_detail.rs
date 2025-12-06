use std::cmp::Ordering;
use bigdecimal::BigDecimal;
use chrono::{Datelike, NaiveDate};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Checkbox, Dialog, EditView, HideableView, LinearLayout, ListView, Panel, SelectView, TextView};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;

use crate::models;
use crate::schema;
use crate::db::establish_connection;
use crate::ui_helpers::toggle_buttons_visible;

// Button name constants
const BILL_EDIT_BUTTON: &str = "ledger_bill_edit_button";
const BILL_TOGGLE_PAID_BUTTON: &str = "ledger_bill_toggle_paid_button";
const BILL_DELETE_BUTTON: &str = "ledger_bill_delete_button";
const BILL_TOGGLE_BUTTONS: &[&str] = &[BILL_EDIT_BUTTON, BILL_TOGGLE_PAID_BUTTON, BILL_DELETE_BUTTON];

const INCOME_DELETE_BUTTON: &str = "ledger_income_delete_button";
const INCOME_TOGGLE_BUTTONS: &[&str] = &[INCOME_DELETE_BUTTON];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BillColumn {
    Name,
    Amount,
    DueDay,
    Paid,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum IncomeColumn {
    Date,
    Amount,
}

#[derive(Clone, Debug)]
struct LedgerBillDisplay {
    id: i32,
    bill_id: i32,
    bill_name: String,
    amount: BigDecimal,
    due_day: String,
    is_payed: bool,
}

#[derive(Clone, Debug)]
struct IncomeDisplay {
    id: i32,
    date: String,
    amount: BigDecimal,
}

impl TableViewItem<BillColumn> for LedgerBillDisplay {
    fn to_column(&self, column: BillColumn) -> String {
        match column {
            BillColumn::Name => self.bill_name.clone(),
            BillColumn::Amount => format!("${}", self.amount),
            BillColumn::DueDay => self.due_day.clone(),
            BillColumn::Paid => if self.is_payed { "✓" } else { "" }.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: BillColumn) -> Ordering {
        match column {
            BillColumn::Name => self.bill_name.cmp(&other.bill_name),
            BillColumn::Amount => self.amount.cmp(&other.amount),
            BillColumn::DueDay => self.due_day.cmp(&other.due_day),
            BillColumn::Paid => self.is_payed.cmp(&other.is_payed),
        }
    }
}

impl TableViewItem<IncomeColumn> for IncomeDisplay {
    fn to_column(&self, column: IncomeColumn) -> String {
        match column {
            IncomeColumn::Date => self.date.clone(),
            IncomeColumn::Amount => format!("${}", self.amount),
        }
    }

    fn cmp(&self, other: &Self, column: IncomeColumn) -> Ordering {
        match column {
            IncomeColumn::Date => self.date.cmp(&other.date),
            IncomeColumn::Amount => self.amount.cmp(&other.amount),
        }
    }
}

pub fn show_ledger_detail(siv: &mut Cursive, target_ledger_id: i32) {
    siv.pop_layer();

    let mut conn = establish_connection();

    // Load ledger
    let ledger_result = schema::ledgers::table
        .find(target_ledger_id)
        .first::<models::Ledger>(&mut conn);

    if ledger_result.is_err() {
        siv.add_layer(Dialog::info("Error loading ledger"));
        return;
    }

    let ledger = ledger_result.unwrap();

    // Load ledger bills with bill names
    let ledger_bill_data: Vec<(models::LedgerBill, models::Bill)> = schema::ledger_bills::table
        .filter(schema::ledger_bills::ledger_id.eq(target_ledger_id))
        .inner_join(schema::bills::table)
        .load(&mut conn)
        .unwrap_or_default();

    let bill_displays: Vec<LedgerBillDisplay> = ledger_bill_data
        .into_iter()
        .map(|(lb, b)| LedgerBillDisplay {
            id: lb.id,
            bill_id: b.id,
            bill_name: b.name,
            amount: lb.amount.clone(),
            due_day: lb.due_day.map_or("-".to_string(), |d| d.format("%d/%m").to_string()),
            is_payed: lb.is_payed,
        })
        .collect();

    // Calculate bill statistics
    let _total_bills_amount: BigDecimal = bill_displays.iter()
        .map(|b| &b.amount)
        .sum();
    let paid_bills_amount: BigDecimal = bill_displays.iter()
        .filter(|b| b.is_payed)
        .map(|b| &b.amount)
        .sum();
    let unpaid_bills_amount: BigDecimal = bill_displays.iter()
        .filter(|b| !b.is_payed)
        .map(|b| &b.amount)
        .sum();
    let paid_bills_count = bill_displays.iter().filter(|b| b.is_payed).count();
    let unpaid_bills_count = bill_displays.iter().filter(|b| !b.is_payed).count();
    let total_bills = bill_displays.iter().count();

    // Load incomes for this ledger
    let ledger_incomes: Vec<models::Income> = schema::incomes::table
        .filter(schema::incomes::ledger_id.eq(target_ledger_id))
        .load(&mut conn)
        .unwrap_or_default();

    let income_displays: Vec<IncomeDisplay> = ledger_incomes
        .iter()
        .map(|i| IncomeDisplay {
            id: i.id,
            date: i.date.format("%d/%m/%Y").to_string(),
            amount: i.amount.clone(),
        })
        .collect();

    let income_count = income_displays.len();

    // Create bills table
    let bills_table = TableView::<LedgerBillDisplay, BillColumn>::new()
        .column(BillColumn::Name, "Bill", |c| c.width_percent(35))
        .column(BillColumn::Amount, "Amount", |c| c.width_percent(25))
        .column(BillColumn::DueDay, "Due", |c| c.width_percent(25))
        .column(BillColumn::Paid, "✓", |c| c.width_percent(15))
        .items(bill_displays)
        .with_name("bills_table")
        .min_size((40, 15));

    // Create income table
    let income_table = TableView::<IncomeDisplay, IncomeColumn>::new()
        .column(IncomeColumn::Date, "Date", |c| c.width_percent(50))
        .column(IncomeColumn::Amount, "Amount", |c| c.width_percent(50))
        .items(income_displays)
        .with_name("income_table")
        .min_height(5)
        .scrollable();

    // Create summary text with two-column bill breakdown
    let summary_text = format!(
        "Bank Balance: ${}\n
         Income: ${} ({} items)\n\n\
         Available Funds: ${}\n\n\
         BILLS         PLANNED     PAID\n\
         ─────────────────────────────\n\
         Amount        ${}    ${}\n\
         Count         {}          {}\n\n\
         Total Expenses: ${}\n\n\
         ───────────────────────────────\n\
         Net: ${}\n\n",
        ledger.bank_balance,
        ledger.income,
        income_count,
        ledger.total.unwrap_or(BigDecimal::from(0)),
        unpaid_bills_amount,
        paid_bills_amount,
        unpaid_bills_count,
        paid_bills_count,
        ledger.expenses,
        ledger.net.unwrap_or(BigDecimal::from(0))
    );

    // Create income section with buttons
    let income_buttons = LinearLayout::horizontal()
        .child(Button::new("Add", move |s| add_income_to_ledger(s, target_ledger_id)))
        .child(HideableView::new(Button::new("Delete", move |s| delete_income_from_ledger(s, target_ledger_id))).with_name(INCOME_DELETE_BUTTON));

    let income_section = LinearLayout::vertical()
        .child(income_table)
        .child(income_buttons)
        .min_height(6);

    // Create bills section with buttons
    let bill_buttons = LinearLayout::horizontal()
        .child(Button::new("Add", move |s| add_bill_to_ledger(s, target_ledger_id)))
        .child(HideableView::new(Button::new("Edit", move |s| edit_ledger_bill(s, target_ledger_id))).with_name(BILL_EDIT_BUTTON))
        .child(HideableView::new(Button::new("Toggle Paid", move |s| toggle_bill_paid(s, target_ledger_id))).with_name(BILL_TOGGLE_PAID_BUTTON))
        .child(HideableView::new(Button::new("Delete", move |s| delete_bill_from_ledger(s, target_ledger_id))).with_name(BILL_DELETE_BUTTON));

    let bills_section = LinearLayout::vertical()
        .child(bills_table)
        .child(bill_buttons);

    // Create summary section with update button
    let summary_content = LinearLayout::vertical()
        .child(TextView::new(summary_text))
        .child(Button::new("Edit", move |s| update_ledger(s, target_ledger_id)));

    // Stack income and summary vertically in right column
    let right_column = LinearLayout::vertical()
        .child(Panel::new(income_section).title("Incomes"))
        .child(Panel::new(summary_content).title("Summary"));

    // Create two-column layout: Bills | (Incomes + Summary)
    let content = LinearLayout::horizontal()
        .child(Panel::new(bills_section).title("Bills").full_width())
        .child(right_column.full_width())
        .scrollable()
        .full_screen();

    let screen = crate::common_layout::create_screen(
        &format!("{} : {}", ledger.name.unwrap_or("Untitled".to_string()), ledger.date.format("%d/%m/%Y")),
        content,
        &crate::common_layout::view_footer()
    );

    siv.add_layer(screen);

    // Toggle button visibility based on item counts
    toggle_buttons_visible(siv, total_bills, BILL_TOGGLE_BUTTONS);
    toggle_buttons_visible(siv, income_count, INCOME_TOGGLE_BUTTONS);
}

fn add_income_to_ledger(siv: &mut Cursive, ledger_id: i32) {
    let mut conn = establish_connection();

    // Get ledger to find its month
    let ledger = schema::ledgers::table
        .find(ledger_id)
        .first::<models::Ledger>(&mut conn)
        .expect("Error loading ledger");

    let ledger_month = ledger.date.month();
    let ledger_year = ledger.date.year();

    // Get unassigned incomes from the same month
    let available_incomes: Vec<models::Income> = schema::incomes::table
        .filter(schema::incomes::ledger_id.is_null())
        .load(&mut conn)
        .expect("Error loading incomes");

    let month_incomes: Vec<models::Income> = available_incomes
        .into_iter()
        .filter(|i| i.date.month() == ledger_month && i.date.year() == ledger_year)
        .collect();

    if month_incomes.is_empty() {
        siv.add_layer(Dialog::info("No unassigned incomes for this month"));
        return;
    }

    let mut select = SelectView::new();
    for income in month_incomes {
        let label = format!("{} - ${}", income.date.format("%d/%m/%Y"), income.amount);
        select.add_item(label, income.id);
    }

    siv.add_layer(
        Dialog::around(select.with_name("income_select"))
            .title("Select Income to Add")
            .button("Add", move |s| {
                let income_id = s.call_on_name("income_select", |v: &mut SelectView<i32>| {
                    v.selection()
                }).unwrap();

                if let Some(selected_id) = income_id {
                    let mut conn = establish_connection();

                    // Assign income to ledger
                    diesel::update(schema::incomes::table.find(*selected_id))
                        .set(schema::incomes::ledger_id.eq(ledger_id))
                        .execute(&mut conn)
                        .expect("Error assigning income");

                    s.pop_layer(); // Close dialog
                    show_ledger_detail(s, ledger_id); // Refresh view
                }
            })
            .button("Cancel", |s| { s.pop_layer(); })
    );
}

fn delete_income_from_ledger(siv: &mut Cursive, ledger_id: i32) {
    let selected = siv.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, IncomeColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(income) = selected {
        siv.add_layer(
            Dialog::text("Remove this income from ledger?")
                .button("Yes", move |s| {
                    let mut conn = establish_connection();

                    // Unassign income from ledger (set ledger_id to null)
                    diesel::update(schema::incomes::table.find(income.id))
                        .set(schema::incomes::ledger_id.eq::<Option<i32>>(None))
                        .execute(&mut conn)
                        .expect("Error removing income");

                    s.pop_layer(); // Close dialog
                    show_ledger_detail(s, ledger_id); // Refresh view
                })
                .button("No", |s| { s.pop_layer(); })
        );
    }
}

fn add_bill_to_ledger(siv: &mut Cursive, ledger_id: i32) {
    let mut conn = establish_connection();

    // Get ledger to find its month
    let ledger = schema::ledgers::table
        .find(ledger_id)
        .first::<models::Ledger>(&mut conn)
        .expect("Error loading ledger");

    let ledger_month = ledger.date.month();
    let ledger_year = ledger.date.year();

    // Get all bills not already in this ledger
    let existing_bill_ids: Vec<i32> = schema::ledger_bills::table
        .filter(schema::ledger_bills::ledger_id.eq(ledger_id))
        .select(schema::ledger_bills::bill_id)
        .load(&mut conn)
        .unwrap_or_default();

    let mut available_bills: Vec<models::Bill> = schema::bills::table
        .load(&mut conn)
        .expect("Error loading bills");

    available_bills.retain(|b| !existing_bill_ids.contains(&b.id));

    if available_bills.is_empty() {
        siv.add_layer(Dialog::info("No bills available to add"));
        return;
    }

    let mut select = SelectView::new();
    for bill in available_bills {
        let due_day_str = bill.due_day.map_or("-".to_string(), |d| d.format("%-d").to_string());
        let label = format!("{} - ${} - {}", bill.name, bill.amount, due_day_str);
        select.add_item(label, (bill.id, bill.amount, bill.due_day, bill.is_auto_pay));
    }

    siv.add_layer(
        Dialog::around(select.with_name("bill_select"))
            .title("Select Bill to Add")
            .button("Add", move |s| {
                let bill_data = s.call_on_name("bill_select", |v: &mut SelectView<(i32, BigDecimal, Option<chrono::NaiveDate>, bool)>| {
                    v.selection()
                }).unwrap();

                if let Some(bill_data_rc) = bill_data {
                    let (bill_id, amount, due_day, autopay) = (*bill_data_rc).clone();
                    let mut conn = establish_connection();

                    // Create due_day for this ledger (use bill's day with ledger's month/year)
                    let ledger_due_day = due_day.and_then(|d| {
                        chrono::NaiveDate::from_ymd_opt(
                            ledger_year,
                            ledger_month,
                            d.day()
                        )
                    });

                    // Insert into ledger_bills
                    use crate::schema::ledger_bills;
                    diesel::insert_into(ledger_bills::table)
                        .values((
                            ledger_bills::ledger_id.eq(ledger_id),
                            ledger_bills::bill_id.eq(bill_id),
                            ledger_bills::amount.eq(amount),
                            ledger_bills::due_day.eq(ledger_due_day),
                            ledger_bills::is_payed.eq(autopay),
                        ))
                        .execute(&mut conn)
                        .expect("Error adding bill to ledger");

                    s.pop_layer(); // Close dialog
                    show_ledger_detail(s, ledger_id); // Refresh view
                }
            })
            .button("Cancel", |s| { s.pop_layer(); })
    );
}

fn toggle_bill_paid(siv: &mut Cursive, ledger_id: i32) {
    let selected = siv.call_on_name("bills_table", |v: &mut TableView<LedgerBillDisplay, BillColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(bill) = selected {
        let mut conn = establish_connection();
        let new_paid_status = !bill.is_payed;

        // Update is_payed status
        diesel::update(schema::ledger_bills::table.find(bill.id))
            .set(schema::ledger_bills::is_payed.eq(new_paid_status))
            .execute(&mut conn)
            .expect("Error updating bill paid status");

        // Refresh view
        show_ledger_detail(siv, ledger_id);
    }
}

fn delete_bill_from_ledger(siv: &mut Cursive, ledger_id: i32) {
    let selected = siv.call_on_name("bills_table", |v: &mut TableView<LedgerBillDisplay, BillColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(bill) = selected {
        siv.add_layer(
            Dialog::text(format!("Remove '{}' from ledger?", bill.bill_name))
                .button("Yes", move |s| {
                    let mut conn = establish_connection();

                    // Delete from ledger_bills
                    diesel::delete(schema::ledger_bills::table.find(bill.id))
                        .execute(&mut conn)
                        .expect("Error deleting bill from ledger");

                    s.pop_layer(); // Close dialog
                    show_ledger_detail(s, ledger_id); // Refresh view
                })
                .button("No", |s| { s.pop_layer(); })
        );
    }
}

fn edit_ledger_bill(siv: &mut Cursive, ledger_id: i32) {
    let selected = siv.call_on_name("bills_table", |v: &mut TableView<LedgerBillDisplay, BillColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(bill) = selected {
        let bill_id = bill.id;

        // Get the current bill data from database to get the actual due_day
        let mut conn = establish_connection();
        let ledger_bill = schema::ledger_bills::table
            .find(bill_id)
            .first::<models::LedgerBill>(&mut conn)
            .expect("Error loading ledger bill");

        let form = ListView::new()
            .child("Amount", EditView::new()
                .content(bill.amount.to_string())
                .with_name("edit_bill_amount")
                .fixed_width(20))
            .child("Due Day (DD/MM)", EditView::new()
                .content(ledger_bill.due_day.map_or("".to_string(), |d| d.format("%d/%m").to_string()))
                .with_name("edit_bill_due_day")
                .fixed_width(20))
            .child("Paid", Checkbox::new()
                .with_checked(bill.is_payed)
                .with_name("edit_bill_paid"));

        siv.add_layer(
            Dialog::around(form)
                .title(format!("Edit: {}", bill.bill_name))
                .button("Save", move |s| {
                    let amount_str = s.call_on_name("edit_bill_amount", |v: &mut EditView| {
                        v.get_content()
                    }).unwrap();

                    let due_day_str = s.call_on_name("edit_bill_due_day", |v: &mut EditView| {
                        v.get_content()
                    }).unwrap();

                    let is_paid = s.call_on_name("edit_bill_paid", |v: &mut Checkbox| {
                        v.is_checked()
                    }).unwrap();

                    // Parse amount
                    let amount = match amount_str.to_string().parse::<BigDecimal>() {
                        Ok(a) => a,
                        Err(_) => {
                            s.add_layer(Dialog::info("Invalid amount format"));
                            return;
                        }
                    };

                    // Parse due day (optional)
                    let due_day = if due_day_str.is_empty() {
                        None
                    } else {
                        match chrono::NaiveDate::parse_from_str(&format!("{}/2024", due_day_str.to_string()), "%d/%m/%Y") {
                            Ok(d) => Some(d),
                            Err(_) => {
                                s.add_layer(Dialog::info("Invalid date format (use DD/MM)"));
                                return;
                            }
                        }
                    };

                    let mut conn = establish_connection();

                    // Update ledger bill
                    diesel::update(schema::ledger_bills::table.find(bill_id))
                        .set((
                            schema::ledger_bills::amount.eq(amount),
                            schema::ledger_bills::due_day.eq(due_day),
                            schema::ledger_bills::is_payed.eq(is_paid),
                        ))
                        .execute(&mut conn)
                        .expect("Error updating ledger bill");

                    s.pop_layer(); // Close dialog
                    show_ledger_detail(s, ledger_id); // Refresh view
                })
                .button("Cancel", |s| { s.pop_layer(); })
        );
    }
}
fn update_ledger(siv: &mut Cursive, ledger_id: i32) {
    let mut conn = establish_connection();

    // Load ledger for editing
    let ledger = schema::ledgers::table
        .find(ledger_id)
        .first::<models::Ledger>(&mut conn)
        .expect("Error loading ledger");

    let current_balance = ledger.bank_balance.to_string();
    let name = ledger.name.unwrap_or_default();
    let date = ledger.date;

    siv.add_layer(
        Dialog::around(

            ListView::new()
                .child("Date", EditView::new()
                    .content(date.format("%d/%m/%Y").to_string())
                    .with_name("ledger_date_input")
                    .fixed_width(20))
                .child("Name", EditView::new()
                    .content(name)
                    .with_name("ledger_name_input")
                    .fixed_width(20))
                .child("Bank Balance", EditView::new()
                    .content(current_balance)
                    .with_name("bank_balance_input")
                    .fixed_width(20))
        )
        .title("Update Ledger")
        .button("Update", move |s| {
            let new_balance = s.call_on_name("bank_balance_input", |v: &mut EditView| {
                v.get_content()
            }).unwrap();

            let name = s.call_on_name("ledger_name_input", |v: &mut EditView| {
                v.get_content()
            }).unwrap().to_string();

            let date = s.call_on_name("ledger_date_input", |v: &mut EditView| {
                v.get_content()
            }).unwrap();

            let parsed_date = NaiveDate::parse_from_str(&date, "%d/%m/%Y");

            if parsed_date.is_err() {
                s.add_layer(Dialog::info("Invalid date format. Use DD/MM/YYYY"));
                return;
            }

            let balance = new_balance.to_string().parse::<BigDecimal>();

            if balance.is_err() {
                s.add_layer(Dialog::info("Invalid balance format"));
                return;
            }

            let mut conn = establish_connection();

            // Update ledger
            diesel::update(schema::ledgers::table.find(ledger_id))
                .set((
                    schema::ledgers::bank_balance.eq(balance.unwrap()),
                    schema::ledgers::date.eq(parsed_date.unwrap()),
                    schema::ledgers::name.eq(name),
                ))
                .execute(&mut conn)
                .expect("Error updating ledger");

            s.pop_layer(); // Close dialog
            show_ledger_detail(s, ledger_id); // Refresh view
        })
        .button("Cancel", |s| { s.pop_layer(); })
    );
}
