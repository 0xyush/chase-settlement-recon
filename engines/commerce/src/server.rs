//! The local HTTP API (optional, feature `server`). The dashboards normally run the engine in the
//! browser as WebAssembly; this server is for the command line and local debugging.
//!
//! - `GET  /api/health`    which engine is answering
//! - `POST /api/transform` `{csv, config}` -> the transform result and its steps
//! - `POST /api/export`    `{csv, config, format}` -> `staging_json` or `report_csv` as a file body
//! - `POST /api/match`     `{entries, hyperswitch_csv}` -> match summary, items and a CSV report

use axum::extract::DefaultBodyLimit;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::api::{self, ExportFormat, MatchResponse, TransformResponse};
use crate::transform::StagingEntry;

const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/transform", post(transform_file))
        .route("/api/export", post(export))
        .route("/api/match", post(match_with_hyperswitch))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}

async fn health() -> Json<Value> {
    Json(json!({"engine": "rust", "version": env!("CARGO_PKG_VERSION")}))
}

#[derive(Deserialize)]
struct TransformRequest {
    /// The CSV report's text.
    csv: String,
    config: Value,
}

/// Runs the pipeline off the async threads.
async fn run(request: TransformRequest) -> TransformResponse {
    tokio::task::spawn_blocking(move || api::transform_file(&request.csv, request.config))
        .await
        .unwrap_or_else(|error| TransformResponse::failed(format!("EngineFailed: {error}")))
}

async fn transform_file(Json(request): Json<TransformRequest>) -> Json<TransformResponse> {
    Json(run(request).await)
}

#[derive(Deserialize)]
struct ExportRequest {
    csv: String,
    config: Value,
    format: ExportFormat,
}

async fn export(Json(request): Json<ExportRequest>) -> Response {
    let format = request.format;
    let result =
        tokio::task::spawn_blocking(move || api::export(&request.csv, request.config, format))
            .await
            .unwrap_or_else(|error| Err(format!("EngineFailed: {error}")));
    match result {
        Ok(body) => ([(header::CONTENT_TYPE, format.content_type())], body).into_response(),
        Err(reason) => (StatusCode::UNPROCESSABLE_ENTITY, reason).into_response(),
    }
}

#[derive(Deserialize)]
struct MatchRequest {
    entries: Vec<StagingEntry>,
    hyperswitch_csv: String,
}

async fn match_with_hyperswitch(Json(request): Json<MatchRequest>) -> Json<MatchResponse> {
    Json(api::match_with_hyperswitch(
        &request.entries,
        &request.hyperswitch_csv,
    ))
}
