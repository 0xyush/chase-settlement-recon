//! Synthetic DFR files for the demo and the tests.
//!
//! J.P. Morgan publishes sample files for Commerce Center reports only; there is no public
//! Orbital / Paymentech DFR sample. These files are made up, deterministically, from:
//!
//! - Chase's Delimited File Reports spec v2.2: record layouts, codes, quoting, number formats;
//! - public code that reads live DFR files: the pipe delimiter, quoted order numbers, positive
//!   amounts padded with a space, a trailing delimiter on data lines;
//! - the layout of a live ACT0010 seen in a real file (day 5): tab delimiter, field 22 filled,
//!   one field the spec does not list, fees with no leading zero, settlement currency last.
//!
//! No real card numbers: account numbers are masked test ranges. Replace these files with a real
//! one from the merchant's Chase account as soon as there is one.

pub const PID: &str = "888888";
pub const COMPANY: &str = "999999";
/// The transaction division (TD) the deposits belong to.
pub const DIVISION: &str = "777001";
const PID_SHORT_NAME: &str = "SAMPLE";
const SECURE_BANK_ACCOUNT: &str = "4401234";
/// Bank-account (ECP) money lands in its own account, apart from card money.
const ECP_BANK_ACCOUNT: &str = "4405678";
/// Routing number of the customer's bank on an ECP row (a Federal Reserve test number).
const ECP_RDFI: &str = "011000015";
/// Merchant category code 4784: tolls and bridge fees.
const MCC: &str = "4784";
const ORDER_ID_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";

pub struct SampleFile {
    pub name: String,
    pub about: &'static str,
    pub contents: String,
}

/// The six sample files, always identical.
pub fn all() -> Vec<SampleFile> {
    let day1 = Day::new(
        "10/01/2026",
        "09/30/2026",
        "10/02/2026",
        "26274",
        "307000101",
    );
    let day2 = Day::new(
        "10/02/2026",
        "10/01/2026",
        "10/03/2026",
        "26275",
        "307000102",
    );
    let day3 = Day::new(
        "10/03/2026",
        "10/02/2026",
        "10/04/2026",
        "26276",
        "307000103",
    );
    let day4 = Day::new(
        "10/04/2026",
        "10/03/2026",
        "10/05/2026",
        "26277",
        "307000104",
    );
    let day5 = Day::new(
        "10/05/2026",
        "10/04/2026",
        "10/06/2026",
        "26278",
        "307000105",
    );

    let comma = Layout {
        delimiter: ',',
        interchange: false,
        live: false,
        trailing_delimiter: false,
        line_end: "\r\n",
    };
    let pipe = Layout {
        delimiter: '|',
        interchange: true,
        live: false,
        trailing_delimiter: true,
        line_end: "\n",
    };
    let live = Layout {
        delimiter: '\t',
        interchange: true,
        live: true,
        trailing_delimiter: false,
        line_end: "\n",
    };

    let chargebacks = chargebacks(&mut Rng::new(17));
    let returned_to_merchant: i64 = chargebacks
        .iter()
        .filter(|c| c.category.financial())
        .map(|c| c.action_cents)
        .sum();
    let day1_txns = transactions(&mut Rng::new(1), 640, &day1, false);
    let day2_txns = transactions(&mut Rng::new(2), 480, &day2, false);
    let day4_txns = transactions(&mut Rng::new(4), 300, &day4, false);
    let day5_txns = transactions(&mut Rng::new(5), 400, &day5, true);

    let cut_off = financial_file(&day4, &day4_txns, &comma, 0);
    let cut_at = cut_off.len() * 45 / 100;

    vec![
        SampleFile {
            name: file_name(&day1, 'B', 101),
            about: "Day 1, financial file (B): FIN0010 deposit summary + ACT0010 deposit \
                    detail, comma-delimited, CRLF line ends. Amex lines are conveyed (C).",
            contents: financial_file(&day1, &day1_txns, &comma, returned_to_merchant),
        },
        SampleFile {
            name: file_name(&day1, 'A', 101),
            about: "Day 1, exception file (A): PDE0017 chargeback activity, summary (S) and \
                    detail (D) records. Only RTM and PARRTM move money.",
            contents: exception_file(&day1, &chargebacks, &comma),
        },
        SampleFile {
            name: file_name(&day2, 'B', 102),
            about: "Day 2, financial file (B) in another setup Chase offers: pipe-delimited, \
                    ACT0010 with the interchange detail option, a trailing pipe on data lines.",
            contents: financial_file(&day2, &day2_txns, &pipe, 0),
        },
        SampleFile {
            name: file_name(&day3, 'B', 103),
            about: "Day 3, no activity: the file header, each report header, the file trailer.",
            contents: financial_file(&day3, &[], &comma, 0),
        },
        SampleFile {
            name: file_name(&day4, 'B', 104),
            about: "Day 4, cut off in transfer: stops mid-line with no *DFREND trailer. \
                    Must be rejected, not half-processed.",
            contents: cut_off[..cut_at].to_string(),
        },
        SampleFile {
            name: file_name(&day5, 'B', 105),
            about: "Day 5, financial file (B) in the layout of a live ACT0010 file: tab-delimited, \
                    field 22 filled (1, 2 or 7), one extra field before the interchange fields, \
                    fees with no leading zero, settlement currency last. Has bank-account (ECP) \
                    rows with action code i. Read it with act0010_deposit_detail_live.json.",
            contents: financial_file(&day5, &day5_txns, &live, 0),
        },
    ]
}

struct Day {
    date: &'static str,
    prior: &'static str,
    generated: &'static str,
    julian: &'static str,
    funds_transfer: &'static str,
}

impl Day {
    fn new(
        date: &'static str,
        prior: &'static str,
        generated: &'static str,
        julian: &'static str,
        funds_transfer: &'static str,
    ) -> Self {
        Self {
            date,
            prior,
            generated,
            julian,
            funds_transfer,
        }
    }

    /// `10/01/2026` -> `261001`, the activity date in a DFR file name.
    fn yymmdd(&self) -> String {
        let (month, rest) = self.date.split_at(2);
        let (day, year) = (&rest[1..3], &rest[6..8]);
        format!("{year}{month}{day}")
    }
}

/// `0000999999.261001.d.B101.dfr`: company (10 digits), activity date, daily, file kind + number.
fn file_name(day: &Day, kind: char, number: u32) -> String {
    format!("{COMPANY:0>10}.{}.d.{kind}{number}.dfr", day.yymmdd())
}

struct Layout {
    delimiter: char,
    /// ACT0010 fields 24 to 30.
    interchange: bool,
    /// The layout of a live ACT0010 file: field 22 filled, an extra field before the
    /// interchange fields, fees with no leading zero, settlement currency last.
    live: bool,
    /// Some live files end every data line with one more delimiter.
    trailing_delimiter: bool,
    line_end: &'static str,
}

impl Layout {
    fn line(&self, fields: &[String]) -> String {
        let mut line = fields.join(&self.delimiter.to_string());
        if self.trailing_delimiter && line.starts_with('R') {
            line.push(self.delimiter);
        }
        line
    }

    fn file(&self, lines: &[String]) -> String {
        let mut text = lines.join(self.line_end);
        text.push_str(self.line_end);
        text
    }

    fn frame(&self, marker: &str) -> String {
        let d = self.delimiter;
        format!("{marker}{d}PID={PID}{d}FREQ=DAILY{d}CO={COMPANY}")
    }

    fn section_header(&self, code: &str, day: &Day, time: &str) -> String {
        let d = self.delimiter;
        format!(
            "H{code}{d}{COMPANY}{d}{}{d}{}{d}{}{d}{time}",
            day.date, day.date, day.generated
        )
    }
}

/// Deterministic xorshift64*: the same seed always gives the same file.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[(self.next() % items.len() as u64) as usize]
    }

    fn text(&mut self, length: usize, alphabet: &[u8]) -> String {
        (0..length)
            .map(|_| char::from(self.pick(alphabet)))
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mop {
    Visa,
    Mastercard,
    Discover,
    Amex,
    /// Electronic check: money taken from a bank account, only in the live-layout sample.
    Ecp,
}

impl Mop {
    const ALL: [Mop; 5] = [Mop::Visa, Mop::Mastercard, Mop::Discover, Mop::Amex, Mop::Ecp];

    fn code(self) -> &'static str {
        match self {
            Self::Visa => "VI",
            Self::Mastercard => "MC",
            Self::Discover => "DI",
            Self::Amex => "AX",
            Self::Ecp => "EC",
        }
    }

    /// Masked test-range card number, the way reports show them.
    fn account(self, rng: &mut Rng) -> String {
        let last4 = rng.text(4, b"0123456789");
        match self {
            Self::Visa => format!("411111XXXXXX{last4}"),
            Self::Mastercard => format!("545454XXXXXX{last4}"),
            Self::Discover => format!("601111XXXXXX{last4}"),
            Self::Amex => format!("371449XXXXX{last4}"),
            Self::Ecp => format!("XXXXXX{last4}"),
        }
    }

    /// Interchange: qualification code, percent fee (in hundredths of a basis point), unit fee.
    /// Amex has none: it is conveyed, so the merchant's Amex contract sets its fees.
    fn interchange(self) -> Option<(&'static str, i64, i64)> {
        match self {
            Self::Visa => Some(("VCNP", 180, 10)),
            Self::Mastercard => Some(("MCMI", 189, 10)),
            Self::Discover => Some(("DCNP", 177, 10)),
            Self::Amex | Self::Ecp => None,
        }
    }

    fn conveyed(self) -> bool {
        self == Self::Amex
    }
}

struct Txn {
    order_id: String,
    mop: Mop,
    refund: bool,
    cents: i64,
    submission: String,
    record: usize,
    account: String,
    expiry: String,
    auth_date: &'static str,
    auth_code: String,
    durbin: &'static str,
    /// ACT0010 action code: DP / RF for cards; DO or i / ER for ECP.
    action: &'static str,
    /// Field 22 in the live layout: 1 phone order, 2 recurring, 7 e-commerce.
    eci: &'static str,
}

impl Txn {
    /// (interchange, assessment) in cents; credited back (negative) on a refund.
    fn fees(&self) -> Option<(i64, i64)> {
        let (_, percent, unit) = self.mop.interchange()?;
        let interchange = (self.cents * percent + 5000) / 10_000 + unit;
        let assessment = (self.cents * 14 + 5000) / 10_000;
        let sign = if self.refund { -1 } else { 1 };
        Some((sign * interchange, sign * assessment))
    }
}

fn transactions(rng: &mut Rng, count: usize, day: &Day, live: bool) -> Vec<Txn> {
    (0..count)
        .map(|index| {
            let mop = match rng.below(100) {
                0..47 => Mop::Visa,
                47..78 => Mop::Mastercard,
                78..87 => Mop::Discover,
                _ => Mop::Amex,
            };
            // Only the live-layout sample has ECP rows, so the other files stay as they were.
            let mop = if live && rng.chance(4) { Mop::Ecp } else { mop };
            let refund = rng.chance(7);
            // Account top-ups are round amounts; pay-by-plate invoices are not.
            let cents = if refund {
                500 + 25 * rng.below(180) as i64
            } else if rng.chance(60) {
                rng.pick(&[2000, 2500, 4000, 5000, 7500, 10_000])
            } else {
                150 + rng.below(4350) as i64
            };
            Txn {
                order_id: rng.text(22, ORDER_ID_CHARS),
                mop,
                refund,
                cents,
                submission: format!("{}.{:04}a", day.julian, index / 200 + 1),
                record: index % 200 + 1,
                account: mop.account(rng),
                expiry: format!("{:02}/{}", 1 + rng.below(12), 27 + rng.below(4)),
                auth_date: if refund {
                    ""
                } else if rng.chance(30) {
                    day.prior
                } else {
                    day.date
                },
                auth_code: if refund {
                    String::new()
                } else {
                    rng.text(6, b"ABCDEFGHJKLMNPQRSTUVWXYZ0123456789")
                },
                durbin: match (mop.interchange(), rng.chance(20)) {
                    (None, _) => "",
                    (Some(_), true) => "Y",
                    (Some(_), false) => "N",
                },
                action: match (mop, refund) {
                    (Mop::Ecp, true) => "ER",
                    (Mop::Ecp, false) if rng.chance(50) => "i",
                    (Mop::Ecp, false) => "DO",
                    (_, true) => "RF",
                    (_, false) => "DP",
                },
                eci: match (live, mop) {
                    (false, _) => "",
                    (true, Mop::Ecp) => "7",
                    (true, _) => match rng.below(100) {
                        0..3 => "1",
                        3..25 => "2",
                        _ => "7",
                    },
                },
            }
        })
        .collect()
}

/// `2000` -> `20.00`, `-1250` -> `-12.50`.
fn money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", cents.abs() / 100, cents.abs() % 100)
}

/// Some reports pad a positive amount with a space instead of a sign: ` 40.00`, `-40.00`.
/// How live files write fees: no leading zero, `-.49`, `.03`, `0`.
fn bare(cents: i64) -> String {
    match cents {
        0 => "0".to_string(),
        -99..=99 => money(cents).replacen("0.", ".", 1),
        _ => money(cents),
    }
}

/// A rate with no leading zero and no trailing zeros: `0.10` -> `.1`.
fn bare_rate(digits: String) -> String {
    format!(".{}", digits.trim_end_matches('0'))
}

fn padded(cents: i64) -> String {
    if cents < 0 {
        money(cents)
    } else {
        format!(" {}", money(cents))
    }
}

fn quoted(value: &str) -> String {
    format!("\"{value}\"")
}

fn fields<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.iter().map(|v| (*v).to_string()).collect()
}

/// A B file: FIN0010 (summary) then ACT0010 (detail), framed by `*DFRBEG` / `*DFREND`.
fn financial_file(day: &Day, txns: &[Txn], layout: &Layout, returned_to_merchant: i64) -> String {
    let mut lines = vec![layout.frame("*DFRBEG")];
    lines.push(layout.section_header("FIN0010", day, "02:15:01"));
    lines.extend(
        summary_rows(day, txns, returned_to_merchant)
            .iter()
            .map(|row| layout.line(row)),
    );
    lines.push(layout.section_header("ACT0010", day, "02:15:01"));
    lines.extend(
        txns.iter()
            .map(|txn| layout.line(&detail_row(day, txn, layout))),
    );
    lines.push(layout.frame("*DFREND"));
    layout.file(&lines)
}

/// ACT0010 data record, fields 1 to 23, plus 24 to 30 with the interchange detail option.
/// The live layout puts one more field before the interchange fields and the currency after.
fn detail_row(day: &Day, txn: &Txn, layout: &Layout) -> Vec<String> {
    let amount = money(if txn.refund { -txn.cents } else { txn.cents });
    let record = txn.record.to_string();
    let mut row = fields([
        "RACT0010",
        day.date,
        PID,
        &quoted(PID_SHORT_NAME),
        &quoted(&txn.submission),
        &record,
        "TD",
        DIVISION,
        "USD",
        &quoted(&txn.order_id),
        if txn.mop == Mop::Ecp { ECP_RDFI } else { "" },
        &txn.account,
        if txn.mop == Mop::Ecp { "" } else { txn.expiry.as_str() },
        &amount,
        txn.mop.code(),
        txn.action,
        txn.auth_date,
        &txn.auth_code,
        if txn.refund { "" } else { "100" },
        "",
        "",
        txn.eci,
        MCC,
    ]);
    if layout.live {
        row.push(String::new());
        // Live files show the fees as negative on a sale, and fill the fields with NONE and
        // zeros when there is no interchange (Amex conveyed, ECP).
        row.extend(match (txn.mop.interchange(), txn.fees()) {
            (Some((code, percent, unit)), Some((interchange, assessment))) => fields([
                code,
                txn.durbin,
                &bare_rate(format!("{unit:02}")),
                &bare_rate(format!("{percent:04}")),
                &bare(-interchange),
                &bare(-assessment),
                "",
            ]),
            _ => fields(["NONE", "N", "0", "0", "0", "0", ""]),
        });
        row.push("USD".to_string());
    } else if layout.interchange {
        row.extend(match (txn.mop.interchange(), txn.fees()) {
            (Some((code, percent, unit)), Some((interchange, assessment))) => fields([
                code,
                txn.durbin,
                &format!("0.{unit:02}"),
                &format!("0.{percent:04}"),
                &money(interchange),
                &money(assessment),
                "",
            ]),
            _ => fields(["", "", "", "", "", "", ""]),
        });
    }
    row
}

/// FIN0010 data records: SALE and REF per card brand (Amex conveyed, the rest settled), then
/// the settled lines with no transaction detail: interchange, Chase's fees, chargebacks.
fn summary_rows(day: &Day, txns: &[Txn], returned_to_merchant: i64) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for mop in Mop::ALL {
        for refund in [false, true] {
            let items: Vec<&Txn> = txns
                .iter()
                .filter(|t| t.mop == mop && t.refund == refund)
                .collect();
            if items.is_empty() {
                continue;
            }
            let total: i64 = items.iter().map(|t| t.cents).sum();
            // A conveyed line is not part of any funds transfer, so it has no FTI or bank account.
            let (funds_transfer, account, flag) = match mop {
                _ if mop.conveyed() => ("", "", "C"),
                Mop::Ecp => (day.funds_transfer, ECP_BANK_ACCOUNT, "S"),
                _ => (day.funds_transfer, SECURE_BANK_ACCOUNT, "S"),
            };
            rows.push(fields([
                "RFIN0010",
                "TD",
                DIVISION,
                funds_transfer,
                account,
                "USD",
                mop.code(),
                if refund { "REF" } else { "SALE" },
                flag,
                &items.len().to_string(),
                &money(if refund { -total } else { total }),
            ]));
        }
    }
    let fees: i64 = txns
        .iter()
        .filter_map(Txn::fees)
        .map(|(interchange, assessment)| interchange + assessment)
        .sum();
    let settled_items = txns.iter().filter(|t| !t.mop.conveyed()).count() as i64;
    let other = [
        ("IA", -fees),
        ("PFEE", -5 * settled_items),
        ("CB", -returned_to_merchant),
    ];
    for (category, cents) in other.into_iter().filter(|(_, cents)| *cents != 0) {
        rows.push(fields([
            "RFIN0010",
            "TD",
            DIVISION,
            day.funds_transfer,
            SECURE_BANK_ACCOUNT,
            "USD",
            "",
            category,
            "S",
            "",
            &money(cents),
        ]));
    }
    rows
}

#[derive(Clone, Copy)]
enum CbCategory {
    /// New chargeback received: into Chase's inventory, nothing debited yet.
    Received,
    /// Chase disputed it with the issuer on the merchant's behalf.
    Represented,
    /// Returned to the merchant: the merchant is debited.
    ReturnedToMerchant,
    /// Part returned to the merchant, the rest represented.
    PartialReturn,
}

impl CbCategory {
    fn code(self) -> &'static str {
        match self {
            Self::Received => "RECD",
            Self::Represented => "REPR",
            Self::ReturnedToMerchant => "RTM",
            Self::PartialReturn => "PARRTM",
        }
    }

    fn financial(self) -> bool {
        matches!(self, Self::ReturnedToMerchant | Self::PartialReturn)
    }
}

struct Chargeback {
    category: CbCategory,
    sequence: u64,
    order_id: String,
    mop: Mop,
    account: String,
    reason: &'static str,
    issuer_cents: i64,
    /// What this action moved, unsigned.
    action_cents: i64,
}

fn chargebacks(rng: &mut Rng) -> Vec<Chargeback> {
    use CbCategory::*;
    [
        Received,
        Received,
        Received,
        Received,
        Represented,
        Represented,
        ReturnedToMerchant,
        ReturnedToMerchant,
        PartialReturn,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, category)| {
        let mop = rng.pick(&[Mop::Visa, Mop::Mastercard, Mop::Discover]);
        let issuer_cents = rng.pick(&[2000, 2500, 4000, 5000, 7500]);
        Chargeback {
            category,
            sequence: 29_733_200 + index as u64 * 7,
            order_id: rng.text(22, ORDER_ID_CHARS),
            mop,
            account: mop.account(rng),
            reason: rng.pick(&["83", "30", "53"]),
            issuer_cents,
            action_cents: match category {
                PartialReturn => issuer_cents / 2,
                _ => issuer_cents,
            },
        }
    })
    .collect()
}

/// An A file: PDE0017 summary (`RPDE0017S`) then detail (`RPDE0017D`) records.
fn exception_file(day: &Day, chargebacks: &[Chargeback], layout: &Layout) -> String {
    let mut lines = vec![layout.frame("*DFRBEG")];
    lines.push(layout.section_header("PDE0017", day, "02:15:07"));

    // Inventory: what Chase was holding, what came in, what left, what it holds now.
    let begin_count = 11_i64;
    let begin_cents = 48_250_i64;
    let mut end_count = begin_count;
    let mut end_cents = begin_cents;
    let mut summary = vec![("BEGINV", "NON", begin_count, begin_cents)];
    for category in [
        CbCategory::Received,
        CbCategory::Represented,
        CbCategory::ReturnedToMerchant,
        CbCategory::PartialReturn,
    ] {
        let items: Vec<&Chargeback> = chargebacks
            .iter()
            .filter(|c| c.category.code() == category.code())
            .collect();
        let cents: i64 = items.iter().map(|c| c.action_cents).sum();
        let count = items.len() as i64;
        // Received adds to inventory; everything else takes away from it.
        let (count, cents) = match category {
            CbCategory::Received => (count, cents),
            _ => (-count, -cents),
        };
        end_count += count;
        end_cents += cents;
        let flag = if category.financial() { "FIN" } else { "NON" };
        summary.push((category.code(), flag, count, cents));
    }
    summary.push(("ENDINV", "NON", end_count, end_cents));
    for (category, flag, count, cents) in summary {
        lines.push(layout.line(&fields([
            "RPDE0017S",
            "TD",
            DIVISION,
            "",
            "",
            "USD",
            "CC",
            category,
            flag,
            &count.to_string(),
            &padded(cents),
            "",
        ])));
    }

    for cb in chargebacks {
        let action = match cb.category {
            CbCategory::Received => cb.action_cents,
            _ => -cb.action_cents,
        };
        lines.push(layout.line(&fields([
            "RPDE0017D",
            "TD",
            DIVISION,
            &padded(cb.issuer_cents),
            "N",
            "USD",
            cb.category.code(),
            "",
            &cb.sequence.to_string(),
            &quoted(&cb.order_id),
            &cb.account,
            cb.reason,
            "08/27/2026",
            "09/24/2026",
            day.date,
            &padded(action),
            "",
            "1",
            cb.mop.code(),
            "08/27/2026",
            "10/14/2026",
            "",
        ])));
    }
    lines.push(layout.frame("*DFREND"));
    layout.file(&lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_deterministic_and_named_like_chase_names_them() {
        let names: Vec<String> = all().into_iter().map(|f| f.name).collect();
        assert_eq!(
            names,
            [
                "0000999999.261001.d.B101.dfr",
                "0000999999.261001.d.A101.dfr",
                "0000999999.261002.d.B102.dfr",
                "0000999999.261003.d.B103.dfr",
                "0000999999.261004.d.B104.dfr",
                "0000999999.261005.d.B105.dfr",
            ]
        );
        let again: Vec<String> = all().into_iter().map(|f| f.contents).collect();
        let first: Vec<String> = all().into_iter().map(|f| f.contents).collect();
        assert_eq!(again, first);
    }

    #[test]
    fn money_formats() {
        assert_eq!(money(2000), "20.00");
        assert_eq!(money(-1250), "-12.50");
        assert_eq!(money(5), "0.05");
        assert_eq!(padded(4000), " 40.00");
        assert_eq!(padded(-4000), "-40.00");
        assert_eq!(bare(-49), "-.49");
        assert_eq!(bare(3), ".03");
        assert_eq!(bare(-205), "-2.05");
        assert_eq!(bare(1050), "10.50");
        assert_eq!(bare(0), "0");
        assert_eq!(bare_rate("10".to_string()), ".1");
        assert_eq!(bare_rate("0180".to_string()), ".018");
    }
}
