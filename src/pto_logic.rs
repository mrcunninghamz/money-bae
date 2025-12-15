use chrono::{Datelike, NaiveDate, Weekday};
use bigdecimal::BigDecimal;

/// Calculate workday hours between start and end dates (inclusive)
/// Counts M-F as 8-hour workdays, subtracts holiday hours in range
pub fn calculate_pto_hours(
    start_date: NaiveDate,
    end_date: NaiveDate,
    holidays: &[(NaiveDate, BigDecimal)],
) -> BigDecimal {
    let mut workdays = 0;
    let mut current = start_date;

    loop {
        match current.weekday() {
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri => {
                workdays += 1;
            }
            _ => {}
        }
        
        if current == end_date {
            break;
        }
        
        current = current.succ_opt().unwrap_or(current);
    }

    let base_hours = BigDecimal::from(workdays * 8);
    
    let holiday_hours: BigDecimal = holidays
        .iter()
        .filter(|(date, _)| *date >= start_date && *date <= end_date)
        .map(|(_, hours)| hours.clone())
        .sum();

    base_hours - holiday_hours
}

/// Calculate hours per day for validation (warn if > 8)
pub fn calculate_hours_per_day(hours: &BigDecimal, start_date: NaiveDate, end_date: NaiveDate) -> BigDecimal {
    let days = (end_date - start_date).num_days() + 1;
    if days == 0 {
        return BigDecimal::from(0);
    }
    hours / BigDecimal::from(days)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    #[test]
    fn test_workday_calculation_simple() {
        // Mon Jan 1 to Fri Jan 5, 2024 = 5 workdays = 40 hours
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let holidays = vec![];
        
        let result = calculate_pto_hours(start, end, &holidays);
        assert_eq!(result, BigDecimal::from(40));
    }

    #[test]
    fn test_workday_calculation_with_weekend() {
        // Mon Jan 1 to Sun Jan 7, 2024 = 5 workdays = 40 hours
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();
        let holidays = vec![];
        
        let result = calculate_pto_hours(start, end, &holidays);
        assert_eq!(result, BigDecimal::from(40));
    }

    #[test]
    fn test_workday_calculation_with_holiday() {
        // Mon Jan 1 to Fri Jan 5, with 8-hour holiday on Jan 3
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let holidays = vec![
            (NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), BigDecimal::from(8))
        ];
        
        let result = calculate_pto_hours(start, end, &holidays);
        assert_eq!(result, BigDecimal::from(32)); // 40 - 8
    }

    #[test]
    fn test_hours_per_day() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let hours = BigDecimal::from(40);
        
        let result = calculate_hours_per_day(&hours, start, end);
        assert_eq!(result, BigDecimal::from(8));
    }
}
