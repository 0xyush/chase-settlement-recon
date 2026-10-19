// Small read helpers over the plain JS objects the engine returns.
import { PRESETS } from "../presets";

export const configFor = (detection) =>
  detection.key
    ? structuredClone(PRESETS[detection.key])
    : {
        report: "Unknown report",
        parsing_config: { file_format: "csv", header_row_index: Math.max(0, detection.headerIndex), footer_marker: "END" },
        skip_configs: [],
        fields: {},
        metadata_fields: [],
        unique_constraint: ["order_id"],
      };

export const reportName = (config) => config.report ?? "Unknown report";
export const headerLine = (records, config) => records[config.parsing_config?.header_row_index ?? 0]?.line ?? 0;
export const metadataText = (entry) => Object.entries(entry.metadata).map(([k, v]) => `${k}=${v ?? "∅"}`).join("  ");
export const orderIdOf = (result, config) => result.entry?.order_id ?? result.row.fields[config.fields?.order_id?.identifier] ?? "";
export const rawCell = (row, column) => row.fields[column] ?? "";
export const rowMatches = (result, query) =>
  !query ||
  `${JSON.stringify(result.entry ?? {})} ${result.details ?? ""} ${Object.values(result.row.fields).join(" ")}`
    .toLowerCase()
    .includes(query.toLowerCase());

// --- Read helpers added for the Control Center style UI ---
const show = (v) => (v === null || v === undefined || v === "" ? "—" : String(v));
export const metadataPairs = (entry) => Object.entries(entry.metadata ?? {}).map(([k, v]) => [k, show(v)]);
export const rawPairs = (row) => Object.entries(row.fields ?? {}).map(([k, v]) => [k, show(v)]);
export const configSummary = (config) => {
  const fields = config.fields ?? {};
  return {
    report: config.report ?? "Unknown report",
    format: config.parsing_config?.file_format ?? "csv",
    headerRow: config.parsing_config?.header_row_index ?? 0,
    footer: config.parsing_config?.footer_marker ?? "",
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
