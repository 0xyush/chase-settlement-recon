// Stand-in for the Control Center's <Icon name="nd-..." /> (its nd-* SVG set). Inline outline paths, 24-unit grid.
let paths = name =>
  switch name {
  | "nd-home" => ["M3 10.5 12 3l9 7.5", "M5 9.5V20a1 1 0 0 0 1 1h4v-6h4v6h4a1 1 0 0 0 1-1V9.5"]
  | "nd-inbox" => [
      "M22 12h-6l-2 3h-4l-2-3H2",
      "M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z",
    ]
  | "nd-reports" => ["M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z", "M14 2v6h6", "M16 13H8", "M16 17H8"]
  | "nd-connectors" => ["m12 2 10 5-10 5L2 7z", "m2 12 10 5 10-5", "m2 17 10 5 10-5"]
  | "nd-workflow" => [
      "M3 5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
      "M7 11v4a2 2 0 0 0 2 2h4",
      "M13 15a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2z",
    ]
  | "nd-settings" => ["M4 21v-7", "M4 10V3", "M12 21v-9", "M12 8V3", "M20 21v-5", "M20 12V3", "M2 14h4", "M10 8h4", "M18 16h4"]
  | "nd-upload" => ["M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4", "m17 8-5-5-5 5", "M12 3v12"]
  | "nd-download" => ["M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4", "m7 10 5 5 5-5", "M12 15V3"]
  | "nd-file" => ["M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z", "M14 2v6h6"]
  | "nd-search" => ["M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14z", "m21 21-4.35-4.35"]
  | "chevron-left" => ["m15 18-6-6 6-6"]
  | "chevron-right" => ["m9 18 6-6-6-6"]
  | "chevron-down" => ["m6 9 6 6 6-6"]
  | "chevron-up" => ["m18 15-6-6-6 6"]
  | "nd-arrow-right" => ["M5 12h14", "m12 5 7 7-7 7"]
  | "arrow-left-right" => ["M8 3 4 7l4 4", "M4 7h16", "m16 21 4-4-4-4", "M20 17H4"]
  | "x" => ["M18 6 6 18", "m6 6 12 12"]
  | "nd-check" => ["M20 6 9 17l-5-5"]
  | "nd-toast-info" => ["M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z", "M12 16v-4", "M12 8h.01"]
  | "nd-alert" => ["m21.7 18-8-14a2 2 0 0 0-3.4 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.7-3z", "M12 9v4", "M12 17h.01"]
  | "bars" => ["M4 6h16", "M4 12h16", "M4 18h16"]
  | "sync-alt" => ["M21 12a9 9 0 0 1-15.5 6.3L3 16", "M3 12a9 9 0 0 1 15.5-6.3L21 8", "M21 3v5h-5", "M3 21v-5h5"]
  | "play" => ["m6 4 14 8-14 8z"]
  | "sun" => [
      "M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8z",
      "M12 2v2",
      "M12 20v2",
      "m4.93 4.93 1.41 1.41",
      "m17.66 17.66 1.41 1.41",
      "M2 12h2",
      "M20 12h2",
      "m6.34 17.66-1.41 1.41",
      "m19.07 4.93-1.41 1.41",
    ]
  | "moon" => ["M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9z"]
  | "rotate-ccw" => ["M3 12a9 9 0 1 0 3-6.7L3 8", "M3 3v5h5"]
  | "nd-store" => ["M3 21h18", "M5 21V7l7-4 7 4v14", "M9 21v-6h6v6"]
  | "nd-code" => ["m16 18 6-6-6-6", "m8 6-6 6 6 6"]
  | _ => []
  }

@react.component
let make = (~name: string, ~size=16, ~className="") =>
  <svg
    width={Int.toString(size)}
    height={Int.toString(size)}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.75"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={`shrink-0 ${className}`}
    ariaHidden=true>
    {paths(name)->Array.mapWithIndex((d, i) => <path key={Int.toString(i)} d />)->React.array}
  </svg>
