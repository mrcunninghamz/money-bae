use std::str::FromStr;
use bigdecimal::BigDecimal;
use cursive::Cursive;
use cursive::views::{Button, HideableView};
use rusty_money::{iso, Money};

pub fn toggle_buttons_visible(siv: &mut Cursive, item_count: usize, button_names: &[&str]) {
    for name in button_names {
        siv.call_on_name(name, |v: &mut HideableView<Button>| {
            v.set_visible(item_count > 0);
        });
    }
}

/// Formats a `BigDecimal` amount as a standard USD currency string using rusty-money.
pub fn format_currency(amount: &BigDecimal) -> String {
    let rounded = amount.round(2);
    let money = Money::from_str(&rounded.to_string(), iso::USD)
        .unwrap_or_else(|_| Money::from_minor(0, iso::USD));
    money.to_string()
}

/// Parses a user-input currency string into a `BigDecimal`.
/// Strips currency symbols, parentheses, and whitespace, delegating numeric and comma parsing to `rusty_money`.
pub fn parse_currency(input: &str) -> Result<BigDecimal, String> {
    let mut s = input.trim();
    if s.is_empty() {
        return Err("Input cannot be empty".to_string());
    }

    let mut is_negative = false;

    // Handle accounting parentheses notation e.g. ($1,234.56) or (1234.56)
    if s.starts_with('(') && s.ends_with(')') && s.len() >= 2 {
        is_negative = true;
        s = &s[1..s.len() - 1];
    }

    // Strip $, spaces, and handle trailing minus e.g. 100-
    let mut cleaned: String = s.chars().filter(|c| *c != '$' && !c.is_whitespace()).collect();
    if let Some(stripped) = cleaned.strip_suffix('-') {
        is_negative = !is_negative;
        cleaned = stripped.to_string();
    }

    if cleaned.is_empty() {
        return Err("Invalid amount format".to_string());
    }

    let money = Money::from_str(&cleaned, iso::USD)
        .map_err(|e| format!("Invalid amount format: {:?}", e))?;

    let mut bd = BigDecimal::from_str(&money.amount().to_string())
        .map_err(|e| format!("Invalid amount format: {}", e))?;

    if is_negative {
        bd = -bd;
    }

    Ok(bd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency_basic() {
        assert_eq!(format_currency(&BigDecimal::from(0)), "$0.00");
        assert_eq!(format_currency(&BigDecimal::from(100)), "$100.00");
        assert_eq!(format_currency(&BigDecimal::from(1000)), "$1,000.00");
        assert_eq!(format_currency(&BigDecimal::from(1000000)), "$1,000,000.00");
    }

    #[test]
    fn test_format_currency_decimals() {
        assert_eq!(format_currency(&BigDecimal::from_str("1234.5").unwrap()), "$1,234.50");
        assert_eq!(format_currency(&BigDecimal::from_str("1234.56").unwrap()), "$1,234.56");
        assert_eq!(format_currency(&BigDecimal::from_str("1234.567").unwrap()), "$1,234.57");
        assert_eq!(format_currency(&BigDecimal::from_str("0.5").unwrap()), "$0.50");
        assert_eq!(format_currency(&BigDecimal::from_str("0.05").unwrap()), "$0.05");
    }

    #[test]
    fn test_format_currency_negative() {
        assert_eq!(format_currency(&BigDecimal::from(-100)), "-$100.00");
        assert_eq!(format_currency(&BigDecimal::from_str("-1234.56").unwrap()), "-$1,234.56");
        assert_eq!(format_currency(&BigDecimal::from_str("-0.00").unwrap()), "$0.00");
    }

    #[test]
    fn test_parse_currency_standard() {
        assert_eq!(parse_currency("100").unwrap(), BigDecimal::from(100));
        assert_eq!(parse_currency("100.50").unwrap(), BigDecimal::from_str("100.50").unwrap());
        assert_eq!(parse_currency("1,000.50").unwrap(), BigDecimal::from_str("1000.50").unwrap());
        assert_eq!(parse_currency("1,000,000.00").unwrap(), BigDecimal::from_str("1000000.00").unwrap());
    }

    #[test]
    fn test_parse_currency_with_dollar_and_spaces() {
        assert_eq!(parse_currency("$100").unwrap(), BigDecimal::from(100));
        assert_eq!(parse_currency(" $ 1,234.56 ").unwrap(), BigDecimal::from_str("1234.56").unwrap());
        assert_eq!(parse_currency("$1,000").unwrap(), BigDecimal::from(1000));
        assert_eq!(parse_currency("1,000$").unwrap(), BigDecimal::from(1000));
    }

    #[test]
    fn test_parse_currency_negative() {
        assert_eq!(parse_currency("-100").unwrap(), BigDecimal::from(-100));
        assert_eq!(parse_currency("-$1,234.56").unwrap(), BigDecimal::from_str("-1234.56").unwrap());
        assert_eq!(parse_currency("$-1,234.56").unwrap(), BigDecimal::from_str("-1234.56").unwrap());
        assert_eq!(parse_currency("($1,234.56)").unwrap(), BigDecimal::from_str("-1234.56").unwrap());
        assert_eq!(parse_currency("(1,234.56)").unwrap(), BigDecimal::from_str("-1234.56").unwrap());
        assert_eq!(parse_currency("100-").unwrap(), BigDecimal::from(-100));
    }

    #[test]
    fn test_parse_currency_invalid() {
        assert!(parse_currency("").is_err());
        assert!(parse_currency("   ").is_err());
        assert!(parse_currency("$").is_err());
        assert!(parse_currency("abc").is_err());
        assert!(parse_currency("1.2.3").is_err());
        assert!(parse_currency(".").is_err());
        assert!(parse_currency("$.").is_err());
    }
}
