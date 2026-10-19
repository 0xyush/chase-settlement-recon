// Step 2: skip configs. Conditions inside one config are AND-ed; configs are OR-ed.
// A condition on a column that is not in the file never matches (silent, like the recon repo).
export function matchSkip(row, skipConfigs = []) {
  for (const config of skipConfigs) {
    const conditions = config.conditions || [];
    if (!conditions.length) continue;
    const matched = conditions.every((c) => {
      if (!(c.identifier in row.fields)) return false;
      const v = row.fields[c.identifier];
      switch (c.operator) {
        case "equals": return v === c.value;
        case "not_equals": return v !== c.value;
        case "starts_with": return v != null && c.value != null && v.startsWith(c.value);
        case "contains": return v != null && c.value != null && v.includes(c.value);
        case "ends_with": return v != null && c.value != null && v.endsWith(c.value);
        default: return false;
      }
    });
    if (matched) return conditions.map((c) => `${c.identifier} ${c.operator} ${JSON.stringify(c.value)}`).join(" AND ");
  }
  return null;
}
