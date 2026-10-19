// The Rust engine (engines/orbital in this repo), compiled to WebAssembly and run right here in
// the browser (built by scripts/build.sh into ./wasm). There is no JavaScript fallback.
import initEngine, { orbital_export, orbital_match, orbital_transform } from "./wasm/recon_wasm.js";

const engine = initEngine();
engine.catch(() => {});
const NOT_LOADED = "The Rust engine (WebAssembly) could not load in this browser";

export async function runTransform(text, config) {
  try {
    await engine;
    const output = JSON.parse(orbital_transform(text, JSON.stringify(config)));
    // Rust sends each parsed row once, inside its result; rebuild the rows list the page counts.
    if (output.results && !output.rows) output.rows = output.results.map((r) => r.row);
    return { ...output, engine: "rust" };
  } catch (error) {
    return { engine: "none", rejected: `${NOT_LOADED} (${error.message})`, results: null, tieout: null };
  }
}

// format: "staging_json" | "report_csv"
export async function exportFile(text, config, format) {
  await engine;
  return orbital_export(text, JSON.stringify(config), format);
}

export async function matchWithHyperswitch(results, hyperswitchCsv) {
  const entries = results.filter((r) => r.outcome === "Success").map((r) => r.entry);
  try {
    await engine;
    return JSON.parse(orbital_match(JSON.stringify(entries), hyperswitchCsv));
  } catch (error) {
    return { rejected: `${NOT_LOADED} (${error.message})` };
  }
}

// The pretend Hyperswitch export generated for each sample (orbital_recon mock-hs).
export const mockExportName = (sampleFile) => sampleFile.replace(/\.dfr$/i, ".hyperswitch_mock.csv");
export const fetchMockExport = (sampleFile) =>
  fetch(`samples/${mockExportName(sampleFile)}`).then((r) =>
    r.ok && !(r.headers.get("content-type") ?? "").includes("text/html") ? r.text() : null,
  );
