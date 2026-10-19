// The Rust engine (engines/commerce in this repo), compiled to WebAssembly and run right here in
// the browser (built by scripts/build.sh into ./wasm). If it cannot load, transforms and exports
// fall back to the JavaScript engine in this folder, and the page says so. Matching needs Rust.
import initEngine, { commerce_export, commerce_match, commerce_transform } from "./wasm/recon_wasm.js";
import { transform as transformInJs } from "./transform.js";
import { rowReportCsv, stagingEntriesJson } from "./export.js";

const engine = initEngine();
engine.catch(() => {});

export async function runTransform(text, records, config) {
  try {
    await engine;
    const output = JSON.parse(commerce_transform(text, JSON.stringify(config)));
    // Rust sends each parsed row once, inside its result; rebuild the rows list the page counts.
    if (output.results && !output.rows) output.rows = output.results.map((r) => r.row);
    return { ...output, engine: "rust" };
  } catch {
    return { ...transformInJs(records, config), engine: "javascript" };
  }
}

// format: "staging_json" | "report_csv"
export async function exportFile(text, config, format, results) {
  try {
    await engine;
    return commerce_export(text, JSON.stringify(config), format);
  } catch {
    return format === "report_csv" ? rowReportCsv(results) : stagingEntriesJson(results);
  }
}

export async function matchWithHyperswitch(results, hyperswitchCsv) {
  const entries = results.filter((r) => r.outcome === "Success").map((r) => r.entry);
  try {
    await engine;
    return JSON.parse(commerce_match(JSON.stringify(entries), hyperswitchCsv));
  } catch (error) {
    return { rejected: `Matching runs in the Rust engine, which could not load in this browser (${error.message}).` };
  }
}

// The pretend Hyperswitch export generated for each sample (chase_recon mock-hs).
export const mockExportName = (sampleFile) => sampleFile.replace(/\.csv$/i, ".hyperswitch_mock.csv");
export const fetchMockExport = (sampleFile) =>
  fetch(`samples/${mockExportName(sampleFile)}`).then((r) =>
    r.ok && !(r.headers.get("content-type") ?? "").includes("text/html") ? r.text() : null,
  );
