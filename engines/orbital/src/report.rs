//! Step 4: write the results out, the two files the recon service keeps for every run.
//!
//! - staging entries as JSON: what would be inserted into `staging_entry`
//! - a row report as CSV, one line per input row: `source_type,identifier,outcome,details`
//!   (the recon repo's `report-v1.csv`)

use crate::transform::{Outcome, TransformOutput};

pub fn staging_entries_json(output: &TransformOutput) -> String {
    let entries: Vec<_> = output.staging_entries().collect();
    serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string())
}

pub fn row_report_csv(output: &TransformOutput) -> String {
    let quote = |value: &str| format!("\"{}\"", value.replace('"', "\"\""));
    let rows = output.results.iter().flatten().map(|result| {
        let details = match result.outcome {
            Outcome::Success => "",
            _ => result.details.as_deref().unwrap_or_default(),
        };
        let line = result.line.to_string();
        ["file_row", line.as_str(), result.outcome.as_str(), details]
            .map(quote)
            .join(",")
    });
    std::iter::once("source_type,identifier,outcome,details".to_string())
        .chain(rows)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TransformationConfig, transform};

    #[test]
    fn report_has_one_line_per_row_with_reasons() {
        let config = TransformationConfig::from_json(
            r#"{"parsing_config": {"section": "ACT0010", "record_type": "RACT0010",
                    "columns": ["Record Type", "Order", "Amount", "Ccy", "Dir", "Date"]},
                "skip_configs": [{"conditions": [{"identifier": "Order", "operator": "equals", "value": "END"}]}],
                "fields": {
                    "currency": {"identifier": "Ccy"}, "amount": {"identifier": "Amount"},
                    "balance_direction": {"identifier": "Dir", "credit_values": ["S"], "debit_values": ["R"]},
                    "effective_at": {"identifier": "Date", "date_format": {"order": "month_day_year", "delimiter": "slash"}},
                    "order_id": {"identifier": "Order"}}}"#,
        )
        .unwrap();
        let file = "*DFRBEG,PID=1,FREQ=DAILY,CO=2\nHACT0010,2,a,b,c,d\n\
            RACT0010,A,1,USD,S,1/2/2025\nRACT0010,B,x,USD,S,1/2/2025\nRACT0010,END,,,,\n\
            *DFREND,PID=1,FREQ=DAILY,CO=2\n";
        let output = transform(file.as_bytes(), &config);
        let csv = row_report_csv(&output);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "source_type,identifier,outcome,details");
        assert_eq!(lines[1], r#""file_row","3","Success","""#);
        assert_eq!(
            lines[2],
            r#""file_row","4","Failed","amount: InvalidType: ""x"" is not a number""#
        );
        assert!(lines[3].starts_with(r#""file_row","5","Skipped","skip: Order equals"#));
        assert!(staging_entries_json(&output).contains("\"order_id\": \"A\""));
    }
}
