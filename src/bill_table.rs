use std::cmp::Ordering;
use std::str::FromStr;
use bigdecimal::BigDecimal;
use chrono::{Datelike, Local, NaiveDate};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Checkbox, Dialog, EditView, HideableView, LinearLayout, ListView, Panel};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;
use crate::db::establish_connection;
use crate::models;
use crate::schema::bills::dsl::*;
use crate::ui_helpers::toggle_buttons_visible;

// Button name constants
const BILL_EDIT_BUTTON: &str = "bill_table_edit_button";
const BILL_DELETE_BUTTON: &str = "bill_table_delete_button";
const TOGGLE_BUTTONS: &[&str] = &[BILL_EDIT_BUTTON, BILL_DELETE_BUTTON];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Name,
    Amount,
    DueDay,
    IsAutoPay
}

#[derive(Clone, Debug)]
struct BillDisplay {
    id: i32,
    name: String,
    amount: BigDecimal,
    due_day: u32,
    is_auto_pay: bool
}

impl From<models::Bill> for BillDisplay {
    fn from(bill: models::Bill) -> Self {
        BillDisplay {
            id: bill.id,
            name: bill.name,
            amount: bill.amount,
            due_day: bill.due_day.day(),
            is_auto_pay: bill.is_auto_pay,
        }
    }
}

impl TableViewItem<BasicColumn> for BillDisplay {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Amount => self.amount.to_string(),
            BasicColumn::DueDay => self.due_day.to_string(),
            BasicColumn::IsAutoPay => self.is_auto_pay.to_string()
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized
    {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Amount => self.amount.cmp(&other.amount),
            BasicColumn::DueDay => self.due_day.cmp(&other.due_day),
            BasicColumn::IsAutoPay => Ordering::Equal,
        }
    }
}

pub struct BillTableView {
    table: TableView<BillDisplay,BasicColumn>,
}

impl BillTableView {
    pub fn new() -> Self {
        let mut conn = establish_connection();
        let results = bills
            .load::<models::Bill>(&mut conn)
            .expect("Error loading bills");

        let bill_displays: Vec<BillDisplay> = results
            .into_iter()
            .map(|b| b.into())
            .collect();

        Self {
            table: TableView::<BillDisplay,BasicColumn>::new()
                .column(BasicColumn::Name, "Name", |c| c.width_percent(30))
                .column(BasicColumn::Amount, "Amount", |c| c.width_percent(25))
                .column(BasicColumn::DueDay, "Due Day", |c| c.width_percent(25))
                .column(BasicColumn::IsAutoPay, "Auto Pay", |c| c.width_percent(20))
                .items(bill_displays)
        }
    }

    pub fn add_table(self, siv: &mut Cursive) {
        siv.pop_layer();

        let buttons = LinearLayout::horizontal()
            .child(Button::new("Add", |s| bill_form(s, None)))
            .child(HideableView::new(Button::new("Edit", |s| {
                let selected = s.call_on_name("bill_table", |v: &mut TableView<BillDisplay, BasicColumn>| {
                    v.borrow_item(v.item().unwrap()).cloned()
                }).flatten();

                if let Some(bill) = selected {
                    bill_form(s, Some(bill));
                }
            })).with_name(BILL_EDIT_BUTTON))
            .child(HideableView::new(Button::new("Delete", |s| delete_bill(s))).with_name(BILL_DELETE_BUTTON));
        let bill_count = self.table.len();
        let content = LinearLayout::vertical()
            .child(Panel::new(
                self.table
                    .with_name("bill_table")
                    .min_size((60, 20))
            ).full_screen())
            .child(buttons);

        let screen = crate::common_layout::create_screen(
            "Bill Table",
            content,
            &crate::common_layout::view_footer()
        );

        siv.add_layer(screen);

        toggle_buttons_visible(siv, bill_count, TOGGLE_BUTTONS);
    }
}

fn bill_form(siv: &mut Cursive, existing: Option<BillDisplay>) {
    let is_edit = existing.is_some();
    let title = if is_edit { "Edit Bill" } else { "Add Bill" };
    let button_label = if is_edit { "Update" } else { "Ok" };

    // Pre-fill values
    let name_value = existing
        .as_ref()
        .map(|b| b.name.clone())
        .unwrap_or_default();

    let amount_value = existing
        .as_ref()
        .map(|b| b.amount.to_string())
        .unwrap_or_default();

    let due_day_value = existing
        .as_ref()
        .map(|b| b.due_day.to_string())
        .unwrap_or_else(|| Local::now().day().to_string());

    let auto_pay_value = existing
        .as_ref()
        .map(|b| b.is_auto_pay)
        .unwrap_or(false);

    let bill_id = existing.map(|b| b.id);

    siv.add_layer(
        Dialog::new()
            .title(title)
            .button(button_label, move |s| {
                let name_str = s.call_on_name("name_input", |v: &mut EditView| {
                    v.get_content()
                }).unwrap();

                let amount_str = s.call_on_name("amount_input", |v: &mut EditView| {
                    v.get_content()
                }).unwrap();

                let due_day_str = s.call_on_name("due_day_input", |v: &mut EditView| {
                    v.get_content()
                }).unwrap();

                let is_auto = s.call_on_name("auto_pay_checkbox", |v: &mut Checkbox| {
                    v.is_checked()
                }).unwrap();

                // Validate name
                if name_str.trim().is_empty() {
                    s.add_layer(Dialog::info("Name cannot be empty"));
                    return;
                }

                // Validate amount
                let amount_bd = BigDecimal::from_str(&amount_str);
                if amount_bd.is_err() {
                    s.add_layer(Dialog::info("Invalid amount format"));
                    return;
                }

                // Validate due day (1-31)
                let day_num: Result<u32, _> = due_day_str.parse();
                if day_num.is_err() || day_num.as_ref().unwrap() < &1 || day_num.as_ref().unwrap() > &31 {
                    s.add_layer(Dialog::info("Due day must be between 1 and 31"));
                    return;
                }

                // Create NaiveDate from day number (use current year/month with the specified day)
                let now = Local::now();
                let due_date = NaiveDate::from_ymd_opt(now.year(), now.month(), day_num.unwrap());
                if due_date.is_none() {
                    s.add_layer(Dialog::info("Invalid day for current month"));
                    return;
                }

                let mut conn = establish_connection();

                if let Some(record_id) = bill_id {
                    // Update existing
                    diesel::update(bills.find(record_id))
                        .set((
                            name.eq(name_str.to_string()),
                            amount.eq(amount_bd.unwrap()),
                            due_day.eq(due_date.unwrap()),
                            is_auto_pay.eq(is_auto),
                        ))
                        .execute(&mut conn)
                        .expect("Error updating bill");
                } else {
                    // Insert new
                    let new_bill = models::NewBill {
                        name: name_str.to_string(),
                        amount: amount_bd.unwrap(),
                        due_day: due_date.unwrap(),
                        is_auto_pay: is_auto,
                    };

                    diesel::insert_into(bills)
                        .values(&new_bill)
                        .execute(&mut conn)
                        .expect("Error saving bill");
                }

                // Reload table
                let results = bills
                    .load::<models::Bill>(&mut conn)
                    .expect("Error loading bills");

                let bill_displays: Vec<BillDisplay> = results
                    .into_iter()
                    .map(|b| b.into())
                    .collect();
                let bill_count = bill_displays.len();

                s.call_on_name("bill_table", |v: &mut TableView<BillDisplay, BasicColumn>| {
                    v.set_items(bill_displays);
                });
                s.pop_layer();

                toggle_buttons_visible(s, bill_count, TOGGLE_BUTTONS);
            })
            .button("Cancel", |s| { s.pop_layer(); })
            .content(
                ListView::new()
                    .child("Name", EditView::new().content(name_value).with_name("name_input").fixed_width(20))
                    .child("Amount", EditView::new().content(amount_value).with_name("amount_input").fixed_width(20))
                    .child("Due Day (1-31)", EditView::new().content(due_day_value).with_name("due_day_input").fixed_width(20))
                    .child("Auto Pay", {
                        let mut cb = Checkbox::new();
                        if auto_pay_value {
                            cb.check();
                        }
                        cb.with_name("auto_pay_checkbox")
                    })
            )
    );
}

fn delete_bill(siv: &mut Cursive) {
    let selected = siv.call_on_name("bill_table", |v: &mut TableView<BillDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(bill) = selected {
        siv.add_layer(
            Dialog::text(format!("Delete bill '{}'?", bill.name))
                .button("Yes", move |s| {
                    let mut conn = establish_connection();

                    diesel::delete(bills.find(bill.id))
                        .execute(&mut conn)
                        .expect("Error deleting bill");

                    // Reload table
                    let results = bills
                        .load::<models::Bill>(&mut conn)
                        .expect("Error loading bills");

                    let bill_displays: Vec<BillDisplay> = results
                        .into_iter()
                        .map(|b| b.into())
                        .collect();
                    let item_count = bill_displays.iter().count();

                    s.call_on_name("bill_table", |v: &mut TableView<BillDisplay, BasicColumn>| {
                        v.set_items(bill_displays);
                    });

                    toggle_buttons_visible(s, item_count, TOGGLE_BUTTONS);

                    s.pop_layer();
                })
                .button("No", |s| { s.pop_layer(); })
        );
    }
}