//! The pipeline as steps, for the dashboard's step strip: what each step did to this file, which
//! source file does it, and how long it took in the engine. Nothing here changes the result; it
//! only describes the run.

use serde::Serialize;

use crate::TransformationConfig;
use crate::tieout::TieOut;
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
    tieout: Option<(&TieOut, u64)>,
    written: Option<Written>,
) -> Vec<Step> {
    let p = &config.parsing_config;
    let t = output.timings;
    let rejected = output.rejected.as_deref();
    let failed_at = match rejected {
        None => 0,
        Some(reason) if reason.starts_with("SectionNotFound") => 2,
        Some(_) => 1,
    };
    let after_failure = "Not run: an earlier step stopped the file.";
    let mut steps = Vec::new();

    // 1. Check the file
    steps.push(match (failed_at, &output.file) {
        (1, _) | (_, None) => step(
            "check",
            "Check the file",
            "src/dfr.rs",
            StepStatus::Failed,
            "rejected",
            rejected.unwrap_or("The engine could not read the file."),
            micros(t.read_ns),
        ),
        (_, Some(file)) => step(
            "check",
            "Check the file",
            "src/dfr.rs",
            StepStatus::Done,
            format!("{} lines", file.lines),
            format!(
                "*DFRBEG on line 1 and *DFREND on line {} carry the same values, so the file \
                 arrived whole. The delimiter is a {}, read from line 1. {} report section(s).",
                file.end.as_ref().map_or(0, |end| end.line),
                file.delimiter.name(),
                file.sections.len()
            ),
            micros(t.read_ns),
        ),
    });

    // 2. Find the section
    let section = output.file.as_ref().and_then(|f| f.section(&p.section));
    steps.push(match (failed_at, section) {
        (1, _) => not_run("section", "Find the section", "src/dfr.rs", after_failure),
        (2, _) | (_, None) => step(
            "section",
            "Find the section",
            "src/dfr.rs",
            StepStatus::Failed,
            "not found",
            rejected.unwrap_or("The section is not in the file."),
            None,
        ),
        (_, Some(section)) => {
            let count = section.records.get(&p.record_type).copied().unwrap_or(0);
            let others: Vec<&str> = output
                .file
                .iter()
                .flat_map(|f| f.sections.iter())
                .filter(|s| s.code != p.section)
                .map(|s| s.code.as_str())
                .collect();
            step(
                "section",
                "Find the section",
                "src/dfr.rs",
                StepStatus::Done,
                format!("{count} {}", p.record_type),
                format!(
                    "H{} ({}) starts on line {}, and {count} {} records follow.{} Timed with step 1: \
                     one pass over the file.",
                    p.section,
                    section.name.unwrap_or("unknown report"),
                    section.line,
                    p.record_type,
                    if others.is_empty() {
                        String::new()
                    } else {
                        format!(" Ignored: {}.", others.join(", "))
                    }
                ),
                None,
            )
        }
    });

    if failed_at > 0 {
        steps.push(not_run(
            "fields",
            "Name the fields",
            "src/dfr.rs",
            after_failure,
        ));
        return finish(steps, config, output, tieout, written, after_failure);
    }

    // 3. Name the fields
    let named = p.columns.len();
    let extra = output
        .header
        .as_ref()
        .map_or(0, |h| h.len().saturating_sub(named));
    let parse_errors = output
        .results
        .iter()
        .flatten()
        .filter(|r| {
            r.details
                .as_deref()
                .is_some_and(|d| d.starts_with("ParseError"))
        })
        .count();
    steps.push(step(
        "fields",
        "Name the fields",
        "src/dfr.rs",
        if parse_errors > 0 {
            StepStatus::Warning
        } else {
            StepStatus::Done
        },
        format!("{named} fields"),
        format!(
            "Each {} line is split on the delimiter, keeping quoted text together. Field N gets the \
             Nth name in the config's column list, copied from Chase's DFR spec.{}{} Timed with step 1.",
            p.record_type,
            if extra > 0 {
                format!(" {extra} extra field(s) kept as field_N.")
            } else {
                String::new()
            },
            if parse_errors > 0 {
                format!(" {parse_errors} line(s) could not be split.")
            } else {
                String::new()
            }
        ),
        None,
    ));

    finish(steps, config, output, tieout, written, after_failure)
}

/// Steps 4 to 8: the same engine as the Commerce Center prototype, plus the tie-out.
fn finish(
    mut steps: Vec<Step>,
    config: &TransformationConfig,
    output: &TransformOutput,
    tieout: Option<(&TieOut, u64)>,
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
        steps.push(not_run("tieout", "Tie out", "src/tieout.rs", why));
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

    // 4. Skip rows
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

    // 5. Map and validate
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

    // 6. Tie out
    steps.push(match tieout {
        Some((tie, nanos)) => {
            let lines = tie.lines.len();
            let differ = lines
                - tie
                    .lines
                    .iter()
                    .filter(|l| l.status == crate::tieout::TieStatus::Tied)
                    .count();
            let money = |map: &std::collections::BTreeMap<String, i64>| {
                map.iter()
                    .map(|(currency, cents)| {
                        format!("{currency} {}.{:02}", cents / 100, (cents % 100).abs())
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            step(
                "tieout",
                "Tie out",
                "src/tieout.rs",
                if tie.tied {
                    StepStatus::Done
                } else {
                    StepStatus::Warning
                },
                if tie.tied {
                    format!("{lines} lines tie")
                } else {
                    format!("{differ} of {lines} differ")
                },
                format!(
                    "Staged entries added up by card brand and sale or refund, then compared with \
                     Chase's own FIN0010 summary in the same file. Chase deposits {}{}.",
                    money(&tie.settled_net),
                    if tie.conveyed_net.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "; {} is conveyed, paid by the network",
                            money(&tie.conveyed_net)
                        )
                    }
                ),
                micros(nanos),
            )
        }
        None => not_run(
            "tieout",
            "Tie out",
            "src/tieout.rs",
            "Runs for Deposit Detail (ACT0010) when the same file has Chase's FIN0010 summary.",
        ),
    });

    // 7. Write outputs
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

    // 8. Match with Hyperswitch
    steps.push(not_run(
        "match",
        "Match with Hyperswitch",
        "src/matching.rs",
        "Runs from the Next step card under the results: entries are paired with a Hyperswitch \
         export on order ID and direction, then the amounts are compared.",
    ));
    steps
}
