// The findings that matter for the merchant call, from the engine's answer.
import { formatMinor } from "./export.js";

const CHASE_ORDER_ID_LIMIT = 22;
const BRANDS = { VI: "Visa", MC: "Mastercard", AX: "Amex", DI: "Discover" };
const rows = (n) => `${n.toLocaleString()} row${n === 1 ? "" : "s"}`;

export function computeChecks(output, config) {
  const results = output.results ?? [];
  const entries = results.filter((r) => r.outcome === "Success").map((r) => r.entry);
  const tie = output.tieout;
  const checks = [];

  if (tie) {
    const differ = tie.lines.filter((l) => l.status !== "tied").length;
    checks.push(
      tie.tied
        ? { level: "ok", text: "Every row adds up to Chase's daily summary. Nothing is missing." }
        : { level: "bad", text: `${differ} card line${differ === 1 ? " doesn't" : "s don't"} add up to Chase's daily summary. See Tie-out.` },
    );
    const conveyed = tie.lines.filter((l) => (l.settled_conveyed ?? "").includes("C"));
    if (conveyed.length) {
      const brands = [...new Set(conveyed.map((l) => BRANDS[l.card_brand] ?? l.card_brand))].join(", ");
      const total = Object.entries(tie.conveyed_net).map(([c, v]) => `${c} ${formatMinor(v)}`).join(", ");
      checks.push({
        level: "warn",
        text: `${brands} (${total}) is paid by ${brands === "Amex" ? "Amex" : "the card network"} directly, not by Chase. It won't be in Chase's deposit.`,
      });
    } else if (entries.some((e) => e.metadata.card_brand === "AX")) {
      checks.push({ level: "ok", text: "Chase pays Amex here, together with the other cards." });
    }
  }

  if (config.parsing_config?.section === "PDE0017" && results.length) {
    const stages = {};
    for (const r of results) {
      const stage = r.row.fields["Category"] ?? "?";
      stages[stage] = (stages[stage] || 0) + 1;
    }
    const list = Object.entries(stages).map(([stage, n]) => `${stage} ${n}`).join(", ");
    checks.push({
      level: "ok",
      text: `Chargebacks: ${list}. Only the returned ones (RTM, PARRTM) take money, so only those are staged.`,
    });
  }

  if (entries.length) {
    const longest = Math.max(...entries.map((e) => e.order_id.length));
    const tooLong = entries.filter((e) => e.order_id.length > CHASE_ORDER_ID_LIMIT).length;
    checks.push(
      tooLong
        ? { level: "bad", text: `${tooLong} order IDs are longer than the ${CHASE_ORDER_ID_LIMIT} characters Chase keeps.` }
        : { level: "warn", text: `Chase keeps only ${CHASE_ORDER_ID_LIMIT} characters of an order ID. The merchant's IDs are 30, so they won't fit as is.` },
    );

    const key = config.unique_constraint || ["order_id"];
    const counts = {};
    for (const e of entries) {
      const k = key
        .map((f) => (f === "order_id" ? e.order_id : f === "balance_direction" ? e.balance_direction : e.metadata[f]))
        .join("|");
      counts[k] = (counts[k] || 0) + 1;
    }
    const duplicates = Object.values(counts).filter((n) => n > 1).length;
    checks.push(
      duplicates
        ? { level: "warn", text: `${duplicates} duplicate row${duplicates === 1 ? "" : "s"} (same ${key.join(" + ")}).` }
        : { level: "ok", text: "No duplicate rows." },
    );
  } else if (results.length === 0 && output.file) {
    checks.push({ level: "ok", text: "Quiet day: the file has no transactions." });
  }

  const reasons = {};
  for (const r of results.filter((x) => x.outcome === "Failed")) {
    const reason = r.details.split(":").slice(0, 2).join(":").replace(/"[^"]*"/, '"…"');
    reasons[reason] = (reasons[reason] || 0) + 1;
  }
  Object.entries(reasons)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 3)
    .forEach(([reason, n]) => checks.push({ level: "bad", text: `${rows(n)} failed: ${reason}.` }));

  return checks;
}
