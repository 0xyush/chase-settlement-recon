//! The two Rust recon engines as WebAssembly functions for the dashboards.
//!
//! Same answers as the engines' local HTTP API (`/api/transform`, `/api/export`, `/api/match`),
//! with JSON strings in and out, so the dashboards' JavaScript barely changes. Built by
//! `scripts/build.sh` with `wasm-bindgen --target web`.

use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;

fn json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        serde_json::json!({ "rejected": format!("EngineFailed: {error}") }).to_string()
    })
}

fn parse(text: &str, what: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|error| format!("InvalidConfig: {what}: {error}"))
}

fn rejected(reason: String) -> String {
    serde_json::json!({ "engine": "rust", "rejected": reason, "results": null }).to_string()
}

fn format_of(format: &str) -> Result<chase_recon::api::ExportFormat, String> {
    serde_json::from_value(Value::String(format.to_string()))
        .map_err(|_| format!("unknown export format {format:?}"))
}

// ---- Commerce Center (CSV reports) ----

/// Transform a Commerce Center CSV report. Returns the transform answer as JSON.
#[wasm_bindgen]
pub fn commerce_transform(csv: &str, config_json: &str) -> String {
    match parse(config_json, "config") {
        Ok(config) => json(&chase_recon::api::transform_file(csv, config)),
        Err(reason) => rejected(reason),
    }
}

/// `format`: `staging_json` or `report_csv`.
#[wasm_bindgen]
pub fn commerce_export(csv: &str, config_json: &str, format: &str) -> Result<String, JsError> {
    let config = parse(config_json, "config").map_err(|e| JsError::new(&e))?;
    let format = format_of(format).map_err(|e| JsError::new(&e))?;
    chase_recon::api::export(csv, config, format).map_err(|e| JsError::new(&e))
}

/// Match staged entries (a JSON array) against a Hyperswitch export CSV.
#[wasm_bindgen]
pub fn commerce_match(entries_json: &str, hyperswitch_csv: &str) -> String {
    match serde_json::from_str::<Vec<chase_recon::transform::StagingEntry>>(entries_json) {
        Ok(entries) => json(&chase_recon::api::match_with_hyperswitch(
            &entries,
            hyperswitch_csv,
        )),
        Err(error) => rejected(format!("entries: {error}")),
    }
}

// ---- Orbital (Delimited File Reports) ----

/// Transform a Chase Orbital DFR file. Returns the transform answer (with tie-out) as JSON.
#[wasm_bindgen]
pub fn orbital_transform(text: &str, config_json: &str) -> String {
    match parse(config_json, "config") {
        Ok(config) => json(&orbital_recon::api::transform_file(text, config)),
        Err(reason) => rejected(reason),
    }
}

/// `format`: `staging_json` or `report_csv`.
#[wasm_bindgen]
pub fn orbital_export(text: &str, config_json: &str, format: &str) -> Result<String, JsError> {
    let config = parse(config_json, "config").map_err(|e| JsError::new(&e))?;
    let format: orbital_recon::api::ExportFormat =
        serde_json::from_value(Value::String(format.to_string()))
            .map_err(|_| JsError::new(&format!("unknown export format {format:?}")))?;
    orbital_recon::api::export(text, config, format).map_err(|e| JsError::new(&e))
}

/// Match staged entries (a JSON array) against a Hyperswitch export CSV.
#[wasm_bindgen]
pub fn orbital_match(entries_json: &str, hyperswitch_csv: &str) -> String {
    match serde_json::from_str::<Vec<orbital_recon::transform::StagingEntry>>(entries_json) {
        Ok(entries) => json(&orbital_recon::api::match_with_hyperswitch(
            &entries,
            hyperswitch_csv,
        )),
        Err(error) => rejected(format!("entries: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commerce_and_orbital_answer_through_json() {
        let root = env!("CARGO_MANIFEST_DIR");
        let csv = std::fs::read_to_string(format!(
            "{root}/../engines/commerce/samples/Deposit_Details_Preset_Report.csv"
        ))
        .unwrap();
        let config = std::fs::read_to_string(format!(
            "{root}/../engines/commerce/configs/deposit_details.json"
        ))
        .unwrap();
        let answer: Value = serde_json::from_str(&commerce_transform(&csv, &config)).unwrap();
        assert_eq!(answer["results"].as_array().unwrap().len(), 825);
        assert_eq!(answer["steps"].as_array().unwrap().len(), 5);

        let dfr = std::fs::read_to_string(format!(
            "{root}/../engines/orbital/samples/0000999999.261001.d.B101.dfr"
        ))
        .unwrap();
        let config = std::fs::read_to_string(format!(
            "{root}/../engines/orbital/configs/act0010_deposit_detail.json"
        ))
        .unwrap();
        let answer: Value = serde_json::from_str(&orbital_transform(&dfr, &config)).unwrap();
        assert_eq!(answer["results"].as_array().unwrap().len(), 640);
        assert_eq!(answer["tieout"]["tied"], true);

        let bad: Value = serde_json::from_str(&orbital_transform(&dfr, "{nope")).unwrap();
        assert!(
            bad["rejected"]
                .as_str()
                .unwrap()
                .starts_with("InvalidConfig")
        );
    }
}
