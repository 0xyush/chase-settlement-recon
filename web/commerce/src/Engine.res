// Bindings to the engines and the shapes they return.
// runTransform / exportFile call the Rust engine (WebAssembly, src/engine/wasm) and fall
// back to the JavaScript engine in src/engine when it is not running. `output.engine` says which.

type record_
type config = JSON.t
type file

type detection = {key: Nullable.t<string>, headerIndex: int, meta: string}
type row = {line: int, fields: Dict.t<Nullable.t<string>>}
type entry = {
  order_id: string,
  amount: int,
  currency: string,
  effective_at: string,
  balance_direction: string,
}
type rowResult = {
  line: int,
  outcome: string,
  details: option<string>,
  entry: option<entry>,
  row: row,
}
// One step of the engine's run, for the step strip (src/steps.rs in the engine).
type step = {
  key: string,
  title: string,
  code: string,
  status: string, // done | warning | failed | not_run
  metric: string,
  detail: string,
  micros: Nullable.t<float>,
}
type output = {
  engine: option<string>, // "rust" or "javascript"
  rejected: option<string>,
  header: option<array<string>>,
  rows: option<array<row>>,
  results: Nullable.t<array<rowResult>>,
  steps: option<array<step>>,
}
type check = {level: string, text: string}
type sample = {file: string, label: string}

// Pipeline
@module("./engine/csv.js") external parseCSV: string => array<record_> = "parseCSV"
@module("./engine/detect.js") external detectReport: array<record_> => detection = "detectReport"
@module("./engine/transform.js") external transform: (array<record_>, config) => output = "transform"
@module("./engine/checks.js")
external computeChecks: (array<rowResult>, config) => array<check> = "computeChecks"

// Rust engine first, JavaScript fallback (src/engine/backend.js)
@module("./engine/backend.js")
external runTransform: (string, array<record_>, config) => promise<output> = "runTransform"
@module("./engine/backend.js")
external exportFile: (string, config, string, array<rowResult>) => promise<string> = "exportFile"

// Exports
@module("./engine/export.js") external stagingEntriesJson: array<rowResult> => string = "stagingEntriesJson"
@module("./engine/export.js") external rowReportCsv: array<rowResult> => string = "rowReportCsv"
@module("./engine/export.js") external download: (string, string, string) => unit = "download"
@module("./engine/export.js") external formatMinor: int => string = "formatMinor"

// Read helpers
@module("./engine/view.js") external configFor: detection => config = "configFor"
@module("./engine/view.js") external reportName: config => string = "reportName"
@module("./engine/view.js") external headerLine: (array<record_>, config) => int = "headerLine"
@module("./engine/view.js") external metadataText: entry => string = "metadataText"
@module("./engine/view.js") external orderIdOf: (rowResult, config) => string = "orderIdOf"
@module("./engine/view.js") external rawCell: (row, string) => string = "rawCell"
@module("./engine/view.js") external rowMatches: (rowResult, string) => bool = "rowMatches"

// Browser I/O
@module("./engine/io.js") external fileFromInput: ReactEvent.Form.t => Nullable.t<file> = "fileFromInput"
@module("./engine/io.js") external fileFromDrop: ReactEvent.Mouse.t => Nullable.t<file> = "fileFromDrop"
@module("./engine/io.js") external fileName: file => string = "fileName"
@module("./engine/io.js") external isCsv: file => bool = "isCsv"
@module("./engine/io.js") external readFileText: file => promise<string> = "readFileText"
@module("./engine/io.js") external fetchSample: string => promise<string> = "fetchSample"
@module("./engine/io.js") external preventWindowDrop: unit => (unit => unit) = "preventWindowDrop"

@module("./presets.js") external samples: array<sample> = "SAMPLES"

// Read helpers for the Control Center style screens (src/engine/view.js)
type configSummary = {
  report: string,
  format: string,
  headerRow: int,
  footer: string,
  uniqueKey: string,
  skipRules: int,
  fields: int,
  metadataFields: int,
}
@module("./engine/view.js") external metadataPairs: entry => array<(string, string)> = "metadataPairs"
@module("./engine/view.js") external rawPairs: row => array<(string, string)> = "rawPairs"
@module("./engine/view.js") external configSummary: config => configSummary = "configSummary"
@module("./engine/view.js") external describeSkip: config => array<string> = "describeSkip"
