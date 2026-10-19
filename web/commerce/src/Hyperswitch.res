// Matching staged Chase entries against a Hyperswitch export (Rust engine, in WebAssembly).

type summary = {
  chase_entries: int,
  hyperswitch_records: int,
  matched: int,
  amount_mismatch: int,
  currency_mismatch: int,
  missing_in_hyperswitch: int,
  missing_in_chase: int,
  ignored: int,
}

type item = {
  outcome: string,
  order_id: string,
  direction: string,
  chase_amount: option<int>,
  hyperswitch_amount: option<int>,
  chase_currency: option<string>,
  hyperswitch_currency: option<string>,
  chase_line: option<int>,
  hyperswitch_line: option<int>,
  note: option<string>,
}

type output = {
  rejected: option<string>,
  summary: option<summary>,
  items: option<array<item>>,
  report_csv: option<string>,
}

@module("./engine/backend.js")
external matchWith: (array<Engine.rowResult>, string) => promise<output> = "matchWithHyperswitch"
@module("./engine/backend.js") external fetchMockExport: string => promise<Nullable.t<string>> = "fetchMockExport"
@module("./engine/backend.js") external mockExportName: string => string = "mockExportName"

let label = outcome =>
  switch outcome {
  | "matched" => "Matched"
  | "amount_mismatch" => "Amount mismatch"
  | "currency_mismatch" => "Currency mismatch"
  | "missing_in_hyperswitch" => "Missing in Hyperswitch"
  | "missing_in_chase" => "Missing in Chase"
  | "ignored" => "Ignored"
  | other => other
  }

let color = (outcome): Tag.color =>
  switch outcome {
  | "matched" => Success
  | "amount_mismatch" | "currency_mismatch" => Error
  | "missing_in_hyperswitch" | "missing_in_chase" => Warning
  | _ => Neutral
  }

// The card each outcome is counted under.
let group = outcome =>
  switch outcome {
  | "amount_mismatch" | "currency_mismatch" => "mismatch"
  | other => other
  }
