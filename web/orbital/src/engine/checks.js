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
        ? { level: "ok", text: "Staged detail ties to Chase's own FIN0010 summary for every card brand: nothing was lost between the file and staging." }
        : { level: "bad", text: `${differ} card brand line${differ === 1 ? "" : "s"} do not tie to Chase's FIN0010 summary. See the Tie-out tab.` },
    );
    const conveyed = tie.lines.filter((l) => (l.settled_conveyed ?? "").includes("C"));
    if (conveyed.length) {
      const brands = [...new Set(conveyed.map((l) => BRANDS[l.card_brand] ?? l.card_brand))].join(", ");
      const total = Object.entries(tie.conveyed_net).map(([c, v]) => `${c} ${formatMinor(v)}`).join(", ");
      checks.push({
        level: "warn",
        text: `${brands} is conveyed (C): Chase lists ${total}, but the card network pays it to the merchant directly. It is not in Chase's deposit.`,
      });
    } else if (entries.some((e) => e.metadata.card_brand === "AX")) {
      checks.push({ level: "ok", text: "Amex is settled (S) in this file: Chase pays it with everything else." });
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
      text: `Chargeback stages in this file: ${list}. Only RTM and PARRTM (returned to the merchant) take money from the merchant, so only they are staged, as debits; RECD and REPR stay in Chase's inventory.`,
    });
  }

  if (entries.length) {
    const longest = Math.max(...entries.map((e) => e.order_id.length));
    const tooLong = entries.filter((e) => e.order_id.length > CHASE_ORDER_ID_LIMIT).length;
    checks.push(
      tooLong
        ? { level: "bad", text: `${tooLong} order ids exceed Chase's ${CHASE_ORDER_ID_LIMIT} characters.` }
        : { level: "warn", text: `Longest order id is ${longest} characters. Chase holds ${CHASE_ORDER_ID_LIMIT}, and the merchant's ids are 30.` },
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
        ? { level: "warn", text: `${duplicates} duplicate key${duplicates === 1 ? "" : "s"} on ${key.join(" + ")}.` }
        : { level: "ok", text: `Unique key ${key.join(" + ")} holds for every staged row.` },
    );
  } else if (results.length === 0 && output.file) {
    checks.push({ level: "ok", text: "No activity: the file has its report headers and no data records, which is how Chase sends a quiet day." });
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
