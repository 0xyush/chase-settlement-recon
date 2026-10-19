// Control Center Button.res: Primary / Secondary, Small (h-8 rounded-lg text-fs-13) / Medium (h-9).
type buttonType = Primary | Secondary
type buttonSize = Small | Medium

@react.component
let make = (
  ~text="",
  ~buttonType=Secondary,
  ~buttonSize=Small,
  ~leftIcon=?,
  ~onClick=?,
  ~disabled=false,
  ~customButtonStyle="",
  ~id=?,
) => {
  let sizeClass = switch buttonSize {
  | Small => "h-8 rounded-lg text-fs-13 px-3"
  | Medium => "h-9 rounded-[10px] text-fs-14 px-4"
  }
  let look = switch (buttonType, disabled) {
  | (Primary, false) => "bg-button-primary-bg text-white hover:bg-button-primary-hoverbg shadow-[0_0_0_1px_rgba(18,114,249,0.35)]"
  | (Primary, true) => "bg-nd_gray-200 text-nd_gray-500 cursor-not-allowed"
  | (Secondary, false) => "bg-white text-nd_gray-600 border border-border_gray hover:text-nd_gray-800 hover:shadow-xs"
  | (Secondary, true) => "bg-nd_gray-50 text-nd_gray-400 border border-border_gray cursor-not-allowed"
  }
  <button
    ?id
    type_="button"
    ?onClick
    disabled
    className={`inline-flex items-center justify-center gap-1.5 font-medium whitespace-nowrap select-none focus:outline-none transition-colors duration-150 ${sizeClass} ${look} ${customButtonStyle}`}>
    {switch leftIcon {
    | Some(icon) => icon
    | None => React.null
    }}
    {text == "" ? React.null : React.string(text)}
  </button>
}
