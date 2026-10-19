//! Step 1: read a Chase report into rows.
//!
//! Chase files start with a `BEGIN` line and a metadata line, so the column names sit on
//! line 3, not line 1. `header_row_index` says which record holds them (0-based). This is the
//! option hyperswitch-recon's CSV parser is missing and its XLSX parser already has.

use indexmap::IndexMap;
use serde::Serialize;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// One data row: the line it starts on in the file, and its cells by column name.
/// An empty cell is `None`, never `Some("")`.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedRow {
    pub line: u64,
    pub fields: IndexMap<String, Option<String>>,
}

impl ParsedRow {
    /// `None`: the file has no such column. `Some(None)`: the column exists, the cell is empty.
    pub fn get(&self, column: &str) -> Option<Option<&str>> {
        self.fields.get(column).map(Option::as_deref)
    }
}

/// A data line either parses into a row or is rejected on its own. One bad line never stops the rest.
#[derive(Debug)]
pub enum RowParse {
    Parsed(ParsedRow),
    Rejected { line: u64, message: String },
}

/// The whole file is either usable, or rejected because it has no header to map onto.
#[derive(Debug)]
pub enum FileParse {
    Parsed {
        header: Vec<String>,
        rows: Vec<RowParse>,
    },
    Rejected {
        reason: String,
    },
}

pub fn parse_report(
    input: &[u8],
    header_row_index: usize,
    footer_marker: Option<&str>,
) -> FileParse {
    let input = input.strip_prefix(UTF8_BOM).unwrap_or(input);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // we pick the header record ourselves
        .flexible(true) // the BEGIN / metadata / END lines have fewer columns
        .from_reader(input);
    let mut records = reader.records();

    // `nth` walks past the lines before the header and returns the header itself.
    let header: Vec<String> = match records.nth(header_row_index) {
        Some(Ok(record)) => record.iter().map(str::to_string).collect(),
        Some(Err(error)) => {
            return FileParse::Rejected {
                reason: format!("InvalidFormat: {error}"),
            };
        }
        None => {
            return FileParse::Rejected {
                reason: format!(
                    "HeaderNotFound: header_row_index {header_row_index} is past the end of the file"
                ),
            };
        }
    };

    // Drop the closing line(s), such as Chase's `END`, the same way the lines above the
    // header are dropped: they frame the file, they are not data.
    let mut records: Vec<_> = records.collect();
    if let Some(marker) = footer_marker {
        while records
            .last()
            .is_some_and(|last| matches!(last, Ok(record) if record.get(0) == Some(marker)))
        {
            records.pop();
        }
    }

    let rows = records
        .into_iter()
        .enumerate()
        .map(|(index, record)| {
            let fallback = u64::try_from(header_row_index + index + 2).unwrap_or(u64::MAX);
            match record {
                Ok(record) => RowParse::Parsed(ParsedRow {
                    line: line_number(input, record.position(), fallback),
                    fields: fields_by_name(&header, &record),
                }),
                Err(error) => RowParse::Rejected {
                    line: line_number(input, error.position(), fallback),
                    message: format!("ParseError: {error}"),
                },
            }
        })
        .collect();

    FileParse::Parsed { header, rows }
}

/// Pairs each cell with its column name. Columns with no name are dropped; when two columns
/// share a name, the first one wins.
fn fields_by_name(
    header: &[String],
    record: &csv::StringRecord,
) -> IndexMap<String, Option<String>> {
    header
        .iter()
        .enumerate()
        .filter(|(_, name)| !name.is_empty())
        .fold(
            IndexMap::with_capacity(header.len()),
            |mut fields, (column, name)| {
                let value = record
                    .get(column)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string);
                fields.entry(name.clone()).or_insert(value);
                fields
            },
        )
}

/// The file line a record starts on. Same correction as hyperswitch-recon's
/// `csv_record_line_number`: with CRLF endings the csv crate can stop a record on `\r` and leave
/// the `\n` pending, which would otherwise make the line number one short.
fn line_number(input: &[u8], position: Option<&csv::Position>, fallback: u64) -> u64 {
    position.map_or(fallback, |position| {
        let byte = usize::try_from(position.byte()).unwrap_or(usize::MAX);
        let pending_line_feeds = input
            .get(byte..)
            .unwrap_or_default()
            .iter()
            .take_while(|b| matches!(**b, b'\r' | b'\n'))
            .filter(|b| **b == b'\n')
            .count();
        position.line() + u64::try_from(pending_line_feeds).unwrap_or(0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHASE_LIKE: &str = "\u{feff}BEGIN,EntityId=1,Frequency=adhoc\n\
        EntityId=1,ReportTypeName=SubmissionDetails\n\
        Submission Date,Merchant Order Number,Amount,Note\n\
        3/3/2025,ord_1,20,\n\
        3/3/2025,ord_2,-5,\"has, comma\"\n\
        END,EntityId=1,Frequency=adhoc\n";

    fn parsed(input: &str, header_row_index: usize) -> (Vec<String>, Vec<ParsedRow>) {
        match parse_report(input.as_bytes(), header_row_index, Some("END")) {
            FileParse::Parsed { header, rows } => (
                header,
                rows.into_iter()
                    .map(|row| match row {
                        RowParse::Parsed(row) => row,
                        RowParse::Rejected { message, .. } => panic!("row rejected: {message}"),
                    })
                    .collect(),
            ),
            FileParse::Rejected { reason } => panic!("file rejected: {reason}"),
        }
    }

    #[test]
    fn header_row_index_skips_the_preamble() {
        let (header, rows) = parsed(CHASE_LIKE, 2);
        assert_eq!(
            header,
            ["Submission Date", "Merchant Order Number", "Amount", "Note"]
        );
        assert_eq!(rows.len(), 2, "two data rows; the END line is dropped");
        assert_eq!(rows[0].get("Merchant Order Number"), Some(Some("ord_1")));
    }

    #[test]
    fn line_numbers_are_file_lines() {
        let (_, rows) = parsed(CHASE_LIKE, 2);
        let lines: Vec<u64> = rows.iter().map(|row| row.line).collect();
        assert_eq!(lines, [4, 5]);
    }

    #[test]
    fn footer_stays_when_no_marker_is_set() {
        match parse_report(CHASE_LIKE.as_bytes(), 2, None) {
            FileParse::Parsed { rows, .. } => assert_eq!(rows.len(), 3),
            FileParse::Rejected { reason } => panic!("{reason}"),
        }
    }

    #[test]
    fn crlf_line_numbers_match_lf() {
        let crlf = CHASE_LIKE.replace('\n', "\r\n");
        let (_, rows) = parsed(&crlf, 2);
        let lines: Vec<u64> = rows.iter().map(|row| row.line).collect();
        assert_eq!(lines, [4, 5]);
    }

    #[test]
    fn empty_cell_is_none_and_quotes_are_respected() {
        let (_, rows) = parsed(CHASE_LIKE, 2);
        assert_eq!(rows[0].get("Note"), Some(None));
        assert_eq!(rows[1].get("Note"), Some(Some("has, comma")));
        assert_eq!(rows[0].get("No Such Column"), None);
    }

    #[test]
    fn byte_order_mark_is_stripped() {
        let (header, _) = parsed("\u{feff}Order,Amount\nA,1\n", 0);
        assert_eq!(header[0], "Order");
    }

    #[test]
    fn header_past_the_end_rejects_the_file() {
        assert!(matches!(
            parse_report(CHASE_LIKE.as_bytes(), 50, None),
            FileParse::Rejected { reason } if reason.starts_with("HeaderNotFound")
        ));
    }
}
