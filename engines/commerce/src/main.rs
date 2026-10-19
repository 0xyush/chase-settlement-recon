//! Command line for the Chase recon prototype.
//!
//! ```text
//! chase_recon serve [--port 8787]                       run the API the dashboard calls
//! chase_recon run <report.csv> <config.json> [--out DIR] transform a file, write JSON + CSV
//! chase_recon match <report.csv> <config.json> <hyperswitch.csv>
//! chase_recon mock-hs <report.csv> <config.json>        print a pretend Hyperswitch export
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use chase_recon::matching;
use chase_recon::report;
use chase_recon::transform::{Outcome, TransformOutput};
use chase_recon::{TransformationConfig, transform};

const USAGE: &str = "usage:
  chase_recon serve [--port 8787]
  chase_recon run <report.csv> <config.json> [--out DIR]
  chase_recon match <report.csv> <config.json> <hyperswitch.csv>
  chase_recon mock-hs <report.csv> <config.json>";

/// Prints a line. A closed pipe (for example `| head`) is not worth crashing over.
fn say(text: &str) {
    let _ = writeln!(std::io::stdout(), "{text}");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None | Some("serve") => serve(&args),
        Some("run") => run(&args),
        Some("match") => match_files(&args),
        Some("mock-hs") => mock_hs(&args),
        Some(_) => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn serve(args: &[String]) -> Result<(), String> {
    let port: u16 = flag(args, "--port")
        .map(|p| p.parse().map_err(|_| format!("bad port: {p}")))
        .transpose()?
        .unwrap_or(8787);
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let address = format!("127.0.0.1:{port}");
        let listener = tokio::net::TcpListener::bind(&address)
            .await
            .map_err(|e| format!("cannot listen on {address}: {e}"))?;
        say(&format!(
            "chase_recon API on http://{address}  (POST /api/transform, /api/export, /api/match)"
        ));
        axum::serve(listener, chase_recon::server::router())
            .await
            .map_err(|e| e.to_string())
    })
}

fn run(args: &[String]) -> Result<(), String> {
    let (csv_path, config_path) = two_paths(args)?;
    let output = load_and_transform(csv_path, config_path)?;
    print_summary(csv_path, &output);
    if let Some(reason) = &output.rejected {
        return Err(format!("file rejected: {reason}"));
    }
    let out_dir = flag(args, "--out").map_or_else(|| PathBuf::from("out"), PathBuf::from);
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let stem = Path::new(csv_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("report");
    let json_path = out_dir.join(format!("{stem}.staging_entries.json"));
    let csv_report_path = out_dir.join(format!("{stem}.report-v1.csv"));
    std::fs::write(&json_path, report::staging_entries_json(&output)).map_err(|e| e.to_string())?;
    std::fs::write(&csv_report_path, report::row_report_csv(&output)).map_err(|e| e.to_string())?;
    say(&format!("wrote {}", json_path.display()));
    say(&format!("wrote {}", csv_report_path.display()));
    Ok(())
}

fn match_files(args: &[String]) -> Result<(), String> {
    let (csv_path, config_path) = two_paths(args)?;
    let hs_path = args.get(3).ok_or(USAGE)?;
    let output = load_and_transform(csv_path, config_path)?;
    let entries: Vec<_> = output.staging_entries().cloned().collect();
    let export = std::fs::read(hs_path).map_err(|e| format!("{hs_path}: {e}"))?;
    let hyperswitch = matching::parse_hyperswitch_export(&export)?;
    let matched = matching::match_entries(&entries, &hyperswitch);
    say(&serde_json::to_string_pretty(&matched.summary).map_err(|e| e.to_string())?);
    Ok(())
}

fn mock_hs(args: &[String]) -> Result<(), String> {
    let (csv_path, config_path) = two_paths(args)?;
    let output = load_and_transform(csv_path, config_path)?;
    let entries: Vec<_> = output.staging_entries().cloned().collect();
    let _ = write!(
        std::io::stdout(),
        "{}",
        matching::mock_hyperswitch_export(&entries)
    );
    Ok(())
}

fn load_and_transform(csv_path: &str, config_path: &str) -> Result<TransformOutput, String> {
    let input = std::fs::read(csv_path).map_err(|e| format!("{csv_path}: {e}"))?;
    let config_text =
        std::fs::read_to_string(config_path).map_err(|e| format!("{config_path}: {e}"))?;
    let config =
        TransformationConfig::from_json(&config_text).map_err(|e| format!("{config_path}: {e}"))?;
    Ok(transform(&input, &config))
}

fn print_summary(name: &str, output: &TransformOutput) {
    let parsed = output.results.as_ref().map_or(0, Vec::len);
    say(&format!(
        "{name}: parsed {parsed}, staged {}, skipped {}, failed {}",
        output.count(Outcome::Success),
        output.count(Outcome::Skipped),
        output.count(Outcome::Failed)
    ));
}

fn two_paths(args: &[String]) -> Result<(&str, &str), String> {
    match (args.get(1), args.get(2)) {
        (Some(csv), Some(config)) => Ok((csv, config)),
        _ => Err(USAGE.to_string()),
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}
