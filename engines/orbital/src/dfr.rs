//! Step 1: read a Chase Delimited File Report (DFR), the file Orbital / Paymentech merchants get.
//!
//! A DFR file is not one table with a header row. It is a stack of lines, and the first field
//! of every line says what the line is:
//!
//! ```text
//! *DFRBEG,PID=888888,FREQ=DAILY,CO=999999         file header: who the file is for, how often
//! HFIN0010,999999,10/01/2026,10/01/2026,...       a report section starts (H + report code)
//! RFIN0010,TD,...                                  that report's data records (R + report code)
//! HACT0010,999999,10/01/2026,10/01/2026,...       the next section
//! RACT0010,10/01/2026,888888,"SAMPLE",...          its data records
//! *DFREND,PID=888888,FREQ=DAILY,CO=999999         file trailer: repeats the header's values
//! ```
//!
//! Reading one is three checks and a lookup:
//! 1. the first line is `*DFRBEG`, and the character after it is the delimiter (`,` `|` or tab);
//! 2. the last line is `*DFREND` with the same values, or the file was cut off in transfer;
//! 3. every line in between is a section header (`H…`) or a data record (`R…`);
//! 4. keep the data records the config asks for and name each field by its position, using the
//!    column list from Chase's spec. Nothing else in the pipeline knows the file was a DFR.

use indexmap::IndexMap;
use serde::Serialize;

use crate::config::ParsingConfig;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const FILE_HEADER: &str = "*DFRBEG";
const FILE_TRAILER: &str = "*DFREND";

/// One data record: the line it is on, and its fields by name.
/// An empty field is `None`, never `Some("")`.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedRow {
    pub line: u64,
    pub fields: IndexMap<String, Option<String>>,
}

impl ParsedRow {
    /// `None`: no such column. `Some(None)`: the column exists, the field is empty.
    pub fn get(&self, column: &str) -> Option<Option<&str>> {
        self.fields.get(column).map(Option::as_deref)
    }
}

/// A data line either reads into a row or is rejected on its own. One bad line never stops the rest.
#[derive(Debug)]
pub enum RowParse {
    Parsed(ParsedRow),
    Rejected { line: u64, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldDelimiter {
    Comma,
    Pipe,
    Tab,
}

impl FieldDelimiter {
    pub fn as_char(self) -> char {
        match self {
            Self::Comma => ',',
            Self::Pipe => '|',
            Self::Tab => '\t',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Comma => "comma",
            Self::Pipe => "pipe",
            Self::Tab => "tab",
        }
    }

    fn from_char(c: char) -> Option<Self> {
        match c {
            ',' => Some(Self::Comma),
            '|' => Some(Self::Pipe),
            '\t' => Some(Self::Tab),
            _ => None,
        }
    }

    /// Config spelling: `","`, `"|"`, `"\t"`, or the names `comma`, `pipe`, `tab`.
    fn from_config(text: &str) -> Option<Self> {
        match text {
            "comma" => Some(Self::Comma),
            "pipe" => Some(Self::Pipe),
            "tab" => Some(Self::Tab),
            _ => {
                let mut chars = text.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Self::from_char(c),
                    _ => None,
                }
            }
        }
    }
}

/// The `*DFRBEG` or `*DFREND` line: `PID=888888, FREQ=DAILY, CO=999999`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    pub line: u64,
    pub values: IndexMap<String, String>,
}

/// One report in the file: its `H` line and how many data records of each type follow it.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    /// The report code, for example `ACT0010`.
    pub code: String,
    /// Chase's name for it, for example "Deposit Detail", when it is one we know.
    pub name: Option<&'static str>,
    pub line: u64,
    pub company_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    /// Generation date and time, Eastern time.
    pub generated: Option<String>,
    /// Data record type -> count, for example `RACT0010` -> 612.
    pub records: IndexMap<String, usize>,
}

/// What the file holds. Returned even when the file is rejected, so you can see why.
#[derive(Debug, Clone, Serialize)]
pub struct DfrFile {
    pub delimiter: FieldDelimiter,
    pub begin: Frame,
    /// `None` when the last line is not `*DFREND`.
    pub end: Option<Frame>,
    pub sections: Vec<Section>,
    pub lines: u64,
    /// Lines that are none of the above. A good file has none.
    pub unexpected_lines: Vec<u64>,
}

impl DfrFile {
    pub fn section(&self, code: &str) -> Option<&Section> {
        self.sections.iter().find(|section| section.code == code)
    }
}

#[derive(Debug)]
pub enum FileRead {
    Parsed {
        file: DfrFile,
        /// Field names in order: the config's columns, then `field_N` for any extra fields.
        header: Vec<String>,
        rows: Vec<RowParse>,
    },
    Rejected {
        reason: String,
        file: Option<DfrFile>,
    },
}

fn rejected(reason: String, file: Option<DfrFile>) -> FileRead {
    FileRead::Rejected { reason, file }
}

pub fn read(input: &[u8], parsing: &ParsingConfig) -> FileRead {
    let input = input.strip_prefix(UTF8_BOM).unwrap_or(input);
    let text = String::from_utf8_lossy(input);
    let lines = numbered_lines(&text);

    // Check 1: the file header, and the delimiter it reveals.
    let Some(&(_, first)) = lines.first() else {
        return rejected("EmptyFile: the file has no lines".to_string(), None);
    };
    let Some(after_header) = first.strip_prefix(FILE_HEADER) else {
        let start: String = first.chars().take(12).collect();
        return rejected(
            format!(
                "NotADfrFile: the first line starts with \"{start}\", not {FILE_HEADER}. Chase's \
                 Delimited File Reports always open with {FILE_HEADER}."
            ),
            None,
        );
    };
    let delimiter = match delimiter_of(after_header, parsing.delimiter.as_deref()) {
        Ok(delimiter) => delimiter,
        Err(reason) => return rejected(reason, None),
    };
    let file = scan(&lines, delimiter);

    // Check 2: the file trailer, last, agreeing with the header.
    let Some(end) = &file.end else {
        let last = lines.last().map_or(0, |(number, _)| *number);
        return rejected(
            format!(
                "Incomplete: the last line ({last}) is not {FILE_TRAILER}, so the file was \
                 probably cut off in transfer. Not processed: fetch it again."
            ),
            Some(file),
        );
    };
    if end.values != file.begin.values {
        let reason = format!(
            "FrameMismatch: {FILE_HEADER} says {}, {FILE_TRAILER} says {}",
            frame_text(&file.begin),
            frame_text(end)
        );
        return rejected(reason, Some(file));
    }

    // Check 3: nothing in between that is not a section header or a data record.
    if let Some(first_bad) = file.unexpected_lines.first() {
        let reason = format!(
            "UnexpectedLines: {} line(s) are not DFR records, first on line {first_bad}",
            file.unexpected_lines.len()
        );
        return rejected(reason, Some(file));
    }

    // The section this config reads.
    if file.section(&parsing.section).is_none() {
        let found: Vec<&str> = file.sections.iter().map(|s| s.code.as_str()).collect();
        let reason = format!(
            "SectionNotFound: this file has no {} section (it has: {}). Each day Chase sends a B \
             file with the financial reports such as ACT0010 and FIN0010, and an A file with the \
             exception reports such as PDE0017.",
            parsing.section,
            if found.is_empty() {
                "none".to_string()
            } else {
                found.join(", ")
            }
        );
        return rejected(reason, Some(file));
    }

    // The lookup: the data records the config asks for, fields named by position.
    let names = Columns::new(&parsing.columns);
    let rows: Vec<RowParse> = lines
        .iter()
        .filter(|(_, line)| first_field(line, delimiter) == parsing.record_type)
        .map(|&(number, line)| names.row(number, line, delimiter))
        .collect();
    let header = names.header(&rows);
    FileRead::Parsed { file, header, rows }
}

/// The file's frame and sections only, with no config: what `orbital_recon scan` prints.
/// Fails only when the file is not a DFR file at all.
pub fn scan_only(input: &[u8]) -> Result<DfrFile, String> {
    let input = input.strip_prefix(UTF8_BOM).unwrap_or(input);
    let text = String::from_utf8_lossy(input);
    let lines = numbered_lines(&text);
    let first = lines.first().map_or("", |(_, line)| *line);
    let after_header = first
        .strip_prefix(FILE_HEADER)
        .ok_or_else(|| format!("NotADfrFile: the first line is not {FILE_HEADER}"))?;
    Ok(scan(&lines, delimiter_of(after_header, None)?))
}

/// Every non-blank line with its 1-based line number. Handles `\n` and `\r\n` endings.
fn numbered_lines(text: &str) -> Vec<(u64, &str)> {
    text.split('\n')
        .zip(1..)
        .map(|(line, number)| (number, line.strip_suffix('\r').unwrap_or(line)))
        .filter(|(_, line)| !line.trim().is_empty())
        .collect()
}

/// The character right after `*DFRBEG` is the delimiter. A config may pin one; then they must agree.
fn delimiter_of(after_header: &str, configured: Option<&str>) -> Result<FieldDelimiter, String> {
    let next = after_header.chars().next();
    let found = next.and_then(FieldDelimiter::from_char).ok_or_else(|| {
        format!(
            "UnknownDelimiter: {FILE_HEADER} is followed by {next:?}; Chase uses comma, pipe or tab"
        )
    })?;
    let Some(configured) = configured else {
        return Ok(found);
    };
    let wanted = FieldDelimiter::from_config(configured).ok_or_else(|| {
        format!("InvalidConfig: delimiter {configured:?} is not \",\", \"|\" or a tab")
    })?;
    if wanted == found {
        Ok(found)
    } else {
        Err(format!(
            "DelimiterMismatch: the config says {}, the file uses {}",
            wanted.name(),
            found.name()
        ))
    }
}

/// One pass over the lines: the frame, each section and its record counts, anything unexpected.
fn scan(lines: &[(u64, &str)], delimiter: FieldDelimiter) -> DfrFile {
    let d = delimiter.as_char();
    let last_number = lines.last().map_or(0, |(number, _)| *number);
    let mut begin = None;
    let mut end = None;
    let mut sections: Vec<Section> = Vec::new();
    let mut unexpected_lines = Vec::new();

    for (index, &(number, line)) in lines.iter().enumerate() {
        let code = first_field(line, delimiter);
        match code {
            FILE_HEADER if index == 0 => begin = Some(frame(number, line, d)),
            FILE_TRAILER if number == last_number => end = Some(frame(number, line, d)),
            _ if code.starts_with('H') && code.len() > 1 => {
                sections.push(section(number, &code[1..], line, d));
            }
            _ if code.starts_with('R') && !sections.is_empty() => {
                if let Some(current) = sections.last_mut() {
                    *current.records.entry(code.to_string()).or_default() += 1;
                }
            }
            _ => unexpected_lines.push(number),
        }
    }

    DfrFile {
        delimiter,
        // `read` only calls `scan` once line 1 is known to be `*DFRBEG`.
        begin: begin.unwrap_or(Frame {
            line: 1,
            values: IndexMap::new(),
        }),
        end,
        sections,
        lines: last_number,
        unexpected_lines,
    }
}

/// The record code: everything before the first delimiter. It is never quoted.
fn first_field(line: &str, delimiter: FieldDelimiter) -> &str {
    line.split(delimiter.as_char())
        .next()
        .unwrap_or_default()
        .trim()
}

/// `*DFRBEG,PID=888888,FREQ=DAILY,CO=999999` -> {PID: 888888, FREQ: DAILY, CO: 999999}.
fn frame(line_number: u64, line: &str, delimiter: char) -> Frame {
    let values = line
        .split(delimiter)
        .skip(1)
        .filter_map(|part| part.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect();
    Frame {
        line: line_number,
        values,
    }
}

fn frame_text(frame: &Frame) -> String {
    frame
        .values
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A section header: `HACT0010,999999,10/01/2026,10/01/2026,10/02/2026,02:15:01`.
fn section(line_number: u64, code: &str, line: &str, delimiter: char) -> Section {
    let fields: Vec<&str> = line.split(delimiter).map(str::trim).collect();
    let field = |index: usize| {
        fields
            .get(index)
            .filter(|value| !value.is_empty())
            .map(|value| (*value).to_string())
    };
    let generated = match (field(4), field(5)) {
        (Some(date), Some(time)) => Some(format!("{date} {time}")),
        (date, _) => date,
    };
    Section {
        code: code.to_string(),
        name: report_name(code),
        line: line_number,
        company_id: field(1),
        date_from: field(2),
        date_to: field(3),
        generated,
        records: IndexMap::new(),
    }
}

/// Field names by position, from the config. Fields past the list are named `field_N`.
struct Columns<'a> {
    names: &'a [String],
    /// `false` for a position whose name already appeared earlier: the first one wins.
    first_use: Vec<bool>,
}

impl<'a> Columns<'a> {
    fn new(names: &'a [String]) -> Self {
        let first_use = names
            .iter()
            .enumerate()
            .map(|(index, name)| !names[..index].contains(name))
            .collect();
        Self { names, first_use }
    }

    fn row(&self, line: u64, text: &str, delimiter: FieldDelimiter) -> RowParse {
        let cells = match split(text, delimiter.as_char()) {
            Ok(cells) => cells,
            Err(message) => return RowParse::Rejected { line, message },
        };
        // Every named column is present, empty unless the line fills it, so a short line
        // (an older layout without the last optional fields) reads as empty fields.
        let mut fields: IndexMap<String, Option<String>> =
            self.names.iter().map(|name| (name.clone(), None)).collect();
        for (index, cell) in cells.into_iter().enumerate() {
            // DFR pads some fields: a positive amount is written " 38.35".
            let value = Some(cell.trim().to_string()).filter(|value| !value.is_empty());
            match self.names.get(index) {
                Some(name) if self.first_use[index] => {
                    fields.insert(name.clone(), value);
                }
                Some(_) => {}
                // An empty field past the spec is the trailing delimiter some files end lines
                // with; only a field with a value gets a `field_N` column.
                None if value.is_some() => {
                    fields.insert(format!("field_{}", index + 1), value);
                }
                None => {}
            }
        }
        RowParse::Parsed(ParsedRow { line, fields })
    }

    /// The header: the configured names, then any `field_N` the rows needed, in position order.
    fn header(&self, rows: &[RowParse]) -> Vec<String> {
        let mut extra: Vec<usize> = rows
            .iter()
            .filter_map(|row| match row {
                RowParse::Parsed(row) => Some(row),
                RowParse::Rejected { .. } => None,
            })
            .flat_map(|row| row.fields.keys().skip(self.names.len()))
            .filter_map(|name| name.strip_prefix("field_")?.parse().ok())
            .collect();
        extra.sort_unstable();
        extra.dedup();
        self.names
            .iter()
            .cloned()
            .chain(
                extra
                    .into_iter()
                    .map(|position| format!("field_{position}")),
            )
            .collect()
    }
}

/// Splits one line on the delimiter. Chase double-quotes free-text fields such as the merchant
/// order number, and a quoted field may contain the delimiter: `"T1,b36046,154"`.
/// A doubled quote inside quotes is a literal quote.
fn split(line: &str, delimiter: char) -> Result<Vec<String>, String> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    cell.push('"');
                    chars.next();
                }
                '"' => quoted = false,
                c => cell.push(c),
            }
        } else if c == delimiter {
            cells.push(std::mem::take(&mut cell));
        } else if c == '"' && cell.trim().is_empty() {
            cell.clear();
            quoted = true;
        } else {
            cell.push(c);
        }
    }
    if quoted {
        return Err("ParseError: a double quote is opened and never closed".to_string());
    }
    cells.push(cell);
    Ok(cells)
}

/// Chase's names for the reports in the DFR spec (v2.2).
pub fn report_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "FIN0010" => "Deposit Activity Summary",
        "FIN0010T" => "Deposit Activity Transfer Summary",
        "FIN0011" => "Service Charge Detail",
        "ACT0002" => "Exception Detail",
        "ACT0010" => "Deposit Detail",
        "ACT0012" => "Submission Listing",
        "ACT0019" => "Notification of Change",
        "ACT0027" => "Terminal/Batch Detail",
        "ACT0033" => "Reject Summary",
        "ACT0036" => "Authorization Detail",
        "ACT0062" => "E-File Image Upload Exception",
        "ANS0013" => "Interchange Qualification Detail",
        "ANS0016" => "Interchange Qualification Summary",
        "ANS0017" => "Interchange Downgrade Summary",
        "ANS0039" => "Debit Authorization Aging",
        "ANS0041" => "Merchant Settlement Aging",
        "ANS0048" => "Green Dot MoneyPak Authorization Aging",
        "PDE0017" => "Chargeback Activity",
        "PDE0018" => "ECP Returns",
        "PDE0020" => "Chargebacks Received",
        "PDE0022" => "ECP Returns Received",
        "PDE0029" => "Retrieval Detail",
        "PDE0036" => "Debit Adjustments",
        "RSK0030" => "MasterCard Excessive Chargeback",
        "RSK0036" => "Discover Chargeback Summary",
        "INF0042" => "Merchant FX Rate File",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(delimiter: Option<&str>) -> ParsingConfig {
        ParsingConfig {
            section: "ACT0010".to_string(),
            record_type: "RACT0010".to_string(),
            columns: [
                "Record Type",
                "Submission Date",
                "Merchant Order #",
                "Amount",
            ]
            .map(String::from)
            .to_vec(),
            delimiter: delimiter.map(String::from),
        }
    }

    const FILE: &str = "*DFRBEG,PID=888888,FREQ=DAILY,CO=999999\n\
        HFIN0010,999999,10/01/2026,10/01/2026,10/02/2026,02:15:01\n\
        RFIN0010,TD,1,2,3,USD,VI,SALE,S,1,20.00\n\
        HACT0010,999999,10/01/2026,10/01/2026,10/02/2026,02:15:01\n\
        RACT0010,10/01/2026,\"ord_1\",20.00\n\
        RACT0010,10/01/2026,\"T1,b36046,154\", 38.35,,,\n\
        RACT0010,10/01/2026,\"ord_3\"\n\
        RACT0010,10/01/2026,\"ord_4\",5.00,EXTRA\n\
        *DFREND,PID=888888,FREQ=DAILY,CO=999999\n";

    fn parsed(input: &str, config: &ParsingConfig) -> (DfrFile, Vec<String>, Vec<ParsedRow>) {
        match read(input.as_bytes(), config) {
            FileRead::Parsed { file, header, rows } => (
                file,
                header,
                rows.into_iter()
                    .map(|row| match row {
                        RowParse::Parsed(row) => row,
                        RowParse::Rejected { message, .. } => panic!("row rejected: {message}"),
                    })
                    .collect(),
            ),
            FileRead::Rejected { reason, .. } => panic!("file rejected: {reason}"),
        }
    }

    fn reason(input: &str, config: &ParsingConfig) -> String {
        match read(input.as_bytes(), config) {
            FileRead::Rejected { reason, .. } => reason,
            FileRead::Parsed { .. } => panic!("expected the file to be rejected"),
        }
    }

    #[test]
    fn keeps_only_the_configured_records_with_file_line_numbers() {
        let (_, _, rows) = parsed(FILE, &config(None));
        let lines: Vec<u64> = rows.iter().map(|row| row.line).collect();
        assert_eq!(
            lines,
            [5, 6, 7, 8],
            "the FIN0010 record and the frame are not rows"
        );
        assert_eq!(rows[0].get("Merchant Order #"), Some(Some("ord_1")));
    }

    #[test]
    fn quoted_field_keeps_its_delimiter_and_padding_is_trimmed() {
        let (_, _, rows) = parsed(FILE, &config(None));
        assert_eq!(rows[1].get("Merchant Order #"), Some(Some("T1,b36046,154")));
        assert_eq!(rows[1].get("Amount"), Some(Some("38.35")));
        assert_eq!(
            rows[1].fields.len(),
            4,
            "trailing empty fields add no columns"
        );
    }

    #[test]
    fn short_line_reads_as_empty_and_extra_field_is_kept() {
        let (_, header, rows) = parsed(FILE, &config(None));
        assert_eq!(rows[2].get("Amount"), Some(None));
        assert_eq!(rows[3].get("field_5"), Some(Some("EXTRA")));
        assert_eq!(header.last().map(String::as_str), Some("field_5"));
    }

    #[test]
    fn scan_lists_sections_and_record_counts() {
        let (file, _, _) = parsed(FILE, &config(None));
        let summary: Vec<(&str, usize)> = file
            .sections
            .iter()
            .map(|s| (s.code.as_str(), s.records.values().sum()))
            .collect();
        assert_eq!(summary, [("FIN0010", 1), ("ACT0010", 4)]);
        assert_eq!(file.sections[1].name, Some("Deposit Detail"));
        assert_eq!(
            file.begin.values.get("CO").map(String::as_str),
            Some("999999")
        );
        assert_eq!(file.delimiter, FieldDelimiter::Comma);
    }

    #[test]
    fn pipe_and_crlf_read_the_same() {
        let pipe = FILE
            .replace(',', "|")
            .replace("T1|b36046|154", "T1,b36046,154");
        let crlf = pipe.replace('\n', "\r\n");
        let (file, _, rows) = parsed(&crlf, &config(None));
        assert_eq!(file.delimiter, FieldDelimiter::Pipe);
        assert_eq!(rows[1].get("Merchant Order #"), Some(Some("T1,b36046,154")));
        assert_eq!(rows[1].line, 6);
    }

    #[test]
    fn missing_trailer_rejects_the_file() {
        let cut = FILE.replace("*DFREND,PID=888888,FREQ=DAILY,CO=999999\n", "");
        assert!(reason(&cut, &config(None)).starts_with("Incomplete"));
    }

    #[test]
    fn trailer_must_match_header() {
        let other = FILE.replace("*DFREND,PID=888888", "*DFREND,PID=111111");
        assert!(reason(&other, &config(None)).starts_with("FrameMismatch"));
    }

    #[test]
    fn not_a_dfr_file() {
        assert!(reason("Order,Amount\nA,1\n", &config(None)).starts_with("NotADfrFile"));
        assert!(reason("", &config(None)).starts_with("EmptyFile"));
    }

    #[test]
    fn missing_section_names_what_the_file_has() {
        let a_file =
            "*DFRBEG,PID=1,FREQ=DAILY,CO=2\nHPDE0017,2,a,b,c,d\n*DFREND,PID=1,FREQ=DAILY,CO=2\n";
        let reason = reason(a_file, &config(None));
        assert!(reason.starts_with("SectionNotFound"), "{reason}");
        assert!(reason.contains("it has: PDE0017"), "{reason}");
    }

    #[test]
    fn stray_line_rejects_the_file() {
        let stray = FILE.replace("HACT0010", "hello\nHACT0010");
        assert!(reason(&stray, &config(None)).starts_with("UnexpectedLines"));
    }

    #[test]
    fn configured_delimiter_must_match_the_file() {
        assert!(reason(FILE, &config(Some("|"))).starts_with("DelimiterMismatch"));
        let (_, _, rows) = parsed(FILE, &config(Some("comma")));
        assert_eq!(rows.len(), 4);
    }

    #[test]
    fn unclosed_quote_rejects_only_that_line() {
        let broken = FILE.replace("\"ord_3\"", "\"ord_3");
        match read(broken.as_bytes(), &config(None)) {
            FileRead::Parsed { rows, .. } => {
                assert!(matches!(&rows[2], RowParse::Rejected { line: 7, .. }));
                assert!(matches!(&rows[3], RowParse::Parsed(_)));
            }
            FileRead::Rejected { reason, .. } => panic!("{reason}"),
        }
    }

    #[test]
    fn empty_day_has_sections_and_no_rows() {
        let empty = "*DFRBEG,PID=1,FREQ=DAILY,CO=2\nHFIN0010,2,a,b,c,d\nHACT0010,2,a,b,c,d\n*DFREND,PID=1,FREQ=DAILY,CO=2\n";
        let (file, _, rows) = parsed(empty, &config(None));
        assert!(rows.is_empty());
        assert_eq!(file.sections.len(), 2);
    }
}
