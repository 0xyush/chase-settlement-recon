import { applyRules, majorToMinor, parseDate } from "./rules.js";
import { matchSkip } from "./skip.js";

/**
 * The whole pipeline for one file: parse -> skip -> map -> validate.
 * This is the seam for the Rust prototype: once it runs, replace this function
 * with a call to it and the UI does not change.
 *
 * Returns { rejected } when the whole file is unusable, otherwise
 * { header, rows, results } with one result per data row, in file order.
 */
export function transform(records, config) {
  const headerIndex = config.parsing_config?.header_row_index ?? 0;
  if (headerIndex >= records.length) {
    return { rejected: `HeaderNotFound: header_row_index ${headerIndex} is past the end of the file` };
  }

  const header = records[headerIndex].cells;
  // Drop closing lines such as Chase's "END" (parsing_config.footer_marker): framing, not data.
  const data = records.slice(headerIndex + 1);
  const footer = config.parsing_config?.footer_marker;
  while (footer && data.length && data[data.length - 1].cells[0] === footer) data.pop();
  const rows = data.map((r) => ({
    line: r.line,
    fields: Object.fromEntries(
      header.filter(Boolean).map((name) => {
        const v = r.cells[header.indexOf(name)];
        return [name, v === undefined || v === "" ? null : v]; // empty cell = absent, never ""
      }),
    ),
  }));

  if (!config.fields || !Object.keys(config.fields).length) return { header, rows, results: null };
  return { header, rows, results: rows.map((row) => transformRow(row, config)) };
}

function transformRow(row, config) {
  const skip = matchSkip(row, config.skip_configs);
  if (skip) return { line: row.line, outcome: "Skipped", details: `skip: ${skip}`, row };

  const fail = (field, message) => ({ line: row.line, outcome: "Failed", details: `${field}: ${message}`, row });
  const read = (spec) => (spec.identifier in row.fields ? row.fields[spec.identifier] : undefined);
  const readMain = (spec) => {
    const raw = read(spec);
    if (raw === undefined) return { error: `column "${spec.identifier}" not in file` };
    const out = applyRules(raw, spec.transformation_rules);
    if (out.error) return out;
    if (out.value === null || out.value === "") return { error: "RequiredFieldMissing" };
    return out;
  };
  const F = config.fields;

  // Main fields fail fast, in the recon engine's order.
  let r = readMain(F.currency);
  if (r.error) return fail("currency", r.error);
  if (!/^[A-Z]{3}$/.test(r.value)) return fail("currency", `InvalidCurrency: "${r.value}"`);
  const currency = r.value;

  r = readMain(F.amount);
  if (r.error) return fail("amount", r.error);
  r = majorToMinor(r.value);
  if (r.error) return fail("amount", r.error);
  const amount = r.value;

  r = readMain(F.balance_direction);
  if (r.error) return fail("balance_direction", r.error);
  const direction = F.balance_direction.credit_values.includes(r.value) ? "credit"
    : F.balance_direction.debit_values.includes(r.value) ? "debit" : null;
  if (!direction) return fail("balance_direction", `InvalidBalanceDirection: "${r.value}"`);

  r = readMain(F.effective_at);
  if (r.error) return fail("effective_at", r.error);
  r = parseDate(r.value, F.effective_at.date_format);
  if (r.error) return fail("effective_at", r.error);
  const effectiveAt = r.value;

  r = readMain(F.order_id);
  if (r.error) return fail("order_id", r.error);
  if (r.value.length > 64) return fail("order_id", `ValueTooLong: ${r.value.length} > 64`);
  const orderId = r.value;

  // Metadata fields collect every error instead of stopping at the first.
  const metadata = {}, exposed = {}, searchable = {}, errors = [];
  for (const m of config.metadata_fields || []) {
    const raw = read(m);
    if (raw === undefined) { if (m.required) errors.push(`${m.field_name}: column "${m.identifier}" not in file`); continue; }
    const out = applyRules(raw, m.transformation_rules);
    if (out.error) { errors.push(`${m.field_name}: ${out.error}`); continue; }
    if ((out.value === null || out.value === "") && m.required) { errors.push(`${m.field_name}: RequiredFieldMissing`); continue; }
    if (out.value !== null && m.type === "major_unit" && !/^-?\d+(\.\d+)?$/.test(out.value)) { errors.push(`${m.field_name}: InvalidType "${out.value}"`); continue; }
    metadata[m.field_name] = out.value;
    if (m.exposed) exposed[m.field_name] = out.value;
    if (m.searchable) searchable[m.field_name] = out.value;
  }
  if (errors.length) return { line: row.line, outcome: "Failed", details: errors.join("; "), row };

  return {
    line: row.line,
    outcome: "Success",
    row,
    entry: {
      order_id: orderId,
      amount,
      currency,
      effective_at: effectiveAt,
      balance_direction: direction,
      metadata,
      exposed_metadata: exposed,
      searchable_metadata: searchable,
      source_line: row.line,
    },
  };
}
