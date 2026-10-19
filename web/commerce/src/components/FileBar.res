// The selected-file row from ReconEnginePipelinesUploadModal (p-3 border rounded-lg bg-nd_gray-50),
// with the detected report as a Tag and the report metadata underneath.
open Typography

@react.component
let make = (
  ~name: string,
  ~report: string,
  ~hasPreset: bool,
  ~meta: string,
  ~rows: int,
  ~headerLine: int,
  ~engine: string,
  ~onReplace: unit => unit,
) => {
  let secondary = `Header line ${Int.toString(headerLine)}` ++ (meta == "" ? "" : ` · ${meta}`)
  <div className="p-3 border border-nd_gray-200 rounded-lg bg-nd_gray-50">
    <div className="flex items-center justify-between gap-3">
      <div className="flex min-w-0 items-center gap-2">
        <Icon name="nd-file" size=16 className="text-nd_gray-500" />
        <span className={`${body.sm.medium} text-nd_gray-700 truncate`} title=name> {React.string(name)} </span>
        <Tag text=report color={hasPreset ? Primary : Neutral} uppercase=false />
        {engine == "rust"
          ? <span title="The Rust engine, compiled to WebAssembly, running in this page">
              <Tag text="Rust engine" color=Success uppercase=false />
            </span>
          : <span title="The Rust engine could not load in this browser, so the JavaScript fallback answered">
              <Tag text="JS fallback" color=Warning uppercase=false />
            </span>}
        <span className={`${body.xs.regular} text-nd_gray-500 shrink-0 hidden sm:inline`}>
          {React.string(`${Int.toLocaleString(rows)} rows`)}
        </span>
      </div>
      <Button buttonType=Secondary buttonSize=Small text="Replace file" onClick={_ => onReplace()} />
    </div>
    <div className={`mt-1.5 pl-6 ${code.md.regular} text-nd_gray-500 truncate`} title=secondary>
      {React.string(secondary)}
    </div>
  </div>
}
