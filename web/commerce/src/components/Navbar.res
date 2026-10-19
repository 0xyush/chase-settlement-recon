// Control Center Navbar.res (min-h-16 row) with the OMP switch pill and ReconEngineStatusIndicator on the right.
open Typography

@react.component
let make = (~onToggleSidebar: unit => unit, ~pending: int) =>
  <div id="navbar" className="w-full">
    <div className="flex flex-row min-h-14 items-center justify-between gap-3">
      <div className="flex items-center gap-2">
        <button
          type_="button"
          ariaLabel="Toggle sidebar"
          onClick={_ => onToggleSidebar()}
          className="h-8 w-8 flex items-center justify-center rounded-lg text-nd_gray-600 hover:bg-nd_gray-50 focus:outline-none">
          <Icon name="bars" size=16 />
        </button>
        <div
          className="flex items-center gap-2 px-3 h-8 rounded-lg border border-nd_gray-200 bg-white shadow-xs cursor-default">
          <Icon name="nd-store" size=14 className="text-nd_gray-500" />
          <span className={`${body.sm.medium} text-nd_gray-700 whitespace-nowrap`}>
            {React.string("Sandbox merchant")}
          </span>
          <Icon name="chevron-down" size=12 className="text-nd_gray-400" />
        </div>
      </div>
      <div className="flex items-center gap-2">
        <div
          title="Recon processor is running"
          className="flex items-center gap-2 px-2 h-8 bg-white border border-nd_gray-300 rounded-lg whitespace-nowrap cursor-default">
          <span className="relative flex h-2 w-2">
            <span
              className="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 bg-nd_green-400"
            />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-nd_green-400" />
          </span>
          <span className={`${body.sm.semibold} text-nd_gray-600`}>
            {React.string(`${Int.toLocaleString(pending)} Pending`)}
          </span>
        </div>
        <div
          className="flex items-center justify-center w-8 h-8 bg-white border border-nd_gray-300 rounded-lg cursor-pointer">
          <Icon
            name="sync-alt"
            size=14
            className="text-nd_gray-600 hover:rotate-180 transition-transform duration-500"
          />
        </div>
      </div>
    </div>
  </div>
