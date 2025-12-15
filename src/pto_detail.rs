use std::cmp::Ordering;
use bigdecimal::BigDecimal;
use chrono::{Datelike, NaiveDate};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, LinearLayout, Panel, SelectView, TextView};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;

use crate::models;
use crate::schema;
use crate::db::establish_connection;
use crate::ui_helpers::toggle_buttons_visible;

fn get_default_date(year: i32) -> String {
    format!("01/01/{}", year)
}

fn calculate_or_custom_hours(
    hours_str: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
    pto_id: i32
) -> (BigDecimal, bool) {
    if hours_str.trim().is_empty() {
        // Auto-calculate: load holidays for this PTO
        let mut conn = establish_connection();
        let holidays = schema::holiday_hours::table
            .filter(schema::holiday_hours::pto_id.eq(pto_id))
            .load::<models::HolidayHours>(&mut conn)
            .expect("Error loading holidays");
        
        let holiday_tuples: Vec<(chrono::NaiveDate, BigDecimal)> = holidays
            .into_iter()
            .map(|h| (h.date, h.hours))
            .collect();
        
        (crate::pto_logic::calculate_pto_hours(start_date, end_date, &holiday_tuples), false)
    } else {
        // Use custom hours provided
        (BigDecimal::parse_bytes(hours_str.as_bytes(), 10).unwrap_or_default(), true)
    }
}

fn parse_date_or_show_error(siv: &mut Cursive, date_str: &str) -> Option<NaiveDate> {
    match NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
        Ok(d) => Some(d),
        Err(_) => {
            siv.add_layer(Dialog::info("Invalid date format. Use MM/DD/YYYY"));
            None
        }
    }
}

const HOLIDAY_EDIT_BUTTON: &str = "holiday_edit_button";
const HOLIDAY_DELETE_BUTTON: &str = "holiday_delete_button";
const HOLIDAY_TOGGLE_BUTTONS: &[&str] = &[HOLIDAY_EDIT_BUTTON, HOLIDAY_DELETE_BUTTON];

const PLAN_EDIT_BUTTON: &str = "plan_edit_button";
const PLAN_DELETE_BUTTON: &str = "plan_delete_button";
const PLAN_VIEW_DESC_BUTTON: &str = "plan_view_desc_button";
const PLAN_TOGGLE_BUTTONS: &[&str] = &[PLAN_EDIT_BUTTON, PLAN_DELETE_BUTTON, PLAN_VIEW_DESC_BUTTON];

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum HolidayColumn {
    Date,
    Name,
    Hours,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum PlanColumn {
    StartDate,
    EndDate,
    Name,
    Hours,
    Status,
}

#[derive(Clone, Debug)]
struct HolidayDisplay {
    id: i32,
    date: NaiveDate,
    name: String,
    hours: BigDecimal,
}

#[derive(Clone, Debug)]
struct PlanDisplay {
    id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    name: String,
    description: Option<String>,
    hours: BigDecimal,
    status: String,
}

impl TableViewItem<HolidayColumn> for HolidayDisplay {
    fn to_column(&self, column: HolidayColumn) -> String {
        match column {
            HolidayColumn::Date => self.date.format("%m/%d/%Y").to_string(),
            HolidayColumn::Name => self.name.clone(),
            HolidayColumn::Hours => format!("{:.2}", self.hours),
        }
    }

    fn cmp(&self, other: &Self, column: HolidayColumn) -> Ordering {
        match column {
            HolidayColumn::Date => self.date.cmp(&other.date),
            HolidayColumn::Name => self.name.cmp(&other.name),
            HolidayColumn::Hours => self.hours.cmp(&other.hours),
        }
    }
}

impl TableViewItem<PlanColumn> for PlanDisplay {
    fn to_column(&self, column: PlanColumn) -> String {
        match column {
            PlanColumn::StartDate => self.start_date.format("%m/%d/%Y").to_string(),
            PlanColumn::EndDate => self.end_date.format("%m/%d/%Y").to_string(),
            PlanColumn::Name => self.name.clone(),
            PlanColumn::Hours => format!("{:.2}", self.hours),
            PlanColumn::Status => self.status.clone(),
        }
    }

    fn cmp(&self, other: &Self, column: PlanColumn) -> Ordering {
        match column {
            PlanColumn::StartDate => self.start_date.cmp(&other.start_date),
            PlanColumn::EndDate => self.end_date.cmp(&other.end_date),
            PlanColumn::Name => self.name.cmp(&other.name),
            PlanColumn::Hours => self.hours.cmp(&other.hours),
            PlanColumn::Status => self.status.cmp(&other.status),
        }
    }
}

pub fn show_pto_detail(siv: &mut Cursive, pto_id: i32) {
    use crate::schema::holiday_hours::dsl as holiday_dsl;
    use crate::schema::pto_plan::dsl as plan_dsl;
    use crate::schema::ptos::dsl::*;

    let mut conn = establish_connection();
    
    let pto = ptos.find(pto_id).first::<models::Pto>(&mut conn).expect("Error loading PTO");
    
    let holidays = holiday_dsl::holiday_hours
        .filter(holiday_dsl::pto_id.eq(pto_id))
        .order(holiday_dsl::date.asc())
        .load::<models::HolidayHours>(&mut conn)
        .expect("Error loading holidays");
    
    let plans = plan_dsl::pto_plan
        .filter(plan_dsl::pto_id.eq(pto_id))
        .order(plan_dsl::start_date.asc())
        .load::<models::PtoPlan>(&mut conn)
        .expect("Error loading PTO plans");

    // Left column: PTO Planning table
    let mut plan_table = TableView::<PlanDisplay, PlanColumn>::new()
        .column(PlanColumn::StartDate, "Start", |c| c.width(11))
        .column(PlanColumn::EndDate, "End", |c| c.width(11))
        .column(PlanColumn::Name, "Name", |c| c.width(18))
        .column(PlanColumn::Hours, "Hours", |c| c.width(7))
        .column(PlanColumn::Status, "Status", |c| c.width(12));

    plan_table.set_items(plans.into_iter().map(|p| PlanDisplay {
        id: p.id,
        start_date: p.start_date,
        end_date: p.end_date,
        name: p.name.clone(),
        description: p.description.clone(),
        hours: p.hours,
        status: p.status,
    }).collect::<Vec<_>>());

    plan_table.set_on_select(|siv: &mut Cursive, _row: usize, _index: usize| {
        let item_count = siv
            .call_on_name("plan_table", |table: &mut TableView<PlanDisplay, PlanColumn>| {
                table.len()
            })
            .unwrap_or(0);
        toggle_buttons_visible(siv, item_count, PLAN_TOGGLE_BUTTONS);
    });

    let plan_buttons = LinearLayout::horizontal()
        .child(Button::new("Add", move |s| show_add_plan_dialog(s, pto_id, pto.year)))
        .child(Button::new("Edit", move |s| edit_selected_plan(s, pto_id)).with_name(PLAN_EDIT_BUTTON))
        .child(Button::new("Delete", |s| delete_selected_plan(s)).with_name(PLAN_DELETE_BUTTON))
        .child(Button::new("View Description", |s| view_plan_description(s)).with_name(PLAN_VIEW_DESC_BUTTON));

    let left_col = LinearLayout::vertical()
        .child(Panel::new(plan_table.with_name("plan_table").full_height()))
        .child(plan_buttons);

    // Right column: Holiday Hours table + Summary
    let mut holiday_table = TableView::<HolidayDisplay, HolidayColumn>::new()
        .column(HolidayColumn::Date, "Date", |c| c.width(12))
        .column(HolidayColumn::Name, "Name", |c| c.width(20))
        .column(HolidayColumn::Hours, "Hours", |c| c.width(8));

    holiday_table.set_items(holidays.into_iter().map(|h| HolidayDisplay {
        id: h.id,
        date: h.date,
        name: h.name,
        hours: h.hours,
    }).collect::<Vec<_>>());

    holiday_table.set_on_select(|siv: &mut Cursive, _row: usize, _index: usize| {
        let item_count = siv
            .call_on_name("holiday_table", |table: &mut TableView<HolidayDisplay, HolidayColumn>| {
                table.len()
            })
            .unwrap_or(0);
        toggle_buttons_visible(siv, item_count, HOLIDAY_TOGGLE_BUTTONS);
    });

    let holiday_buttons = LinearLayout::horizontal()
        .child(Button::new("Add", move |s| show_add_holiday_dialog(s, pto_id, pto.year)))
        .child(Button::new("Edit", move |s| edit_selected_holiday(s, pto_id)).with_name(HOLIDAY_EDIT_BUTTON))
        .child(Button::new("Delete", |s| delete_selected_holiday(s)).with_name(HOLIDAY_DELETE_BUTTON))
        .child(Button::new("Copy from Last Year", move |s| copy_holidays_from_last_year(s, pto_id, pto.year)));

    let summary = TextView::new(format!(
        "Year: {}\nAvailable Hours: {:.2}\nHours Planned: {:.2}\nHours Used: {:.2}\nHours Remaining: {:.2}",
        pto.year, pto.available_hours, pto.hours_planned, pto.hours_used, pto.hours_remaining
    ));

    let summary_panel = Panel::new(summary).title("Summary");

    let right_col = LinearLayout::vertical()
        .child(Panel::new(holiday_table.with_name("holiday_table").full_height()))
        .child(holiday_buttons)
        .child(summary_panel);

    let layout = LinearLayout::horizontal()
        .child(Panel::new(left_col).title("Planned PTO").full_width())
        .child(Panel::new(right_col).title("Holiday Hours & Summary").full_width());

    siv.add_layer(Dialog::around(layout).title(format!("PTO Detail - {}", pto.year)).full_screen());
}

fn view_plan_description(siv: &mut Cursive) {
    let selected = siv
        .call_on_name("plan_table", |table: &mut TableView<PlanDisplay, PlanColumn>| {
            table.borrow_item(table.row().unwrap()).cloned()
        })
        .flatten();

    if let Some(plan) = selected {
        let desc_text = plan.description.clone().unwrap_or_else(|| "No description available.".to_string());
        
        let dialog = Dialog::around(TextView::new(desc_text))
            .title(format!("Description: {}", plan.name))
            .button("Close", |s| {
                s.pop_layer();
            });
        
        siv.add_layer(dialog);
    }
}

fn show_add_holiday_dialog(siv: &mut Cursive, pto_id: i32, pto_year: i32) {
    let dialog = Dialog::new()
        .title("Add Holiday")
        .content(
            LinearLayout::vertical()
                .child(TextView::new("Date (MM/DD/YYYY):"))
                .child(EditView::new().content(get_default_date(pto_year)).with_name("date").fixed_width(15))
                .child(TextView::new("Name:"))
                .child(EditView::new().with_name("name").fixed_width(25))
                .child(TextView::new("Hours:"))
                .child(EditView::new().content("8").with_name("hours").fixed_width(10))
        )
        .button("Ok", move |s| {
            let date_str = s.call_on_name("date", |v: &mut EditView| v.get_content()).unwrap();
            let name_str = s.call_on_name("name", |v: &mut EditView| v.get_content()).unwrap();
            let hours_str = s.call_on_name("hours", |v: &mut EditView| v.get_content()).unwrap();

            let date_val = match parse_date_or_show_error(s, &date_str) {
                Some(d) => d,
                None => return,
            };
            let hours_val = BigDecimal::parse_bytes(hours_str.as_bytes(), 10).unwrap_or_default();

            let mut conn = establish_connection();
            let new_holiday = models::NewHolidayHours {
                pto_id,
                date: date_val,
                name: name_str.to_string(),
                hours: hours_val,
            };

            diesel::insert_into(schema::holiday_hours::table)
                .values(&new_holiday)
                .execute(&mut conn)
                .expect("Error saving holiday");

            s.pop_layer();
            s.pop_layer();
            show_pto_detail(s, pto_id);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn edit_selected_holiday(siv: &mut Cursive, pto_id: i32) {
    let selected = siv
        .call_on_name("holiday_table", |table: &mut TableView<HolidayDisplay, HolidayColumn>| {
            table.borrow_item(table.row().unwrap()).cloned()
        })
        .flatten();

    if let Some(holiday) = selected {
        let dialog = Dialog::new()
            .title("Edit Holiday")
            .content(
                LinearLayout::vertical()
                    .child(TextView::new("Date (MM/DD/YYYY):"))
                    .child(EditView::new().content(holiday.date.format("%m/%d/%Y").to_string()).with_name("date").fixed_width(15))
                    .child(TextView::new("Name:"))
                    .child(EditView::new().content(holiday.name.clone()).with_name("name").fixed_width(25))
                    .child(TextView::new("Hours:"))
                    .child(EditView::new().content(holiday.hours.to_string()).with_name("hours").fixed_width(10))
            )
            .button("Ok", move |s| {
                let date_str = s.call_on_name("date", |v: &mut EditView| v.get_content()).unwrap();
                let name_str = s.call_on_name("name", |v: &mut EditView| v.get_content()).unwrap();
                let hours_str = s.call_on_name("hours", |v: &mut EditView| v.get_content()).unwrap();

                let date_val = match parse_date_or_show_error(s, &date_str) {
                    Some(d) => d,
                    None => return,
                };
                let hours_val = BigDecimal::parse_bytes(hours_str.as_bytes(), 10).unwrap_or_default();

                let mut conn = establish_connection();
                diesel::update(schema::holiday_hours::table.find(holiday.id))
                    .set((
                        schema::holiday_hours::date.eq(date_val),
                        schema::holiday_hours::name.eq(name_str.to_string()),
                        schema::holiday_hours::hours.eq(hours_val),
                    ))
                    .execute(&mut conn)
                    .expect("Error updating holiday");

                s.pop_layer();
                s.pop_layer();
                show_pto_detail(s, pto_id);
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}

fn delete_selected_holiday(siv: &mut Cursive) {
    let selected_id = siv
        .call_on_name("holiday_table", |table: &mut TableView<HolidayDisplay, HolidayColumn>| {
            table.borrow_item(table.row().unwrap()).map(|item| item.id)
        })
        .flatten();

    if let Some(holiday_id) = selected_id {
        let dialog = Dialog::text("Delete this holiday?")
            .button("Yes", move |s| {
                let mut conn = establish_connection();
                diesel::delete(schema::holiday_hours::table.find(holiday_id))
                    .execute(&mut conn)
                    .expect("Error deleting holiday");

                s.pop_layer();
            })
            .button("No", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}

fn copy_holidays_from_last_year(siv: &mut Cursive, pto_id: i32, current_year: i32) {
    let mut conn = establish_connection();
    
    // Find PTO record for previous year
    let prev_year = current_year - 1;
    let prev_pto = schema::ptos::table
        .filter(schema::ptos::year.eq(prev_year))
        .first::<models::Pto>(&mut conn)
        .optional()
        .expect("Error loading previous year PTO");
    
    if let Some(prev_pto) = prev_pto {
        // Load holidays from previous year
        let prev_holidays = schema::holiday_hours::table
            .filter(schema::holiday_hours::pto_id.eq(prev_pto.id))
            .load::<models::HolidayHours>(&mut conn)
            .expect("Error loading previous year holidays");
        
        if prev_holidays.is_empty() {
            siv.add_layer(Dialog::info(format!("No holidays found for year {}", prev_year)));
            return;
        }
        
        let count = prev_holidays.len();
        
        // Copy holidays with updated year
        for holiday in prev_holidays {
            let new_date = holiday.date.with_year(current_year)
                .unwrap_or(holiday.date);
            
            let new_holiday = models::NewHolidayHours {
                pto_id,
                date: new_date,
                name: holiday.name.clone(),
                hours: holiday.hours.clone(),
            };
            
            diesel::insert_into(schema::holiday_hours::table)
                .values(&new_holiday)
                .execute(&mut conn)
                .expect("Error copying holiday");
        }
        
        siv.add_layer(Dialog::info(format!("Copied {} holidays from {}", count, prev_year))
            .button("Ok", move |s| {
                s.pop_layer();
                s.pop_layer();
                show_pto_detail(s, pto_id);
            }));
    } else {
        siv.add_layer(Dialog::info(format!("No PTO record found for year {}", prev_year)));
    }
}

fn show_add_plan_dialog(siv: &mut Cursive, pto_id: i32, pto_year: i32) {
    let dialog = Dialog::new()
        .title("Add PTO Plan")
        .content(
            LinearLayout::vertical()
                .child(TextView::new("Start Date (MM/DD/YYYY):"))
                .child(EditView::new().content(get_default_date(pto_year)).with_name("start_date").fixed_width(15))
                .child(TextView::new("End Date (MM/DD/YYYY):"))
                .child(EditView::new().content(get_default_date(pto_year)).with_name("end_date").fixed_width(15))
                .child(TextView::new("Name:"))
                .child(EditView::new().with_name("name").fixed_width(25))
                .child(TextView::new("Description:"))
                .child(EditView::new().with_name("description").fixed_width(40))
                .child(TextView::new("Hours (leave blank for auto calculation):"))
                .child(EditView::new().with_name("hours").fixed_width(10))
        )
        .button("Ok", move |s| {
            let start_str = s.call_on_name("start_date", |v: &mut EditView| v.get_content()).unwrap();
            let end_str = s.call_on_name("end_date", |v: &mut EditView| v.get_content()).unwrap();
            let name_str = s.call_on_name("name", |v: &mut EditView| v.get_content()).unwrap();
            let desc_str = s.call_on_name("description", |v: &mut EditView| v.get_content()).unwrap();
            let hours_str = s.call_on_name("hours", |v: &mut EditView| v.get_content()).unwrap();

            let start_val = match parse_date_or_show_error(s, &start_str) {
                Some(d) => d,
                None => return,
            };
            let end_val = match parse_date_or_show_error(s, &end_str) {
                Some(d) => d,
                None => return,
            };
            
            let (hours_val, custom_hours_val) = calculate_or_custom_hours(&hours_str, start_val, end_val, pto_id);

            let mut conn = establish_connection();
            let new_plan = models::NewPtoPlan {
                pto_id,
                start_date: start_val,
                end_date: end_val,
                name: name_str.to_string(),
                description: if desc_str.is_empty() { None } else { Some(desc_str.to_string()) },
                hours: hours_val,
                status: "Planned".to_string(),
                custom_hours: custom_hours_val,
            };

            diesel::insert_into(schema::pto_plan::table)
                .values(&new_plan)
                .execute(&mut conn)
                .expect("Error saving PTO plan");

            s.pop_layer();
            s.pop_layer();
            show_pto_detail(s, pto_id);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn edit_selected_plan(siv: &mut Cursive, pto_id: i32) {
    let selected = siv
        .call_on_name("plan_table", |table: &mut TableView<PlanDisplay, PlanColumn>| {
            table.borrow_item(table.row().unwrap()).cloned()
        })
        .flatten();

    if let Some(plan) = selected {
        let dialog = Dialog::new()
            .title("Edit PTO Plan")
            .content(
                LinearLayout::vertical()
                    .child(TextView::new("Start Date (MM/DD/YYYY):"))
                    .child(EditView::new().content(plan.start_date.format("%m/%d/%Y").to_string()).with_name("start_date").fixed_width(15))
                    .child(TextView::new("End Date (MM/DD/YYYY):"))
                    .child(EditView::new().content(plan.end_date.format("%m/%d/%Y").to_string()).with_name("end_date").fixed_width(15))
                    .child(TextView::new("Name:"))
                    .child(EditView::new().content(plan.name.clone()).with_name("name").fixed_width(25))
                    .child(TextView::new("Description:"))
                    .child(EditView::new().content(plan.description.clone().unwrap_or_default()).with_name("description").fixed_width(40))
                    .child(TextView::new("Hours (leave blank for auto calculation):"))
                    .child(EditView::new().content(plan.hours.to_string()).with_name("hours").fixed_width(10))
                    .child(TextView::new("Status:"))
                    .child({
                        let mut select = SelectView::new();
                        let current_status = models::PtoStatus::from(plan.status.clone());
                        
                        for status in models::PtoStatus::all() {
                            select.add_item(status.to_string(), status);
                        }
                        
                        select.set_selection(current_status as usize);
                        select.with_name("status")
                    })
            )
            .button("Ok", move |s| {
                let start_str = s.call_on_name("start_date", |v: &mut EditView| v.get_content()).unwrap();
                let end_str = s.call_on_name("end_date", |v: &mut EditView| v.get_content()).unwrap();
                let name_str = s.call_on_name("name", |v: &mut EditView| v.get_content()).unwrap();
                let desc_str = s.call_on_name("description", |v: &mut EditView| v.get_content()).unwrap();
                let hours_str = s.call_on_name("hours", |v: &mut EditView| v.get_content()).unwrap();
                let status = s.call_on_name("status", |v: &mut SelectView<models::PtoStatus>| {
                    v.selection().map(|s| *s.as_ref())
                }).unwrap().unwrap_or(models::PtoStatus::Planned);

                let start_val = match parse_date_or_show_error(s, &start_str) {
                    Some(d) => d,
                    None => return,
                };
                let end_val = match parse_date_or_show_error(s, &end_str) {
                    Some(d) => d,
                    None => return,
                };
                
                let (hours_val, custom_hours_val) = calculate_or_custom_hours(&hours_str, start_val, end_val, pto_id);

                let mut conn = establish_connection();
                diesel::update(schema::pto_plan::table.find(plan.id))
                    .set((
                        schema::pto_plan::start_date.eq(start_val),
                        schema::pto_plan::end_date.eq(end_val),
                        schema::pto_plan::name.eq(name_str.to_string()),
                        schema::pto_plan::description.eq(if desc_str.is_empty() { None } else { Some(desc_str.to_string()) }),
                        schema::pto_plan::hours.eq(hours_val),
                        schema::pto_plan::status.eq(String::from(status)),
                        schema::pto_plan::custom_hours.eq(custom_hours_val),
                    ))
                    .execute(&mut conn)
                    .expect("Error updating PTO plan");

                s.pop_layer();
                s.pop_layer();
                show_pto_detail(s, pto_id);
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}

fn delete_selected_plan(siv: &mut Cursive) {
    let selected_id = siv
        .call_on_name("plan_table", |table: &mut TableView<PlanDisplay, PlanColumn>| {
            table.borrow_item(table.row().unwrap()).map(|item| item.id)
        })
        .flatten();

    if let Some(plan_id) = selected_id {
        let dialog = Dialog::text("Delete this PTO plan?")
            .button("Yes", move |s| {
                let mut conn = establish_connection();
                diesel::delete(schema::pto_plan::table.find(plan_id))
                    .execute(&mut conn)
                    .expect("Error deleting PTO plan");

                s.pop_layer();
            })
            .button("No", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}
