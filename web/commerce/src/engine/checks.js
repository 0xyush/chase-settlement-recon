// The findings that matter for the merchant call, computed from staged rows.
const CHASE_ORDER_ID_LIMIT = 22;
const rows = (n) => `${n} row${n === 1 ? "" : "s"}`;

export function computeChecks(results, config) {
  const entries = results.filter((r) => r.outcome === "Success").map((r) => r.entry);
  const checks = [];

  const longest = Math.max(0, ...entries.map((e) => e.order_id.length));
  const tooLong = entries.filter((e) => e.order_id.length > CHASE_ORDER_ID_LIMIT).length;
  checks.push(tooLong
    ? { level: "bad", text: `${tooLong} order ids exceed Chase's ${CHASE_ORDER_ID_LIMIT} characters.` }
    : { level: "warn", text: `Longest order id is ${longest} characters. Chase holds ${CHASE_ORDER_ID_LIMIT}, and the merchant's ids are 30.` });

  const amex = entries.filter((e) => e.metadata.card_brand === "AX").length;
  if (amex) checks.push({ level: "warn", text: `${rows(amex)} are Amex. Amex may be conveyed: Chase lists them, but Amex pays the merchant.` });

  const key = config.unique_constraint || ["order_id"];
  const counts = {};
  for (const e of entries) {
    const k = key.map((f) => (f === "order_id" ? e.order_id : f === "balance_direction" ? e.balance_direction : e.metadata[f])).join("|");
    counts[k] = (counts[k] || 0) + 1;
  }
  const duplicates = Object.values(counts).filter((n) => n > 1).length;
  checks.push(duplicates
    ? { level: "warn", text: `${duplicates} duplicate key${duplicates === 1 ? "" : "s"} on ${key.join(" + ")}.` }
    : { level: "ok", text: `Unique key ${key.join(" + ")} holds for every staged row.` });

  if (entries.some((e) => "txn_type" in e.metadata)) {
    const moto = entries.filter((e) => ["2", "3"].includes(e.metadata.txn_type)).length;
    checks.push({ level: "ok", text: `${rows(moto)} are phone or mail orders (MOTO).` });
  }

  const reasons = {};
  for (const r of results.filter((x) => x.outcome === "Failed")) {
    const reason = r.details.split(":").slice(0, 2).join(":").replace(/"[^"]*"/, '"…"');
    reasons[reason] = (reasons[reason] || 0) + 1;
  }
  Object.entries(reasons)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 3)
    .forEach(([reason, n]) => checks.push({ level: "bad", text: `${rows(n)} failed on ${reason}.` }));

  return checks;
}
