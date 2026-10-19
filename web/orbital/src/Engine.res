// Bindings to the Rust engine (engines/orbital) and the shapes it returns.
// runTransform / exportFile call it as WebAssembly in the page; there is no JavaScript fallback.

type config = JSON.t
type file

type detection = {key: Nullable.t<string>, isDfr: bool, sections: array<string>}
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

// What the DFR file holds: its frame (*DFRBEG / *DFREND) and its report sections.
type frame = {line: int, values: Dict.t<string>}
type section = {
  code: string,
  name: Nullable.t<string>,
  line: int,
  company_id: Nullable.t<string>,
  date_from: Nullable.t<string>,
  date_to: Nullable.t<string>,
  generated: Nullable.t<string>,
  records: Dict.t<int>,
}
type dfrFile = {
  delimiter: string,
  @as("begin") begin_: frame,
  @as("end") end_: Nullable.t<frame>,
  sections: array<section>,
  lines: int,
  unexpected_lines: array<int>,
}

// Staged ACT0010 detail against Chase's FIN0010 summary in the same file.
type tieLine = {
  currency: string,
  card_brand: string,
  category: string,
  settled_conveyed: Nullable.t<string>,
  summary_count: Nullable.t<int>,
  summary_amount: Nullable.t<int>,
  detail_count: Nullable.t<int>,
  detail_amount: Nullable.t<int>,
  status: string,
}
type otherLine = {
  currency: string,
  category: string,
  name: string,
  settled_conveyed: Nullable.t<string>,
  amount: int,
}
type tieout = {
  tied: bool,
  lines: array<tieLine>,
  other: array<otherLine>,
  settled_net: Dict.t<int>,
  conveyed_net: Dict.t<int>,
  problems: array<string>,
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
  engine: option<string>, // "rust", or "none" when the engine is not reachable
  rejected: option<string>,
  file: option<dfrFile>,
  header: option<array<string>>,
  rows: option<array<row>>,
  results: Nullable.t<array<rowResult>>,
  steps: option<array<step>>,
  tieout: Nullable.t<tieout>,
}
type check = {level: string, text: string}
type sample = {file: string, label: string}

// Pipeline
@module("./engine/detect.js") external detectFile: string => detection = "detectFile"
@module("./engine/backend.js") external runTransform: (string, config) => promise<output> = "runTransform"
@module("./engine/backend.js")
external exportFile: (string, config, string) => promise<string> = "exportFile"
@module("./engine/checks.js") external computeChecks: (output, config) => array<check> = "computeChecks"

// Downloads and money
@module("./engine/export.js") external download: (string, string, string) => unit = "download"
@module("./engine/export.js") external formatMinor: int => string = "formatMinor"

// Read helpers
@module("./engine/view.js") external configFor: detection => config = "configFor"
@module("./engine/view.js") external reportName: config => string = "reportName"
@module("./engine/view.js") external metadataText: entry => string = "metadataText"
@module("./engine/view.js") external orderIdOf: (rowResult, config) => string = "orderIdOf"
@module("./engine/view.js") external rawCell: (row, string) => string = "rawCell"
@module("./engine/view.js") external rowMatches: (rowResult, string) => bool = "rowMatches"
@module("./engine/view.js") external metadataPairs: entry => array<(string, string)> = "metadataPairs"
@module("./engine/view.js") external rawPairs: row => array<(string, string)> = "rawPairs"
@module("./engine/view.js") external describeSkip: config => array<string> = "describeSkip"
@module("./engine/view.js") external frameText: frame => string = "frameText"
@module("./engine/view.js") external recordsText: section => string = "recordsText"
@module("./engine/view.js") external fileLine: dfrFile => string = "fileLine"
@module("./engine/view.js") external netText: Dict.t<int> => string = "netText"

type configSummary = {
  report: string,
  format: string,
  section: string,
  recordType: string,
  columns: int,
  delimiter: string,
  uniqueKey: string,
  skipRules: int,
  fields: int,
  metadataFields: int,
}
@module("./engine/view.js") external configSummary: config => configSummary = "configSummary"

// Browser I/O
@module("./engine/io.js") external fileFromInput: ReactEvent.Form.t => Nullable.t<file> = "fileFromInput"
@module("./engine/io.js") external fileFromDrop: ReactEvent.Mouse.t => Nullable.t<file> = "fileFromDrop"
@module("./engine/io.js") external fileName: file => string = "fileName"
@module("./engine/io.js") external isAccepted: file => bool = "isAccepted"
@module("./engine/io.js") external readFileText: file => promise<string> = "readFileText"
@module("./engine/io.js") external fetchSample: string => promise<string> = "fetchSample"
@module("./engine/io.js") external preventWindowDrop: unit => (unit => unit) = "preventWindowDrop"

@module("./presets.js") external samples: array<sample> = "SAMPLES"
