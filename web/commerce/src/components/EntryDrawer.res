// Right-hand drawer like ReconEnginePipelinesTransformationRunDetails (Modal with
// "h-screen w-1/3 float-right", heading.sm title, px-6 py-5 body, sticky primary OK button).
open Typography

@val @scope("window")
external addKeyListener: (@as("keydown") _, ReactEvent.Keyboard.t => unit) => unit = "addEventListener"
@val @scope("window")
external removeKeyListener: (@as("keydown") _, ReactEvent.Keyboard.t => unit) => unit = "removeEventListener"

@react.component
let make = (~result: option<Engine.rowResult>, ~config: Engine.config, ~onClose: unit => unit) => {
  let open_ = result->Option.isSome
  React.useEffect1(() => {
    if open_ {
      let onKey = e => ReactEvent.Keyboard.key(e) == "Escape" ? onClose() : ()
      addKeyListener(onKey)
      Some(() => removeKeyListener(onKey))
    } else {
      None
    }
  }, [open_])

  switch result {
  | None => React.null
  | Some(r) =>
    let orderId = Engine.orderIdOf(r, config)
    <div className="fixed inset-0 z-50 flex justify-end">
      <div className="absolute inset-0 bg-nd_gray-950 bg-opacity-40" onClick={_ => onClose()} />
      <div
        role="dialog"
        className="relative h-full w-full max-w-md bg-white shadow-hyperswitch_box_shadow flex flex-col overflow-hidden">
        <div className="flex items-center justify-between p-4 pl-6 border-b border-nd_gray-150 bg-white">
          <div className={`${heading.sm.semibold} text-nd_gray-800`}> {React.string("Entry details")} </div>
          <button
            type_="button"
            ariaLabel="Close"
            onClick={_ => onClose()}
            className="h-8 w-8 flex items-center justify-center rounded-lg text-nd_gray-500 hover:bg-nd_gray-50 focus:outline-none">
            <Icon name="x" size=16 />
          </button>
        </div>
        <div className="flex-1 overflow-y-auto px-6 py-5 pb-24">
          <div className="flex flex-col gap-5">
            <div className="flex flex-col gap-1.5">
              <div className="flex items-center gap-2 flex-wrap">
                <p className={`${code.lg.semibold} text-nd_gray-800 break-all`}>
                  {React.string(orderId == "" ? `Row ${Int.toString(r.line)}` : orderId)}
                </p>
                <Tag text={Tag.outcomeText(r.outcome)} color={Tag.ofOutcome(r.outcome)} />
              </div>
              <div className={`flex items-center gap-1.5 ${body.sm.regular} text-nd_gray-500`}>
                <span> {React.string(Engine.reportName(config))} </span>
                <Icon name="nd-arrow-right" size=10 className="text-nd_gray-300" />
                <span className={`${body.sm.medium} text-nd_gray-700`}>
                  {React.string(`line ${Int.toString(r.line)}`)}
                </span>
              </div>
            </div>
            {switch (r.outcome, r.details) {
            | ("Failed", Some(details)) =>
              <div
                className={`${body.sm.regular} bg-nd_red-50 border border-nd_red-100 rounded-lg px-2.5 py-1.5 text-nd_red-600 break-words`}>
                {React.string(details)}
              </div>
            | ("Skipped", Some(details)) =>
              <div
                className={`${body.sm.regular} bg-nd_gray-50 border border-nd_gray-150 rounded-lg px-2.5 py-1.5 text-nd_gray-700 break-words`}>
                {React.string(details)}
              </div>
            | _ => React.null
            }}
            {switch r.entry {
            | Some(e) =>
              <>
                <div className="flex flex-col gap-2">
                  <Helpers.SectionTitle> {React.string("Staged entry")} </Helpers.SectionTitle>
                  <div className="flex flex-col gap-1.5">
                    <Helpers.MetaRow label="Order ID" value=e.order_id mono=true />
                    <Helpers.MetaRow label="Entry type" value={EntriesTable.capitalize(e.balance_direction)} />
                    <Helpers.MetaRow label="Amount" value={`${e.currency} ${Engine.formatMinor(e.amount)}`} />
                    <Helpers.MetaRow label="Amount (minor units)" value={Int.toString(e.amount)} />
                    <Helpers.MetaRow label="Effective at" value=e.effective_at />
                  </div>
                </div>
                <div className="h-px bg-nd_gray-150" />
                <div className="flex flex-col gap-2">
                  <Helpers.SectionTitle count={Engine.metadataPairs(e)->Array.length}>
                    {React.string("Metadata")}
                  </Helpers.SectionTitle>
                  <Helpers.KeyValueList pairs={Engine.metadataPairs(e)} emptyText="No metadata fields in this config." />
                </div>
              </>
            | None => React.null
            }}
            <div className="flex flex-col gap-2">
              <Helpers.SectionTitle count={Engine.rawPairs(r.row)->Array.length}>
                {React.string("Raw CSV row")}
              </Helpers.SectionTitle>
              <Helpers.KeyValueList pairs={Engine.rawPairs(r.row)} emptyText="No columns parsed." />
            </div>
          </div>
        </div>
        <div className="absolute bottom-0 left-0 right-0 bg-white p-4 border-t border-nd_gray-150">
          <Button customButtonStyle="!w-full" buttonType=Primary buttonSize=Medium text="OK" onClick={_ => onClose()} />
        </div>
      </div>
    </div>
  }
}
