use std::rc::Rc;
use diesel::prelude::*;
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, Datelike};

use crate::db::PgConnector;
use crate::models::{HolidayHours, NewHolidayHours};
use crate::schema::holiday_hours;

pub struct HolidayHoursRepo {
    pg_connector: Rc<PgConnector>,
}

impl HolidayHoursRepo {
    pub fn new(pg_connector: Rc<PgConnector>) -> Self {
        Self { pg_connector }
    }

    pub fn pg_connector(&self) -> Rc<PgConnector> {
        Rc::clone(&self.pg_connector)
    }

    pub fn find_by_pto_id(&self, pto_id: i32) -> Vec<HolidayHours> {
        let mut conn = self.pg_connector.get_connection();
        holiday_hours::table
            .filter(holiday_hours::pto_id.eq(pto_id))
            .order(holiday_hours::date.asc())
            .load::<HolidayHours>(&mut *conn)
            .expect("Error loading holiday hours")
    }

    pub fn find_by_id(&self, holiday_id: i32) -> Option<HolidayHours> {
        let mut conn = self.pg_connector.get_connection();
        holiday_hours::table
            .find(holiday_id)
            .first::<HolidayHours>(&mut *conn)
            .ok()
    }

    pub fn create(
        &self,
        holiday_pto_id: i32,
        holiday_date: NaiveDate,
        holiday_name: String,
        holiday_hours: BigDecimal,
    ) -> HolidayHours {
        let mut conn = self.pg_connector.get_connection();
        let new_holiday = NewHolidayHours {
            pto_id: holiday_pto_id,
            date: holiday_date,
            name: holiday_name,
            hours: holiday_hours,
        };

        diesel::insert_into(holiday_hours::table)
            .values(&new_holiday)
            .returning(HolidayHours::as_returning())
            .get_result(&mut *conn)
            .expect("Error saving new holiday hours")
    }

    pub fn update(
        &self,
        holiday_id: i32,
        holiday_date: NaiveDate,
        holiday_name: String,
        holiday_hours_val: BigDecimal,
    ) -> HolidayHours {
        let mut conn = self.pg_connector.get_connection();
        diesel::update(holiday_hours::table.filter(holiday_hours::id.eq(holiday_id)))
            .set((
                holiday_hours::date.eq(holiday_date),
                holiday_hours::name.eq(holiday_name),
                holiday_hours::hours.eq(holiday_hours_val),
            ))
            .returning(HolidayHours::as_returning())
            .get_result(&mut *conn)
            .expect("Error updating holiday hours")
    }

    pub fn delete(&self, holiday_id: i32) -> bool {
        let mut conn = self.pg_connector.get_connection();
        diesel::delete(holiday_hours::table.filter(holiday_hours::id.eq(holiday_id)))
            .execute(&mut *conn)
            .is_ok()
    }

    pub fn copy_from_previous_year(&self, source_pto_id: i32, target_pto_id: i32, year_offset: i32) -> usize {
        let source_holidays = self.find_by_pto_id(source_pto_id);
        let mut conn = self.pg_connector.get_connection();
        
        let new_holidays: Vec<NewHolidayHours> = source_holidays
            .into_iter()
            .map(|h| {
                let new_date = NaiveDate::from_ymd_opt(
                    h.date.year() + year_offset,
                    h.date.month(),
                    h.date.day()
                ).unwrap_or(h.date);
                
                NewHolidayHours {
                    pto_id: target_pto_id,
                    date: new_date,
                    name: h.name,
                    hours: h.hours,
                }
            })
            .collect();

        diesel::insert_into(holiday_hours::table)
            .values(&new_holidays)
            .execute(&mut *conn)
            .unwrap_or(0)
    }
}
