//! The small steps between a raw cell and a field: transformation rules, amounts, dates.

use std::collections::HashMap;

use regex::Regex;

use crate::config::{DateFormat, DateOrder, Rule};

/// Compiles each regex pattern once per file instead of once per row.
#[derive(Default)]
pub struct RegexCache(HashMap<String, Regex>);

impl RegexCache {
    fn get(&mut self, pattern: &str) -> Result<&Regex, String> {
        if !self.0.contains_key(pattern) {
            let compiled = Regex::new(pattern)
                .map_err(|error| format!("RegexCompileFailed: {pattern}: {error}"))?;
            self.0.insert(pattern.to_string(), compiled);
        }
        self.0
            .get(pattern)
            .ok_or_else(|| format!("RegexCompileFailed: {pattern}"))
    }
}

/// Runs the rules on the raw cell, in order. Like hyperswitch-recon, rules run before any
/// parsing, and every rule except `default_value` leaves a missing cell (`None`) alone.
pub fn apply_rules(
    value: Option<&str>,
    rules: &[Rule],
    regexes: &mut RegexCache,
) -> Result<Option<String>, String> {
    rules
        .iter()
        .try_fold(value.map(str::to_string), |current, rule| {
            let Some(value) = current else {
                return Ok(match rule {
                    Rule::DefaultValue { value } => Some(value.clone()),
                    _ => None,
                });
            };
            let next = match rule {
                Rule::Trim => value.trim().to_string(),
                Rule::Absolute => value
                    .strip_prefix('-')
                    .map_or(value.clone(), str::to_string),
                Rule::Negate if value.is_empty() => value,
                Rule::Negate => value
                    .strip_prefix('-')
                    .map_or_else(|| format!("-{value}"), str::to_string),
                Rule::StartsWith {
                    prefix,
                    then,
                    otherwise,
                } => {
                    if value.starts_with(prefix.as_str()) {
                        then.clone()
                    } else {
                        otherwise.clone()
                    }
                }
                Rule::Regex { pattern, group } => {
                    let captures = regexes.get(pattern)?.captures(&value).ok_or_else(|| {
                        format!("UnmatchedRegex: \"{value}\" does not match {pattern}")
                    })?;
                    return Ok(captures
                        .get(group.unwrap_or(0))
                        .map(|m| m.as_str().to_string()));
                }
                Rule::DefaultValue { .. } => value,
            };
            Ok(Some(next))
        })
}

/// A decimal amount in major units to minor units: "20" -> 2000, "-0.46" -> -46.
/// Integer maths only, never floats, like `MinorUnit` in the recon repo.
pub fn major_to_minor(raw: &str) -> Result<i64, String> {
    let invalid = || format!("InvalidType: \"{raw}\" is not a number");
    let (negative, unsigned) = match raw.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, raw),
    };
    let (whole, fraction) = match unsigned.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (unsigned, None),
    };
    if !is_digits(whole) || fraction.is_some_and(|f| !is_digits(f)) {
        return Err(invalid());
    }
    let fraction = fraction.unwrap_or_default();
    if fraction.len() > 2 {
        return Err(format!("PrecisionLoss: \"{raw}\" has more than 2 decimals"));
    }
    let whole: i64 = whole.parse().map_err(|_| invalid())?;
    let cents: i64 = format!("{fraction:0<2}").parse().map_err(|_| invalid())?;
    let minor = whole
        .checked_mul(100)
        .and_then(|w| w.checked_add(cents))
        .ok_or_else(invalid)?;
    Ok(if negative { -minor } else { minor })
}

/// True for a plain decimal such as "12", "-0.46" or "3.00".
pub fn is_decimal(raw: &str) -> bool {
    let unsigned = raw.strip_prefix('-').unwrap_or(raw);
    match unsigned.split_once('.') {
        Some((whole, fraction)) => is_digits(whole) && is_digits(fraction),
        None => is_digits(unsigned),
    }
}

fn is_digits(part: &str) -> bool {
    !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())
}

/// "3/3/2025" with month_day_year + slash -> "2025-03-03 00:00:00". No leading zeros needed.
pub fn parse_date(raw: &str, format: &DateFormat) -> Result<String, String> {
    let wrong_shape = || format!("InvalidFormat: \"{raw}\" is not {}", format.pattern());
    let parts: Vec<&str> = raw.split(format.delimiter.as_char()).collect();
    let [first, second, third] = parts.as_slice() else {
        return Err(wrong_shape());
    };
    let (year, month, day) = match format.order {
        DateOrder::MonthDayYear => (third, first, second),
        DateOrder::DayMonthYear => (third, second, first),
        DateOrder::YearMonthDay => (first, second, third),
    };
    let fits =
        |part: &str, min: usize, max: usize| (min..=max).contains(&part.len()) && is_digits(part);
    if !fits(year, 4, 4) || !fits(month, 1, 2) || !fits(day, 1, 2) {
        return Err(wrong_shape());
    }
    let (year, month, day): (u32, u32, u32) = (
        year.parse().map_err(|_| wrong_shape())?,
        month.parse().map_err(|_| wrong_shape())?,
        day.parse().map_err(|_| wrong_shape())?,
    );
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
        return Err(format!("InvalidFormat: \"{raw}\" is not a real date"));
    }
    Ok(format!("{year:04}-{month:02}-{day:02} 00:00:00"))
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Delimiter;

    fn run(value: Option<&str>, rules: &[Rule]) -> Result<Option<String>, String> {
        apply_rules(value, rules, &mut RegexCache::default())
    }

    #[test]
    fn rules_run_in_order() {
        assert_eq!(
            run(Some(" -20 "), &[Rule::Trim, Rule::Absolute]),
            Ok(Some("20".into()))
        );
        assert_eq!(run(Some("20"), &[Rule::Negate]), Ok(Some("-20".into())));
        let sign = Rule::StartsWith {
            prefix: "-".into(),
            then: "debit".into(),
            otherwise: "credit".into(),
        };
        assert_eq!(
            run(Some("-5"), std::slice::from_ref(&sign)),
            Ok(Some("debit".into()))
        );
        assert_eq!(run(Some("5"), &[sign]), Ok(Some("credit".into())));
    }

    #[test]
    fn missing_cell_only_takes_a_default() {
        assert_eq!(run(None, &[Rule::Trim, Rule::Absolute]), Ok(None));
        let default = Rule::DefaultValue {
            value: "USD".into(),
        };
        assert_eq!(
            run(None, std::slice::from_ref(&default)),
            Ok(Some("USD".into()))
        );
        assert_eq!(run(Some(""), &[default]), Ok(Some(String::new())));
    }

    #[test]
    fn regex_keeps_a_group_or_fails_loudly() {
        let last4 = Rule::Regex {
            pattern: r"(\d{4})$".into(),
            group: Some(1),
        };
        assert_eq!(
            run(Some("476134XXXXXX0050"), std::slice::from_ref(&last4)),
            Ok(Some("0050".into()))
        );
        assert_eq!(
            run(Some("IBAN"), &[last4]),
            Err(r#"UnmatchedRegex: "IBAN" does not match (\d{4})$"#.into())
        );
    }

    #[test]
    fn amounts_become_minor_units() {
        assert_eq!(major_to_minor("20"), Ok(2000));
        assert_eq!(major_to_minor("1.06"), Ok(106));
        assert_eq!(major_to_minor("20.5"), Ok(2050));
        assert_eq!(major_to_minor("-0.46"), Ok(-46));
        assert!(
            major_to_minor("1.234")
                .unwrap_err()
                .starts_with("PrecisionLoss")
        );
        assert!(
            major_to_minor("12a")
                .unwrap_err()
                .starts_with("InvalidType")
        );
        assert!(major_to_minor("5.").unwrap_err().starts_with("InvalidType"));
        assert!(
            major_to_minor("99999999999999999999")
                .unwrap_err()
                .starts_with("InvalidType")
        );
    }

    #[test]
    fn dates_parse_without_padding_and_reject_impossible_days() {
        let mdy = DateFormat {
            order: DateOrder::MonthDayYear,
            delimiter: Delimiter::Slash,
        };
        assert_eq!(
            parse_date("3/3/2025", &mdy),
            Ok("2025-03-03 00:00:00".into())
        );
        assert_eq!(
            parse_date("2/29/2024", &mdy),
            Ok("2024-02-29 00:00:00".into())
        );
        assert!(
            parse_date("2/29/2025", &mdy)
                .unwrap_err()
                .contains("not a real date")
        );
        assert_eq!(
            parse_date("2025-03-03", &mdy),
            Err(r#"InvalidFormat: "2025-03-03" is not M/D/YYYY"#.into())
        );
    }
}
