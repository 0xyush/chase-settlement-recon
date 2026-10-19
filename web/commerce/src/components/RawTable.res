// Parsed rows for a report with no mapping yet, in the same LoadedTable look with pagination.
@react.component
let make = (~header: array<string>, ~rows: array<Engine.row>) => {
  let (offset, setOffset) = React.useState(() => 0)
  let (perPage, setPerPage) = React.useState(() => 10)
  let names = header->Array.filter(c => c != "")
  let columns = [Table.col(~align=Right, "line", "Line")]->Array.concat(names->Array.map(c => Table.col(c, c)))
  let total = rows->Array.length
  let page = rows->Array.slice(~start=offset, ~end=offset + perPage)

  <div>
    <Table.Wrapper withPagination={total > 0}>
      <Table.Head columns />
      <tbody>
        {total == 0
          ? <Table.Empty colSpan={columns->Array.length} title="No rows parsed." message="The file has no data rows under its header." />
          : page
            ->Array.map(row =>
              <Table.Row key={Int.toString(row.line)}>
                <Table.Cell align=Right className="text-nd_gray-400"> {React.string(Int.toString(row.line))} </Table.Cell>
                {names
                ->Array.map(c => {
                  let v = Engine.rawCell(row, c)
                  <Table.Cell key=c>
                    <span title=v className="inline-block max-w-56 truncate align-middle">
                      {React.string(v == "" ? "—" : v)}
                    </span>
                  </Table.Cell>
                })
                ->React.array}
              </Table.Row>
            )
            ->React.array}
      </tbody>
    </Table.Wrapper>
    {total > 0
      ? <Table.Paginator
          total offset perPage setOffset={o => setOffset(_ => o)} setPerPage={n => {
            setPerPage(_ => n)
            setOffset(_ => 0)
          }}
        />
      : React.null}
  </div>
}
