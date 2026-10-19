// Picks the config for a DFR file from its section headers (lines starting with H).
// The engine does the real reading; this only chooses which config to send with the file.
export function detectFile(text) {
  const lines = text.split(/\r?\n/).filter((line) => line.trim());
  const isDfr = (lines[0] ?? "").startsWith("*DFRBEG");
  const delimiter = isDfr ? lines[0].charAt(7) : ",";
  const sections = [];
  const records = {};
  let current = null;
  for (const line of lines) {
    const code = line.split(delimiter)[0].trim();
    if (code.startsWith("H") && code.length > 1) {
      current = code.slice(1);
      sections.push(current);
      records[current] = {};
    } else if (code.startsWith("R") && current) {
      records[current][code] = (records[current][code] ?? 0) + 1;
    }
  }
  const key = sections.includes("ACT0010")
    ? act0010Layout(lines, delimiter)
    : sections.includes("PDE0017")
      ? "pde0017_chargebacks"
      : null;
  // No config for this report yet: read its first section, using its most common data record.
  const first = sections[0] ?? "";
  const common = Object.entries(records[first] ?? {}).sort((a, b) => b[1] - a[1])[0]?.[0];
  return { key, isDfr, sections, fallbackSection: first, fallbackRecord: common ?? `R${first}` };
}

// Chase's spec gives ACT0010 30 fields with the interchange option. Live files have 32: one more
// field before the fees and the settlement currency last. Count the first data line to choose.
function act0010Layout(lines, delimiter) {
  const row = lines.find((line) => line.startsWith("RACT0010"));
  if (!row) return "act0010_deposit_detail";
  const fields = [];
  let cell = "";
  let quoted = false;
  for (const c of row) {
    if (c === '"') quoted = !quoted;
    else if (c === delimiter && !quoted) {
      fields.push(cell);
      cell = "";
    } else cell += c;
  }
  fields.push(cell);
  // A trailing delimiter adds an empty field, not a column.
  while (fields.length && !fields[fields.length - 1].trim()) fields.pop();
  return fields.length >= 31 ? "act0010_deposit_detail_live" : "act0010_deposit_detail";
}
