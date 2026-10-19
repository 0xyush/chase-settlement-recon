// Tabs.res -> TabsBinding (Blend Tabs, variant Underline, size Lg).
open Typography

type tab = {title: string, count: option<int>}

@react.component
let make = (~tabs: array<tab>, ~selected: int, ~onSelect: int => unit) =>
  <div className="flex items-center gap-6 border-b border-nd_gray-150 overflow-x-auto no-scrollbar">
    {tabs
    ->Array.mapWithIndex((tab, i) => {
      let look = i == selected
        ? "border-primary text-nd_gray-800"
        : "border-transparent text-nd_gray-500 hover:text-nd_gray-700"
      <button
        key=tab.title
        type_="button"
        onClick={_ => onSelect(i)}
        className={`-mb-px pb-2.5 pt-1 ${body.md.medium} whitespace-nowrap border-b-2 focus:outline-none ${look}`}>
        {React.string(tab.title)}
        {switch tab.count {
        | Some(n) =>
          <span className={`ml-1.5 ${body.sm.medium} text-nd_gray-400`}> {React.string(Int.toLocaleString(n))} </span>
        | None => React.null
        }}
      </button>
    })
    ->React.array}
  </div>
