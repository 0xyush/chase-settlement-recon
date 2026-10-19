// The "Next step" card right under the results: match the staged entries with Hyperswitch.
// Before a match it is a one-line call to action; after one it shows the outcome, and each number
// opens the full list in the Match results tab.
open Typography

let primaryButton = "group inline-flex items-center justify-center gap-2 h-10 px-4 rounded-xl text-fs-14 font-semibold bg-button-primary-bg text-button-primary-text hover:bg-button-primary-hoverbg shadow-[0_1px_2px_rgba(0,0,0,0.12),0_6px_16px_-6px_rgba(18,114,249,0.6)] transition-all duration-150 active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
let secondaryButton = "inline-flex items-center justify-center gap-2 h-10 px-4 rounded-xl text-fs-14 font-medium bg-white text-nd_gray-700 border border-nd_gray-200 hover:border-nd_gray-300 hover:bg-nd_gray-25 transition-colors duration-150 cursor-pointer"

module Stat = {
  @react.component
  let make = (~title: string, ~value: int, ~dot: string, ~onClick: unit => unit) =>
    <button
      type_="button"
      onClick={_ => onClick()}
      className="group flex flex-col items-start gap-1 rounded-xl border border-nd_gray-150 bg-white px-3.5 py-2.5 text-left transition-colors hover:border-primary">
      <span className={`flex items-center gap-1.5 ${body.xs.medium} text-nd_gray-500`}>
        <span className={`h-1.5 w-1.5 rounded-full ${dot}`} />
        {React.string(title)}
      </span>
      <span className={`${heading.sm.semibold} text-nd_gray-800`}> {React.string(Int.toLocaleString(value))} </span>
    </button>
}

@react.component
let make = (
  ~staged: int,
  ~connector: string,
  ~run: option<(string, Hyperswitch.output)>,
  ~matching: bool,
  ~onUseMock: unit => unit,
  ~onUpload: (string, string) => unit,
  ~onViewItems: string => unit,
) => {
  let onUploadChange = e =>
    switch Engine.fileFromInput(e)->Nullable.toOption {
    | Some(file) => Engine.readFileText(file)->Promise.thenResolve(text => onUpload(Engine.fileName(file), text))->ignore
    | None => ()
    }
  let upload = (~label) =>
    <label className=secondaryButton>
      <Icon name="nd-upload" size=15 className="text-nd_gray-500" />
      {React.string(label)}
      <input type_="file" accept=".csv,text/csv" hidden=true onChange=onUploadChange />
    </label>
  let summary = switch run {
  | Some((source, {summary: Some(s)})) => Some((source, s))
  | _ => None
  }

  <div
    className="shrink-0 rounded-2xl border border-nd_gray-150 bg-gradient-to-br from-nd_primary_blue-25 via-white to-white p-5">
    <div className="flex flex-wrap items-center justify-between gap-4">
      <div className="flex items-center gap-3.5 min-w-0">
        <span
          className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-button-primary-bg text-button-primary-text shadow-[0_6px_16px_-6px_rgba(18,114,249,0.7)]">
          <Icon name="arrow-left-right" size=18 />
        </span>
        <div className="flex flex-col gap-0.5 min-w-0">
          <span className="text-fs-11 leading-16 font-semibold uppercase tracking-[0.08em] text-primary">
            {React.string("Next step")}
          </span>
          <span className={`${body.lg.semibold} text-nd_gray-800`}> {React.string("Match with Hyperswitch")} </span>
          <span className={`${body.sm.regular} text-nd_gray-500`}>
            {React.string(
              switch summary {
              | Some((source, s)) =>
                `${Int.toLocaleString(s.matched)} of ${Int.toLocaleString(s.chase_entries)} matched · ${source}`
              | None =>
                `Compare these ${Int.toLocaleString(staged)} rows with Hyperswitch payments and refunds.`
              },
            )}
          </span>
        </div>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        {switch (matching, summary) {
        | (true, _) =>
          <span className={`inline-flex items-center gap-2 h-10 px-4 ${body.sm.medium} text-nd_gray-600`}>
            <span className="h-4 w-4 rounded-full border-2 border-nd_gray-200 border-t-primary animate-spin" />
            {React.string("Matching…")}
          </span>
        | (false, Some(_)) =>
          <>
            <button type_="button" id="match-view-items" className=primaryButton onClick={_ => onViewItems("all")}>
              {React.string("See every item")}
              <Icon name="nd-arrow-right" size=15 className="transition-transform group-hover:translate-x-0.5" />
            </button>
            {upload(~label="Try another export")}
          </>
        | (false, None) =>
          <>
            <button
              type_="button"
              id="match-next-step"
              className=primaryButton
              disabled={staged == 0}
              onClick={_ => onUseMock()}>
              {React.string("Run match")}
              <Icon name="nd-arrow-right" size=15 className="transition-transform group-hover:translate-x-0.5" />
            </button>
            {upload(~label="Upload export")}
          </>
        }}
      </div>
    </div>
    {switch (matching, run, summary) {
    | (true, _, _) => React.null
    | (false, _, Some((_, s))) =>
      let mismatched = s.amount_mismatch + s.currency_mismatch
      <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-2.5">
        <Stat title="Matched" value=s.matched dot="bg-nd_green-400" onClick={() => onViewItems("matched")} />
        <Stat title="Amount mismatch" value=mismatched dot="bg-nd_red-500" onClick={() => onViewItems("mismatch")} />
        <Stat
          title="Missing in Hyperswitch"
          value=s.missing_in_hyperswitch
          dot="bg-nd_orange-300"
          onClick={() => onViewItems("missing_in_hyperswitch")}
        />
        <Stat
          title="Missing in Chase"
          value=s.missing_in_chase
          dot="bg-nd_orange-300"
          onClick={() => onViewItems("missing_in_chase")}
        />
      </div>
    | (false, Some((_, {rejected: Some(reason)})), _) =>
      <div className="mt-4"> <Helpers.InfoBox tone=Error message=reason /> </div>
    | (false, _, None) =>
      <p className={`mt-3 flex items-center gap-1.5 ${body.xs.regular} text-nd_gray-400`}>
        <Icon name="nd-toast-info" size=13 />
        {React.string(
          staged == 0
            ? "Nothing to match in this file."
            : `No Hyperswitch export yet, so Run match uses a test export made from this file (${connector}).`,
        )}
      </p>
    }}
  </div>
}
