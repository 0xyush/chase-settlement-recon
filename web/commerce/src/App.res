// Page shell copied from HyperSwitchApp.res (sidebar + navbar + "p-6 md:px-12 md:py-8 flex flex-col gap-*"
// content column), page built from the Recon Engine screens' pieces.
type loaded = {name: string, text: string, records: array<Engine.record_>, detection: Engine.detection}

let count = (results: array<Engine.rowResult>, outcome) =>
  results->Array.filter(r => r.outcome == outcome)->Array.length

let wideScreen: bool = %raw(`typeof window !== "undefined" && window.innerWidth >= 1024`)

module Results = {
  @react.component
  let make = (
    ~file: loaded,
    ~config: Engine.config,
    ~output: Engine.output,
    ~onApply: Engine.config => unit,
    ~onReset: unit => unit,
    ~onSelect: Engine.rowResult => unit,
    ~onMatched: option<Hyperswitch.summary> => unit,
  ) => {
    let (filter, setFilter) = React.useState(() => "All")
    let (tab, setTab) = React.useState(() => "entries")
    let (matchRun, setMatchRun) = React.useState((): option<(string, Hyperswitch.output)> => None)
    let (matching, setMatching) = React.useState(() => false)
    let (matchFilter, setMatchFilter) = React.useState(() => "all")

    // A new engine answer (another file or config) makes an earlier match stale.
    React.useEffect1(() => {
      setMatchRun(_ => None)
      onMatched(None)
      None
    }, [output])

    // The next step: match the staged entries with a Hyperswitch export in the Rust engine.
    let runMatch = (source, csv) =>
      switch output.results->Nullable.toOption {
      | Some(results) =>
        setMatching(_ => true)
        Hyperswitch.matchWith(results, csv)
        ->Promise.thenResolve(out => {
          setMatchRun(_ => Some((source, out)))
          setMatching(_ => false)
          onMatched(out.summary)
        })
        ->ignore
      | None => ()
      }
    let useMock = () =>
      Hyperswitch.fetchMockExport(file.name)
      ->Promise.thenResolve(text =>
        switch text->Nullable.toOption {
        | Some(csv) => runMatch(Hyperswitch.mockExportName(file.name), csv)
        | None =>
          setMatchRun(_ => Some((
            "",
            {
              rejected: Some("There is no mock Hyperswitch export for this file. Upload one instead."),
              summary: None,
              items: None,
              report_csv: None,
            },
          )))
        }
      )
      ->ignore
    let viewItems = f => {
      setMatchFilter(_ => f)
      setTab(_ => "match")
    }

    // Each view is a tab with an id, so a button elsewhere on the page can open it.
    let show = (views: array<(string, Tabs.tab, React.element)>) => {
      let ids = views->Array.map(((id, _, _)) => id)
      let found = ids->Array.indexOf(tab)
      let selected = found < 0 ? 0 : found
      <div className="flex flex-col gap-4">
        <Tabs
          tabs={views->Array.map(((_, t, _)) => t)}
          selected
          onSelect={i => setTab(_ => ids->Array.get(i)->Option.getOr("entries"))}
        />
        {views->Array.get(selected)->Option.mapOr(React.null, ((_, _, view)) => view)}
      </div>
    }
    let editor = ("config", {Tabs.title: "Transformation config", count: None}, <ConfigEditor config onApply onReset />)
    let matchTab = switch matchRun {
    | Some((source, {summary: Some(s), items: Some(items), report_csv})) => [
        (
          "match",
          {Tabs.title: "Match results", count: Some(items->Array.length)},
          <MatchPanel
            summary=s
            items
            reportCsv={report_csv->Option.getOr("")}
            source
            fileName=file.name
            filter=matchFilter
            setFilter={f => setMatchFilter(_ => f)}
          />,
        ),
      ]
    | _ => []
    }
    let nextStep = (~staged, ~connector) =>
      <MatchNextStep staged connector run=matchRun matching onUseMock=useMock onUpload=runMatch onViewItems=viewItems />
    let cards = (rows, staged, skipped, failed) =>
      <SummaryCards
        cards=[
          {title: "Parsed", value: rows, filter: "All", valueClass: "text-nd_gray-800"},
          {title: "Staged", value: staged, filter: "Success", valueClass: "text-nd_green-400"},
          {title: "Skipped", value: skipped, filter: "Skipped", valueClass: skipped > 0 ? "text-nd_orange-600" : "text-nd_gray-800"},
          {title: "Failed", value: failed, filter: "Failed", valueClass: failed > 0 ? "text-nd_red-500" : "text-nd_gray-800"},
        ]
        active=filter
        onSelect={f => {
          setFilter(_ => f)
          setTab(_ => "entries")
        }}
      />
    switch output.rejected {
    | Some(reason) =>
      <>
        <Helpers.InfoBox tone=Error message={"File rejected: " ++ reason} />
        <ConfigEditor config onApply onReset />
      </>
    | None =>
      let rows = output.rows->Option.getOr([])
      switch output.results->Nullable.toOption {
      | None =>
        <>
          <Helpers.InfoBox
            message="No transformation config matched this report, so nothing was mapped. The parsed rows are below; add its columns under Transformation config to stage them."
          />
          {show([
            ("rows", {Tabs.title: "Parsed rows", count: Some(rows->Array.length)}, <RawTable header={output.header->Option.getOr([])} rows />),
            editor,
          ])}
        </>
      | Some(results) =>
        let staged = count(results, "Success")
        <>
          {cards(rows->Array.length, staged, count(results, "Skipped"), count(results, "Failed"))}
          <Findings checks={Engine.computeChecks(results, config)} />
          {nextStep(~staged, ~connector="jpmorgan")}
          {show(
            [
              (
                "entries",
                {Tabs.title: "Entries", count: Some(results->Array.length)},
                <EntriesTable results config text=file.text filter fileName=file.name onSelect />,
              ),
            ]
            ->Array.concat(matchTab)
            ->Array.concat([editor]),
          )}
        </>
      }
    }
  }
}

@react.component
let make = () => {
  let (file, setFile) = React.useState((): option<loaded> => None)
  let (config, setConfig) = React.useState((): option<Engine.config> => None)
  let (selected, setSelected) = React.useState((): option<Engine.rowResult> => None)
  let (sidebarExpanded, setSidebarExpanded) = React.useState(() => wideScreen)

  let (output, setOutput) = React.useState((): option<Engine.output> => None)
  let (matched, setMatched) = React.useState((): option<Hyperswitch.summary> => None)
  let latestRequest = React.useRef(0)

  let load = (name, text) => {
    let records = Engine.parseCSV(text)
    let detection = Engine.detectReport(records)
    setOutput(_ => None)
    setMatched(_ => None)
    setFile(_ => Some({name, text, records, detection}))
    setConfig(_ => Some(Engine.configFor(detection)))
  }
  let replace = () => {
    setMatched(_ => None)
    setFile(_ => None)
    setConfig(_ => None)
    setOutput(_ => None)
    setSelected(_ => None)
  }

  // Ask the Rust engine (JavaScript fallback) whenever the file or the config changes.
  // Only the newest answer is kept, so a slow older request cannot overwrite a newer one.
  React.useEffect2(() => {
    switch (file, config) {
    | (Some(f), Some(c)) =>
      latestRequest.current = latestRequest.current + 1
      let request = latestRequest.current
      Engine.runTransform(f.text, f.records, c)
      ->Promise.thenResolve(out =>
        if latestRequest.current == request {
          setOutput(_ => Some(out))
        }
      )
      ->ignore
    | _ => ()
    }
    None
  }, (file, config))

  let pending = switch output {
  | Some(out) => out.results->Nullable.toOption->Option.mapOr(0, r => count(r, "Success"))
  | None => 0
  }

  <div className="flex h-screen bg-hyperswitch_background">
    <Sidebar expanded=sidebarExpanded />
    <div id="page-scroll" className="flex relative flex-col flex-1 min-w-0 overflow-y-auto overflow-x-hidden">
      <div className="w-full max-w-fixedPageWidth md:px-12 px-5 pt-3">
        <Navbar onToggleSidebar={() => setSidebarExpanded(v => !v)} pending />
      </div>
      <div className="p-6 md:px-12 md:py-8 flex flex-col gap-6 max-w-fixedPageWidth min-h-full">
        <PageHeading
          breadcrumb=["Pipelines", "Chase transformation"]
          title="Chase transformation"
          subTitle="Drop a J.P. Morgan settlement report. The Rust engine parses it, skips, maps and validates every row into a staging entry, writes the reports, and matches the entries against Hyperswitch."
        />
        {switch (file, config, output) {
        | (Some(f), Some(c), Some(out)) =>
          <>
            <FileBar
              name=f.name
              report={Engine.reportName(c)}
              hasPreset={f.detection.key->Nullable.toOption->Option.isSome}
              meta=f.detection.meta
              rows={out.rows->Option.getOr([])->Array.length}
              headerLine={Engine.headerLine(f.records, c)}
              engine={out.engine->Option.getOr("javascript")}
              onReplace=replace
            />
            {switch out.steps {
            | Some(steps) if steps->Array.length > 0 => <StepsSnackbar steps matched fileName=f.name />
            | _ => React.null
            }}
            <Results
              key=f.name
              file=f
              config=c
              output=out
              onApply={c => setConfig(_ => Some(c))}
              onReset={() => setConfig(_ => Some(Engine.configFor(f.detection)))}
              onSelect={r => setSelected(_ => Some(r))}
              onMatched={summary => setMatched(_ => summary)}
            />
          </>
        | (Some(_), Some(_), None) => <Helpers.InfoBox message="Running the Rust engine on this file…" />
        | _ => <UploadDropzone onLoad=load />
        }}
      </div>
    </div>
    {switch config {
    | Some(c) => <EntryDrawer result=selected config=c onClose={() => setSelected(_ => None)} />
    | None => React.null
    }}
  </div>
}
