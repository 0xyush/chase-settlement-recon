// TableUtils.LabelCell -> TagBinding (Blend Tag, variant Subtle, shape Squarical, size Xs).
type color = Success | Error | Neutral | Primary | Warning

@react.component
let make = (~text: string, ~color=Neutral, ~uppercase=true) => {
  let look = switch color {
  | Success => "bg-nd_green-50 text-nd_green-600"
  | Error => "bg-nd_red-50 text-nd_red-600"
  | Neutral => "bg-nd_gray-150 text-nd_gray-600"
  | Primary => "bg-nd_primary_blue-50 text-nd_primary_blue-600"
  | Warning => "bg-nd_orange-50 text-nd_orange-600"
  }
  let case = uppercase ? "uppercase tracking-wide" : ""
  <span
    className={`inline-flex items-center h-5 px-1.5 rounded text-fs-11 leading-16 font-medium whitespace-nowrap ${case} ${look}`}>
    {React.string(text)}
  </span>
}

let ofOutcome = outcome =>
  switch outcome {
  | "Success" => Success
  | "Failed" => Error
  | _ => Neutral
  }

let outcomeText = outcome => outcome == "Success" ? "Staged" : outcome
