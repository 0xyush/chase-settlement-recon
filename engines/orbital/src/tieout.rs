//! Step 6 (Orbital only): does the detail add up to Chase's own summary?
//!
//! A daily B file carries two views of the same deposits:
//!
//! - FIN0010, Deposit Activity Summary: one line per card brand and category (SALE, REF, fees,
//!   chargebacks), with a count and an amount;
//! - ACT0010, Deposit Detail: one line per transaction, which is what we stage.
//!
//! Adding up the staged ACT0010 entries by currency, card brand and sale/refund must give the
//! FIN0010 SALE and REF lines. When it does, nothing was lost between Chase's file and our
//! staging. When it does not, the line that differs says where to look.
//!
//! FIN0010 also marks every line Settled (`S`: part of Chase's deposit to the merchant's bank) or
//! Conveyed (`C`: listed for information; the card network pays the merchant directly). Conveyed
//! is how Amex works for a merchant with its own Amex contract, which may be the case here, so this
//! is where the "settled or conveyed" question gets its answer from a real file.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::TransformationConfig;
use crate::config::ParsingConfig;
use crate::dfr::{self, FileRead, RowParse};
use crate::rules;
use crate::transform::{Direction, TransformOutput};

/// FIN0010 field names by position (DFR spec v2.2). Fields 12 and 13 only appear for
/// cross-currency merchants.
pub const FIN0010_COLUMNS: [&str; 13] = [
    "Record Type",
    "Entity Type",
    "Entity #",
    "Funds Transfer Instruction #",
    "Secure BA #",
    "Presentment Currency",
    "MOP",
    "Category",
    "Settled/Conveyed",
    "Count",
    "Amount",
    "Settlement Currency",
    "Foreign Exchange Rate",
];

/// The staged metadata field that holds the card brand (DFR's MOP code).
const CARD_BRAND: &str = "card_brand";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TieStatus {
    Tied,
    Differs,
    /// Chase's summary has the line, nothing staged adds up to it.
    SummaryOnly,
    /// Staged entries with no summary line.
    DetailOnly,
}

/// One SALE or REF line: Chase's summary next to the sum of what we staged. Amounts are minor
/// units, without sign; the category says which way the money went.
#[derive(Debug, Clone, Serialize)]
pub struct TieLine {
    pub currency: String,
    pub card_brand: String,
    pub category: String,
    /// `S`, `C`, or both when the summary splits the line. `None` when only detail exists.
    pub settled_conveyed: Option<String>,
    pub summary_count: Option<i64>,
    pub summary_amount: Option<i64>,
    pub detail_count: Option<i64>,
    pub detail_amount: Option<i64>,
    pub status: TieStatus,
}

/// A FIN0010 line with no transaction detail behind it in ACT0010: fees, interchange,
/// chargebacks, adjustments. Signed as in the file: negative is money out.
#[derive(Debug, Clone, Serialize)]
pub struct OtherLine {
    pub currency: String,
    pub category: String,
    pub name: &'static str,
    pub settled_conveyed: Option<String>,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TieOut {
    /// Every SALE and REF line ties.
    pub tied: bool,
    pub lines: Vec<TieLine>,
    pub other: Vec<OtherLine>,
    /// Per currency: what Chase's deposit to the bank should be (all `S` lines, refunds and fees
    /// subtracted).
    pub settled_net: BTreeMap<String, i64>,
    /// Per currency: sales minus refunds on `C` lines, paid by the network, not by Chase.
    pub conveyed_net: BTreeMap<String, i64>,
    /// FIN0010 lines that could not be read, with the reason.
    pub problems: Vec<String>,
}

/// Runs only for an ACT0010 config on a file that also has a FIN0010 section.
pub fn tie_out(
    input: &[u8],
    config: &TransformationConfig,
    output: &TransformOutput,
) -> Option<TieOut> {
    if config.parsing_config.section != "ACT0010" {
        return None;
    }
    output.file.as_ref()?.section("FIN0010")?;
    let results = output.results.as_ref()?;

    let parsing = ParsingConfig {
        section: "FIN0010".to_string(),
        record_type: "RFIN0010".to_string(),
        columns: FIN0010_COLUMNS.map(String::from).to_vec(),
        delimiter: None,
    };
    let FileRead::Parsed { rows, .. } = dfr::read(input, &parsing) else {
        return None;
    };

    type Key = (String, String, String); // currency, card brand, SALE | REF
    let mut summary: BTreeMap<Key, (i64, i64, Vec<String>)> = BTreeMap::new();
    let mut other = Vec::new();
    let mut settled_net: BTreeMap<String, i64> = BTreeMap::new();
    let mut conveyed_net: BTreeMap<String, i64> = BTreeMap::new();
    let mut problems = Vec::new();

    for row in rows {
        let row = match row {
            RowParse::Parsed(row) => row,
            RowParse::Rejected { line, message } => {
                problems.push(format!("line {line}: {message}"));
                continue;
            }
        };
        let text = |name: &str| row.get(name).flatten().unwrap_or_default().to_string();
        let (currency, category, flag) = (
            text("Presentment Currency"),
            text("Category"),
            text("Settled/Conveyed"),
        );
        let amount = match rules::major_to_minor(&text("Amount")) {
            Ok(amount) => amount,
            Err(error) => {
                problems.push(format!("line {}: Amount: {error}", row.line));
                continue;
            }
        };
        // SALE adds, REF subtracts whatever sign the file used; other lines are signed in the file.
        let signed = match category.as_str() {
            "SALE" => amount.abs(),
            "REF" => -amount.abs(),
            _ => amount,
        };
        let net = if flag == "C" {
            &mut conveyed_net
        } else {
            &mut settled_net
        };
        *net.entry(currency.clone()).or_default() += signed;

        if matches!(category.as_str(), "SALE" | "REF") {
            let count = text("Count").parse::<i64>().unwrap_or(0).abs();
            let line =
                summary
                    .entry((currency, text("MOP"), category))
                    .or_insert((0, 0, Vec::new()));
            line.0 += count;
            line.1 += amount.abs();
            if !line.2.contains(&flag) {
                line.2.push(flag);
            }
        } else {
            other.push(OtherLine {
                currency,
                name: category_name(&category),
                category,
                settled_conveyed: Some(flag).filter(|f| !f.is_empty()),
                amount,
            });
        }
    }

    let mut detail: BTreeMap<Key, (i64, i64)> = BTreeMap::new();
    for entry in results.iter().filter_map(|r| r.entry.as_ref()) {
        let brand = entry
            .metadata
            .get(CARD_BRAND)
            .cloned()
            .flatten()
            .unwrap_or_default();
        let category = match entry.balance_direction {
            Direction::Credit => "SALE",
            Direction::Debit => "REF",
        };
        let line = detail
            .entry((entry.currency.clone(), brand, category.to_string()))
            .or_default();
        line.0 += 1;
        line.1 += entry.amount;
    }

    // Chase's order: by currency, card brand, then sales before refunds.
    let mut keys: Vec<&Key> = summary.keys().chain(detail.keys()).collect();
    keys.sort_by_key(|(currency, brand, category)| {
        let brand_rank = ["VI", "MC", "DI", "AX"]
            .iter()
            .position(|b| b == brand)
            .unwrap_or(usize::MAX);
        (currency, brand_rank, brand, category != "SALE")
    });
    keys.dedup();
    let lines: Vec<TieLine> = keys
        .into_iter()
        .map(|key| {
            let s = summary.get(key);
            let d = detail.get(key);
            let status = match (s, d) {
                (Some(s), Some(d)) if (s.0, s.1) == *d => TieStatus::Tied,
                (Some(_), Some(_)) => TieStatus::Differs,
                (Some(_), None) => TieStatus::SummaryOnly,
                (None, _) => TieStatus::DetailOnly,
            };
            TieLine {
                currency: key.0.clone(),
                card_brand: key.1.clone(),
                category: key.2.clone(),
                settled_conveyed: s.map(|s| s.2.join(", ")),
                summary_count: s.map(|s| s.0),
                summary_amount: s.map(|s| s.1),
                detail_count: d.map(|d| d.0),
                detail_amount: d.map(|d| d.1),
                status,
            }
        })
        .collect();

    Some(TieOut {
        tied: lines.iter().all(|line| line.status == TieStatus::Tied),
        lines,
        other,
        settled_net,
        conveyed_net,
        problems,
    })
}

/// FIN0010 category names (DFR spec v2.2, Appendix C).
fn category_name(code: &str) -> &'static str {
    match code {
        "SALE" => "Sales",
        "REF" => "Refunds",
        "BUW" => "Back up withholding",
        "CB" => "Net chargeback activity",
        "DBADJ" => "Net debit adjustment activity",
        "ERET" => "Net ECP return activity",
        "IA" => "Interchange and assessment fees",
        "OTH" => "Other adjustments",
        "PFEE" => "Chase Paymentech fees",
        "RES" => "Reserve adjustments",
        _ => "Other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform;

    const CONFIG: &str = r#"{
        "parsing_config": {"section": "ACT0010", "record_type": "RACT0010",
            "columns": ["Record Type", "Date", "Order", "MOP", "Action", "Amount", "Ccy"]},
        "fields": {
            "currency": {"identifier": "Ccy"},
            "amount": {"identifier": "Amount", "transformation_rules": [{"type": "absolute"}]},
            "balance_direction": {"identifier": "Action", "credit_values": ["DP"], "debit_values": ["RF"]},
            "effective_at": {"identifier": "Date", "date_format": {"order": "month_day_year", "delimiter": "slash"}},
            "order_id": {"identifier": "Order"}
        },
        "metadata_fields": [{"identifier": "MOP", "field_name": "card_brand"}]
    }"#;

    const FILE: &str = "*DFRBEG,PID=1,FREQ=DAILY,CO=2\n\
        HFIN0010,2,a,b,c,d\n\
        RFIN0010,TD,7,300,400,USD,VI,SALE,S,2,30.00\n\
        RFIN0010,TD,7,300,400,USD,VI,REF,S,1,-5.00\n\
        RFIN0010,TD,7,,,USD,AX,SALE,C,1,40.00\n\
        RFIN0010,TD,7,300,400,USD,MC,SALE,S,1,9.99\n\
        RFIN0010,TD,7,300,400,USD,,PFEE,S,,-0.50\n\
        HACT0010,2,a,b,c,d\n\
        RACT0010,10/01/2026,o1,VI,DP,10.00,USD\n\
        RACT0010,10/01/2026,o2,VI,DP,20.00,USD\n\
        RACT0010,10/01/2026,o3,VI,RF,-5.00,USD\n\
        RACT0010,10/01/2026,o4,AX,DP,40.00,USD\n\
        RACT0010,10/01/2026,o5,MC,DP,9.00,USD\n\
        *DFREND,PID=1,FREQ=DAILY,CO=2\n";

    fn run() -> TieOut {
        let config = TransformationConfig::from_json(CONFIG).unwrap();
        let output = transform(FILE.as_bytes(), &config);
        tie_out(FILE.as_bytes(), &config, &output).unwrap()
    }

    fn status(tie: &TieOut, brand: &str, category: &str) -> TieStatus {
        tie.lines
            .iter()
            .find(|l| l.card_brand == brand && l.category == category)
            .unwrap()
            .status
    }

    #[test]
    fn matching_lines_tie_and_a_short_one_differs() {
        let tie = run();
        assert_eq!(status(&tie, "VI", "SALE"), TieStatus::Tied);
        assert_eq!(status(&tie, "VI", "REF"), TieStatus::Tied);
        assert_eq!(status(&tie, "AX", "SALE"), TieStatus::Tied);
        assert_eq!(
            status(&tie, "MC", "SALE"),
            TieStatus::Differs,
            "9.00 staged, 9.99 in the summary"
        );
        assert!(!tie.tied);
    }

    #[test]
    fn conveyed_amex_is_not_in_the_settled_deposit() {
        let tie = run();
        // 30.00 + 9.99 - 5.00 - 0.50 settled; the 40.00 Amex sale is conveyed.
        assert_eq!(tie.settled_net.get("USD"), Some(&3449));
        assert_eq!(tie.conveyed_net.get("USD"), Some(&4000));
        assert_eq!(tie.other[0].name, "Chase Paymentech fees");
    }
}
