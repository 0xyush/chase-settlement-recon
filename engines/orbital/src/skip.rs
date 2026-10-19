//! Step 2: decide whether a row is skipped.
//!
//! Conditions inside one skip config must all match (AND). Several skip configs are
//! alternatives (OR). A condition on a column the file does not have never matches, the same
//! permissive choice hyperswitch-recon makes.

use crate::config::{SkipCondition, SkipConfig, SkipOperator};
use crate::dfr::ParsedRow;

/// The reason the row is skipped, or `None` when it is kept.
pub fn match_skip(row: &ParsedRow, skip_configs: &[SkipConfig]) -> Option<String> {
    skip_configs
        .iter()
        .filter(|config| !config.conditions.is_empty())
        .find(|config| config.conditions.iter().all(|c| condition_matches(row, c)))
        .map(|config| {
            config
                .conditions
                .iter()
                .map(describe)
                .collect::<Vec<_>>()
                .join(" AND ")
        })
}

fn condition_matches(row: &ParsedRow, condition: &SkipCondition) -> bool {
    let Some(cell) = row.get(&condition.identifier) else {
        return false;
    };
    let expected = condition.value.as_deref();
    match condition.operator {
        SkipOperator::Equals => cell == expected,
        SkipOperator::NotEquals => cell != expected,
        SkipOperator::StartsWith => both(cell, expected, |c, e| c.starts_with(e)),
        SkipOperator::Contains => both(cell, expected, |c, e| c.contains(e)),
        SkipOperator::EndsWith => both(cell, expected, |c, e| c.ends_with(e)),
    }
}

/// Substring checks only make sense between two real strings; a missing side keeps the row.
fn both(cell: Option<&str>, expected: Option<&str>, test: impl Fn(&str, &str) -> bool) -> bool {
    matches!((cell, expected), (Some(c), Some(e)) if test(c, e))
}

fn describe(condition: &SkipCondition) -> String {
    let value = serde_json::to_string(&condition.value).unwrap_or_default();
    format!(
        "{} {} {value}",
        condition.identifier,
        condition.operator.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn row(cells: &[(&str, Option<&str>)]) -> ParsedRow {
        ParsedRow {
            line: 9,
            fields: cells
                .iter()
                .map(|(k, v)| (k.to_string(), v.map(str::to_string)))
                .collect::<IndexMap<_, _>>(),
        }
    }

    fn config(conditions: &[(&str, SkipOperator, Option<&str>)]) -> SkipConfig {
        SkipConfig {
            conditions: conditions
                .iter()
                .map(|(identifier, operator, value)| SkipCondition {
                    identifier: identifier.to_string(),
                    operator: *operator,
                    value: value.map(str::to_string),
                })
                .collect(),
        }
    }

    #[test]
    fn end_row_is_skipped_with_a_reason() {
        let skips = [config(&[(
            "Submission Date",
            SkipOperator::Equals,
            Some("END"),
        )])];
        assert_eq!(
            match_skip(&row(&[("Submission Date", Some("END"))]), &skips),
            Some(r#"Submission Date equals "END""#.into())
        );
        assert_eq!(
            match_skip(&row(&[("Submission Date", Some("3/3/2025"))]), &skips),
            None
        );
    }

    #[test]
    fn conditions_and_configs_combine() {
        let and = [config(&[
            ("Type", SkipOperator::Equals, Some("AUTH")),
            ("Amount", SkipOperator::StartsWith, Some("-")),
        ])];
        assert!(match_skip(&row(&[("Type", Some("AUTH")), ("Amount", Some("5"))]), &and).is_none());
        let or = [
            config(&[("Type", SkipOperator::Equals, Some("AUTH"))]),
            config(&[("Type", SkipOperator::Equals, Some("END"))]),
        ];
        assert!(match_skip(&row(&[("Type", Some("END"))]), &or).is_some());
    }

    #[test]
    fn missing_column_and_empty_config_never_skip() {
        let skips = [
            config(&[("Nope", SkipOperator::Equals, None)]),
            SkipConfig { conditions: vec![] },
        ];
        assert_eq!(match_skip(&row(&[("Type", None)]), &skips), None);
    }
}
