// Step 1 of the pipeline: split raw CSV text into records.
// RFC 4180 quoting, CRLF tolerant. Blank lines are dropped, like Rust's csv crate.
// Each record keeps the file line it starts on, so errors point at the right line.
export function parseCSV(text) {
  if (text.charCodeAt(0) === 0xfeff) text = text.slice(1); // strip the byte order mark
  const records = [];
  let cells = [], field = "", quoted = false, line = 1, start = 1;

  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (quoted) {
      if (c === '"') {
        if (text[i + 1] === '"') { field += '"'; i++; } else quoted = false;
      } else {
        if (c === "\n") line++;
        field += c;
      }
      continue;
    }
    if (c === '"') quoted = true;
    else if (c === ",") { cells.push(field); field = ""; }
    else if (c === "\r") { /* ignore */ }
    else if (c === "\n") {
      cells.push(field);
      records.push({ cells, line: start });
      cells = []; field = ""; line++; start = line;
    } else field += c;
  }
  if (field !== "" || cells.length) { cells.push(field); records.push({ cells, line: start }); }

  return records.filter((r) => !(r.cells.length === 1 && r.cells[0] === ""));
}
