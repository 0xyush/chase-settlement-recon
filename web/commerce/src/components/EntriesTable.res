// Transformed entries table (ReconEngineDataTransformedEntries + EntriesTableEntity columns):
// search row, LoadedTable look, status labels, mono order ids, right-aligned amounts, pagination.
open Typography

let columns = [
  Table.col(~align=Right, "line", "Line"),
  Table.col("status", "Status"),
  Table.col("order_id", "Order ID"),
  Table.col("entry_type", "Entry Type"),
  Table.col(~align=Right, "amount", "Amount"),
  Table.col("currency", "Currency"),
  Table.col("effective_at", "Effective At"),
  Table.col("metadata", "Metadata / Reason"),
]

let capitalize = s => String.toUpperCase(String.slice(s, ~start=0, ~end=1)) ++ String.sliceToEnd(s, ~start=1)

@react.component
let make = (
  ~results: array<Engine.rowResult>,
  ~config: Engine.config,
  ~text: string,
  ~filter: string,
  ~fileName: string,
  ~onSelect: Engine.rowResult => unit,
) => {
  let (query, setQuery) = React.useState(() => "")
  let (offset, setOffset) = React.useState(() => 0)
  let (perPage, setPerPage) = React.useState(() => 10)

  React.useEffect3(() => {
    setOffset(_ => 0)
    None
  }, (filter, query, perPage))

  let visible = React.useMemo3(
    () => results->Array.filter(r => (filter == "All" || r.outcome == filter) && Engine.rowMatches(r, query)),
    (results, filter, query),
  )
  let total = visible->Array.length
  let page = visible->Array.slice(~start=offset, ~end=offset + perPage)
  let base = fileName->String.replaceRegExp(%re("/\.csv$/i"), "")

  <div className="flex flex-col gap-4">
    <div className="flex flex-wrap items-center justify-between gap-3">
      <SearchInput value=query onChange={q => setQuery(_ => q)} placeholder="Search by order ID, value or reason" />
      <div className="flex items-center gap-2">
        <Button
          buttonType=Secondary
          buttonSize=Small
          text="Staging entries JSON"
          leftIcon={<Icon name="nd-download" size=14 className="text-nd_gray-500" />}
          onClick={_ =>
            Engine.exportFile(text, config, "staging_json", results)
            ->Promise.thenResolve(body => Engine.download(base ++ ".staging_entries.json", body, "application/json"))
            ->ignore}
        />
        <Button
          buttonType=Secondary
          buttonSize=Small
          text="Row report CSV"
          leftIcon={<Icon name="nd-download" size=14 className="text-nd_gray-500" />}
          onClick={_ =>
            Engine.exportFile(text, config, "report_csv", results)
            ->Promise.thenResolve(body => Engine.download(base ++ ".report-v1.csv", body, "text/csv"))
            ->ignore}
        />
      </div>
    </div>
    <div>
      <Table.Wrapper withPagination={total > 0}>
        <Table.Head columns />
        <tbody>
          {total == 0
            ? <Table.Empty
                colSpan=8
                title="No entries to show."
                message={query == "" ? "No rows have this status." : "No rows match your search."}
              />
            : page
              ->Array.map(r =>
                <Table.Row key={Int.toString(r.line)} onClick={_ => onSelect(r)}>
                  <Table.Cell align=Right className="text-nd_gray-400"> {React.string(Int.toString(r.line))} </Table.Cell>
                  <Table.Cell> <Tag text={Tag.outcomeText(r.outcome)} color={Tag.ofOutcome(r.outcome)} /> </Table.Cell>
                  <Table.Cell>
                    {
                      let id = Engine.orderIdOf(r, config)
                      <span title=id className={`${code.md.medium} text-nd_gray-700 inline-block max-w-44 truncate align-middle`}>
                        {React.string(id == "" ? "—" : id)}
                      </span>
                    }
                  </Table.Cell>
                  {switch r.entry {
                  | Some(e) =>
                    <>
                      <Table.Cell> {React.string(capitalize(e.balance_direction))} </Table.Cell>
                      <Table.Cell align=Right className="text-nd_gray-700"> {React.string(Engine.formatMinor(e.amount))} </Table.Cell>
                      <Table.Cell> {React.string(e.currency)} </Table.Cell>
                      <Table.Cell> {React.string(String.slice(e.effective_at, ~start=0, ~end=10))} </Table.Cell>
                      <Table.Cell>
                        {
                          let text = Engine.metadataText(e)
                          <span title=text className="inline-block max-w-xs truncate align-middle text-nd_gray-500">
                            {React.string(text)}
                          </span>
                        }
                      </Table.Cell>
                    </>
                  | None =>
                    <Table.Cell colSpan=5>
                      {
                        let text = r.details->Option.getOr("")
                        <span
                          title=text
                          className={`inline-block max-w-md truncate align-middle ${r.outcome == "Failed"
                              ? "text-nd_red-500"
                              : "text-nd_gray-500"}`}>
                          {React.string(text)}
                        </span>
                      }
                    </Table.Cell>
                  }}
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
