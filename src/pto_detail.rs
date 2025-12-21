use std::cmp::Ordering;
use std::rc::Rc;
use bigdecimal::BigDecimal;
use chrono::{Datelike, NaiveDate};
use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, LinearLayout, Panel, SelectView, TextView};
use cursive_table_view::{TableView, TableViewItem};
use diesel::prelude::*;

use crate::models;
use crate::schema;
use crate::repositories::pto_repo::PtoRepo;
use crate::repositories::pto_plan_repo::PtoPlanRepo;
use crate::repositories::holiday_hours_repo::HolidayHoursRepo;
use crate::ui_helpers::toggle_buttons_visible;

fn get_default_date(year: i32) -> String {
    format!("01/01/{}", year)
}

fn calculate_or_custom_hours(
    hours_str: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
    pto_id: i32,
    holiday_repo: &Rc<HolidayHoursRepo>
) -> (BigDecimal, bool) {
    if hours_str.trim().is_empty() {
        // Auto-calculate: load holidays for this PTO
        let holidays = holiday_repo.find_by_pto_id(pto_id);
        
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

pub fn show_pto_detail(siv: &mut Cursive, pto_id: i32, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let pto = pto_repo.find_by_id(pto_id).expect("Error loading PTO");
    let holidays = holiday_repo.find_by_pto_id(pto_id);
    let plans = pto_plan_repo.find_by_pto_id(pto_id);

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

    let repo_add_plan = Rc::clone(pto_plan_repo);
    let pto_repo_add_plan = Rc::clone(pto_repo);
    let holiday_repo_add_plan = Rc::clone(holiday_repo);
    let repo_edit_plan = Rc::clone(pto_plan_repo);
    let pto_repo_edit_plan = Rc::clone(pto_repo);
    let holiday_repo_edit_plan = Rc::clone(holiday_repo);
    let repo_delete_plan = Rc::clone(pto_plan_repo);
    let pto_repo_delete_plan = Rc::clone(pto_repo);
    let holiday_repo_delete_plan = Rc::clone(holiday_repo);
    let holiday_repo_for_plan = Rc::clone(holiday_repo);
    let plan_buttons = LinearLayout::horizontal()
        .child(Button::new("Add", move |s| show_add_plan_dialog(s, pto_id, pto.year, &pto_repo_add_plan, &repo_add_plan, &holiday_repo_add_plan)))
        .child(Button::new("Edit", move |s| edit_selected_plan(s, pto_id, &pto_repo_edit_plan, &repo_edit_plan, &holiday_repo_edit_plan)).with_name(PLAN_EDIT_BUTTON))
        .child(Button::new("Delete", move |s| delete_selected_plan(s, &pto_repo_delete_plan, &repo_delete_plan, &holiday_repo_delete_plan)).with_name(PLAN_DELETE_BUTTON))
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

    let repo_add_holiday = Rc::clone(holiday_repo);
    let pto_repo_add = Rc::clone(pto_repo);
    let plan_repo_add = Rc::clone(pto_plan_repo);
    let repo_edit_holiday = Rc::clone(holiday_repo);
    let pto_repo_edit = Rc::clone(pto_repo);
    let plan_repo_edit = Rc::clone(pto_plan_repo);
    let repo_delete_holiday = Rc::clone(holiday_repo);
    let pto_repo_delete = Rc::clone(pto_repo);
    let plan_repo_delete = Rc::clone(pto_plan_repo);
    let repo_copy_holiday = Rc::clone(holiday_repo);
    let pto_repo_copy = Rc::clone(pto_repo);
    let plan_repo_copy = Rc::clone(pto_plan_repo);
    let holiday_buttons = LinearLayout::horizontal()
        .child(Button::new("Add", move |s| show_add_holiday_dialog(s, pto_id, pto.year, &pto_repo_add, &plan_repo_add, &repo_add_holiday)))
        .child(Button::new("Edit", move |s| edit_selected_holiday(s, pto_id, &pto_repo_edit, &plan_repo_edit, &repo_edit_holiday)).with_name(HOLIDAY_EDIT_BUTTON))
        .child(Button::new("Delete", move |s| delete_selected_holiday(s, &pto_repo_delete, &plan_repo_delete, &repo_delete_holiday)).with_name(HOLIDAY_DELETE_BUTTON))
        .child(Button::new("Copy from Last Year", move |s| copy_holidays_from_last_year(s, pto_id, pto.year, &pto_repo_copy, &plan_repo_copy, &repo_copy_holiday)));

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

    let screen = crate::common_layout::create_screen(
        &format!("PTO Detail - {}", pto.year),
        layout,
        &crate::common_layout::view_footer()
    );

    siv.add_layer(screen);
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

fn show_add_holiday_dialog(siv: &mut Cursive, pto_id: i32, pto_year: i32, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let repo_ok = Rc::clone(holiday_repo);
    let pto_repo_ok = Rc::clone(pto_repo);
    let plan_repo_ok = Rc::clone(pto_plan_repo);
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

            repo_ok.create(pto_id, date_val, name_str.to_string(), hours_val);

            s.pop_layer();
            s.pop_layer();
            show_pto_detail(s, pto_id, &pto_repo_ok, &plan_repo_ok, &repo_ok);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn edit_selected_holiday(siv: &mut Cursive, pto_id: i32, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let selected = siv
        .call_on_name("holiday_table", |table: &mut TableView<HolidayDisplay, HolidayColumn>| {
            table.borrow_item(table.row().unwrap()).cloned()
        })
        .flatten();

    if let Some(holiday) = selected {
        let repo_ok = Rc::clone(holiday_repo);
        let pto_repo_ok = Rc::clone(pto_repo);
        let plan_repo_ok = Rc::clone(pto_plan_repo);
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

                repo_ok.update(holiday.id, date_val, name_str.to_string(), hours_val);

                s.pop_layer();
                s.pop_layer();
                show_pto_detail(s, pto_id, &pto_repo_ok, &plan_repo_ok, &repo_ok);
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}

fn delete_selected_holiday(siv: &mut Cursive, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let selected_id = siv
        .call_on_name("holiday_table", |table: &mut TableView<HolidayDisplay, HolidayColumn>| {
            table.borrow_item(table.row().unwrap()).map(|item| item.id)
        })
        .flatten();

    if let Some(holiday_id) = selected_id {
        let repo_yes = Rc::clone(holiday_repo);
        let dialog = Dialog::text("Delete this holiday?")
            .button("Yes", move |s| {
                repo_yes.delete(holiday_id);

                s.pop_layer();
            })
            .button("No", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}

fn copy_holidays_from_last_year(siv: &mut Cursive, pto_id: i32, current_year: i32, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let prev_year = current_year - 1;
    
    // Find PTO record for previous year
    let pg_connector = holiday_repo.pg_connector();
    let prev_pto = {
        let mut conn = pg_connector.get_connection();
        schema::ptos::table
            .filter(schema::ptos::year.eq(prev_year))
            .first::<models::Pto>(&mut *conn)
            .optional()
            .expect("Error loading previous year PTO")
    };
    
    if let Some(prev_pto) = prev_pto {
        let count = holiday_repo.copy_from_previous_year(prev_pto.id, pto_id, 1);
        
        if count == 0 {
            siv.add_layer(Dialog::info(format!("No holidays found for year {}", prev_year)));
            return;
        }
        
        let repo_ok = Rc::clone(pto_repo);
        let plan_repo_ok = Rc::clone(pto_plan_repo);
        let holiday_repo_ok = Rc::clone(holiday_repo);
        siv.add_layer(Dialog::info(format!("Copied {} holidays from {}", count, prev_year))
            .button("Ok", move |s| {
                s.pop_layer();
                s.pop_layer();
                show_pto_detail(s, pto_id, &repo_ok, &plan_repo_ok, &holiday_repo_ok);
            }));
    } else {
        siv.add_layer(Dialog::info(format!("No PTO record found for year {}", prev_year)));
    }
}

fn show_add_plan_dialog(siv: &mut Cursive, pto_id: i32, pto_year: i32, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let repo_ok = Rc::clone(pto_plan_repo);
    let pto_repo_ok = Rc::clone(pto_repo);
    let plan_repo_ok = Rc::clone(pto_plan_repo);
    let holiday_repo_calc = Rc::clone(holiday_repo);
    let holiday_repo_ok = Rc::clone(holiday_repo);
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
            
            let (hours_val, custom_hours_val) = calculate_or_custom_hours(&hours_str, start_val, end_val, pto_id, &holiday_repo_calc);

            repo_ok.create(
                pto_id,
                start_val,
                end_val,
                name_str.to_string(),
                if desc_str.is_empty() { None } else { Some(desc_str.to_string()) },
                hours_val,
                "Planned".to_string(),
                custom_hours_val,
            );

            s.pop_layer();
            s.pop_layer();
            show_pto_detail(s, pto_id, &pto_repo_ok, &plan_repo_ok, &holiday_repo_ok);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    siv.add_layer(dialog);
}

fn edit_selected_plan(siv: &mut Cursive, pto_id: i32, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let selected = siv
        .call_on_name("plan_table", |table: &mut TableView<PlanDisplay, PlanColumn>| {
            table.borrow_item(table.row().unwrap()).cloned()
        })
        .flatten();

    if let Some(plan) = selected {
        let repo_ok = Rc::clone(pto_plan_repo);
        let pto_repo_ok = Rc::clone(pto_repo);
        let plan_repo_ok = Rc::clone(pto_plan_repo);
        let holiday_repo_calc = Rc::clone(holiday_repo);
        let holiday_repo_ok = Rc::clone(holiday_repo);
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
                
                let (hours_val, custom_hours_val) = calculate_or_custom_hours(&hours_str, start_val, end_val, pto_id, &holiday_repo_calc);

                repo_ok.update(
                    plan.id,
                    start_val,
                    end_val,
                    name_str.to_string(),
                    if desc_str.is_empty() { None } else { Some(desc_str.to_string()) },
                    hours_val,
                    String::from(status),
                    custom_hours_val,
                );

                s.pop_layer();
                s.pop_layer();
                show_pto_detail(s, pto_id, &pto_repo_ok, &plan_repo_ok, &holiday_repo_ok);
            })
            .button("Cancel", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}

fn delete_selected_plan(siv: &mut Cursive, pto_repo: &Rc<PtoRepo>, pto_plan_repo: &Rc<PtoPlanRepo>, holiday_repo: &Rc<HolidayHoursRepo>) {
    let selected_id = siv
        .call_on_name("plan_table", |table: &mut TableView<PlanDisplay, PlanColumn>| {
            table.borrow_item(table.row().unwrap()).map(|item| item.id)
        })
        .flatten();

    if let Some(plan_id) = selected_id {
        let repo_yes = Rc::clone(pto_plan_repo);
        let dialog = Dialog::text("Delete this PTO plan?")
            .button("Yes", move |s| {
                repo_yes.delete(plan_id);

                s.pop_layer();
            })
            .button("No", |s| {
                s.pop_layer();
            });

        siv.add_layer(dialog);
    }
}
