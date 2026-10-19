//! Step 5: match the staged Chase entries against a Hyperswitch export.
//!
//! Match key = order id + direction: a Hyperswitch payment pairs with a Chase credit, a refund
//! with a Chase debit. Inside one key, equal amounts pair first; anything left over is a
//! mismatch, or missing on one side. Hyperswitch rows whose status will never settle are set
//! aside as `ignored`.
//!
//! The Hyperswitch export here is a simple CSV:
//! `payment_id,entry_type,amount,currency,status,connector,created_at`, amounts in minor units.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::transform::{Direction, StagingEntry};

/// Hyperswitch statuses that end in money at Chase.
const SETTLING_STATUSES: [&str; 3] = ["succeeded", "partially_captured", "success"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    Payment,
    Refund,
}

impl EntryType {
    fn direction(self) -> Direction {
        match self {
            Self::Payment => Direction::Credit,
            Self::Refund => Direction::Debit,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyperswitchRecord {
    pub payment_id: String,
    pub entry_type: EntryType,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    #[serde(default)]
    pub connector: String,
    #[serde(default)]
    pub created_at: String,
}

/// Declared in the order results are listed: problems first, matches last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcome {
    AmountMismatch,
    CurrencyMismatch,
    MissingInHyperswitch,
    MissingInChase,
    Ignored,
    Matched,
}

impl MatchOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AmountMismatch => "amount_mismatch",
            Self::CurrencyMismatch => "currency_mismatch",
            Self::MissingInHyperswitch => "missing_in_hyperswitch",
            Self::MissingInChase => "missing_in_chase",
            Self::Ignored => "ignored",
            Self::Matched => "matched",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchItem {
    pub outcome: MatchOutcome,
    pub order_id: String,
    pub direction: Direction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chase_amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperswitch_amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chase_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperswitch_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chase_line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperswitch_line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MatchSummary {
    pub chase_entries: usize,
    pub hyperswitch_records: usize,
    pub matched: usize,
    pub amount_mismatch: usize,
    pub currency_mismatch: usize,
    pub missing_in_hyperswitch: usize,
    pub missing_in_chase: usize,
    pub ignored: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchOutput {
    pub summary: MatchSummary,
    pub items: Vec<MatchItem>,
}

/// Reads a Hyperswitch export CSV (header on line 1). Returns each record with its file line.
pub fn parse_hyperswitch_export(input: &[u8]) -> Result<Vec<(u64, HyperswitchRecord)>, String> {
    let input = input.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(input);
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(input);
    let headers = reader
        .headers()
        .map_err(|error| format!("Hyperswitch export: {error}"))?
        .clone();
    reader
        .records()
        .map(|record| {
            let record = record.map_err(|error| format!("Hyperswitch export: {error}"))?;
            let line = record.position().map_or(0, csv::Position::line);
            let parsed = record
                .deserialize(Some(&headers))
                .map_err(|error| format!("Hyperswitch export line {line}: {error}"))?;
            Ok((line, parsed))
        })
        .collect()
}

pub fn match_entries(
    entries: &[StagingEntry],
    hyperswitch: &[(u64, HyperswitchRecord)],
) -> MatchOutput {
    type Key = (String, Direction);
    let mut chase: BTreeMap<Key, Vec<&StagingEntry>> = BTreeMap::new();
    for entry in entries {
        chase
            .entry((entry.order_id.clone(), entry.balance_direction))
            .or_default()
            .push(entry);
    }

    let mut items = Vec::new();
    let mut settling: BTreeMap<Key, Vec<(u64, &HyperswitchRecord)>> = BTreeMap::new();
    for (line, record) in hyperswitch {
        let key = (record.payment_id.clone(), record.entry_type.direction());
        if SETTLING_STATUSES.contains(&record.status.as_str()) {
            settling.entry(key).or_default().push((*line, record));
        } else {
            let mut item = item(MatchOutcome::Ignored, &key, None, Some((*line, record)));
            item.note = Some(format!("status {} never settles at Chase", record.status));
            items.push(item);
        }
    }

    let keys: BTreeSet<&Key> = chase.keys().chain(settling.keys()).collect();
    for key in keys {
        let chase_side = chase.get(key).cloned().unwrap_or_default();
        let mut hyperswitch_side = settling.get(key).cloned().unwrap_or_default();

        // 1. Same amount and currency on both sides: matched.
        let mut unpaired = Vec::new();
        for entry in chase_side {
            let same = hyperswitch_side
                .iter()
                .position(|(_, r)| r.amount == entry.amount && r.currency == entry.currency);
            match same {
                Some(index) => {
                    let pair = hyperswitch_side.remove(index);
                    items.push(item(MatchOutcome::Matched, key, Some(entry), Some(pair)));
                }
                None => unpaired.push(entry),
            }
        }
        // 2. Same key, different amount or currency: a mismatch. 3. Leftovers: missing.
        let mut leftovers = hyperswitch_side.into_iter();
        for entry in unpaired {
            match leftovers.next() {
                Some(pair) => {
                    let outcome = if pair.1.currency == entry.currency {
                        MatchOutcome::AmountMismatch
                    } else {
                        MatchOutcome::CurrencyMismatch
                    };
                    items.push(item(outcome, key, Some(entry), Some(pair)));
                }
                None => items.push(item(
                    MatchOutcome::MissingInHyperswitch,
                    key,
                    Some(entry),
                    None,
                )),
            }
        }
        items.extend(
            leftovers.map(|pair| item(MatchOutcome::MissingInChase, key, None, Some(pair))),
        );
    }

    items.sort_by_key(|i| {
        (
            i.outcome,
            i.chase_line.unwrap_or(u64::MAX),
            i.hyperswitch_line.unwrap_or(u64::MAX),
        )
    });
    let count = |outcome| items.iter().filter(|i| i.outcome == outcome).count();
    let summary = MatchSummary {
        chase_entries: entries.len(),
        hyperswitch_records: hyperswitch.len(),
        matched: count(MatchOutcome::Matched),
        amount_mismatch: count(MatchOutcome::AmountMismatch),
        currency_mismatch: count(MatchOutcome::CurrencyMismatch),
        missing_in_hyperswitch: count(MatchOutcome::MissingInHyperswitch),
        missing_in_chase: count(MatchOutcome::MissingInChase),
        ignored: count(MatchOutcome::Ignored),
    };
    MatchOutput { summary, items }
}

fn item(
    outcome: MatchOutcome,
    key: &(String, Direction),
    chase: Option<&StagingEntry>,
    hyperswitch: Option<(u64, &HyperswitchRecord)>,
) -> MatchItem {
    MatchItem {
        outcome,
        order_id: key.0.clone(),
        direction: key.1,
        chase_amount: chase.map(|e| e.amount),
        hyperswitch_amount: hyperswitch.map(|(_, r)| r.amount),
        chase_currency: chase.map(|e| e.currency.clone()),
        hyperswitch_currency: hyperswitch.map(|(_, r)| r.currency.clone()),
        chase_line: chase.map(|e| e.source_line),
        hyperswitch_line: hyperswitch.map(|(line, _)| line),
        note: None,
    }
}

/// The match result as CSV, one line per item.
pub fn match_report_csv(output: &MatchOutput) -> String {
    let quote = |value: &str| format!("\"{}\"", value.replace('"', "\"\""));
    let text = |value: Option<String>| value.unwrap_or_default();
    let number = |value: Option<i64>| value.map(|v| v.to_string()).unwrap_or_default();
    let line = |value: Option<u64>| value.map(|v| v.to_string()).unwrap_or_default();
    let rows = output.items.iter().map(|i| {
        let direction = match i.direction {
            Direction::Credit => "credit",
            Direction::Debit => "debit",
        };
        [
            i.outcome.as_str().to_string(),
            i.order_id.clone(),
            direction.to_string(),
            number(i.chase_amount),
            number(i.hyperswitch_amount),
            text(i.chase_currency.clone()),
            text(i.hyperswitch_currency.clone()),
            line(i.chase_line),
            line(i.hyperswitch_line),
            text(i.note.clone()),
        ]
        .map(|v| quote(&v))
        .join(",")
    });
    std::iter::once(
        "outcome,order_id,direction,chase_amount_minor,hyperswitch_amount_minor,chase_currency,hyperswitch_currency,chase_line,hyperswitch_line,note"
            .to_string(),
    )
    .chain(rows)
    .collect::<Vec<_>>()
    .join("\n")
}

/// A pretend Hyperswitch export built from staged Chase entries, with differences planted on
/// purpose so the matching has something to find. Deterministic: the same input always gives
/// the same file.
///
/// - every 97th entry (from the 14th) is left out: shows as missing in Hyperswitch
/// - every 89th entry (from the 22nd) is 1.00 higher: shows as an amount mismatch
/// - one payment and one refund only Hyperswitch knows about: missing in Chase
/// - one failed payment: ignored, it never settles
///
/// It assumes Hyperswitch sends its payment id to Chase as the order number. For the merchant
/// that is the open question: its ids are 30 characters and Chase keeps 22.
pub fn mock_hyperswitch_export(entries: &[StagingEntry]) -> String {
    let currency = entries.first().map_or("USD", |e| e.currency.as_str());
    let created = entries
        .first()
        .map_or_else(String::new, |e| iso(&e.effective_at));
    let planted = [
        ("hs_only_payment_0001", "payment", 2500, "succeeded"),
        ("hs_only_refund_0001", "refund", 500, "success"),
        ("hs_failed_payment_0001", "payment", 1200, "failed"),
    ];
    let rows = entries
        .iter()
        .enumerate()
        .filter(|(index, _)| index % 97 != 13)
        .map(|(index, entry)| {
            let amount = if index % 89 == 21 {
                entry.amount + 100
            } else {
                entry.amount
            };
            let (entry_type, status) = match entry.balance_direction {
                Direction::Credit => ("payment", "succeeded"),
                Direction::Debit => ("refund", "success"),
            };
            format!(
                "{},{entry_type},{amount},{},{status},jpmorgan,{}",
                csv_field(&entry.order_id),
                entry.currency,
                iso(&entry.effective_at)
            )
        })
        .chain(planted.iter().map(|(id, entry_type, amount, status)| {
            format!("{id},{entry_type},{amount},{currency},{status},jpmorgan,{created}")
        }));
    std::iter::once("payment_id,entry_type,amount,currency,status,connector,created_at".to_string())
        .chain(rows)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn iso(effective_at: &str) -> String {
    format!("{}Z", effective_at.replacen(' ', "T", 1))
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    fn entry(order_id: &str, amount: i64, direction: Direction, line: u64) -> StagingEntry {
        StagingEntry {
            order_id: order_id.into(),
            amount,
            currency: "USD".into(),
            effective_at: "2025-03-03 00:00:00".into(),
            balance_direction: direction,
            metadata: IndexMap::new(),
            exposed_metadata: IndexMap::new(),
            searchable_metadata: IndexMap::new(),
            source_line: line,
        }
    }

    #[test]
    fn every_outcome_is_found() {
        let entries = [
            entry("a", 2000, Direction::Credit, 4),
            entry("a", 2000, Direction::Debit, 5),
            entry("b", 500, Direction::Credit, 6),
            entry("c", 700, Direction::Credit, 7),
        ];
        let export = "payment_id,entry_type,amount,currency,status,connector,created_at\n\
            a,payment,2000,USD,succeeded,jpmorgan,\n\
            a,refund,2000,USD,success,jpmorgan,\n\
            b,payment,600,USD,succeeded,jpmorgan,\n\
            d,payment,900,USD,succeeded,jpmorgan,\n\
            e,payment,100,USD,failed,jpmorgan,\n";
        let hyperswitch = parse_hyperswitch_export(export.as_bytes()).unwrap();
        let output = match_entries(&entries, &hyperswitch);
        assert_eq!(
            output.summary,
            MatchSummary {
                chase_entries: 4,
                hyperswitch_records: 5,
                matched: 2,
                amount_mismatch: 1,
                currency_mismatch: 0,
                missing_in_hyperswitch: 1,
                missing_in_chase: 1,
                ignored: 1,
            }
        );
        assert_eq!(
            output.items[0].outcome,
            MatchOutcome::AmountMismatch,
            "problems are listed first"
        );
        assert_eq!(output.items.last().unwrap().outcome, MatchOutcome::Matched);
    }

    #[test]
    fn duplicate_keys_pair_by_amount() {
        let entries = [
            entry("a", 100, Direction::Credit, 4),
            entry("a", 300, Direction::Credit, 5),
        ];
        let export = "payment_id,entry_type,amount,currency,status\n\
            a,payment,300,USD,succeeded\na,payment,100,USD,succeeded\n";
        let output = match_entries(
            &entries,
            &parse_hyperswitch_export(export.as_bytes()).unwrap(),
        );
        assert_eq!(output.summary.matched, 2);
    }

    #[test]
    fn mock_export_round_trips_with_its_planted_differences() {
        let entries: Vec<StagingEntry> = (0..200)
            .map(|i| {
                entry(
                    &format!("ord_{i}"),
                    1000 + i,
                    Direction::Credit,
                    4 + i as u64,
                )
            })
            .collect();
        let export = mock_hyperswitch_export(&entries);
        let output = match_entries(
            &entries,
            &parse_hyperswitch_export(export.as_bytes()).unwrap(),
        );
        let s = &output.summary;
        assert_eq!(s.missing_in_hyperswitch, 2, "entries 13 and 110 left out");
        assert_eq!(
            s.amount_mismatch, 2,
            "entries 21 and 199 changed; 110 was left out"
        );
        assert_eq!(s.missing_in_chase, 2);
        assert_eq!(s.ignored, 1);
        assert_eq!(s.matched, 200 - 2 - 2);
    }
}
