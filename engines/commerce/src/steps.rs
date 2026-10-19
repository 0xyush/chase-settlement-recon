//! The pipeline as steps, for the dashboard's step strip: what each step did to this file, which
//! source file does it, and how long it took in the engine. Nothing here changes the result; it
//! only describes the run.

use serde::Serialize;

use crate::TransformationConfig;
use crate::transform::{Outcome, TransformOutput};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Done,
    Warning,
    Failed,
    NotRun,
}

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub key: &'static str,
    pub title: &'static str,
    /// The source file that does this step.
    pub code: &'static str,
    pub status: StepStatus,
    /// A few words for the strip, for example "640 staged".
    pub metric: String,
    /// What happened, in a sentence or two.
    pub detail: String,
    /// Time in the engine, in microseconds. `None` when the step did not run on its own clock.
    pub micros: Option<f64>,
}

/// What writing the outputs produced: entries, report lines, nanoseconds.
pub struct Written {
    pub entries: usize,
    pub report_lines: usize,
    pub nanos: u64,
}

fn micros(nanos: u64) -> Option<f64> {
    Some(nanos as f64 / 1000.0)
}

fn step(
    key: &'static str,
    title: &'static str,
    code: &'static str,
    status: StepStatus,
    metric: impl Into<String>,
    detail: impl Into<String>,
    micros: Option<f64>,
) -> Step {
    Step {
        key,
        title,
        code,
        status,
        metric: metric.into(),
        detail: detail.into(),
        micros,
    }
}

fn not_run(key: &'static str, title: &'static str, code: &'static str, why: &str) -> Step {
    step(key, title, code, StepStatus::NotRun, "not run", why, None)
}

pub fn build(
    config: &TransformationConfig,
    output: &TransformOutput,
    written: Option<Written>,
) -> Vec<Step> {
    let p = &config.parsing_config;
    let after_failure = "Not run: the file was rejected.";
    let rows = output
        .results
        .as_ref()
        .map(Vec::len)
        .or(output.rows.as_ref().map(Vec::len))
        .unwrap_or(0);
    let read = match output.rejected.as_deref() {
        Some(reason) => step(
            "read",
            "Read the CSV",
            "src/parser.rs",
            StepStatus::Failed,
            "rejected",
            reason,
            micros(output.timings.read_ns),
        ),
        None => step(
            "read",
            "Read the CSV",
            "src/parser.rs",
            StepStatus::Done,
            format!("{rows} rows"),
            format!(
                "Column names come from line {} (header_row_index {}), and the {rows} lines under \
                 it are data.{} Empty cells become \"missing\".",
                p.header_row_index + 1,
                p.header_row_index,
                p.footer_marker
                    .as_deref()
                    .map(|marker| format!(" The closing \"{marker}\" line is dropped."))
                    .unwrap_or_default()
            ),
            micros(output.timings.read_ns),
        ),
    };
    finish(vec![read], config, output, written, after_failure)
}

/// Steps 2 to 5: skip, map and validate, write, match.
fn finish(
    mut steps: Vec<Step>,
    config: &TransformationConfig,
    output: &TransformOutput,
    written: Option<Written>,
    after_failure: &str,
) -> Vec<Step> {
    let Some(results) = &output.results else {
        let why = if output.rejected.is_some() {
            after_failure
        } else {
            "Not run: the config maps no fields yet, so rows are only read."
        };
        steps.push(not_run("skip", "Skip rows", "src/skip.rs", why));
        steps.push(not_run("map", "Map and validate", "src/transform.rs", why));
        steps.push(not_run("write", "Write outputs", "src/report.rs", why));
        steps.push(not_run(
            "match",
            "Match with Hyperswitch",
            "src/matching.rs",
            why,
        ));
        return steps;
    };
    let t = output.timings;
    let count = |outcome| results.iter().filter(|r| r.outcome == outcome).count();
    let (staged, skipped, failed) = (
        count(Outcome::Success),
        count(Outcome::Skipped),
        count(Outcome::Failed),
    );
    let first_reason = |outcome| {
        results
            .iter()
            .find(|r| r.outcome == outcome)
            .and_then(|r| r.details.clone())
            .unwrap_or_default()
    };

    // 2. Skip rows
    let rules = config.skip_configs.len();
    steps.push(step(
        "skip",
        "Skip rows",
        "src/skip.rs",
        StepStatus::Done,
        format!("{skipped} skipped"),
        if rules == 0 {
            "No skip rules in the config, so every row goes on to mapping.".to_string()
        } else if skipped == 0 {
            format!("{rules} skip rule(s); no row matched them.")
        } else {
            format!(
                "{rules} skip rule(s); {skipped} row(s) matched and stop here. First: {}",
                first_reason(Outcome::Skipped)
            )
        },
        micros(t.skip_ns),
    ));

    // 3. Map and validate
    let metadata = config.metadata_fields.len();
    steps.push(step(
        "map",
        "Map and validate",
        "src/transform.rs + src/rules.rs",
        if failed > 0 {
            StepStatus::Warning
        } else {
            StepStatus::Done
        },
        format!("{staged} staged"),
        format!(
            "Each kept row becomes a staging entry: the five main fields (order_id, amount, \
             currency, effective_at, balance_direction) plus {metadata} metadata fields, with \
             amounts turned into cents. {}",
            if failed == 0 {
                "No row failed.".to_string()
            } else {
                format!(
                    "{failed} row(s) failed and say why. First: {}",
                    first_reason(Outcome::Failed)
                )
            }
        ),
        micros(t.map_ns),
    ));

    // 4. Write outputs
    steps.push(match written {
        Some(w) => step(
            "write",
            "Write outputs",
            "src/report.rs",
            StepStatus::Done,
            format!("{} entries", w.entries),
            format!(
                "staging_entries.json holds the {} staged entries; report-v1.csv has one line per \
                 row ({}), with the reason for every skip and failure. Download both in Entries.",
                w.entries, w.report_lines
            ),
            micros(w.nanos),
        ),
        None => not_run("write", "Write outputs", "src/report.rs", "Not run."),
    });

    // 5. Match with Hyperswitch
    steps.push(not_run(
        "match",
        "Match with Hyperswitch",
        "src/matching.rs",
        "Runs from the Next step card under the results: entries are paired with a Hyperswitch \
         export on order ID and direction, then the amounts are compared.",
    ));
    steps
}
