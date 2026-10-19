// Row-by-row check that the Rust engine and the dashboard's JavaScript engine agree.
// Usage: node scripts/parity.mjs   (needs the Rust API running: chase_recon serve)
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const dashboard = path.resolve(root, "../../web/commerce/src");
const { parseCSV } = await import(path.join(dashboard, "engine/csv.js"));
const { transform } = await import(path.join(dashboard, "engine/transform.js"));
const api = process.env.API ?? "http://127.0.0.1:8787";

const cases = [
  ["Deposit_Details_Preset_Report.csv", "deposit_details.json"],
  ["Transaction_Fee_Details_Preset_Report.csv", "transaction_fee_details.json"],
  ["Funded_Transaction_Fee_Details_Preset_Report.csv", "funded_txn_fee_details.json"],
];

// The comparable part of one row result: what happened, why, and the entry.
const view = (r) => JSON.stringify({ line: r.line, outcome: r.outcome, details: r.details ?? null, entry: r.entry ?? null, fields: r.row.fields });

let failures = 0;
for (const [file, configFile] of cases) {
  const csv = fs.readFileSync(path.join(root, "samples", file), "utf8");
  const config = JSON.parse(fs.readFileSync(path.join(root, "configs", configFile), "utf8"));
  const js = transform(parseCSV(csv), config);
  const response = await fetch(`${api}/api/transform`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ csv, config }),
  });
  const rust = await response.json();
  const diffs = [];
  if (js.results.length !== rust.results.length) diffs.push(`row count js=${js.results.length} rust=${rust.results.length}`);
  js.results.forEach((r, i) => {
    const a = view(r), b = rust.results[i] ? view(rust.results[i]) : "(missing)";
    if (a !== b && diffs.length < 5) diffs.push(`row ${i}:\n  js:   ${a.slice(0, 400)}\n  rust: ${b.slice(0, 400)}`);
  });
  const count = (o) => rust.results.filter((r) => r.outcome === o).length;
  console.log(`${diffs.length ? "DIFF" : "SAME"}  ${file}: ${rust.results.length} rows (staged ${count("Success")}, skipped ${count("Skipped")}, failed ${count("Failed")}), engine=${rust.engine}`);
  diffs.forEach((d) => console.log("  " + d));
  failures += diffs.length;
}
process.exit(failures ? 1 : 0);
