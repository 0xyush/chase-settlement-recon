// Control Center Table.res / LoadedTable.res / Paginator.res look:
// header bg-nd_gray-25 text-nd_gray-400 text-fs-13 h-12, rows text-fs-14 text-nd_gray-600 hover:bg-nd_gray-50,
// border-nd_br_gray-150, rounded-t-lg table + rounded-b-lg "Showing x-y of n" pagination footer.
open Typography

type align = Left | Right
type column = {key: string, title: string, align: align}

let col = (~align=Left, key, title) => {key, title, align}

module Wrapper = {
  @react.component
  let make = (~withPagination=true, ~children) =>
    <div
      className={`border border-nd_br_gray-150 ${withPagination ? "rounded-t-lg" : "rounded-lg"} overflow-x-auto bg-white`}>
      <table className="table-auto w-full border-collapse"> {children} </table>
    </div>
}

module Head = {
  @react.component
  let make = (~columns: array<column>) =>
    <thead>
      <tr>
        {columns
        ->Array.map(c =>
          <th
            key=c.key
            className={`bg-nd_gray-25 h-12 px-4 py-3 text-fs-13 leading-18 font-normal text-nd_gray-400 whitespace-pre select-none ${c.align ==
                Right
                ? "text-right"
                : "text-left"}`}>
            {React.string(c.title)}
          </th>
        )
        ->React.array}
      </tr>
    </thead>
}

module Row = {
  @react.component
  let make = (~onClick=?, ~children) =>
    <tr
      ?onClick
      className={`bg-white hover:bg-nd_gray-50 transition duration-300 ease-in-out ${onClick->Option.isSome
          ? "cursor-pointer"
          : ""}`}>
      {children}
    </tr>
}

module Cell = {
  @react.component
  let make = (~align=Left, ~className="", ~colSpan=?, ~children) =>
    <td
      ?colSpan
      className={`px-4 py-3 border-t border-nd_br_gray-150 text-fs-14 leading-20 text-nd_gray-600 whitespace-nowrap align-middle ${align ==
          Right
          ? "text-right tabular-nums"
          : "text-left"} ${className}`}>
      {children}
    </td>
}

module Empty = {
  @react.component
  let make = (~colSpan: int, ~title: string, ~message: string) =>
    <tr>
      <td colSpan className="border-t border-nd_br_gray-150">
        <div className="h-40 flex flex-col justify-center items-center gap-1 text-center px-4">
          <p className={`${body.md.semibold} text-nd_gray-800`}> {React.string(title)} </p>
          <p className={`${body.sm.medium} text-nd_gray-500`}> {React.string(message)} </p>
        </div>
      </td>
    </tr>
}

module PageButton = {
  @react.component
  let make = (~icon: string, ~disabled: bool, ~onClick: unit => unit, ~label: string) =>
    <button
      type_="button"
      ariaLabel=label
      disabled
      onClick={_ => onClick()}
      className={`h-8 w-8 flex items-center justify-center rounded-lg focus:outline-none ${disabled
          ? "text-nd_gray-300 cursor-not-allowed"
          : "text-nd_gray-500 hover:bg-nd_gray-150"}`}>
      <Icon name=icon size=14 />
    </button>
}

let perPageOptions = [10, 25, 50, 100]

module Paginator = {
  @react.component
  let make = (
    ~total: int,
    ~offset: int,
    ~perPage: int,
    ~setOffset: int => unit,
    ~setPerPage: int => unit,
  ) => {
    let pages = Math.Int.max(1, (total + perPage - 1) / perPage)
    let current = offset / perPage + 1
    let from = total == 0 ? 0 : offset + 1
    let to = Math.Int.min(offset + perPage, total)
    // Same windowing as Pagination.res (btnCount = 4).
    let btnCount = 4
    let pageToLeft = btnCount - (pages - current) < btnCount / 2 ? btnCount / 2 : btnCount - (pages - current)
    let startIndex = Math.Int.max(1, current - pageToLeft)
    let endIndex = Math.Int.min(startIndex + btnCount, pages)
    let numbers = Array.fromInitializer(~length=endIndex - startIndex + 1, i => startIndex + i)
    let goTo = page => setOffset((page - 1) * perPage)

    <div
      className="flex flex-wrap items-center justify-between gap-2 bg-nd_gray-25 border border-t-0 rounded-b-lg border-nd_br_gray-150 px-4 py-2">
      <div className={`flex items-center gap-1 ${body.md.medium} text-nd_gray-500 whitespace-pre`}>
        {React.string("Showing ")}
        <span className="text-nd_gray-700">
          {React.string(`${Int.toLocaleString(from)}-${Int.toLocaleString(to)}`)}
        </span>
        {React.string(" of ")}
        <span className="text-nd_gray-700"> {React.string(Int.toLocaleString(total))} </span>
      </div>
      <div className="flex items-center gap-2">
        <select
          ariaLabel="Rows per page"
          value={Int.toString(perPage)}
          onChange={e =>
            setPerPage(ReactEvent.Form.target(e)["value"]->Int.fromString->Option.getOr(10))}
          className="h-8 rounded-lg border border-nd_gray-200 bg-white px-2 text-fs-13 text-nd_gray-600 focus:outline-none">
          {perPageOptions
          ->Array.map(n =>
            <option key={Int.toString(n)} value={Int.toString(n)}>
              {React.string(`${Int.toString(n)} / page`)}
            </option>
          )
          ->React.array}
        </select>
        <div className="flex items-center gap-0.5">
          <PageButton icon="chevron-left" label="Previous page" disabled={current <= 1} onClick={() => goTo(current - 1)} />
          {numbers
          ->Array.map(n =>
            <button
              key={Int.toString(n)}
              type_="button"
              onClick={_ => goTo(n)}
              className={`h-8 min-w-8 px-2 rounded-lg text-fs-12 font-medium focus:outline-none ${n == current
                  ? "bg-primary bg-opacity-10 text-primary"
                  : "text-nd_gray-500 hover:bg-nd_gray-150"}`}>
              {React.string(Int.toString(n))}
            </button>
          )
          ->React.array}
          <PageButton icon="chevron-right" label="Next page" disabled={current >= pages} onClick={() => goTo(current + 1)} />
        </div>
      </div>
    </div>
  }
}
