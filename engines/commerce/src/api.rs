//! The engine's requests and answers in one place, shared by the local HTTP server
//! (`server.rs`) and the WebAssembly build the static site runs in the browser (`../../wasm`).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_time::Instant;

use crate::matching::{self, MatchOutput};
use crate::report;
use crate::steps::{self, Step, Written};
use crate::transform::{ENGINE, StagingEntry, TransformOutput};
use crate::{TransformationConfig, transform};

/// The transform answer: the result and the steps.
#[derive(Debug, Serialize)]
pub struct TransformResponse {
    #[serde(flatten)]
    pub output: TransformOutput,
    /// The run as steps, for the dashboard's step snackbar.
    pub steps: Vec<Step>,
}

impl TransformResponse {
    pub fn failed(reason: String) -> Self {
        Self {
            output: TransformOutput::rejected(reason),
            steps: Vec::new(),
        }
    }
}

fn nanos(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

/// Runs the whole pipeline on one CSV report's text with a config given as JSON.
pub fn transform_file(text: &str, config: Value) -> TransformResponse {
    let config = match serde_json::from_value::<TransformationConfig>(config) {
        Ok(config) => config,
        Err(error) => return TransformResponse::failed(format!("InvalidConfig: {error}")),
    };
    let output = transform(text.as_bytes(), &config);
    let written = output.results.as_ref().map(|_| {
        let clock = Instant::now();
        let json = report::staging_entries_json(&output);
        let csv = report::row_report_csv(&output);
        let nanos = nanos(clock.elapsed());
        drop(json);
        Written {
            entries: output.staging_entries().count(),
            report_lines: csv.lines().count().saturating_sub(1),
            nanos,
        }
    });
    let steps = steps::build(&config, &output, written);
    TransformResponse { output, steps }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    StagingJson,
    ReportCsv,
}

impl ExportFormat {
    pub fn content_type(self) -> &'static str {
        match self {
            Self::StagingJson => "application/json",
            Self::ReportCsv => "text/csv",
        }
    }
}

/// The staging entries (JSON) or the row report (CSV) for one file, or why the file was rejected.
pub fn export(text: &str, config: Value, format: ExportFormat) -> Result<String, String> {
    let output = transform_file(text, config).output;
    if let Some(reason) = output.rejected {
        return Err(reason);
    }
    Ok(match format {
        ExportFormat::StagingJson => report::staging_entries_json(&output),
        ExportFormat::ReportCsv => report::row_report_csv(&output),
    })
}

#[derive(Debug, Serialize)]
pub struct MatchResponse {
    pub engine: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected: Option<String>,
    #[serde(flatten)]
    pub result: Option<MatchOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_csv: Option<String>,
}

/// Matches staged entries against a Hyperswitch export CSV.
pub fn match_with_hyperswitch(entries: &[StagingEntry], hyperswitch_csv: &str) -> MatchResponse {
    match matching::parse_hyperswitch_export(hyperswitch_csv.as_bytes()) {
        Ok(hyperswitch) => {
            let output = matching::match_entries(entries, &hyperswitch);
            MatchResponse {
                engine: ENGINE,
                rejected: None,
                report_csv: Some(matching::match_report_csv(&output)),
                result: Some(output),
            }
        }
        Err(reason) => MatchResponse {
            engine: ENGINE,
            rejected: Some(reason),
            result: None,
            report_csv: None,
        },
    }
}
