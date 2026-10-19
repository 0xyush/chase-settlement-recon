// "Transformation config" tab: Parsing / Skip rules summary (TransformationRunDetails sections)
// next to the JSON editor with Re-run / Reset in Control Center button styles.
open Typography

let pretty = config => config->JSON.stringifyWithIndent(2)

@react.component
let make = (~config: Engine.config, ~onApply: Engine.config => unit, ~onReset: unit => unit) => {
  let (text, setText) = React.useState(() => pretty(config))
  let (error, setError) = React.useState((): option<string> => None)

  React.useEffect1(() => {
    setText(_ => pretty(config))
    setError(_ => None)
    None
  }, [config])

  let apply = _ =>
    switch JSON.parseExn(text) {
    | parsed => onApply(parsed)
    | exception Exn.Error(e) => setError(_ => Some("Invalid JSON: " ++ e->Exn.message->Option.getOr("")))
    }

  let summary = Engine.configSummary(config)
  let skips = Engine.describeSkip(config)
  let dirty = text != pretty(config)

  <div className="grid grid-cols-1 md:grid-cols-[minmax(0,2fr)_minmax(0,3fr)] gap-4">
    <div className="flex flex-col gap-5 border border-nd_gray-150 rounded-lg p-4 bg-white self-start">
      <div className="flex flex-col gap-2">
        <Helpers.SectionTitle> {React.string("Parsing")} </Helpers.SectionTitle>
        <div className="flex flex-col gap-1.5">
          <Helpers.MetaRow label="Report" value=summary.report />
          <Helpers.MetaRow label="Format" value={String.toUpperCase(summary.format)} />
          <Helpers.MetaRow label="Section" value={`H${summary.section}`} mono=true />
          <Helpers.MetaRow label="Data records" value=summary.recordType mono=true />
          <Helpers.MetaRow label="Named fields" value={`${Int.toString(summary.columns)}, by position`} />
          <Helpers.MetaRow label="Delimiter" value=summary.delimiter />
          <Helpers.MetaRow label="Unique key" value=summary.uniqueKey mono=true />
          <Helpers.MetaRow label="Main fields" value={Int.toString(summary.fields)} />
          <Helpers.MetaRow label="Metadata fields" value={Int.toString(summary.metadataFields)} />
        </div>
      </div>
      <div className="flex flex-col gap-2">
        <Helpers.SectionTitle count={skips->Array.length}> {React.string("Skip rules")} </Helpers.SectionTitle>
        {skips->Array.length == 0
          ? <p className={`${body.xs.regular} text-nd_gray-500`}>
              {React.string("No rows skipped — every parsed row is mapped.")}
            </p>
          : <div className="flex flex-col gap-1.5">
              {skips
              ->Array.mapWithIndex((rule, i) =>
                <div
                  key={Int.toString(i)}
                  className={`${code.md.regular} bg-nd_gray-50 border border-nd_gray-150 rounded-lg px-2.5 py-1.5 text-nd_gray-700 break-words`}>
                  {React.string(rule)}
                </div>
              )
              ->React.array}
            </div>}
      </div>
    </div>
    <div className="flex flex-col gap-3">
      <textarea
        ariaLabel="Transformation config JSON"
        value=text
        spellCheck=false
        onChange={e => setText(_ => ReactEvent.Form.target(e)["value"])}
        className={`w-full h-96 ${code.md.regular} text-nd_gray-800 bg-white border border-nd_gray-200 rounded-lg p-3 resize-y focus:outline-none focus:border-nd_primary_blue-500`}
      />
      <div className="flex flex-wrap items-center gap-2">
        <Button
          buttonType=Primary
          buttonSize=Small
          text="Re-run"
          leftIcon={<Icon name="play" size=12 />}
          onClick=apply
        />
        <Button
          buttonType=Secondary
          buttonSize=Small
          text="Reset to preset"
          leftIcon={<Icon name="rotate-ccw" size=14 className="text-nd_gray-500" />}
          onClick={_ => onReset()}
        />
        {switch error {
        | Some(message) => <span className={`${body.sm.medium} text-nd_red-500`}> {React.string(message)} </span>
        | None =>
          dirty
            ? <span className={`${body.sm.medium} text-nd_gray-500`}> {React.string("Unsaved edits")} </span>
            : React.null
        }}
      </div>
    </div>
  </div>
}
