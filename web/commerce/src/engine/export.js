// Downloads: the staged entries, and a per-row report in the recon repo's report-v1.csv shape.
export function stagingEntriesJson(results) {
  return JSON.stringify(results.filter((r) => r.outcome === "Success").map((r) => r.entry), null, 2);
}

export function rowReportCsv(results) {
  const q = (v) => `"${String(v).replace(/"/g, '""')}"`;
  const lines = results.map((r) => ["file_row", r.line, r.outcome, r.outcome === "Success" ? "" : r.details].map(q).join(","));
  return ["source_type,identifier,outcome,details", ...lines].join("\n");
}

export function download(filename, text, type) {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([text], { type }));
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}

export const formatMinor = (minor) => {
  const sign = minor < 0 ? "-" : "";
  const abs = Math.abs(minor);
  return `${sign}${Math.floor(abs / 100).toLocaleString()}.${String(abs % 100).padStart(2, "0")}`;
};
