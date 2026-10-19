//! Command line for the Orbital (DFR) recon prototype.
//!
//! ```text
//! orbital_recon serve [--port 8788]                     run the API the dashboard calls
//! orbital_recon scan <file.dfr>                         show the file's frame and sections
//! orbital_recon run <file.dfr> <config.json> [--out DIR] transform a file, write JSON + CSV
//! orbital_recon match <file.dfr> <config.json> <hyperswitch.csv>
//! orbital_recon mock-hs <file.dfr> <config.json>        print a pretend Hyperswitch export
//! orbital_recon samples [--out DIR]                     write the synthetic sample files
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use orbital_recon::dfr::{self, DfrFile};
use orbital_recon::tieout::{self, TieStatus};
use orbital_recon::transform::{Outcome, TransformOutput};
use orbital_recon::{TransformationConfig, matching, report, sample, transform};

const USAGE: &str = "usage:
  orbital_recon serve [--port 8788]
  orbital_recon scan <file.dfr>
  orbital_recon run <file.dfr> <config.json> [--out DIR]
  orbital_recon match <file.dfr> <config.json> <hyperswitch.csv>
  orbital_recon mock-hs <file.dfr> <config.json>
  orbital_recon samples [--out DIR]";

/// Prints a line. A closed pipe (for example `| head`) is not worth crashing over.
fn say(text: &str) {
    let _ = writeln!(std::io::stdout(), "{text}");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None | Some("serve") => serve(&args),
        Some("scan") => scan(&args),
        Some("run") => run(&args),
        Some("match") => match_files(&args),
        Some("mock-hs") => mock_hs(&args),
        Some("samples") => samples(&args),
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
        .unwrap_or(8788);
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    runtime.block_on(async move {
        let address = format!("127.0.0.1:{port}");
        let listener = tokio::net::TcpListener::bind(&address)
            .await
            .map_err(|e| format!("cannot listen on {address}: {e}"))?;
        say(&format!(
            "orbital_recon API on http://{address}  (POST /api/transform, /api/export, /api/match)"
        ));
        axum::serve(listener, orbital_recon::server::router())
            .await
            .map_err(|e| e.to_string())
    })
}

fn scan(args: &[String]) -> Result<(), String> {
    let path = args.get(1).ok_or(USAGE)?;
    let input = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    print_structure(&dfr::scan_only(&input)?);
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    let (file_path, config_path) = two_paths(args)?;
    let (input, config) = load(file_path, config_path)?;
    let output = transform(&input, &config);
    if let Some(file) = &output.file {
        print_structure(file);
    }
    if let Some(reason) = &output.rejected {
        return Err(format!("file rejected: {reason}"));
    }
    print_summary(file_path, &output);
    if let Some(tie) = tieout::tie_out(&input, &config, &output) {
        let differs = tie
            .lines
            .iter()
            .filter(|l| l.status != TieStatus::Tied)
            .count();
        say(&format!(
            "tie-out against FIN0010: {}",
            if tie.tied {
                "every card brand ties".to_string()
            } else {
                format!("{differs} line(s) differ")
            }
        ));
        for (currency, cents) in &tie.settled_net {
            say(&format!("  settled by Chase: {currency} {}", money(*cents)));
        }
        for (currency, cents) in &tie.conveyed_net {
            say(&format!(
                "  conveyed (paid by the network): {currency} {}",
                money(*cents)
            ));
        }
    }
    let out_dir = flag(args, "--out").map_or_else(|| PathBuf::from("out"), PathBuf::from);
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let stem = Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("report")
        .trim_end_matches(".dfr");
    let json_path = out_dir.join(format!("{stem}.staging_entries.json"));
    let csv_report_path = out_dir.join(format!("{stem}.report-v1.csv"));
    std::fs::write(&json_path, report::staging_entries_json(&output)).map_err(|e| e.to_string())?;
    std::fs::write(&csv_report_path, report::row_report_csv(&output)).map_err(|e| e.to_string())?;
    say(&format!("wrote {}", json_path.display()));
    say(&format!("wrote {}", csv_report_path.display()));
    Ok(())
}

fn match_files(args: &[String]) -> Result<(), String> {
    let (file_path, config_path) = two_paths(args)?;
    let hs_path = args.get(3).ok_or(USAGE)?;
    let (input, config) = load(file_path, config_path)?;
    let output = transform(&input, &config);
    let entries: Vec<_> = output.staging_entries().cloned().collect();
    let export = std::fs::read(hs_path).map_err(|e| format!("{hs_path}: {e}"))?;
    let hyperswitch = matching::parse_hyperswitch_export(&export)?;
    let matched = matching::match_entries(&entries, &hyperswitch);
    say(&serde_json::to_string_pretty(&matched.summary).map_err(|e| e.to_string())?);
    Ok(())
}

fn mock_hs(args: &[String]) -> Result<(), String> {
    let (file_path, config_path) = two_paths(args)?;
    let (input, config) = load(file_path, config_path)?;
    let output = transform(&input, &config);
    let entries: Vec<_> = output.staging_entries().cloned().collect();
    let _ = write!(
        std::io::stdout(),
        "{}",
        matching::mock_hyperswitch_export(&entries)
    );
    Ok(())
}

fn samples(args: &[String]) -> Result<(), String> {
    let out_dir = flag(args, "--out").map_or_else(|| PathBuf::from("samples"), PathBuf::from);
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let files = sample::all();
    let mut about = String::from(
        "Synthetic Chase Orbital / Paymentech Delimited File Reports (DFR).\n\n\
         There is no public DFR sample, so these are made up from Chase's DFR spec v2.2,\n\
         deterministically (orbital_recon samples). Masked test card numbers only.\n\
         Replace them with a real file from the merchant's Chase account when there is one.\n\n",
    );
    for file in &files {
        std::fs::write(out_dir.join(&file.name), &file.contents).map_err(|e| e.to_string())?;
        about.push_str(&format!("{}\n  {}\n\n", file.name, file.about));
        say(&format!("wrote {}", out_dir.join(&file.name).display()));
    }
    std::fs::write(out_dir.join("ABOUT THESE FILES.txt"), about).map_err(|e| e.to_string())?;
    Ok(())
}

fn load(file_path: &str, config_path: &str) -> Result<(Vec<u8>, TransformationConfig), String> {
    let input = std::fs::read(file_path).map_err(|e| format!("{file_path}: {e}"))?;
    let config_text =
        std::fs::read_to_string(config_path).map_err(|e| format!("{config_path}: {e}"))?;
    let config =
        TransformationConfig::from_json(&config_text).map_err(|e| format!("{config_path}: {e}"))?;
    Ok((input, config))
}

fn print_structure(file: &DfrFile) {
    let frame = |values: &indexmap::IndexMap<String, String>| {
        values
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    say(&format!(
        "line {:>5}  *DFRBEG  {}  ({}-delimited)",
        file.begin.line,
        frame(&file.begin.values),
        file.delimiter.name()
    ));
    for section in &file.sections {
        let records = section
            .records
            .iter()
            .map(|(kind, count)| format!("{kind} x{count}"))
            .collect::<Vec<_>>()
            .join(", ");
        say(&format!(
            "line {:>5}  H{:<8} {:<26} {}",
            section.line,
            section.code,
            section.name.unwrap_or("?"),
            if records.is_empty() {
                "no records".to_string()
            } else {
                records
            }
        ));
    }
    match &file.end {
        Some(end) => say(&format!(
            "line {:>5}  *DFREND  {}",
            end.line,
            frame(&end.values)
        )),
        None => say(&format!(
            "line {:>5}  (no *DFREND: the file is incomplete)",
            file.lines
        )),
    }
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

fn money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", cents.abs() / 100, cents.abs() % 100)
}

fn two_paths(args: &[String]) -> Result<(&str, &str), String> {
    match (args.get(1), args.get(2)) {
        (Some(file), Some(config)) => Ok((file, config)),
        _ => Err(USAGE.to_string()),
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}
