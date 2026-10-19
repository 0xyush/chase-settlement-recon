# Orbital recon dashboard

The Commerce Center dashboard (`../commerce`), adapted for Chase Orbital /
Paymentech **Delimited File Reports (DFR)**. Same Hyperswitch Control Center look, ReScript +
React + Tailwind. All reading, transforming, tie-out and matching happens in the Rust engine
(`../../engines/orbital`); there is no JavaScript fallback.

## Run

```
../../scripts/build.sh   # once: builds the Rust engine to WebAssembly into src/engine/wasm
npm install
npm run dev
```

The Rust engine from `../../engines/orbital` runs right in the browser as WebAssembly; there is
no server and no JavaScript fallback.

## What is different from the Commerce dashboard

- Drop a `.dfr` file, or pick one of the five synthetic samples (served from `public/samples`, a
  link to `../../engines/orbital/samples`).
- The config is picked from the file's sections: `HACT0010` -> Deposit Detail,
  `HPDE0017` -> Chargeback Activity (`src/engine/detect.js`).
- **Tie-out** tab: staged ACT0010 detail per card brand next to Chase's FIN0010 summary, with the
  Settled / Conveyed flag, Chase's net deposit, and fee / chargeback lines.
- **File structure** tab: the file line by line: `*DFRBEG`, each section with its record counts,
  `*DFREND` (or that it is missing).
- Findings cover the tie-out, conveyed Amex, and which chargeback stages move money.
- Configs in `src/presets/` are copies of the engine's `configs/`; refresh with
  `npm run sync-configs`.
