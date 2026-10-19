// Control Center Sidebar.res + ReconEngineSidebarValues.res: Operate / Monitor / Configure sections.
open Typography

type item = {name: string, icon: string, active: bool}
type section = {heading: string, items: array<item>}

let sections = [
  {
    heading: "Operate",
    items: [
      {name: "Overview", icon: "nd-home", active: false},
      {name: "Exceptions", icon: "nd-inbox", active: false},
    ],
  },
  {
    heading: "Monitor",
    items: [
      {name: "Transactions", icon: "nd-reports", active: false},
      {name: "Transformed Entries", icon: "nd-connectors", active: false},
      {name: "Pipelines", icon: "nd-workflow", active: true},
    ],
  },
  {heading: "Configure", items: [{name: "Rules Library", icon: "nd-settings", active: false}]},
]

module Link = {
  @react.component
  let make = (~item: item, ~expanded: bool) => {
    let selected = item.active
      ? `bg-sidebar-hoverColor text-sidebar-textColorPrimary ${body.md.semibold}`
      : `text-sidebar-textColor hover:bg-sidebar-hoverColor ${body.md.medium}`
    let layout = expanded ? "gap-4 px-3 py-1.5" : "justify-center py-2"
    <div
      title=item.name
      className={`relative overflow-hidden flex flex-row items-center rounded-lg cursor-pointer whitespace-nowrap transition duration-300 ${layout} ${selected}`}>
      <Icon
        size=18 name=item.icon className={item.active ? "text-sidebar-textColorPrimary" : "text-nd_gray-500"}
      />
      {expanded ? React.string(item.name) : React.null}
    </div>
  }
}

// Pages kept off the main path. One button at the bottom of the sidebar opens them.
type page = {label: string, icon: string, href: string}
let otherPages = [
  {label: "Commerce dashboard", icon: "nd-store", href: "../commerce/"},
]

module MorePages = {
  @react.component
  let make = (~expanded: bool) => {
    let (opened, setOpened) = React.useState(() => false)
    let layout = expanded ? "gap-4 px-3 py-1.5" : "justify-center py-2"
    let row = `flex flex-row items-center rounded-lg whitespace-nowrap text-sidebar-textColor hover:bg-sidebar-hoverColor transition duration-300 ${layout} ${body.md.medium}`
    <div className="flex flex-col gap-1 p-2.5 border-t border-sidebar-borderColor">
      <button type_="button" title="More pages" ariaExpanded=opened onClick={_ => setOpened(v => !v)} className=row>
        <Icon size=18 name="bars" className="text-nd_gray-500" />
        {expanded
          ? <>
              <span className="flex-1 text-left"> {React.string("More pages")} </span>
              <Icon size=14 name={opened ? "chevron-up" : "chevron-down"} className="text-nd_gray-400" />
            </>
          : React.null}
      </button>
      {opened
        ? otherPages
          ->Array.map(page =>
            <a key=page.href href=page.href title=page.label className={`${row} ${expanded ? "pl-6" : ""}`}>
              <Icon size=16 name=page.icon className="text-nd_gray-500" />
              {expanded ? React.string(page.label) : React.null}
            </a>
          )
          ->React.array
        : React.null}
    </div>
  }
}

@react.component
let make = (~expanded: bool) => {
  let width = expanded ? "w-56" : "w-14"
  <aside
    className={`${width} shrink-0 h-screen flex flex-col bg-sidebar-primary border-r border-sidebar-borderColor transition-all duration-200 overflow-hidden`}>
    // The official Juspay hyperswitch logo (hyperswitch repo, docs/imgs); the mark alone when collapsed.
    <div className="flex items-center mt-4 mb-5 px-3.5 h-8 cursor-default">
      {expanded
        ? <>
            <img src="brand/hyperswitch-logo-light.svg" alt="Juspay hyperswitch" className="h-5 w-auto dark:hidden" />
            <img src="brand/hyperswitch-logo-dark.svg" alt="Juspay hyperswitch" className="h-5 w-auto hidden dark:block" />
          </>
        : <img src="brand/hyperswitch-icon.svg" alt="Juspay hyperswitch" className="h-7 w-7 shrink-0" />}
    </div>
    <nav className="flex-1 overflow-y-auto overflow-x-hidden sidebar-scrollbar">
      <div className="flex flex-col gap-2 p-2.5 pt-0">
        {sections
        ->Array.map(section =>
          <React.Fragment key=section.heading>
            {expanded
              ? <div className={`px-3 pt-2 text-nd_gray-400 tracking-widest uppercase ${body.sm.semibold}`}>
                  {React.string(section.heading)}
                </div>
              : <div className="mx-2 mt-2 border-t border-sidebar-borderColor" />}
            {section.items
            ->Array.map(item => <Link key=item.name item expanded />)
            ->React.array}
          </React.Fragment>
        )
        ->React.array}
      </div>
    </nav>
    <MorePages expanded />
    <div className="flex items-center gap-3 p-3 border-t border-sidebar-borderColor">
      <div
        className="w-7 h-7 shrink-0 rounded-full bg-nd_primary_blue-50 text-nd_primary_blue-600 flex items-center justify-center text-fs-12 font-semibold">
        {React.string("M")}
      </div>
      {expanded
        ? <div className="min-w-0">
            <div className={`${body.md.medium} text-nd_gray-700 truncate`}> {React.string("Merchant admin")} </div>
            <div className={`${body.sm.regular} text-nd_gray-500 truncate`}> {React.string("Sandbox")} </div>
          </div>
        : React.null}
    </div>
  </aside>
}
