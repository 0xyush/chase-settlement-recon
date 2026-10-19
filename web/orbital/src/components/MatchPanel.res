// Match results tab: every matched, mismatched and missing item from the last match, with the
// outcome cards as filters and the match report CSV. The match itself starts from the Next step
// card under the results.
open Typography

let columns = [
  Table.col("status", "Result"),
  Table.col("order_id", "Chase order ID"),
  Table.col("payment_id", "Hyperswitch payment"),
  Table.col("direction", "Entry Type"),
  Table.col(~align=Right, "chase", "Chase amount"),
  Table.col(~align=Right, "hyperswitch", "Hyperswitch amount"),
  Table.col(~align=Right, "chase_line", "Chase line"),
  Table.col(~align=Right, "hs_line", "Hyperswitch line"),
  Table.col("note", "Note"),
]

let amount = value => value->Option.mapOr("—", Engine.formatMinor)
let line = value => value->Option.mapOr("—", Int.toString)
let capitalize = s => String.toUpperCase(String.slice(s, ~start=0, ~end=1)) ++ String.sliceToEnd(s, ~start=1)

@react.component
let make = (
  ~summary: Hyperswitch.summary,
  ~items: array<Hyperswitch.item>,
  ~reportCsv: string,
  ~source: string,
  ~fileName: string,
  ~filter: string,
  ~setFilter: string => unit,
) => {
  let (offset, setOffset) = React.useState(() => 0)
  let (perPage, setPerPage) = React.useState(() => 10)
  React.useEffect1(() => {
    setOffset(_ => 0)
    None
  }, [filter])

  let s = summary
  let visible = filter == "all" ? items : items->Array.filter(i => Hyperswitch.group(i.outcome) == filter)
  let total = visible->Array.length
  let page = visible->Array.slice(~start=offset, ~end=offset + perPage)
  let base = fileName->String.replaceRegExp(%re("/\.(csv|dfr)$/i"), "")

  <div className="flex flex-col gap-4">
    <div className={`flex flex-wrap items-center justify-between gap-2 ${body.sm.regular} text-nd_gray-500`}>
      <span>
        {React.string(
          `${Int.toLocaleString(s.chase_entries)} staged Chase entries against ${Int.toLocaleString(
              s.hyperswitch_records,
            )} Hyperswitch records from ${source}`,
        )}
      </span>
      <Button
        buttonType=Secondary
        text="Match report CSV"
        leftIcon={<Icon name="nd-download" size=14 className="text-nd_gray-500" />}
        onClick={_ => Engine.download(base ++ ".match-report.csv", reportCsv, "text/csv")}
      />
    </div>
    <SummaryCards
      cards=[
        {title: "Matched", value: s.matched, filter: "matched", valueClass: "text-nd_green-400"},
        {
          title: "Amount mismatch",
          value: s.amount_mismatch + s.currency_mismatch,
          filter: "mismatch",
          valueClass: s.amount_mismatch + s.currency_mismatch > 0 ? "text-nd_red-500" : "text-nd_gray-800",
        },
        {
          title: "Missing in Hyperswitch",
          value: s.missing_in_hyperswitch,
          filter: "missing_in_hyperswitch",
          valueClass: s.missing_in_hyperswitch > 0 ? "text-nd_orange-600" : "text-nd_gray-800",
        },
        {
          title: "Missing in Chase",
          value: s.missing_in_chase,
          filter: "missing_in_chase",
          valueClass: s.missing_in_chase > 0 ? "text-nd_orange-600" : "text-nd_gray-800",
        },
      ]
      active=filter
      onSelect={f => setFilter(filter == f ? "all" : f)}
    />
    <div>
      <Table.Wrapper withPagination={total > 0}>
        <Table.Head columns />
        <tbody>
          {total == 0
            ? <Table.Empty colSpan=9 title="Nothing here." message="No items have this result." />
            : page
              ->Array.mapWithIndex((i, index) =>
                <Table.Row key={`${i.order_id}-${i.outcome}-${Int.toString(offset + index)}`}>
                  <Table.Cell> <Tag text={Hyperswitch.label(i.outcome)} color={Hyperswitch.color(i.outcome)} /> </Table.Cell>
                  <Table.Cell>
                    <span title=i.order_id className={`${code.md.medium} text-nd_gray-700 inline-block max-w-44 truncate align-middle`}>
                      {React.string(i.order_id)}
                    </span>
                  </Table.Cell>
                  <Table.Cell>
                    <span title={i.payment_id->Option.getOr("")} className={`${code.md.medium} text-nd_gray-500 inline-block max-w-44 truncate align-middle`}>
                      {React.string(i.payment_id->Option.getOr("—"))}
                    </span>
                  </Table.Cell>
                  <Table.Cell> {React.string(capitalize(i.direction))} </Table.Cell>
                  <Table.Cell align=Right className="text-nd_gray-700"> {React.string(amount(i.chase_amount))} </Table.Cell>
                  <Table.Cell align=Right className="text-nd_gray-700"> {React.string(amount(i.hyperswitch_amount))} </Table.Cell>
                  <Table.Cell align=Right className="text-nd_gray-400"> {React.string(line(i.chase_line))} </Table.Cell>
                  <Table.Cell align=Right className="text-nd_gray-400"> {React.string(line(i.hyperswitch_line))} </Table.Cell>
                  <Table.Cell className="text-nd_gray-500"> {React.string(i.note->Option.getOr(""))} </Table.Cell>
                </Table.Row>
              )
              ->React.array}
        </tbody>
      </Table.Wrapper>
      {total > 0
        ? <Table.Paginator total offset perPage setOffset={o => setOffset(_ => o)} setPerPage={n => setPerPage(_ => n)} />
        : React.null}
    </div>
  </div>
}
