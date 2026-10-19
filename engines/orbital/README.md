# Orbital recon prototype (Rust)

Turns a Chase Orbital / Paymentech **Delimited File Report (DFR)** into hyperswitch-recon staging
entries, ties them out against Chase's own summary in the same file, writes the reports, and
matches the entries against a Hyperswitch export. The twin of `../commerce` (which
reads Commerce Center CSV reports), for the case where the merchant's account is on Orbital and its
reports still come from Paymentech reporting. The dashboard in `../../web/orbital`
calls it.

## Run

Rust is installed through Homebrew's rustup; put it on your PATH first:

```
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
```

```
cargo test                                         # 54 tests, including every sample file
cargo run --release -- serve                       # API on http://127.0.0.1:8788 for the dashboard
cargo run --release -- scan samples/0000999999.261001.d.B101.dfr
cargo run --release -- run samples/0000999999.261001.d.B101.dfr configs/act0010_deposit_detail.json --out out
cargo run --release -- run samples/0000999999.261001.d.A101.dfr configs/pde0017_chargebacks.json --out out
cargo run --release -- run samples/0000999999.261005.d.B105.dfr configs/act0010_deposit_detail_live.json --out out
cargo run --release -- match samples/0000999999.261001.d.B101.dfr configs/act0010_deposit_detail.json samples/0000999999.261001.d.B101.hyperswitch_mock.csv
cargo run --release -- samples --out samples       # rewrite the synthetic sample files
```

The dashboards don't need `serve`: they run this engine as WebAssembly (`../../wasm`).

## What a DFR file looks like

Not a table with a header row: a stack of lines whose first field says what the line is.

```
*DFRBEG,PID=888888,FREQ=DAILY,CO=999999         file header
HFIN0010,999999,10/01/2026,10/01/2026,...       report section: Deposit Activity Summary
RFIN0010,TD,777001,307000101,...,VI,SALE,S,281,12110.84
HACT0010,999999,10/01/2026,10/01/2026,...       report section: Deposit Detail
RACT0010,10/01/2026,888888,"SAMPLE","26274.0001a",1,TD,777001,USD,"W2AJHsBkMc5Hyx7NUAA7i2",...
*DFREND,PID=888888,FREQ=DAILY,CO=999999         file trailer, repeats the header
```

Each day Chase sends a **B** file (financial reports: FIN0010, ACT0010, ...) and an **A** file
(exception reports: PDE0017 chargebacks, ...), named `<company>.<YYMMDD>.d.<A|B>nnn.dfr`.
Comma, pipe or tab delimiter, chosen per merchant.

## How it works

| Step | File | Status | What it does |
|---|---|---|---|
| 1. Read | `src/dfr.rs` | **new** | Checks the frame (`*DFRBEG` first, `*DFREND` last and agreeing), reads the delimiter from line 1, lists the sections, keeps the configured data records and names each field by its position from the config. Rejects a cut-off file whole. Handles quoted fields with delimiters inside, padded amounts, CRLF, a trailing delimiter. |
| 2. Skip | `src/skip.rs` | copied | Drops rows the config names: AR authorization reversals in deposits (they move no money), and every chargeback stage except RTM / PARRTM. |
| 3. Map + validate | `src/transform.rs`, `src/rules.rs` | copied, 26 code lines changed | Maps fields to the five main fields and metadata, cents with integer maths, Failed rows with a reason. Only change: rows come from `dfr.rs`, output carries the file structure. |
| 4. Write | `src/report.rs` | copied | Staging entries JSON and `report-v1.csv`. |
| 5. Match | `src/matching.rs` | copied | Pairs entries with a Hyperswitch export on order ID + direction, then amount. Connector name in the mock is `jpmorgan_orbital`. |
| 6. Tie-out | `src/tieout.rs` | **new** | Adds up the staged ACT0010 entries per card brand and sale/refund and compares them with Chase's FIN0010 lines in the same file. Reads FIN0010's Settled / Conveyed flag: what Chase deposits versus what the card network (Amex) pays directly. |
| Config | `src/config.rs`, `configs/*.json` | copied, parsing section changed | `parsing_config` names the section, the record type and every field position (from Chase's spec), instead of a header row. |
| Samples | `src/sample.rs` | **new** | Generates the synthetic files. |
| API | `src/server.rs` | copied | `POST /api/transform` (`{text, config}`, answer includes `file` and `tieout`), `/api/export`, `/api/match`, `GET /api/health`. |

Size: the reader is about 425 lines of code without tests and comments. Everything after it is the
Commerce prototype's engine.

Every `/api/transform` answer also carries `steps`: the run as 8 steps, each with what it did to
this file, the source file that does it, and the time it took in the engine (`src/steps.rs`).
The dashboard shows them as the "What the engine did" strip.

## Verified (synthetic samples)

| File | What it shows | Parsed | Staged | Skipped | Failed |
|---|---|---|---|---|---|
| `…261001.d.B101.dfr` | Day 1 B file, comma, CRLF | 640 | 640 | 0 | 0 |
| `…261001.d.A101.dfr` | Day 1 A file, PDE0017 chargebacks | 9 | 3 | 6 | 0 |
| `…261002.d.B102.dfr` | Day 2 B file, pipe, interchange detail, trailing pipe | 480 | 480 | 0 | 0 |
| `…261003.d.B103.dfr` | No activity: headers only | 0 | 0 | 0 | 0 |
| `…261004.d.B104.dfr` | Cut off mid-line, no `*DFREND` | rejected | | | |
| `…261005.d.B105.dfr` | Day 5 B file in the live layout (below), tab, ECP rows | 400 | 400 | 0 | 0 |

All three B files tie out on every card brand. Amex is conveyed (USD 3,291.96 on day 1). The A file's
returned-to-merchant total (65.00) equals the B file's FIN0010 chargeback line. Matching day 1
against its mock export: 627 matched, 6 amount mismatches, 7 missing in Hyperswitch, 2 missing in
Chase, 1 ignored. All pinned in `tests/samples.rs`.

## Checked against a live file

A real DFR pair from another Chase merchant (a B file and an A file from 2022) was run through the
engine locally. The files hold real, masked card data, so they are not in this repo; `B105` copies
their layout with made-up values.

| File | Parsed | Staged | Skipped | Failed | Tie-out |
|---|---|---|---|---|---|
| B file, ACT0010 (`act0010_deposit_detail_live.json`) | 9,186 | 9,186 | 0 | 0 | every card brand ties to FIN0010 |
| A file, PDE0017 (`pde0017_chargebacks.json`) | 92 | 20 | 72 | 0 | |

What the live file confirmed:

- **Amex is conveyed.** Every Amex line in FIN0010 is `C`. Its ACT0010 rows have interchange
  `NONE` and zero fees. No Amex row is in the PDE0017 chargebacks: Amex disputes come from Amex.
- **The detail rebuilds Chase's summary.** ACT0010 counts and amounts equal FIN0010 for every
  division, card brand and sale / refund.
- **Card money and ECP money go to different bank accounts** in FIN0010.

What it changed:

- **ACT0010 has 32 fields, not 30.** Field 22 (reserved in the spec) holds 1, 2 or 7, which look
  like Chase's transaction type codes (1 phone order, 2 recurring, 7 e-commerce); it is read as
  `transaction_type`. One field the spec does not list sits before the interchange fields, and
  the settlement currency is last. `configs/act0010_deposit_detail_live.json` reads this layout.
  The dashboard picks it by counting the fields on the first ACT0010 line.
- **Fees have no leading zero** (`-.49`, `.1`) and are negative on a sale. The amount parser
  accepts that now.
- **A money field must fit 2 decimals.** With the spec's positions on a live file, a rate such as
  `.018` lands in the interchange fee column. That row now fails with `PrecisionLoss` instead of
  staging a wrong fee.
- **ECP action code `i`.** It is not in the spec's action code list, but FIN0010 counts those rows
  as sales, so the configs treat it as money in, like `DO`.
- **File names.** Chase's live files are named like `Chase.d.B123.dfr_resp`; the dashboard
  accepts `.dfr_resp`.

## What is synthetic, and what to confirm with a real file

There is no public DFR sample (J.P. Morgan publishes Commerce Center samples only), so the files in
`samples/` are generated from
Chase's DFR spec v2.2 (2013) plus details seen in public code that reads live files: pipe delimiter,
quoted order numbers that can contain commas, positive amounts padded with a space, a trailing
delimiter. Masked test card numbers only. Before relying on it:

- **Spec version.** Field positions after 23 change between versions and setups: spec v2.2 lists
  30, the live file above has 32 (`configs/act0010_deposit_detail_live.json` reads it), one seen in
  public code has 36. Get the merchant's current DFR spec from Chase and check `columns` in the
  config. The first 23 fields are stable.
- **Authorization reversals.** Chase's 2018 ACT-0010 guide lists AR (authorization reversal) as an
  action code. The deposit config skips AR rows with a reason instead of failing them.
- **Refund sign.** The spec implies refunds are negative. The configs drop the sign and read the
  direction from the action code (`DP` sale, `RF` refund), so either way works.
- **Chargebacks.** Which PDE0017 stages move money depends on the merchant's setup with Chase. The
  config assumes RTM / PARRTM (returned to merchant) are the debits.
- **Fees.** Whether interchange and Chase fees (FIN0010 `IA`, `PFEE`) come out of the daily deposit
  or are billed monthly depends on the merchant's pricing setup.
- **Order IDs.** Chase keeps 22 characters of the merchant order number here too; the merchant's
  ids are 30. In a sandbox payment through Hyperswitch to Authorize.net, the full 30-character id
  reached only Authorize.net's description field; its invoice number held a random 20-character
  value. Which field Chase prints as the order number for Authorize.net traffic is still open.
- **Delivery.** DFR files arrive by SFTP (or are pulled from Paymentech Online). Nothing here
  fetches files; the recon service would need them landed where its ingestion can read them.
