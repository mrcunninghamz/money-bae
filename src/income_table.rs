use std::cmp::Ordering;
use std::str::FromStr;
use bigdecimal::BigDecimal;
use chrono::{Local, NaiveDate};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, HideableView, LinearLayout, ListView, Panel};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;

use crate::models;
use crate::schema::incomes::dsl::*;
use crate::db::establish_connection;
use crate::ui_helpers::toggle_buttons_visible;

// Button name constants
const INCOME_EDIT_BUTTON: &str = "income_table_edit_button";
const INCOME_DELETE_BUTTON: &str = "income_table_delete_button";
const TOGGLE_BUTTONS: &[&str] = &[INCOME_EDIT_BUTTON, INCOME_DELETE_BUTTON];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Date,
    Amount,
}

#[derive(Clone, Debug)]
struct IncomeDisplay {
    id: i32,
    date: NaiveDate,
    amount: BigDecimal,
}

impl From<models::Income> for IncomeDisplay {
    fn from(income: models::Income) -> Self {
        IncomeDisplay {
            id: income.id,
            date: income.date,
            amount: income.amount,
        }
    }
}

impl TableViewItem<BasicColumn> for IncomeDisplay {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Date => self.date.format("%d/%m/%Y").to_string(),
            BasicColumn::Amount => self.amount.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized
    {
        match column {
            BasicColumn::Date => self.date.cmp(&other.date),
            BasicColumn::Amount => self.amount.cmp(&other.amount),
        }
    }
}

pub struct IncomeTableView {
    table: TableView<IncomeDisplay,BasicColumn>,
}

impl IncomeTableView {
    pub fn new() -> Self {
        let mut conn = establish_connection();
        let results = incomes
            .load::<models::Income>(&mut conn)
            .expect("Error loading incomes");

        let income_displays: Vec<IncomeDisplay> = results
            .into_iter()
            .map(|i| i.into())
            .collect();

        Self {
            table: TableView::<IncomeDisplay,BasicColumn>::new()
                .column(BasicColumn::Date, "Date", |c| c.width_percent(40))
                .column(BasicColumn::Amount, "Amount", |c| c.width_percent(60))
                .items(income_displays)
        }
    }

    pub fn add_table(self, siv: &mut Cursive) {
        siv.pop_layer();

        let buttons = LinearLayout::horizontal()
            .child(Button::new("Add", |s| income_form(s, None)))
            .child(HideableView::new(Button::new("Edit", |s| {
                let selected = s.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
                    v.borrow_item(v.item().unwrap()).cloned()
                }).flatten();

                if let Some(income) = selected {
                    income_form(s, Some(income));
                }
            })).with_name(INCOME_EDIT_BUTTON))
            .child(HideableView::new(Button::new("Delete", |s| delete_income(s))).with_name(INCOME_DELETE_BUTTON));

        let income_count = self.table.len();
        let content = LinearLayout::vertical()
            .child(Panel::new(
                self.table
                    .with_name("income_table")
                    .min_size((50, 20))
            ).full_screen())
            .child(buttons);

        let screen = crate::common_layout::create_screen(
            "Income Table",
            content,
            &crate::common_layout::view_footer()
        );

        siv.add_layer(screen);

        toggle_buttons_visible(siv, income_count, TOGGLE_BUTTONS);
    }
}

fn income_form(siv: &mut Cursive, existing: Option<IncomeDisplay>) {
    let is_edit = existing.is_some();
    let title = if is_edit { "Edit Income" } else { "Add Income" };
    let button_label = if is_edit { "Update" } else { "Ok" };

    // Pre-fill values
    let date_value = existing
        .as_ref()
        .map(|i| i.date.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|| Local::now().format("%d/%m/%Y").to_string());

    let amount_value = existing
        .as_ref()
        .map(|i| i.amount.to_string())
        .unwrap_or_default();

    let income_id = existing.map(|i| i.id);

    siv.add_layer(
        Dialog::new()
            .title(title)
            .button(button_label, move |s| {
                let date_str = s.call_on_name("date_input", |v: &mut EditView| {
                    v.get_content()
                }).unwrap();

                let amount_str = s.call_on_name("amount_input", |v: &mut EditView| {
                    v.get_content()
                }).unwrap();

                // Validate date format DD/MM/YYYY
                let parsed_date = NaiveDate::parse_from_str(&date_str, "%d/%m/%Y");
                let amount_bd = BigDecimal::from_str(&amount_str);

                if parsed_date.is_err() {
                    s.add_layer(Dialog::info("Invalid date format. Use DD/MM/YYYY"));
                    return;
                }

                if amount_bd.is_err() {
                    s.add_layer(Dialog::info("Invalid amount format"));
                    return;
                }

                let mut conn = establish_connection();

                if let Some(record_id) = income_id {
                    // Update existing
                    diesel::update(incomes.find(record_id))
                        .set((
                            date.eq(parsed_date.unwrap()),
                            amount.eq(amount_bd.unwrap()),
                        ))
                        .execute(&mut conn)
                        .expect("Error updating income");
                } else {
                    // Insert new
                    let new_income = models::NewIncome {
                        date: parsed_date.unwrap(),
                        amount: amount_bd.unwrap(),
                    };

                    diesel::insert_into(incomes)
                        .values(&new_income)
                        .execute(&mut conn)
                        .expect("Error saving income");
                }

                // Reload table
                let results = incomes
                    .load::<models::Income>(&mut conn)
                    .expect("Error loading incomes");

                let income_displays: Vec<IncomeDisplay> = results
                    .into_iter()
                    .map(|i| i.into())
                    .collect();
                let income_count = income_displays.len();

                s.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
                    v.set_items(income_displays);
                });
                s.pop_layer();

                toggle_buttons_visible(s, income_count, TOGGLE_BUTTONS);
            })
            .button("Cancel", |s| { s.pop_layer(); })
            .content(
                ListView::new()
                    .child("Date (DD/MM/YYYY)", EditView::new().content(date_value).with_name("date_input").fixed_width(20))
                    .child("Amount", EditView::new().content(amount_value).with_name("amount_input").fixed_width(20))
            )
    );
}

fn delete_income(siv: &mut Cursive) {
    let selected = siv.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(income) = selected {
        siv.add_layer(
            Dialog::text("Delete this income?")
                .button("Yes", move |s| {
                    let mut conn = establish_connection();

                    diesel::delete(incomes.find(income.id))
                        .execute(&mut conn)
                        .expect("Error deleting income");

                    // Reload table
                    let results = incomes
                        .load::<models::Income>(&mut conn)
                        .expect("Error loading incomes");

                    let income_displays: Vec<IncomeDisplay> = results
                        .into_iter()
                        .map(|i| i.into())
                        .collect();
                    let income_count = income_displays.len();

                    s.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
                        v.set_items(income_displays);
                    });

                    toggle_buttons_visible(s, income_count, TOGGLE_BUTTONS);

                    s.pop_layer();
                })
                .button("No", |s| { s.pop_layer(); })
        );
    }
}
