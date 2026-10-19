// Works out which Chase report a file is, and which line holds the column names.
export function detectReport(records) {
  const metaRecord = records.slice(0, 3).find((r) => r.cells.join(",").includes("ReportType"));
  const headerIndex = records.findIndex(
    (r, i) =>
      i < 15 &&
      r.cells.filter(Boolean).length >= 4 &&
      !r.cells.some((c) => c.includes("=")) &&
      r.cells[0] !== "BEGIN",
  );
  const header = headerIndex >= 0 ? records[headerIndex].cells : [];

  let key = null;
  if (header.includes("Merchant Action Code") && header.includes("Submission Date")) key = "deposit_details";
  else if (header.includes("Settled Deposit Amount")) key = "transaction_fee_details";
  else if (header.includes("Fund Transfer Identifier") && header.includes("Action Type Code")) key = "funded_txn_fee_details";

  return {
    key,
    headerIndex,
    meta: metaRecord ? metaRecord.cells.filter(Boolean).join(", ") : "",
  };
}
