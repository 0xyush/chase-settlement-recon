//! Chase Orbital (Paymentech) settlement file -> recon staging entries -> matched against Hyperswitch.
//!
//! Same pipeline as the Commerce Center prototype (`../commerce`); only step 1
//! is new, because Orbital merchants on Paymentech reporting get Delimited File Reports (DFR)
//! instead of CSV reports with a header row.
//!
//! 1. [`dfr`]: check the file's `*DFRBEG` / `*DFREND` frame, find the report section the config
//!    reads (for example `ACT0010`), and name each field by its position in Chase's spec. New.
//! 2. [`skip`]: drop rows the config says to skip. Copied.
//! 3. [`transform`]: map fields to the five main fields + metadata, validate, build entries.
//!    Copied, calling [`dfr`] instead of the CSV parser.
//! 4. [`report`]: write the staging entries (JSON) and a per-row report (CSV). Copied.
//! 5. [`matching`]: compare the entries with a Hyperswitch export. Copied.
//! 6. [`tieout`]: check the staged detail adds up to Chase's own FIN0010 summary in the same
//!    file, and show what is settled by Chase versus conveyed (paid by Amex directly). New.
//!
//! [`config`] holds the JSON config that drives steps 1 to 3; [`rules`] the small value steps;
//! [`sample`] builds the synthetic sample files.

pub mod api;
pub mod config;
pub mod dfr;
pub mod matching;
pub mod report;
pub mod rules;
pub mod sample;
#[cfg(feature = "server")]
pub mod server;
pub mod skip;
pub mod steps;
pub mod tieout;
pub mod transform;

pub use config::TransformationConfig;
pub use transform::{TransformOutput, transform};
