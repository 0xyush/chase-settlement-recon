// Tie-out: the staged ACT0010 detail added up per card brand and sale / refund, next to Chase's own
// FIN0010 summary line from the same file; then the summary lines with no detail behind them.
open Typography

let columns = [
  Table.col("status", "Result"),
  Table.col("brand", "Card brand"),
  Table.col("category", "Category"),
  Table.col("sc", "Settled / conveyed"),
  Table.col(~align=Right, "chase_count", "Chase count"),
  Table.col(~align=Right, "staged_count", "Staged count"),
  Table.col(~align=Right, "chase_amount", "Chase amount"),
  Table.col(~align=Right, "staged_amount", "Staged amount"),
]

let otherColumns = [
  Table.col("category", "Category"),
  Table.col("name", "What it is"),
  Table.col("sc", "Settled / conveyed"),
  Table.col(~align=Right, "amount", "Amount"),
]

let brandName = code =>
  switch code {
  | "VI" => "Visa"
  | "MC" => "Mastercard"
  | "AX" => "Amex"
  | "DI" => "Discover"
  | "" => "—"
  | other => other
  }

let categoryName = code =>
  switch code {
  | "SALE" => "Sales"
  | "REF" => "Refunds"
  | other => other
  }

let settledText = (value: Nullable.t<string>) =>
  switch value->Nullable.toOption {
  | Some("S") => "S · settled by Chase"
  | Some("C") => "C · conveyed, network pays"
  | Some(other) => other
  | None => "—"
  }

let money = (value: Nullable.t<int>) => value->Nullable.toOption->Option.mapOr("—", Engine.formatMinor)
let count = (value: Nullable.t<int>) => value->Nullable.toOption->Option.mapOr("—", Int.toLocaleString)

let statusTag = status =>
  switch status {
  | "tied" => <Tag text="Tied" color=Tag.Success />
  | "differs" => <Tag text="Differs" color=Tag.Error />
  | "summary_only" => <Tag text="Only in Chase's summary" color=Tag.Warning />
  | _ => <Tag text="Only in staged detail" color=Tag.Warning />
  }

module Stat = {
  @react.component
  let make = (~title: string, ~value: string, ~hint: string, ~valueClass="text-nd_gray-800") =>
    <div className="flex flex-col gap-1.5 bg-white border border-nd_gray-200 rounded-xl px-4 py-3 shadow-xs">
      <div className={`${body.md.medium} text-nd_gray-400`}> {React.string(title)} </div>
      <div className={`${heading.md.semibold} ${valueClass}`}> {React.string(value)} </div>
      <div className={`${body.xs.regular} text-nd_gray-500`}> {React.string(hint)} </div>
    </div>
}

@react.component
let make = (~tieout: Engine.tieout) => {
  let differ = tieout.lines->Array.filter(l => l.status != "tied")->Array.length
  <div className="flex flex-col gap-4">
    <p className={`${body.sm.regular} text-nd_gray-500`}>
      {React.string(
        "The B file carries Chase's own summary of the same deposits (FIN0010) next to the transaction detail (ACT0010). Each line adds up the staged entries for one card brand and sale or refund, and puts the total next to Chase's line. If every line ties, nothing was lost between Chase's file and staging. FIN0010 also marks each line Settled (S: in Chase's deposit to the bank) or Conveyed (C: Chase lists it, but the card network pays the merchant directly).",
      )}
    </p>
    <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
      <Stat
        title="Staged detail vs Chase's summary"
        value={tieout.tied ? "Ties" : `${Int.toString(differ)} line${differ == 1 ? "" : "s"} differ`}
        valueClass={tieout.tied ? "text-nd_green-400" : "text-nd_red-500"}
        hint={`${Int.toString(tieout.lines->Array.length)} card brand lines checked`}
      />
      <Stat
        title="Chase's deposit to the bank"
        value={Engine.netText(tieout.settled_net)}
        hint="S lines: sales, less refunds, fees and chargebacks"
      />
      <Stat
        title="Conveyed, paid by the network"
        value={Engine.netText(tieout.conveyed_net)}
        hint="C lines: listed by Chase, not in its deposit"
      />
    </div>
    <Table.Wrapper withPagination=false>
      <Table.Head columns />
      <tbody>
        {tieout.lines->Array.length == 0
          ? <Table.Empty colSpan=8 title="Nothing to tie out." message="No sales or refunds in this file." />
          : tieout.lines
            ->Array.map(l =>
              <Table.Row key={`${l.currency}-${l.card_brand}-${l.category}`}>
                <Table.Cell> {statusTag(l.status)} </Table.Cell>
                <Table.Cell> {React.string(brandName(l.card_brand))} </Table.Cell>
                <Table.Cell> {React.string(categoryName(l.category))} </Table.Cell>
                <Table.Cell className="text-nd_gray-500"> {React.string(settledText(l.settled_conveyed))} </Table.Cell>
                <Table.Cell align=Right className="text-nd_gray-700"> {React.string(count(l.summary_count))} </Table.Cell>
                <Table.Cell align=Right className="text-nd_gray-700"> {React.string(count(l.detail_count))} </Table.Cell>
                <Table.Cell align=Right className="text-nd_gray-700"> {React.string(money(l.summary_amount))} </Table.Cell>
                <Table.Cell align=Right className="text-nd_gray-700"> {React.string(money(l.detail_amount))} </Table.Cell>
              </Table.Row>
            )
            ->React.array}
      </tbody>
    </Table.Wrapper>
    <Helpers.SectionTitle count={tieout.other->Array.length}>
      {React.string("Summary lines with no transaction detail")}
    </Helpers.SectionTitle>
    <Table.Wrapper withPagination=false>
      <Table.Head columns=otherColumns />
      <tbody>
        {tieout.other->Array.length == 0
          ? <Table.Empty colSpan=4 title="None." message="No fees, chargebacks or adjustments in this file." />
          : tieout.other
            ->Array.mapWithIndex((o, i) =>
              <Table.Row key={Int.toString(i)}>
                <Table.Cell> <span className={`${code.md.medium} text-nd_gray-700`}> {React.string(o.category)} </span> </Table.Cell>
                <Table.Cell> {React.string(o.name)} </Table.Cell>
                <Table.Cell className="text-nd_gray-500"> {React.string(settledText(o.settled_conveyed))} </Table.Cell>
                <Table.Cell align=Right className={o.amount < 0 ? "text-nd_red-500" : "text-nd_gray-700"}>
                  {React.string(Engine.formatMinor(o.amount))}
                </Table.Cell>
              </Table.Row>
            )
            ->React.array}
      </tbody>
    </Table.Wrapper>
    {tieout.problems->Array.length == 0
      ? React.null
      : <Helpers.InfoBox tone=Error message={"Unreadable summary lines: " ++ tieout.problems->Array.joinWith("; ")} />}
  </div>
}
