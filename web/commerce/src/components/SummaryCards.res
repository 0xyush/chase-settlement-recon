// ReconEngineDataTransformedEntriesHelper.TransformedEntriesOverviewCard: clickable cards that
// double as the status filter (active = border-primary / text-primary).
open Typography

type card = {title: string, value: int, filter: string, valueClass: string}

@react.component
let make = (~cards: array<card>, ~active: string, ~onSelect: string => unit) =>
  <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
    {cards
    ->Array.map(card => {
      let isActive = card.filter == active
      let (border, title, value) = isActive
        ? ("border-primary", "text-primary", "text-primary")
        : ("border-nd_gray-200", "text-nd_gray-400", card.valueClass)
      <div
        key=card.filter
        onClick={_ => onSelect(card.filter)}
        className={`flex flex-col gap-2 bg-white border ${border} rounded-xl px-4 py-3 shadow-xs cursor-pointer hover:bg-nd_gray-50 transition-colors`}>
        <div className={`${body.md.medium} ${title}`}> {React.string(card.title)} </div>
        <div className={`${heading.md.semibold} ${value}`}> {React.string(Int.toLocaleString(card.value))} </div>
      </div>
    })
    ->React.array}
  </div>
