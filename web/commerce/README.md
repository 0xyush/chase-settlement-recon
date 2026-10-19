# Chase recon prototype dashboard

Drop a Chase (J.P. Morgan) settlement report CSV and see how the hyperswitch-recon
pipeline would turn it into staging entries: parse, skip, map, validate.
Built in ReScript + React with the same versions and colors as the Hyperswitch Control Center.

## Run

```
../../scripts/build.sh   # once: builds the Rust engine to WebAssembly into src/engine/wasm
npm install
npm run dev              # compiles ReScript once, then starts Vite
npm run res:watch        # optional, in a second terminal: recompile .res files on save
```

## Layout

The UI copies the Hyperswitch Control Center's structure (its Recon Engine screens): the
sidebar + navbar shell, `PageHeading`, the pipelines upload dropzone, overview stat cards,
`LoadedTable` tables with the "Showing x-y of n" paginator, Blend-style status tags and a
right-hand details drawer. Tailwind 3 (same major as the Control Center) with the `nd_*`
color scales, `fs-*` font sizes and `theme.json` colors copied into `tailwind.config.js`.

```
src/
  Main.res              mounts the app and imports theme.css
  App.res               page shell (sidebar + navbar + content column) and the page itself
  Engine.res            bindings to the engines and the shapes they return
  Hyperswitch.res       match types, labels and colors for the Match with Hyperswitch tab
  theme.css             Tailwind directives + base styles (tokens live in tailwind.config.js)
  components/
    Sidebar, Navbar     app shell: Recon Engine sidebar (Operate / Monitor / Configure), top bar
    PageHeading         breadcrumb + title + one-line description
    UploadDropzone      drag-and-drop zone with "Browse Files" and the three sample buttons
    FileBar             compact loaded-file row: name, report tag, row count, header line, Replace file
    SummaryCards        Parsed / Staged / Skipped / Failed cards; clicking one filters the table
    Findings            the checks, with status dots
    Tabs                Entries | Transformation config
    EntriesTable        search, downloads (written by Rust), paginated entries table; row click opens the drawer
    MatchPanel          Match with Hyperswitch tab: mock or uploaded export, result cards, paginated items
    RawTable            parsed rows for a report with no mapping
    EntryDrawer         entry details: staged fields, metadata, raw CSV row, or the failure/skip reason
    ConfigEditor        parsing / skip-rule summary + JSON editor with Re-run and Reset
    Table               table primitives (wrapper, head, row, cell, empty state, paginator)
    Button, Tag, Icon, SearchInput, Helpers, Typography   Control Center style primitives
  engine/               the pipeline, in plain JavaScript
    csv.js              1. parse CSV into records with line numbers
    detect.js           which Chase report it is, where the header row is
    skip.js             2. skip configs (AND inside a config, OR across configs)
    rules.js            transformation rules, amount to minor units, M/D/YYYY dates
    transform.js        3-4. map and validate each row into a staging entry (JS fallback for the Rust engine)
    backend.js          calls the Rust engine (/api/transform, /api/export, /api/match), falls back to JS
    checks.js           findings for the merchant call (22-char ids, Amex, duplicates, MOTO)
    export.js           staging entries JSON, row report CSV
    view.js             read helpers the components use (config summary, metadata / raw row pairs)
  presets.js            transformation config per Chase report (Deposit Details, Transaction Fee, Funded Fee)
tailwind.config.js      Control Center token subset (nd_gray, nd_primary_blue, nd_green, nd_red, nd_orange, ...)
public/samples/         three J.P. Morgan sample reports
```

## Rust engine

The page runs the Rust engine from `../../engines/commerce` right in the browser, compiled to
WebAssembly (`src/engine/wasm`, built by `scripts/build.sh` at the repo root). No server needed.

The file bar shows which engine answered: **Rust engine**, or **JS fallback** if the
WebAssembly could not load (then `src/engine/transform.js` does the work and matching is
unavailable). `src/engine/backend.js` is the only place that knows about the two engines.
