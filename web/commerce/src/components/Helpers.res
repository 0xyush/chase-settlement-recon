// Small pieces copied from ReconEnginePipelinesHelper.res (SectionTitle, MetaRow),
// ReconEngineDataHelper.StatusIndicator and NoDataFound (InfoBox).
open Typography

module SectionTitle = {
  @react.component
  let make = (~count=?, ~children) =>
    <h4 className={`flex items-center gap-1.5 ${body.md.semibold} text-nd_gray-800`}>
      {children}
      {switch count {
      | Some(n) => <span className="text-nd_gray-500"> {React.string(`(${Int.toString(n)})`)} </span>
      | None => React.null
      }}
    </h4>
}

module MetaRow = {
  @react.component
  let make = (~label: string, ~value: string, ~mono=false) =>
    <div className={`flex items-baseline justify-between gap-3 ${body.sm.regular}`}>
      <span className="text-nd_gray-500 shrink-0"> {React.string(label)} </span>
      <span
        title=value
        className={`min-w-0 truncate text-right text-nd_gray-700 ${mono ? code.md.medium : body.sm.medium}`}>
        {React.string(value)}
      </span>
    </div>
}

// Key/value list in the FieldRow look: bordered, divided rows.
module KeyValueList = {
  @react.component
  let make = (~pairs: array<(string, string)>, ~mono=false, ~emptyText="Nothing here.") =>
    if pairs->Array.length == 0 {
      <p className={`${body.xs.regular} text-nd_gray-500`}> {React.string(emptyText)} </p>
    } else {
      <div className="border border-nd_gray-150 rounded-lg overflow-hidden">
        {pairs
        ->Array.map(((key, value)) =>
          <div
            key
            className={`flex items-center justify-between gap-3 px-3 py-2 border-b border-nd_gray-150 last:border-0 ${body.sm.regular}`}>
            <span className="min-w-0 truncate text-nd_gray-500" title=key> {React.string(key)} </span>
            <span
              title=value
              className={`min-w-0 truncate text-right text-nd_gray-700 ${mono ? code.md.medium : body.sm.medium}`}>
              {React.string(value)}
            </span>
          </div>
        )
        ->React.array}
      </div>
    }
}

module StatusDot = {
  @react.component
  let make = (~level: string) => {
    let bg = switch level {
    | "ok" => "bg-nd_green-300"
    | "bad" => "bg-nd_red-400"
    | _ => "bg-nd_orange-300"
    }
    <span className="relative flex h-2 w-2 shrink-0">
      <span className={`absolute inline-flex h-full w-full rounded-full ${bg} opacity-75`} />
      <span className={`relative inline-flex rounded-full h-2 w-2 ${bg}`} />
    </span>
  }
}

type tone = Info | Error

module InfoBox = {
  @react.component
  let make = (~message: string, ~tone=Info) => {
    let (box, icon, iconName) = switch tone {
    | Info => ("border-nd_gray-200 bg-white text-nd_gray-500", "text-nd_gray-400", "nd-toast-info")
    | Error => ("border-nd_red-100 bg-nd_red-50 text-nd_red-600", "text-nd_red-500", "nd-alert")
    }
    <div className={`border ${box} p-3 pl-4 rounded-lg flex items-center gap-3`}>
      <Icon name=iconName size=18 className=icon />
      <div className={body.md.medium}> {React.string(message)} </div>
    </div>
  }
}
