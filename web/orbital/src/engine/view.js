// Small read helpers over the plain JS objects the engine returns.
import { PRESETS } from "../presets.js";

export const configFor = (detection) =>
  detection.key
    ? structuredClone(PRESETS[detection.key])
    : {
        report: detection.fallbackSection ? `${detection.fallbackSection} (no config yet)` : "Unknown report",
        parsing_config: {
          file_format: "dfr",
          section: detection.fallbackSection,
          record_type: detection.fallbackRecord,
          columns: ["Record Type"],
        },
        skip_configs: [],
        fields: {},
        metadata_fields: [],
        unique_constraint: ["order_id"],
      };

export const reportName = (config) => config.report ?? "Unknown report";
export const metadataText = (entry) => Object.entries(entry.metadata).map(([k, v]) => `${k}=${v ?? "∅"}`).join("  ");
export const orderIdOf = (result, config) => result.entry?.order_id ?? result.row.fields[config.fields?.order_id?.identifier] ?? "";
export const rawCell = (row, column) => row.fields[column] ?? "";
export const rowMatches = (result, query) =>
  !query ||
  `${JSON.stringify(result.entry ?? {})} ${result.details ?? ""} ${Object.values(result.row.fields).join(" ")}`
    .toLowerCase()
    .includes(query.toLowerCase());

const show = (v) => (v === null || v === undefined || v === "" ? "—" : String(v));
export const metadataPairs = (entry) => Object.entries(entry.metadata ?? {}).map(([k, v]) => [k, show(v)]);
export const rawPairs = (row) => Object.entries(row.fields ?? {}).map(([k, v]) => [k, show(v)]);

export const configSummary = (config) => {
  const parsing = config.parsing_config ?? {};
  const fields = config.fields ?? {};
  return {
    report: config.report ?? "Unknown report",
    format: parsing.file_format ?? "dfr",
    section: parsing.section ?? "",
    recordType: parsing.record_type ?? "",
    columns: (parsing.columns ?? []).length,
    delimiter: parsing.delimiter ?? "read from the *DFRBEG line",
    uniqueKey: (config.unique_constraint ?? ["order_id"]).join(" + "),
    skipRules: (config.skip_configs ?? []).length,
    fields: Object.keys(fields).length,
    metadataFields: (config.metadata_fields ?? []).length,
  };
};

export const describeSkip = (config) =>
  (config.skip_configs ?? []).map((s) =>
    (s.conditions ?? []).map((c) => `${c.identifier} ${c.operator} "${c.value}"`).join(" AND "),
  );

// File structure
export const frameText = (frame) =>
  Object.entries(frame?.values ?? {})
    .map(([k, v]) => `${k}=${v}`)
    .join("  ");
export const recordsText = (section) =>
  Object.entries(section.records ?? {})
    .map(([kind, n]) => `${kind} × ${n.toLocaleString()}`)
    .join(", ");
export const fileLine = (file) => {
  if (!file) return "";
  const v = file.begin?.values ?? {};
  const parts = [v.PID && `PID ${v.PID}`, v.CO && `CO ${v.CO}`, v.FREQ, `${file.delimiter}-delimited`];
  return parts.filter(Boolean).join(" · ") + ` · sections ${file.sections.map((s) => s.code).join(", ") || "none"}`;
};
export const netText = (byCurrency) =>
  Object.entries(byCurrency ?? {})
    .map(([currency, cents]) => `${currency} ${formatMoney(cents)}`)
    .join(", ") || "—";

const formatMoney = (minor) => {
  const sign = minor < 0 ? "-" : "";
  const abs = Math.abs(minor);
  return `${sign}${Math.floor(abs / 100).toLocaleString()}.${String(abs % 100).padStart(2, "0")}`;
};
