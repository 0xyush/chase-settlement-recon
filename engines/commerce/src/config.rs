//! The transformation config: which Chase column feeds which recon field.
//!
//! This is the same JSON the dashboard sends (its `src/presets.js`), a flatter version of
//! hyperswitch-recon's `POST /transformations/v2/configs` request: a parsing config, skip
//! rules, the five main fields, metadata fields, and the unique key.

use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Deserialize)]
pub struct TransformationConfig {
    /// Display name, for example "Deposit Details".
    #[serde(default)]
    pub report: Option<String>,
    #[serde(default)]
    pub parsing_config: ParsingConfig,
    /// A row is skipped when every condition of at least one skip config matches.
    #[serde(default)]
    pub skip_configs: Vec<SkipConfig>,
    /// The five main fields. Absent or `{}` means no mapping yet, so rows are only parsed.
    #[serde(default, deserialize_with = "none_when_empty")]
    pub fields: Option<MainFields>,
    #[serde(default)]
    pub metadata_fields: Vec<MetadataField>,
    /// Fields that together identify one entry, for example `["order_id", "balance_direction"]`.
    #[serde(default)]
    pub unique_constraint: Vec<String>,
}

impl TransformationConfig {
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ParsingConfig {
    /// 0-based record that holds the column names. Chase reports use 2, which is line 3.
    #[serde(default)]
    pub header_row_index: usize,
    /// First cell of a closing line that is not data. Chase reports end with an `END` line,
    /// so `"END"` drops it before any row is counted.
    #[serde(default)]
    pub footer_marker: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkipConfig {
    #[serde(default)]
    pub conditions: Vec<SkipCondition>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkipCondition {
    /// Column name in the file.
    pub identifier: String,
    pub operator: SkipOperator,
    /// `null` matches an empty cell.
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipOperator {
    Equals,
    NotEquals,
    StartsWith,
    Contains,
    EndsWith,
}

impl SkipOperator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::NotEquals => "not_equals",
            Self::StartsWith => "starts_with",
            Self::Contains => "contains",
            Self::EndsWith => "ends_with",
        }
    }
}

/// The five fields every recon entry must have.
#[derive(Debug, Clone, Deserialize)]
pub struct MainFields {
    pub currency: FieldSpec,
    pub amount: FieldSpec,
    pub balance_direction: DirectionSpec,
    pub effective_at: DateSpec,
    pub order_id: FieldSpec,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldSpec {
    /// The column name in the Chase file.
    pub identifier: String,
    #[serde(default)]
    pub transformation_rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectionSpec {
    pub identifier: String,
    /// Cell values that mean money coming in, for example `["DP", "DO"]`.
    pub credit_values: Vec<String>,
    /// Cell values that mean money going out, for example `["RF"]`.
    pub debit_values: Vec<String>,
    #[serde(default)]
    pub transformation_rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DateSpec {
    pub identifier: String,
    pub date_format: DateFormat,
    #[serde(default)]
    pub transformation_rules: Vec<Rule>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DateFormat {
    pub order: DateOrder,
    pub delimiter: Delimiter,
}

impl DateFormat {
    /// Human form of the format, used in error messages: `M/D/YYYY`.
    pub fn pattern(self) -> String {
        let d = self.delimiter.as_char();
        match self.order {
            DateOrder::MonthDayYear => format!("M{d}D{d}YYYY"),
            DateOrder::DayMonthYear => format!("D{d}M{d}YYYY"),
            DateOrder::YearMonthDay => format!("YYYY{d}M{d}D"),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateOrder {
    MonthDayYear,
    DayMonthYear,
    YearMonthDay,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Delimiter {
    Slash,
    Hyphen,
    Dot,
}

impl Delimiter {
    pub fn as_char(self) -> char {
        match self {
            Self::Slash => '/',
            Self::Hyphen => '-',
            Self::Dot => '.',
        }
    }
}

/// A transformation rule, run on the raw cell before it is parsed.
/// Same names as hyperswitch-recon's rules.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Rule {
    Trim,
    /// Drops a leading minus sign: "-20" becomes "20".
    Absolute,
    /// Flips the sign: "-20" becomes "20" and "20" becomes "-20".
    Negate,
    /// Replaces the value with `then` when it starts with `prefix`, otherwise with `otherwise`.
    StartsWith {
        prefix: String,
        then: String,
        otherwise: String,
    },
    /// Keeps capture group `group` of the first match (the whole match when absent).
    Regex {
        pattern: String,
        #[serde(default)]
        group: Option<usize>,
    },
    /// Fills a missing cell. An empty string is not missing.
    DefaultValue {
        value: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetadataField {
    /// The column name in the Chase file.
    pub identifier: String,
    /// Our clean name: the key in the stored metadata, used by recon rules as `metadata.<name>`.
    pub field_name: String,
    #[serde(default)]
    pub required: bool,
    /// Also stored unencrypted, so reports can read it.
    #[serde(default)]
    pub exposed: bool,
    /// Also searchable. Implies exposed in the real recon service.
    #[serde(default)]
    pub searchable: bool,
    #[serde(default, rename = "type")]
    pub value_type: MetadataType,
    #[serde(default)]
    pub transformation_rules: Vec<Rule>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataType {
    #[default]
    String,
    /// A decimal amount such as "-0.46". Checked, but stored as text.
    MajorUnit,
}

/// `"fields": {}` and `"fields": null` both mean "no mapping yet".
fn none_when_empty<'de, D>(deserializer: D) -> Result<Option<MainFields>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match &value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Object(map) if map.is_empty() => Ok(None),
        _ => serde_json::from_value(value)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_fields_mean_no_mapping() {
        let config = TransformationConfig::from_json(r#"{"fields": {}}"#).unwrap();
        assert!(config.fields.is_none());
        assert_eq!(config.parsing_config.header_row_index, 0);
    }

    #[test]
    fn shipped_configs_load() {
        for name in [
            "deposit_details",
            "transaction_fee_details",
            "funded_txn_fee_details",
        ] {
            let path = format!("{}/configs/{name}.json", env!("CARGO_MANIFEST_DIR"));
            let text = std::fs::read_to_string(&path).unwrap();
            let config = TransformationConfig::from_json(&text).unwrap();
            assert!(config.fields.is_some(), "{name} has no field mapping");
            assert_eq!(config.parsing_config.header_row_index, 2, "{name}");
        }
    }
}
