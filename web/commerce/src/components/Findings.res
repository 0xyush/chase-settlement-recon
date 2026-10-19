// Findings card: SectionTitle header + FieldRow-style divided rows with ReconEngineDataHelper.StatusIndicator dots.
open Typography

@react.component
let make = (~checks: array<Engine.check>) =>
  <div className="border border-nd_gray-150 rounded-lg bg-white">
    <div className="px-4 py-2.5 border-b border-nd_gray-150">
      <Helpers.SectionTitle count={checks->Array.length}> {React.string("Findings")} </Helpers.SectionTitle>
    </div>
    {checks
    ->Array.mapWithIndex((check, index) =>
      <div
        key={Int.toString(index)}
        className={`flex items-start gap-3 px-4 py-2.5 border-b border-nd_gray-150 last:border-0 ${body.sm.regular} text-nd_gray-700`}>
        <span className="mt-1.5 flex"> <Helpers.StatusDot level=check.level /> </span>
        <span> {React.string(check.text)} </span>
      </div>
    )
    ->React.array}
  </div>
