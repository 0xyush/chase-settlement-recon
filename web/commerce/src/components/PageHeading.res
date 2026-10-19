// PageUtils.PageHeading (heading.lg.semibold title + muted subtitle) under a BreadCrumbNavigation row.
open Typography

@react.component
let make = (~title: string, ~subTitle="", ~breadcrumb: array<string>=[]) =>
  <div className="flex flex-col gap-2">
    {breadcrumb->Array.length == 0
      ? React.null
      : <div className={`flex items-center gap-1 ${body.md.medium} text-nd_gray-500`}>
          {breadcrumb
          ->Array.mapWithIndex((crumb, i) => {
            let last = i == breadcrumb->Array.length - 1
            <React.Fragment key=crumb>
              {i == 0 ? React.null : <Icon name="chevron-right" size=12 className="text-nd_gray-400" />}
              <span className={last ? "text-nd_gray-800" : "hover:text-nd_gray-700 cursor-default"}>
                {React.string(crumb)}
              </span>
            </React.Fragment>
          })
          ->React.array}
        </div>}
    <div>
      <div className={`${heading.lg.semibold} text-nd_gray-800`}> {React.string(title)} </div>
      {subTitle == ""
        ? React.null
        : <div className={`mt-1 ${body.md.medium} text-nd_gray-500`}> {React.string(subTitle)} </div>}
    </div>
  </div>
