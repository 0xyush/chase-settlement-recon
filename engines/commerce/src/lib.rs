//! Chase settlement file -> recon staging entries -> matched against Hyperswitch.
//!
//! The steps mirror hyperswitch-recon's transformation engine:
//!
//! 1. [`parser`]: read the CSV, starting from the configured header row.
//! 2. [`skip`]: drop rows the config says to skip (the `END` trailer, approval-only rows).
//! 3. [`transform`]: map columns to the five main fields + metadata, validate, build entries.
//! 4. [`report`]: write the staging entries (JSON) and a per-row report (CSV).
//! 5. [`matching`]: compare the entries with a Hyperswitch export.
//!
//! [`config`] holds the JSON config that drives steps 1 to 3; [`rules`] the small value steps.

pub mod api;
pub mod config;
pub mod matching;
pub mod parser;
pub mod report;
pub mod rules;
#[cfg(feature = "server")]
pub mod server;
pub mod skip;
pub mod steps;
pub mod transform;

pub use config::TransformationConfig;
pub use transform::{TransformOutput, transform};
