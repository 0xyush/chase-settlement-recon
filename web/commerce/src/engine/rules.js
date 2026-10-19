// Transformation rules run on the raw string, in array order, before any parsing.
// Mirrors the rules in hyperswitch-recon (trim, absolute, negate, starts_with, regex, default_value).
export function applyRules(value, rules = []) {
  let v = value;
  for (const rule of rules) {
    if (v === null) {
      if (rule.type === "default_value") v = rule.value; // only fires on a missing cell
      continue;
    }
    switch (rule.type) {
      case "trim": v = v.trim(); break;
      case "absolute": v = v.startsWith("-") ? v.slice(1) : v; break;
      case "negate": v = v === "" ? v : v.startsWith("-") ? v.slice(1) : `-${v}`; break;
      case "starts_with": v = v.startsWith(rule.prefix) ? rule.then : rule.otherwise; break;
      case "regex": {
        const match = new RegExp(rule.pattern).exec(v);
        if (!match) return { error: `UnmatchedRegex: "${v}" does not match ${rule.pattern}` };
        v = match[rule.group ?? 0] ?? null;
        break;
      }
      default: break;
    }
  }
  return { value: v };
}

// "20.5" -> 2050. String math, never floats, like MinorUnit in the recon repo.
export function majorToMinor(s) {
  const m = /^(-?)(\d+)(?:\.(\d+))?$/.exec(s);
  if (!m) return { error: `InvalidType: "${s}" is not a number` };
  const fraction = m[3] || "";
  if (fraction.length > 2) return { error: `PrecisionLoss: "${s}" has more than 2 decimals` };
  const minor = parseInt(m[2], 10) * 100 + parseInt((fraction + "00").slice(0, 2), 10);
  return { value: m[1] ? -minor : minor };
}

// "3/3/2025" with { order: "month_day_year", delimiter: "slash" } -> "2025-03-03 00:00:00"
export function parseDate(s, format) {
  const m = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(s);
  if (!m || format?.order !== "month_day_year") return { error: `InvalidFormat: "${s}" is not M/D/YYYY` };
  const [month, day, year] = [+m[1], +m[2], +m[3]];
  const date = new Date(Date.UTC(year, month - 1, day));
  if (date.getUTCMonth() !== month - 1 || date.getUTCDate() !== day) return { error: `InvalidFormat: "${s}" is not a real date` };
  const pad = (n) => String(n).padStart(2, "0");
  return { value: `${year}-${pad(month)}-${pad(day)} 00:00:00` };
}
