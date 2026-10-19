// The engine's run as a snackbar at the bottom right. When a file loads, its steps tick off one
// after another; click a step to check what it did, the source file that does it, and how long it
// took in the engine. Minimize it to a pill. The steps come from the engine (`steps` in the
// transform answer); the match step fills in once a match has run in the Match tab.
open Typography

let duration = (micros: Nullable.t<float>) =>
  switch micros->Nullable.toOption {
  | None => "—"
  | Some(us) if us >= 1000.0 => `${Float.toFixedWithPrecision(us /. 1000.0, ~digits=1)} ms`
  | Some(us) => `${Float.toFixedWithPrecision(us, ~digits=0)} µs`
  }

// The match step is known only after a match runs.
let withMatch = (step: Engine.step, matched: option<Hyperswitch.summary>): Engine.step =>
  switch (step.key, matched) {
  | ("match", Some(s)) =>
    let mismatched = s.amount_mismatch + s.currency_mismatch
    let missing = s.missing_in_hyperswitch + s.missing_in_chase
    {
      ...step,
      status: mismatched + missing > 0 ? "warning" : "done",
      metric: `${Int.toLocaleString(s.matched)} matched`,
      detail: `${Int.toLocaleString(s.matched)} matched, ${Int.toString(
          mismatched,
        )} with a different amount or currency, ${Int.toString(
          s.missing_in_hyperswitch,
        )} missing in Hyperswitch, ${Int.toString(s.missing_in_chase)} missing in Chase, ${Int.toString(
          s.ignored,
        )} set aside because they never settle.`,
    }
  | _ => step
  }

module StatusIcon = {
  @react.component
  let make = (~status: string, ~pending: bool) => {
    let circle = "flex h-[18px] w-[18px] shrink-0 items-center justify-center rounded-full"
    if pending {
      <span className="h-[18px] w-[18px] shrink-0 rounded-full border-2 border-nd_gray-200 border-t-primary animate-spin" />
    } else {
      switch status {
      | "done" =>
        <span className={`${circle} bg-nd_green-400 text-button-primary-text`}> <Icon name="nd-check" size=11 /> </span>
      | "warning" =>
        <span className={`${circle} bg-nd_orange-300 text-button-primary-text text-fs-11 font-semibold`}>
          {React.string("!")}
        </span>
      | "failed" =>
        <span className={`${circle} bg-nd_red-500 text-button-primary-text`}> <Icon name="x" size=11 /> </span>
      | _ => <span className={`${circle} border border-dashed border-nd_gray-300`} />
      }
    }
  }
}

@react.component
let make = (~steps: array<Engine.step>, ~matched: option<Hyperswitch.summary>, ~fileName: string) => {
  let steps = steps->Array.map(s => withMatch(s, matched))
  let count = steps->Array.length
  let (revealed, setRevealed) = React.useState(() => 0)
  let (isOpen, setIsOpen) = React.useState(() => true)
  let (expanded, setExpanded) = React.useState((): option<string> => None)
  let failedKey = steps->Array.find(s => s.status == "failed")->Option.map(s => s.key)
  // Once you click into the snackbar it stays as you left it; otherwise it tucks itself away.
  let touched = React.useRef(false)
  let tuckAway = () =>
    if !touched.current {
      setIsOpen(_ => false)
    }

  // A new run: open, tick the steps off in order, and open the step that stopped the file.
  let runKey =
    steps->Array.filter(s => s.key != "match")->Array.map(s => s.key ++ s.metric)->Array.joinWith("|")
  React.useEffect1(() => {
    setRevealed(_ => 0)
    setIsOpen(_ => true)
    setExpanded(_ => failedKey)
    touched.current = false
    let timer = setInterval(() => setRevealed(n => n + 1), 140)
    let stop = setTimeout(() => clearInterval(timer), (count + 1) * 140)
    // A clean run tucks into the pill after a few seconds; a run that stopped stays open.
    let tuck = setTimeout(() => failedKey->Option.isNone ? tuckAway() : (), (count + 1) * 140 + 3500)
    Some(
      () => {
        clearInterval(timer)
        clearTimeout(stop)
        clearTimeout(tuck)
      },
    )
  }, [runKey])

  // A match result: open and show the match step.
  let matchedKey = matched->Option.mapOr("", s => Int.toString(s.matched) ++ "/" ++ Int.toString(s.chase_entries))
  React.useEffect1(() => {
    if matchedKey != "" {
      touched.current = false
      setIsOpen(_ => true)
      setExpanded(_ => Some("match"))
      let tuck = setTimeout(tuckAway, 6000)
      Some(() => clearTimeout(tuck))
    } else {
      None
    }
  }, [matchedKey])

  let running = revealed < count
  let failedAt = steps->Array.findIndex(s => s.status == "failed")
  let warnings = steps->Array.filter(s => s.status == "warning")->Array.length
  let finished = steps->Array.filter(s => s.status != "not_run")->Array.length
  let total =
    steps->Array.reduce(0.0, (sum, s) => sum +. s.micros->Nullable.toOption->Option.getOr(0.0))
  let (headline, tone) = if running {
    ("Running the Rust engine…", "text-nd_gray-800")
  } else if failedAt >= 0 {
    (`Stopped at step ${Int.toString(failedAt + 1)}`, "text-nd_red-500")
  } else if warnings > 0 {
    (`Done · ${Int.toString(warnings)} to look at`, "text-nd_orange-600")
  } else {
    ("All steps done", "text-nd_green-400")
  }
  let summaryStatus = running ? "" : failedAt >= 0 ? "failed" : warnings > 0 ? "warning" : "done"
  let progress = count == 0 ? 0 : Math.Int.min(revealed, count) * 100 / count

  <div className="fixed bottom-5 right-5 z-40 w-[23rem] max-w-[calc(100vw-2.5rem)]">
    {isOpen
      ? <div
          role="status"
          onClick={_ => touched.current = true}
          className="flex flex-col max-h-[72vh] rounded-xl border border-nd_gray-200 bg-white shadow-generic_shadow overflow-hidden">
          <div className="flex items-start gap-3 px-4 pt-3 pb-2.5">
            <span className="mt-0.5"> <StatusIcon status=summaryStatus pending=running /> </span>
            <div className="flex min-w-0 flex-1 flex-col">
              <span className={`${body.sm.semibold} ${tone}`}> {React.string(headline)} </span>
              <span className={`${body.xs.regular} text-nd_gray-500 truncate`} title=fileName>
                {React.string(
                  `${fileName} · ${Int.toString(finished)} of ${Int.toString(count)} steps ran · ${duration(
                      Nullable.make(total),
                    )}`,
                )}
              </span>
            </div>
            <button
              type_="button"
              id="steps-minimize"
              ariaLabel="Minimize"
              title="Minimize"
              onClick={_ => setIsOpen(_ => false)}
              className="h-7 w-7 flex items-center justify-center rounded-md text-nd_gray-500 hover:bg-nd_gray-50">
              <Icon name="chevron-down" size=14 />
            </button>
          </div>
          <div className="h-0.5 bg-nd_gray-150">
            <div
              className={`h-full transition-all duration-150 ${failedAt >= 0 && !running
                  ? "bg-nd_red-500"
                  : "bg-primary"}`}
              style={{width: `${Int.toString(progress)}%`}}
            />
          </div>
          <div className="overflow-y-auto py-1">
            {steps
            ->Array.mapWithIndex((s, i) => {
              let pending = i >= revealed
              let isExpanded = expanded == Some(s.key)
              <div key=s.key className="px-2">
                <button
                  type_="button"
                  id={`step-${s.key}`}
                  onClick={_ => setExpanded(current => current == Some(s.key) ? None : Some(s.key))}
                  className={`w-full flex items-center gap-2.5 px-2 py-1.5 rounded-lg text-left focus:outline-none ${isExpanded
                      ? "bg-nd_gray-50"
                      : "hover:bg-nd_gray-25"}`}>
                  <StatusIcon status=s.status pending />
                  <span
                    className={`flex-1 min-w-0 truncate ${body.sm.medium} ${s.status == "not_run" && !pending
                        ? "text-nd_gray-400"
                        : "text-nd_gray-700"}`}>
                    {React.string(`${Int.toString(i + 1)}. ${s.title}`)}
                  </span>
                  <span className={`${body.xs.regular} text-nd_gray-500 shrink-0`}>
                    {React.string(pending ? "" : s.metric)}
                  </span>
                  <Icon
                    name={isExpanded ? "chevron-down" : "chevron-right"}
                    size=12
                    className="text-nd_gray-400"
                  />
                </button>
                {isExpanded && !pending
                  ? <div className="flex flex-col gap-1.5 pl-9 pr-2 pt-1 pb-2.5">
                      <p className={`${body.xs.regular} leading-18 text-nd_gray-600`}> {React.string(s.detail)} </p>
                      <div className={`flex flex-wrap gap-x-4 gap-y-0.5 ${body.xs.regular} text-nd_gray-500`}>
                        <span>
                          {React.string("Code ")}
                          <span className={`${code.sm.medium} text-nd_gray-700`}> {React.string(s.code)} </span>
                        </span>
                        <span>
                          {React.string("Time ")}
                          <span className="text-nd_gray-700"> {React.string(duration(s.micros))} </span>
                        </span>
                      </div>
                    </div>
                  : React.null}
              </div>
            })
            ->React.array}
          </div>
        </div>
      : <button
          type_="button"
          id="steps-open"
          onClick={_ => setIsOpen(_ => true)}
          className="ml-auto flex items-center gap-2.5 h-10 pl-3 pr-3.5 rounded-full border border-nd_gray-200 bg-white shadow-generic_shadow hover:bg-nd_gray-25">
          <StatusIcon status=summaryStatus pending=running />
          <span className={`${body.sm.medium} ${tone}`}> {React.string(headline)} </span>
          <span className={`${body.xs.regular} text-nd_gray-500`}>
            {React.string(`${Int.toString(finished)}/${Int.toString(count)} · ${duration(Nullable.make(total))}`)}
          </span>
          <Icon name="chevron-up" size=12 className="text-nd_gray-400" />
        </button>}
  </div>
}
