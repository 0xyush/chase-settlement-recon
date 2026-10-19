//! End-to-end checks on the synthetic sample files (`orbital_recon samples`). The counts are
//! pinned, so any change in reading, skipping, validation or tie-out shows up here.

use orbital_recon::matching::{self, MatchSummary};
use orbital_recon::tieout::{self, TieOut, TieStatus};
use orbital_recon::transform::{Direction, Outcome};
use orbital_recon::{TransformOutput, TransformationConfig, sample, transform};

const DAY1_B: &str = "0000999999.261001.d.B101.dfr";
const DAY1_A: &str = "0000999999.261001.d.A101.dfr";
const DAY2_B_PIPE: &str = "0000999999.261002.d.B102.dfr";
const DAY3_EMPTY: &str = "0000999999.261003.d.B103.dfr";
const DAY4_CUT: &str = "0000999999.261004.d.B104.dfr";
const DAY5_LIVE: &str = "0000999999.261005.d.B105.dfr";

fn file(name: &str) -> String {
    sample::all()
        .into_iter()
        .find(|f| f.name == name)
        .unwrap()
        .contents
}

fn config(name: &str) -> TransformationConfig {
    let path = format!("{}/configs/{name}.json", env!("CARGO_MANIFEST_DIR"));
    TransformationConfig::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn deposits(text: &str) -> (TransformOutput, Option<TieOut>) {
    deposits_with(text, "act0010_deposit_detail")
}

fn deposits_with(text: &str, config_name: &str) -> (TransformOutput, Option<TieOut>) {
    let config = config(config_name);
    let output = transform(text.as_bytes(), &config);
    let tie = tieout::tie_out(text.as_bytes(), &config, &output);
    (output, tie)
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
fn samples_on_disk_match_the_generator() {
    for f in sample::all() {
        let path = format!("{}/samples/{}", env!("CARGO_MANIFEST_DIR"), f.name);
        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert!(
            on_disk == f.contents,
            "{} is stale: run `orbital_recon samples`",
            f.name
        );
    }
}

#[test]
fn day1_deposit_detail_stages_every_row_and_ties_out() {
    let (output, tie) = deposits(&file(DAY1_B));
    assert_eq!(counts(&output), (640, 640, 0, 0));
    let first = output.results.as_ref().unwrap()[0].clone();
    assert_eq!(
        first.line, 15,
        "frame, FIN0010 section and ACT0010 header come first"
    );
    let tie = tie.expect("the B file has a FIN0010 section");
    assert!(tie.tied, "every card brand's sales and refunds add up");
    assert_eq!(tie.settled_net.get("USD"), Some(&1_960_650));
    assert_eq!(tie.conveyed_net.get("USD"), Some(&329_196));
}

#[test]
fn amex_is_conveyed_and_the_rest_settled() {
    let (_, tie) = deposits(&file(DAY1_B));
    for line in tie.unwrap().lines {
        let expected = if line.card_brand == "AX" { "C" } else { "S" };
        assert_eq!(
            line.settled_conveyed.as_deref(),
            Some(expected),
            "{}",
            line.card_brand
        );
    }
}

#[test]
fn a_dropped_row_breaks_the_tie_out_on_its_card_brand() {
    let text = file(DAY1_B);
    let mut config = config("act0010_deposit_detail");
    let skip =
        r#"{"conditions": [{"identifier": "Record #", "operator": "equals", "value": "7"}]}"#;
    config
        .skip_configs
        .push(serde_json::from_str(skip).unwrap());
    let output = transform(text.as_bytes(), &config);
    let tie = tieout::tie_out(text.as_bytes(), &config, &output).unwrap();
    let differing: Vec<_> = tie
        .lines
        .iter()
        .filter(|l| l.status != TieStatus::Tied)
        .collect();
    assert_eq!(
        output.count(Outcome::Skipped),
        4,
        "record 7 of each of 4 submissions"
    );
    assert!(!tie.tied);
    assert!(!differing.is_empty() && differing.len() <= 4);
}

#[test]
fn delimiter_does_not_change_the_result() {
    let comma = file(DAY1_B);
    let pipe = comma.replace(',', "|");
    let entries = |text: &str| {
        let (output, _) = deposits(text);
        output.staging_entries().cloned().collect::<Vec<_>>()
    };
    assert_eq!(entries(&comma), entries(&pipe));
}

#[test]
fn day2_pipe_file_with_interchange_detail() {
    let (output, tie) = deposits(&file(DAY2_B_PIPE));
    assert_eq!(counts(&output), (480, 480, 0, 0));
    assert_eq!(
        output.header.as_ref().map(Vec::len),
        Some(30),
        "the trailing pipe adds no column"
    );
    let entries: Vec<_> = output.staging_entries().collect();
    let card = |e: &&orbital_recon::transform::StagingEntry| {
        e.metadata.get("card_brand").cloned().flatten()
    };
    assert!(
        entries
            .iter()
            .filter(|e| card(e).as_deref() != Some("AX"))
            .all(|e| e
                .metadata
                .get("interchange_fee")
                .is_some_and(Option::is_some))
    );
    assert!(tie.unwrap().tied);
}

#[test]
fn day5_live_layout_stages_every_row_and_ties_out() {
    let (output, tie) = deposits_with(&file(DAY5_LIVE), "act0010_deposit_detail_live");
    assert_eq!(counts(&output), (400, 400, 0, 0));
    assert_eq!(output.header.as_ref().map(Vec::len), Some(32));
    let tie = tie.unwrap();
    assert!(tie.tied, "every card brand and ECP ties to FIN0010");
    assert_eq!(tie.conveyed_net.get("USD"), Some(&146_514));

    let entries: Vec<_> = output.staging_entries().collect();
    let meta = |e: &orbital_recon::transform::StagingEntry, key: &str| {
        e.metadata.get(key).cloned().flatten()
    };
    assert!(
        entries
            .iter()
            .all(|e| matches!(meta(e, "transaction_type").as_deref(), Some("1" | "2" | "7"))),
        "field 22 (phone order, recurring, e-commerce) is read on every row"
    );
    let ecp_i: Vec<_> = entries
        .iter()
        .filter(|e| meta(e, "card_brand").as_deref() == Some("EC"))
        .filter(|e| e.balance_direction == Direction::Credit)
        .collect();
    assert_eq!(ecp_i.len(), 18, "ECP sales with action code DO or i are money in");
}

#[test]
fn spec_layout_config_on_a_live_layout_file_fails_loudly() {
    // The live layout has one more field before the fees. Read with the spec's positions, a rate
    // such as ".018" lands in the interchange fee column; it must fail, not stage a wrong fee.
    let (output, tie) = deposits(&file(DAY5_LIVE));
    let failed = output.count(Outcome::Failed);
    assert!(failed > 300, "{failed} rows failed");
    let first = output
        .results
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.outcome == Outcome::Failed)
        .and_then(|r| r.details.clone())
        .unwrap();
    assert!(first.contains("interchange_fee: PrecisionLoss"), "{first}");
    assert!(!tie.unwrap().tied);
}

#[test]
fn no_activity_day_is_valid_and_empty() {
    let (output, _) = deposits(&file(DAY3_EMPTY));
    assert!(output.rejected.is_none());
    assert_eq!(counts(&output), (0, 0, 0, 0));
}

#[test]
fn cut_off_file_is_rejected_whole() {
    let (output, tie) = deposits(&file(DAY4_CUT));
    assert!(
        output
            .rejected
            .as_deref()
            .unwrap()
            .starts_with("Incomplete")
    );
    assert!(output.results.is_none(), "not one row is staged");
    assert!(tie.is_none());
    assert_eq!(output.file.unwrap().sections.len(), 2);
}

#[test]
fn chargebacks_stage_only_what_moves_money() {
    let text = file(DAY1_A);
    let output = transform(text.as_bytes(), &config("pde0017_chargebacks"));
    assert_eq!(
        counts(&output),
        (9, 3, 6, 0),
        "RPDE0017S summary lines are not read; RECD and REPR are skipped"
    );
    let entries: Vec<_> = output.staging_entries().collect();
    assert!(
        entries
            .iter()
            .all(|e| e.balance_direction == Direction::Debit)
    );
    let debited: i64 = entries.iter().map(|e| e.amount).sum();
    // The same money shows up as the CB line of the day 1 B file's FIN0010 summary.
    let (_, tie) = deposits(&file(DAY1_B));
    let cb = tie
        .unwrap()
        .other
        .into_iter()
        .find(|o| o.category == "CB")
        .unwrap();
    assert_eq!(debited, -cb.amount);
}

#[test]
fn deposit_config_on_the_exception_file_says_where_to_look() {
    let (output, _) = deposits(&file(DAY1_A));
    let reason = output.rejected.unwrap();
    assert!(reason.starts_with("SectionNotFound"), "{reason}");
    assert!(reason.contains("it has: PDE0017"));
}

#[test]
fn chase_order_ids_never_exceed_22_characters() {
    let (output, _) = deposits(&file(DAY1_B));
    let longest = output.staging_entries().map(|e| e.order_id.len()).max();
    assert_eq!(
        longest,
        Some(22),
        "30-character merchant ids would not fit"
    );
}

#[test]
fn matching_finds_the_planted_differences() {
    let (output, _) = deposits(&file(DAY1_B));
    let entries: Vec<_> = output.staging_entries().cloned().collect();
    let export = matching::mock_hyperswitch_export(&entries);
    let hyperswitch = matching::parse_hyperswitch_export(export.as_bytes()).unwrap();
    let summary = matching::match_entries(&entries, &hyperswitch).summary;
    assert_eq!(
        summary,
        MatchSummary {
            chase_entries: 640,
            hyperswitch_records: 636,
            matched: 627,
            amount_mismatch: 6,
            currency_mismatch: 0,
            missing_in_hyperswitch: 7,
            missing_in_chase: 2,
            ignored: 1,
        }
    );
}

#[test]
fn steps_say_where_a_file_stopped() {
    use orbital_recon::steps::{self, StepStatus};
    let statuses = |name: &str, config_name: &str| {
        let text = file(name);
        let config = config(config_name);
        let output = transform(text.as_bytes(), &config);
        steps::build(&config, &output, None, None)
            .into_iter()
            .map(|s| s.status)
            .collect::<Vec<_>>()
    };
    let day1 = statuses(DAY1_B, "act0010_deposit_detail");
    assert_eq!(day1.len(), 8);
    assert!(day1[..5].iter().all(|s| *s == StepStatus::Done));
    let cut = statuses(DAY4_CUT, "act0010_deposit_detail");
    assert_eq!(
        cut[0],
        StepStatus::Failed,
        "the frame check stops a cut-off file"
    );
    assert!(cut[1..].iter().all(|s| *s == StepStatus::NotRun));
    let wrong_file = statuses(DAY1_A, "act0010_deposit_detail");
    assert_eq!(&wrong_file[..2], &[StepStatus::Done, StepStatus::Failed]);
}

#[test]
fn authorization_reversals_are_skipped_not_failed() {
    // Chase's 2018 ACT-0010 guide lists AR (authorization reversal) as an action code. It moves
    // no money, so the deposit config skips it instead of failing it as an unknown direction.
    let text = file(DAY1_B).replacen(",DP,", ",AR,", 1);
    let (output, _) = deposits(&text);
    assert_eq!(counts(&output), (640, 639, 1, 0));
    let skipped = output
        .results
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.outcome == Outcome::Skipped)
        .unwrap();
    assert_eq!(
        skipped.details.as_deref(),
        Some(r#"skip: Action Code equals "AR""#)
    );
}
