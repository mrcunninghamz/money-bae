use std::cmp::Ordering;
use bigdecimal::BigDecimal;
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, LinearLayout, Panel};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;

use crate::models;
use crate::schema::ptos::dsl::*;
use crate::db::establish_connection;
use crate::ui_helpers::toggle_buttons_visible;

const PTO_VIEW_BUTTON: &str = "pto_table_view_button";
const PTO_EDIT_BUTTON: &str = "pto_table_edit_button";
const PTO_DELETE_BUTTON: &str = "pto_table_delete_button";
const TOGGLE_BUTTONS: &[&str] = &[PTO_VIEW_BUTTON, PTO_EDIT_BUTTON, PTO_DELETE_BUTTON];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum PtoColumn {
    Year,
    AvailableHours,
    HoursPlanned,
    HoursUsed,
    HoursRemaining,
}

#[derive(Clone, Debug)]
struct PtoDisplay {
    id: i32,
    year: i32,
    available_hours: BigDecimal,
    hours_planned: BigDecimal,
    hours_used: BigDecimal,
    hours_remaining: BigDecimal,
}

impl From<models::Pto> for PtoDisplay {
    fn from(pto: models::Pto) -> Self {
        PtoDisplay {
            id: pto.id,
            year: pto.year,
            available_hours: pto.available_hours,
            hours_planned: pto.hours_planned,
            hours_used: pto.hours_used,
            hours_remaining: pto.hours_remaining,
        }
    }
}

impl TableViewItem<PtoColumn> for PtoDisplay {
    fn to_column(&self, column: PtoColumn) -> String {
        match column {
            PtoColumn::Year => self.year.to_string(),
            PtoColumn::AvailableHours => format!("{:.2}", self.available_hours),
            PtoColumn::HoursPlanned => format!("{:.2}", self.hours_planned),
            PtoColumn::HoursUsed => format!("{:.2}", self.hours_used),
            PtoColumn::HoursRemaining => format!("{:.2}", self.hours_remaining),
        }
    }

    fn cmp(&self, other: &Self, column: PtoColumn) -> Ordering where Self: Sized {
        match column {
            PtoColumn::Year => self.year.cmp(&other.year),
            PtoColumn::AvailableHours => self.available_hours.cmp(&other.available_hours),
            PtoColumn::HoursPlanned => self.hours_planned.cmp(&other.hours_planned),
            PtoColumn::HoursUsed => self.hours_used.cmp(&other.hours_used),
            PtoColumn::HoursRemaining => self.hours_remaining.cmp(&other.hours_remaining),
        }
    }
}

pub fn show_pto_table_view(siv: &mut Cursive) {
    let mut conn = establish_connection();
    
    let pto_records = ptos
        .order(year.desc())
        .load::<models::Pto>(&mut conn)
        .expect("Error loading PTOs");

    let mut table = TableView::<PtoDisplay, PtoColumn>::new()
        .column(PtoColumn::Year, "Year", |c| c.width(10))
        .column(PtoColumn::AvailableHours, "Available", |c| c.width(12))
        .column(PtoColumn::HoursPlanned, "Planned", |c| c.width(12))
        .column(PtoColumn::HoursUsed, "Used", |c| c.width(12))
        .column(PtoColumn::HoursRemaining, "Remaining", |c| c.width(12));

    table.set_items(pto_records.into_iter().map(PtoDisplay::from).collect::<Vec<_>>());

    table.set_on_select(|siv: &mut Cursive, _row: usize, _index: usize| {
        let item_count = siv
            .call_on_name("pto_table", |table: &mut TableView<PtoDisplay, PtoColumn>| {
                table.len()
            })
            .unwrap_or(0);
        toggle_buttons_visible(siv, item_count, TOGGLE_BUTTONS);
    });

    let table_view = Panel::new(table.with_name("pto_table").full_screen())
        .title("PTO Records");

    let buttons = LinearLayout::horizontal()
        .child(Button::new("Add", |s| show_add_pto_dialog(s)))
        .child(Button::new("View", |s| view_selected_pto(s)).with_name(PTO_VIEW_BUTTON))
        .child(Button::new("Edit", |s| edit_selected_pto(s)).with_name(PTO_EDIT_BUTTON))
        .child(Button::new("Delete", |s| delete_selected_pto(s)).with_name(PTO_DELETE_BUTTON));

    let layout = LinearLayout::vertical()
        .child(table_view)
        .child(buttons);

    siv.add_layer(layout);
}

fn show_add_pto_dialog(siv: &mut Cursive) {
    let dialog = Dialog::new()
        .title("Add PTO Record")
        .content(
            LinearLayout::vertical()
                .child(
                    LinearLayout::horizontal()
                        .child(Panel::new(EditView::new().with_name("year").fixed_width(10)))
                        .child(Panel::new(EditView::new().with_name("available_hours").fixed_width(10)))
                )
        )
        .button("Ok", |s| {
            let year_str = s.call_on_name("year", |v: &mut EditView| v.get_content()).unwrap();
            let available_str = s.call_on_name("available_hours", |v: &mut EditView| v.get_content()).unwrap();

            let year_val: i32 = year_str.parse().unwrap_or(0);
            let available_val = BigDecimal::parse_bytes(available_str.as_bytes(), 10).unwrap_or_default();

            let mut conn = establish_connection();
            let new_pto = models::NewPto {
                year: year_val,
                available_hours: available_val,
            };

            diesel::insert_into(ptos)
                .values(&new_pto)
                .execute(&mut conn)
                .expect("Error saving PTO");

            s.pop_layer();
            s.pop_layer();
            show_pto_table_view(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn view_selected_pto(siv: &mut Cursive) {
    let selected_id = siv
        .call_on_name("pto_table", |table: &mut TableView<PtoDisplay, PtoColumn>| {
            table.borrow_item(table.row().unwrap()).map(|item| item.id)
        })
        .flatten();

    if let Some(pto_id) = selected_id {
        siv.pop_layer();
        crate::pto_detail::show_pto_detail(siv, pto_id);
    }
}

fn edit_selected_pto(siv: &mut Cursive) {
    let selected_id = siv
        .call_on_name("pto_table", |table: &mut TableView<PtoDisplay, PtoColumn>| {
            table.borrow_item(table.row().unwrap()).map(|item| item.id)
        })
        .flatten();

    if let Some(pto_id) = selected_id {
        let mut conn = establish_connection();
        let pto_record = ptos.find(pto_id).first::<models::Pto>(&mut conn).expect("Error loading PTO");

        let dialog = Dialog::new()
            .title("Edit PTO Record")
            .content(
                LinearLayout::vertical()
                    .child(
                        LinearLayout::horizontal()
                            .child(Panel::new(EditView::new().content(pto_record.available_hours.to_string()).with_name("available_hours").fixed_width(10)))
                    )
            )
            .button("Ok", move |s| {
                let available_str = s.call_on_name("available_hours", |v: &mut EditView| v.get_content()).unwrap();
                let available_val = BigDecimal::parse_bytes(available_str.as_bytes(), 10).unwrap_or_default();

                let mut conn = establish_connection();
                diesel::update(ptos.find(pto_id))
                    .set(available_hours.eq(available_val))
                    .execute(&mut conn)
                    .expect("Error updating PTO");

                s.pop_layer();
                s.pop_layer();
                show_pto_table_view(s);
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}

fn delete_selected_pto(siv: &mut Cursive) {
    let selected_id = siv
        .call_on_name("pto_table", |table: &mut TableView<PtoDisplay, PtoColumn>| {
            table.borrow_item(table.row().unwrap()).map(|item| item.id)
        })
        .flatten();

    if let Some(pto_id) = selected_id {
        let dialog = Dialog::text("Delete this PTO record?")
            .button("Yes", move |s| {
                let mut conn = establish_connection();
                diesel::delete(ptos.find(pto_id))
                    .execute(&mut conn)
                    .expect("Error deleting PTO");

                s.pop_layer();
                s.pop_layer();
                show_pto_table_view(s);
            })
            .button("No", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}
