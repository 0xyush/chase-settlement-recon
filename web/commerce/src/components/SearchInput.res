// SearchInput.res: bordered rounded-lg box, nd-search icon, focus-within primary border.
open Typography

@react.component
let make = (~value: string, ~onChange: string => unit, ~placeholder="Search by ID", ~widthClass="w-full sm:w-72") =>
  <div
    className={`${widthClass} relative flex items-center h-9 border border-nd_br_gray-200 rounded-lg bg-nd_gray-0 hover:bg-nd_gray-50 focus-within:border-nd_primary_blue-500 transition-all duration-200`}>
    <div className="flex items-center pl-3"> <Icon name="nd-search" size=16 className="text-nd_gray-500" /> </div>
    <input
      type_="text"
      value
      placeholder
      onChange={e => onChange(ReactEvent.Form.target(e)["value"])}
      className={`flex-1 min-w-0 h-full px-2.5 bg-transparent ${body.md.regular} text-nd_gray-700 placeholder:text-nd_gray-400 focus:outline-none`}
    />
    {value == ""
      ? React.null
      : <button
          type_="button"
          ariaLabel="Clear search"
          onClick={_ => onChange("")}
          className="flex items-center pr-3 text-nd_gray-400 hover:text-nd_gray-600 focus:outline-none">
          <Icon name="x" size=14 />
        </button>}
  </div>
