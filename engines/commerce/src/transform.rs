//! Steps 2 and 3: skip rows, then map and validate each kept row into a staging entry.
//!
//! Mirrors hyperswitch-recon's per-row engine: a bad row is an answer about the input
//! (`Failed` with a reason), never an error that stops the file. Main fields stop at the first
//! problem; metadata fields report every problem at once.

use std::time::Duration;

use web_time::Instant;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::config::{MainFields, MetadataType, Rule, TransformationConfig};
use crate::parser::{self, FileParse, ParsedRow, RowParse};
use crate::rules::{self, RegexCache};
use crate::skip;

pub const ENGINE: &str = "rust";
const ORDER_ID_MAX: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Money coming in: a sale.
    Credit,
    /// Money going out: a refund.
    Debit,
}

/// One recon entry, the shape the recon service stores in its `staging_entry` table:
/// five main fields plus metadata. Amounts are minor units (cents).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagingEntry {
    pub order_id: String,
    pub amount: i64,
    pub currency: String,
    pub effective_at: String,
    pub balance_direction: Direction,
    pub metadata: IndexMap<String, Option<String>>,
    pub exposed_metadata: IndexMap<String, Option<String>>,
    pub searchable_metadata: IndexMap<String, Option<String>>,
    pub source_line: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Outcome {
    Success,
    Skipped,
    Failed,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Skipped => "Skipped",
            Self::Failed => "Failed",
        }
    }
}

/// What happened to one data row, in file order.
#[derive(Debug, Clone, Serialize)]
pub struct RowResult {
    pub line: u64,
    pub outcome: Outcome,
    /// The skip or failure reason. Absent for a staged row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// The staging entry. Present only for a staged row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<StagingEntry>,
    /// The parsed row as it came out of the file.
    pub row: ParsedRow,
}

impl RowResult {
    fn not_staged(row: ParsedRow, outcome: Outcome, details: String) -> Self {
        Self {
            line: row.line,
            outcome,
            details: Some(details),
            entry: None,
            row,
        }
    }
}

/// The result of one file. Same JSON shape the dashboard's JavaScript engine returns.
#[derive(Debug, Serialize)]
pub struct TransformOutput {
    pub engine: &'static str,
    /// Set when the whole file could not be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Vec<String>>,
    /// Only when the config maps no fields: the parsed rows as they are.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<ParsedRow>>,
    /// One result per data row, in file order. `None` when the config maps no fields.
    pub results: Option<Vec<RowResult>>,
    /// How long each step took. Reported through `steps`, not serialized here.
    #[serde(skip)]
    pub timings: Timings,
}

/// Time spent in each step, in nanoseconds. Skip and map run row by row, so their times are
/// summed over the rows.
#[derive(Debug, Clone, Copy, Default)]
pub struct Timings {
    pub read_ns: u64,
    pub skip_ns: u64,
    pub map_ns: u64,
}

fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

impl TransformOutput {
    pub fn rejected(reason: String) -> Self {
        Self {
            engine: ENGINE,
            rejected: Some(reason),
            header: None,
            rows: None,
            results: None,
            timings: Timings::default(),
        }
    }

    pub fn count(&self, outcome: Outcome) -> usize {
        self.results
            .iter()
            .flatten()
            .filter(|r| r.outcome == outcome)
            .count()
    }

    pub fn staging_entries(&self) -> impl Iterator<Item = &StagingEntry> {
        self.results
            .iter()
            .flatten()
            .filter_map(|r| r.entry.as_ref())
    }
}

/// The whole pipeline for one file: read it, then skip, map and validate every row.
pub fn transform(input: &[u8], config: &TransformationConfig) -> TransformOutput {
    let parsing = &config.parsing_config;
    let clock = Instant::now();
    let parsed = parser::parse_report(
        input,
        parsing.header_row_index,
        parsing.footer_marker.as_deref(),
    );
    let mut timings = Timings {
        read_ns: nanos(clock.elapsed()),
        ..Timings::default()
    };
    let (header, rows) = match parsed {
        FileParse::Rejected { reason } => {
            let mut output = TransformOutput::rejected(reason);
            output.timings = timings;
            return output;
        }
        FileParse::Parsed { header, rows } => (header, rows),
    };

    let Some(fields) = &config.fields else {
        let rows = rows
            .into_iter()
            .filter_map(|row| match row {
                RowParse::Parsed(row) => Some(row),
                RowParse::Rejected { .. } => None,
            })
            .collect();
        return TransformOutput {
            engine: ENGINE,
            rejected: None,
            header: Some(header),
            rows: Some(rows),
            results: None,
            timings,
        };
    };

    let mut regexes = RegexCache::default();
    let results = rows
        .into_iter()
        .map(|row| match row {
            RowParse::Parsed(row) => transform_row(row, config, fields, &mut regexes, &mut timings),
            RowParse::Rejected { line, message } => RowResult::not_staged(
                ParsedRow {
                    line,
                    fields: IndexMap::new(),
                },
                Outcome::Failed,
                message,
            ),
        })
        .collect();

    TransformOutput {
        engine: ENGINE,
        rejected: None,
        header: Some(header),
        rows: None,
        results: Some(results),
        timings,
    }
}

fn transform_row(
    row: ParsedRow,
    config: &TransformationConfig,
    fields: &MainFields,
    regexes: &mut RegexCache,
    timings: &mut Timings,
) -> RowResult {
    let clock = Instant::now();
    let skipped = skip::match_skip(&row, &config.skip_configs);
    timings.skip_ns += nanos(clock.elapsed());
    if let Some(reason) = skipped {
        return RowResult::not_staged(row, Outcome::Skipped, format!("skip: {reason}"));
    }
    let clock = Instant::now();
    let mapped = map_row(&row, config, fields, regexes);
    timings.map_ns += nanos(clock.elapsed());
    match mapped {
        Ok(entry) => RowResult {
            line: row.line,
            outcome: Outcome::Success,
            details: None,
            entry: Some(entry),
            row,
        },
        Err(details) => RowResult::not_staged(row, Outcome::Failed, details),
    }
}

/// Column mapping + validation. Main fields in the recon engine's order:
/// currency, amount, balance_direction, effective_at, order_id.
fn map_row(
    row: &ParsedRow,
    config: &TransformationConfig,
    f: &MainFields,
    regexes: &mut RegexCache,
) -> Result<StagingEntry, String> {
    let label = |field: &'static str| move |message: String| format!("{field}: {message}");

    let currency = read_main(
        row,
        &f.currency.identifier,
        &f.currency.transformation_rules,
        regexes,
    )
    .map_err(label("currency"))?;
    if !(currency.len() == 3 && currency.bytes().all(|b| b.is_ascii_uppercase())) {
        return Err(format!("currency: InvalidCurrency: \"{currency}\""));
    }

    let amount = read_main(
        row,
        &f.amount.identifier,
        &f.amount.transformation_rules,
        regexes,
    )
    .and_then(|raw| rules::major_to_minor(&raw))
    .map_err(label("amount"))?;

    let direction = &f.balance_direction;
    let raw_direction = read_main(
        row,
        &direction.identifier,
        &direction.transformation_rules,
        regexes,
    )
    .map_err(label("balance_direction"))?;
    let balance_direction = if direction.credit_values.contains(&raw_direction) {
        Direction::Credit
    } else if direction.debit_values.contains(&raw_direction) {
        Direction::Debit
    } else {
        return Err(format!(
            "balance_direction: InvalidBalanceDirection: \"{raw_direction}\""
        ));
    };

    let effective_at = read_main(
        row,
        &f.effective_at.identifier,
        &f.effective_at.transformation_rules,
        regexes,
    )
    .and_then(|raw| rules::parse_date(&raw, &f.effective_at.date_format))
    .map_err(label("effective_at"))?;

    let order_id = read_main(
        row,
        &f.order_id.identifier,
        &f.order_id.transformation_rules,
        regexes,
    )
    .map_err(label("order_id"))?;
    let order_id_length = order_id.chars().count();
    if order_id_length > ORDER_ID_MAX {
        return Err(format!(
            "order_id: ValueTooLong: {order_id_length} > {ORDER_ID_MAX}"
        ));
    }

    // Metadata: collect every problem instead of stopping at the first.
    let mut metadata = IndexMap::new();
    let mut exposed_metadata = IndexMap::new();
    let mut searchable_metadata = IndexMap::new();
    let mut errors = Vec::new();
    for field in &config.metadata_fields {
        let name = &field.field_name;
        let Some(raw) = row.get(&field.identifier) else {
            if field.required {
                errors.push(format!(
                    "{name}: column \"{}\" not in file",
                    field.identifier
                ));
            }
            continue;
        };
        let value = match rules::apply_rules(raw, &field.transformation_rules, regexes) {
            Ok(value) => value,
            Err(error) => {
                errors.push(format!("{name}: {error}"));
                continue;
            }
        };
        if field.required && value.as_deref().is_none_or(str::is_empty) {
            errors.push(format!("{name}: RequiredFieldMissing"));
            continue;
        }
        if let (Some(v), MetadataType::MajorUnit) = (&value, field.value_type)
            && !rules::is_decimal(v)
        {
            errors.push(format!("{name}: InvalidType \"{v}\""));
            continue;
        }
        if field.exposed {
            exposed_metadata.insert(name.clone(), value.clone());
        }
        if field.searchable {
            searchable_metadata.insert(name.clone(), value.clone());
        }
        metadata.insert(name.clone(), value);
    }
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }

    Ok(StagingEntry {
        order_id,
        amount,
        currency,
        effective_at,
        balance_direction,
        metadata,
        exposed_metadata,
        searchable_metadata,
        source_line: row.line,
    })
}

/// Reads one main field: the column must exist, the rules must pass, and the value must not be empty.
fn read_main(
    row: &ParsedRow,
    identifier: &str,
    rules: &[Rule],
    regexes: &mut RegexCache,
) -> Result<String, String> {
    let raw = row
        .get(identifier)
        .ok_or_else(|| format!("column \"{identifier}\" not in file"))?;
    match rules::apply_rules(raw, rules, regexes)? {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err("RequiredFieldMissing".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"{
        "parsing_config": {"header_row_index": 2},
        "skip_configs": [{"conditions": [{"identifier": "Date", "operator": "equals", "value": "END"}]}],
        "fields": {
            "currency": {"identifier": "Ccy", "transformation_rules": [{"type": "trim"}]},
            "amount": {"identifier": "Amount", "transformation_rules": [{"type": "trim"}, {"type": "absolute"}]},
            "balance_direction": {"identifier": "Action", "credit_values": ["DP"], "debit_values": ["RF"]},
            "effective_at": {"identifier": "Date", "date_format": {"order": "month_day_year", "delimiter": "slash"}},
            "order_id": {"identifier": "Order"}
        },
        "metadata_fields": [
            {"identifier": "Card", "field_name": "card_brand", "required": true, "exposed": true},
            {"identifier": "Fee", "field_name": "fee", "type": "major_unit"}
        ]
    }"#;

    const FILE: &str = "BEGIN,x\nmeta\n\
        Date,Order,Action,Amount,Ccy,Card,Fee\n\
        3/3/2025,ord_1,DP,20,USD,VI,-0.46\n\
        3/3/2025,ord_1,RF,-20,USD,VI,0.41\n\
        3/3/2025,ord_2,XX,5,USD,MC,0\n\
        3/3/2025,ord_3,DP,1.234,USD,,abc\n\
        3/3/2025,ord_4,DP,5,USD,,abc\n\
        END,x\n";

    fn run() -> Vec<RowResult> {
        let config = TransformationConfig::from_json(CONFIG).unwrap();
        transform(FILE.as_bytes(), &config).results.unwrap()
    }

    #[test]
    fn sale_and_refund_are_staged_with_direction_and_cents() {
        let results = run();
        let sale = results[0].entry.as_ref().unwrap();
        assert_eq!(
            (sale.amount, sale.balance_direction),
            (2000, Direction::Credit)
        );
        assert_eq!(sale.effective_at, "2025-03-03 00:00:00");
        assert_eq!(
            sale.metadata.get("card_brand"),
            Some(&Some("VI".to_string()))
        );
        assert_eq!(sale.exposed_metadata.len(), 1);
        let refund = results[1].entry.as_ref().unwrap();
        assert_eq!(
            (refund.amount, refund.balance_direction),
            (2000, Direction::Debit)
        );
    }

    #[test]
    fn failures_say_which_field_and_why() {
        let results = run();
        assert_eq!(results[2].outcome, Outcome::Failed);
        assert_eq!(
            results[2].details.as_deref(),
            Some(r#"balance_direction: InvalidBalanceDirection: "XX""#)
        );
        // A main-field failure stops there.
        assert_eq!(
            results[3].details.as_deref(),
            Some(r#"amount: PrecisionLoss: "1.234" has more than 2 decimals"#)
        );
        // Metadata failures are all reported together.
        assert_eq!(
            results[4].details.as_deref(),
            Some(r#"card_brand: RequiredFieldMissing; fee: InvalidType "abc""#)
        );
    }

    #[test]
    fn end_row_is_skipped_and_order_is_kept() {
        let results = run();
        let last = results.last().unwrap();
        assert_eq!(last.outcome, Outcome::Skipped);
        let lines: Vec<u64> = results.iter().map(|r| r.line).collect();
        assert_eq!(lines, [4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn no_mapping_returns_raw_rows() {
        let config =
            TransformationConfig::from_json(r#"{"parsing_config": {"header_row_index": 2}}"#)
                .unwrap();
        let output = transform(FILE.as_bytes(), &config);
        assert!(output.results.is_none());
        assert_eq!(output.rows.unwrap().len(), 6);
    }
}
