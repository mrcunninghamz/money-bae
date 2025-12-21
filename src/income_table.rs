use std::cmp::Ordering;
use std::rc::Rc;
use std::str::FromStr;
use bigdecimal::BigDecimal;
use chrono::{Local, NaiveDate};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, HideableView, LinearLayout, ListView, Panel, TextArea};
use cursive_table_view::{TableView, TableViewItem};

use crate::models;
use crate::repositories::IncomeRepo;
use crate::ui_helpers::toggle_buttons_visible;

// Button name constants
const INCOME_EDIT_BUTTON: &str = "income_table_edit_button";
const INCOME_DUPLICATE_BUTTON: &str = "income_table_duplicate_button";
const INCOME_DELETE_BUTTON: &str = "income_table_delete_button";
const TOGGLE_BUTTONS: &[&str] = &[INCOME_EDIT_BUTTON, INCOME_DUPLICATE_BUTTON, INCOME_DELETE_BUTTON];

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
    notes: Option<String>,
}

impl From<models::Income> for IncomeDisplay {
    fn from(income: models::Income) -> Self {
        IncomeDisplay {
            id: income.id,
            date: income.date,
            amount: income.amount,
            notes: income.notes,
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
    income_repo: Rc<IncomeRepo>,
}

impl IncomeTableView {
    pub fn new(income_repo: Rc<IncomeRepo>) -> Self {
        let results = income_repo.find_all();

        let income_displays: Vec<IncomeDisplay> = results
            .into_iter()
            .map(|i| i.into())
            .collect();

        Self {
            table: TableView::<IncomeDisplay,BasicColumn>::new()
                .column(BasicColumn::Date, "Date", |c| c.width_percent(40))
                .column(BasicColumn::Amount, "Amount", |c| c.width_percent(60))
                .items(income_displays),
            income_repo,
        }
    }

    pub fn add_table(self, siv: &mut Cursive) {
        siv.pop_layer();

        let repo_add = Rc::clone(&self.income_repo);
        let repo_edit = Rc::clone(&self.income_repo);
        let repo_duplicate = Rc::clone(&self.income_repo);
        let repo_delete = Rc::clone(&self.income_repo);

        let buttons = LinearLayout::horizontal()
            .child(Button::new("Add", move |s| income_form(s, None, &repo_add)))
            .child(HideableView::new(Button::new("Edit", move |s| {
                let selected = s.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
                    v.borrow_item(v.item().unwrap()).cloned()
                }).flatten();

                if let Some(income) = selected {
                    income_form(s, Some(income), &repo_edit);
                }
            })).with_name(INCOME_EDIT_BUTTON))
            .child(HideableView::new(Button::new("Duplicate", move |s| duplicate_income(s, &repo_duplicate))).with_name(INCOME_DUPLICATE_BUTTON))
            .child(HideableView::new(Button::new("Delete", move |s| delete_income(s, &repo_delete))).with_name(INCOME_DELETE_BUTTON));

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

fn income_form(siv: &mut Cursive, existing: Option<IncomeDisplay>, income_repo: &Rc<IncomeRepo>) {
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
    
    let notes_value = existing
        .as_ref()
        .and_then(|i| i.notes.clone())
        .unwrap_or_default();

    let income_id = existing.map(|i| i.id);
    let repo_form = Rc::clone(income_repo);

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

                let notes_str = s.call_on_name("notes_input", |v: &mut TextArea| {
                    v.get_content().to_string()
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

                let notes_opt = if notes_str.is_empty() { None } else { Some(notes_str.to_string()) };

                if let Some(record_id) = income_id {
                    repo_form.update(record_id, parsed_date.unwrap(), amount_bd.unwrap(), notes_opt);
                } else {
                    repo_form.create(parsed_date.unwrap(), amount_bd.unwrap(), notes_opt);
                }

                // Reload table
                let income_displays = repo_form.find_all()
                    .into_iter()
                    .map(|i| IncomeDisplay::from(i))
                    .collect::<Vec<_>>();
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
                    .child("Notes", TextArea::new().content(notes_value).with_name("notes_input").min_size((40, 3)))
            )
    );
}

fn delete_income(siv: &mut Cursive, income_repo: &Rc<IncomeRepo>) {
    let selected = siv.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(income) = selected {
        let repo_delete = Rc::clone(income_repo);
        siv.add_layer(
            Dialog::text("Delete this income?")
                .button("Yes", move |s| {
                    repo_delete.delete(income.id);

                    // Reload table
                    let income_displays = repo_delete.find_all()
                        .into_iter()
                        .map(|i| IncomeDisplay::from(i))
                        .collect::<Vec<_>>();
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

fn duplicate_income(siv: &mut Cursive, income_repo: &Rc<IncomeRepo>) {
    let selected = siv.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
        v.borrow_item(v.item().unwrap()).cloned()
    }).flatten();

    if let Some(income) = selected {
        income_repo.create(Local::now().date_naive(), income.amount, None);

        // Reload table
        let income_displays = income_repo.find_all()
            .into_iter()
            .map(|i| IncomeDisplay::from(i))
            .collect::<Vec<_>>();
        let income_count = income_displays.len();

        siv.call_on_name("income_table", |v: &mut TableView<IncomeDisplay, BasicColumn>| {
            v.set_items(income_displays);
        });

        toggle_buttons_visible(siv, income_count, TOGGLE_BUTTONS);
    }
}
