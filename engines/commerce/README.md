# Chase recon prototype (Rust)

Turns a Chase / J.P. Morgan settlement report into hyperswitch-recon staging entries, writes the
reports, and matches the entries against a Hyperswitch export. A scrappy prototype for the
Toll-merchant demo, built to mirror how hyperswitch-recon's transformation engine works.
The dashboard in `../../web/commerce` calls it.

## Run

Rust is installed through Homebrew's rustup; put it on your PATH first:

```
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
```

```
cargo test                                   # 32 tests, including the three sample files
cargo run --release -- serve                 # optional local API on :8787 (the dashboard uses WebAssembly)
cargo run --release -- run samples/Deposit_Details_Preset_Report.csv configs/deposit_details.json --out out
cargo run --release -- match samples/Deposit_Details_Preset_Report.csv configs/deposit_details.json samples/Deposit_Details_Preset_Report.hyperswitch_mock.csv
cargo run --release -- mock-hs samples/Deposit_Details_Preset_Report.csv configs/deposit_details.json > hs.csv
node scripts/parity.mjs                      # row-by-row check against the dashboard's JS engine (needs `serve`)
```

## How it works

| Step | File | What it does |
|---|---|---|
| 1. Read | `src/parser.rs` | Reads the CSV. Chase puts a `BEGIN` line and a metadata line first and an `END` line last, so `header_row_index: 2` picks line 3 as the column names and `footer_marker: "END"` drops the closing line. Empty cells become "missing". Every row keeps its file line number. |
| 2. Skip | `src/skip.rs` | Drops rows the config names, such as approval-only `AUTH` rows, account-fee `OTHER` rows, or fee rows with no order number. Conditions in one rule must all match; any rule can skip. |
| 3. Map + validate | `src/transform.rs`, `src/rules.rs` | Maps columns to the five main fields (order_id, amount, currency, effective_at, balance_direction) and to metadata fields. Amounts become cents with integer maths. A bad row is marked Failed with the reason and never stops the file. |
| 4. Write | `src/report.rs` | Staging entries as JSON, and a row report CSV (`source_type,identifier,outcome,details`), the recon repo's `report-v1.csv`. |
| 5. Match | `src/matching.rs` | Pairs entries with a Hyperswitch export on order ID + direction (payment = credit, refund = debit), then compares amounts. Results: matched, amount mismatch, missing in Hyperswitch, missing in Chase, ignored (statuses that never settle). |
| Config | `src/config.rs`, `configs/*.json` | The JSON that drives steps 1 to 3, one per Chase report. Same shape the dashboard sends. |
| API | `src/server.rs` | `POST /api/transform`, `/api/export`, `/api/match`, `GET /api/health`. |

Every `/api/transform` answer also carries `steps`: the run as 5 steps, each with what it did to
this file, the source file that does it, and the time it took in the engine (`src/steps.rs`).
The dashboard shows them as the "What the engine did" strip.

## Verified

| Sample | Parsed | Staged | Skipped | Failed |
|---|---|---|---|---|
| Deposit Details | 825 | 825 | 0 | 0 |
| Transaction Fee Details | 1,208 | 1,207 | 1 | 0 |
| Funded Transaction Fee Details | 5,205 | 247 | 4,958 | 0 |

These counts are pinned in `tests/samples.rs`, and `scripts/parity.mjs` confirms every row's
outcome, reason and entry is identical to the dashboard's JavaScript engine.

## What is mocked

- The Chase files are J.P. Morgan's public sample reports, not merchant data.
- The Hyperswitch exports (`samples/*.hyperswitch_mock.csv`) are generated from those samples
  with differences planted on purpose. They assume Hyperswitch sends its payment id to Chase as
  the order number; for the merchant that is an open question, because its ids are 30
  characters and Chase keeps 22.
- Nothing talks to the real recon service, Hyperswitch, or Chase.
