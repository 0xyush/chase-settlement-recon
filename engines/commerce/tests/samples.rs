//! End-to-end checks on J.P. Morgan's public sample reports: the counts are pinned, so any
//! change in parsing, skipping or validation shows up here.

use chase_recon::matching::{self, MatchSummary};
use chase_recon::transform::{Direction, Outcome};
use chase_recon::{TransformOutput, TransformationConfig, transform};

fn run(sample: &str, config: &str) -> TransformOutput {
    let root = env!("CARGO_MANIFEST_DIR");
    let input = std::fs::read(format!("{root}/samples/{sample}")).unwrap();
    let config_text = std::fs::read_to_string(format!("{root}/configs/{config}")).unwrap();
    transform(
        &input,
        &TransformationConfig::from_json(&config_text).unwrap(),
    )
}

fn counts(output: &TransformOutput) -> (usize, usize, usize, usize) {
    (
        output.results.as_ref().map_or(0, Vec::len),
        output.count(Outcome::Success),
        output.count(Outcome::Skipped),
        output.count(Outcome::Failed),
    )
}

#[test]
fn deposit_details() {
    let output = run("Deposit_Details_Preset_Report.csv", "deposit_details.json");
    // The END line is dropped by footer_marker, so every one of the 825 rows is data.
    assert_eq!(counts(&output), (825, 825, 0, 0));
    let results = output.results.as_ref().unwrap();
    assert_eq!(
        results[0].line, 4,
        "the header is on line 3, so data starts on line 4"
    );
    let first = results[0].entry.as_ref().unwrap();
    assert_eq!(first.order_id, "c4kLuNMbfSz9e8JfbpBLc5");
    assert_eq!(first.amount, 2000, "refund of 20.00, stored as 2000 cents");
}

#[test]
fn bank_debit_rows_stage_without_a_card_last4() {
    let output = run("Deposit_Details_Preset_Report.csv", "deposit_details.json");
    let last4 = |entry: &chase_recon::transform::StagingEntry| {
        entry.metadata.get("card_last4").cloned().flatten()
    };
    let (bank, card): (Vec<_>, Vec<_>) = output
        .staging_entries()
        .partition(|e| e.metadata.get("card_brand") == Some(&Some("ED".to_string())));
    assert_eq!(
        bank.len(),
        59,
        "European direct debits: account number is \"IBAN\""
    );
    assert!(bank.iter().all(|e| last4(e).is_none()));
    assert!(card.iter().filter_map(|e| last4(e)).all(|v| v.len() == 4));
}

#[test]
fn transaction_fee_details() {
    let output = run(
        "Transaction_Fee_Details_Preset_Report.csv",
        "transaction_fee_details.json",
    );
    // One account-level fee row has no order number: skipped, not failed.
    assert_eq!(counts(&output), (1208, 1207, 1, 0));
}

#[test]
fn funded_transaction_fee_details() {
    let output = run(
        "Funded_Transaction_Fee_Details_Preset_Report.csv",
        "funded_txn_fee_details.json",
    );
    // AUTH rows carry fees but no money, OTHER rows are account fees: both skipped.
    assert_eq!(counts(&output), (5205, 247, 4958, 0));
    let chargeback = output
        .staging_entries()
        .find(|e| e.metadata.get("action_type") == Some(&Some("DISPUTE".to_string())))
        .expect("the chargeback is staged");
    assert_eq!(chargeback.balance_direction, Direction::Debit);
    assert_eq!(chargeback.amount, 999_999);
}

#[test]
fn chase_order_ids_never_exceed_22_characters() {
    let output = run("Deposit_Details_Preset_Report.csv", "deposit_details.json");
    let longest = output
        .staging_entries()
        .map(|e| e.order_id.len())
        .max()
        .unwrap();
    assert_eq!(longest, 22, "30-character merchant ids would not fit");
}

#[test]
fn matching_finds_the_planted_differences() {
    let output = run("Deposit_Details_Preset_Report.csv", "deposit_details.json");
    let entries: Vec<_> = output.staging_entries().cloned().collect();
    let export = matching::mock_hyperswitch_export(&entries);
    let hyperswitch = matching::parse_hyperswitch_export(export.as_bytes()).unwrap();
    let summary = matching::match_entries(&entries, &hyperswitch).summary;
    assert_eq!(
        summary,
        MatchSummary {
            chase_entries: 825,
            hyperswitch_records: 819,
            matched: 807,
            amount_mismatch: 9,
            currency_mismatch: 0,
            missing_in_hyperswitch: 9,
            missing_in_chase: 2,
            ignored: 1,
        }
    );
}

#[test]
fn steps_describe_the_run() {
    use chase_recon::steps::{self, StepStatus};
    let config_text = std::fs::read_to_string(format!(
        "{}/configs/deposit_details.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let config = TransformationConfig::from_json(&config_text).unwrap();
    let output = run("Deposit_Details_Preset_Report.csv", "deposit_details.json");
    let built = steps::build(&config, &output, None);
    let keys: Vec<&str> = built.iter().map(|s| s.key).collect();
    assert_eq!(keys, ["read", "skip", "map", "write", "match"]);
    assert_eq!(built[0].status, StepStatus::Done);
    assert_eq!(built[0].metric, "825 rows");
    let rejected = transform(b"just one line", &config);
    assert_eq!(
        steps::build(&config, &rejected, None)[0].status,
        StepStatus::Failed
    );
}
