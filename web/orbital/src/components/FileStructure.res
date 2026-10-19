// The DFR file line by line: its frame (*DFRBEG / *DFREND) and each report section with its
// record counts, in the LoadedTable look. The section this config reads is tagged.
open Typography

let columns = [
  Table.col(~align=Right, "line", "Line"),
  Table.col("record", "Record"),
  Table.col("what", "What it is"),
  Table.col("detail", "Detail"),
  Table.col("records", "Data records"),
]

let orDash = (value: Nullable.t<string>) => value->Nullable.toOption->Option.getOr("—")

module Mono = {
  @react.component
  let make = (~text: string, ~className="text-nd_gray-700") =>
    <span title=text className={`${code.md.medium} ${className} inline-block max-w-md truncate align-middle`}>
      {React.string(text)}
    </span>
}

@react.component
let make = (~file: Engine.dfrFile, ~section: string, ~recordType: string) => {
  let header = Engine.frameText(file.begin_)
  <div className="flex flex-col gap-3">
    <p className={`${body.sm.regular} text-nd_gray-500`}>
      {React.string(
        `A DFR file is a stack of lines, and the first field says what each line is: the file header, a report section (H + report code), that report's data records (R + code), and the file trailer, which repeats the header. This config reads the ${recordType} records of the ${section} section and ignores the rest.`,
      )}
    </p>
    <Table.Wrapper withPagination=false>
      <Table.Head columns />
      <tbody>
        <Table.Row>
          <Table.Cell align=Right className="text-nd_gray-400"> {React.string(Int.toString(file.begin_.line))} </Table.Cell>
          <Table.Cell> <Mono text="*DFRBEG" /> </Table.Cell>
          <Table.Cell> {React.string("File header")} </Table.Cell>
          <Table.Cell> <Mono text=header /> </Table.Cell>
          <Table.Cell> <Tag text={`${file.delimiter}-delimited`} uppercase=false /> </Table.Cell>
        </Table.Row>
        {file.sections
        ->Array.map(s => {
          let records = Engine.recordsText(s)
          <Table.Row key={Int.toString(s.line)}>
            <Table.Cell align=Right className="text-nd_gray-400"> {React.string(Int.toString(s.line))} </Table.Cell>
            <Table.Cell> <Mono text={"H" ++ s.code} /> </Table.Cell>
            <Table.Cell>
              <span className="inline-flex items-center gap-2">
                {React.string(s.name->Nullable.toOption->Option.getOr("Report not in the v2.2 spec"))}
                {s.code == section ? <Tag text="read by this config" color=Primary uppercase=false /> : React.null}
              </span>
            </Table.Cell>
            <Table.Cell className="text-nd_gray-500">
              <div className="flex flex-col">
                <span>
                  {React.string(
                    s.date_from == s.date_to
                      ? `Activity ${orDash(s.date_from)}`
                      : `Activity ${orDash(s.date_from)} to ${orDash(s.date_to)}`,
                  )}
                </span>
                <span className={`${body.xs.regular} text-nd_gray-400`}>
                  {React.string(`Generated ${orDash(s.generated)} ET`)}
                </span>
              </div>
            </Table.Cell>
            <Table.Cell>
              {records == "" ? <span className="text-nd_gray-400"> {React.string("none")} </span> : <Mono text=records />}
            </Table.Cell>
          </Table.Row>
        })
        ->React.array}
        {switch file.end_->Nullable.toOption {
        | Some(trailer) =>
          let same = Engine.frameText(trailer) == header
          <Table.Row>
            <Table.Cell align=Right className="text-nd_gray-400"> {React.string(Int.toString(trailer.line))} </Table.Cell>
            <Table.Cell> <Mono text="*DFREND" /> </Table.Cell>
            <Table.Cell> {React.string("File trailer")} </Table.Cell>
            <Table.Cell> <Mono text={Engine.frameText(trailer)} /> </Table.Cell>
            <Table.Cell>
              {same
                ? <Tag text="matches the header" color=Success uppercase=false />
                : <Tag text="differs from the header" color=Error uppercase=false />}
            </Table.Cell>
          </Table.Row>
        | None =>
          <Table.Row>
            <Table.Cell align=Right className="text-nd_gray-400"> {React.string(Int.toString(file.lines))} </Table.Cell>
            <Table.Cell> <Mono text="*DFREND" className="text-nd_red-500" /> </Table.Cell>
            <Table.Cell> <Tag text="missing" color=Error uppercase=false /> </Table.Cell>
            <Table.Cell colSpan=2 className="text-nd_red-500 !whitespace-normal">
              {React.string("The file stops here with no trailer: it was cut off in transfer, so nothing in it is staged.")}
            </Table.Cell>
          </Table.Row>
        }}
      </tbody>
    </Table.Wrapper>
    {switch file.unexpected_lines->Array.length {
    | 0 => React.null
    | n =>
      <Helpers.InfoBox
        tone=Error
        message={`${Int.toString(n)} line(s) are not DFR records, first on line ${file.unexpected_lines
          ->Array.get(0)
          ->Option.mapOr("?", Int.toString)}.`}
      />
    }}
  </div>
}
