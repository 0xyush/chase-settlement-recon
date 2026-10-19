// ReconEnginePipelinesUploadModal.UploadDropzone: dashed rounded-xl zone, upload icon,
// "Choose files or drag & drop them here", Browse Files button, drag depth counter.
open Typography

@react.component
let make = (~onLoad: (string, string) => unit) => {
  let (dragging, setDragging) = React.useState(() => false)
  let (error, setError) = React.useState((): option<string> => None)
  let depth = React.useRef(0)

  React.useEffect0(() => Some(Engine.preventWindowDrop()))

  let readFile = file =>
    if Engine.isAccepted(file) {
      setError(_ => None)
      Engine.readFileText(file)->Promise.thenResolve(text => onLoad(Engine.fileName(file), text))->ignore
    } else {
      setError(_ => Some(`${Engine.fileName(file)}: unsupported file type. Chase DFR files (.dfr) only.`))
    }

  let loadSample = (sample: Engine.sample) =>
    Engine.fetchSample(sample.file)->Promise.thenResolve(text => onLoad(sample.file, text))->ignore

  let onDragEnter = e => {
    ReactEvent.Mouse.preventDefault(e)
    depth.current = depth.current + 1
    setDragging(_ => true)
  }
  let onDragOver = e => ReactEvent.Mouse.preventDefault(e)
  let onDragLeave = _ => {
    depth.current = depth.current > 0 ? depth.current - 1 : 0
    if depth.current == 0 {
      setDragging(_ => false)
    }
  }
  let onDrop = e => {
    ReactEvent.Mouse.preventDefault(e)
    depth.current = 0
    setDragging(_ => false)
    switch Engine.fileFromDrop(e)->Nullable.toOption {
    | Some(file) => readFile(file)
    | None => setError(_ => Some("No file selected. Please choose a file."))
    }
  }
  let onChange = e =>
    switch Engine.fileFromInput(e)->Nullable.toOption {
    | Some(file) => readFile(file)
    | None => ()
    }

  let zoneClass = dragging
    ? "border-nd_primary_blue-500 bg-nd_primary_blue-25 shadow-sm"
    : "border-nd_gray-300 bg-white hover:border-nd_gray-400"

  <div className="flex flex-col gap-4">
    <input id="pipelinesFileUploadInput" type_="file" accept=".dfr,.dfr_resp,.txt,.dat,.csv" hidden=true onChange />
    <label
      htmlFor="pipelinesFileUploadInput"
      onDragEnter
      onDragOver
      onDragLeave
      onDrop
      className={`flex flex-col items-center justify-center w-full min-h-64 border border-dashed ${zoneClass} rounded-xl cursor-pointer transition-all duration-200 ease-out`}>
      <div className="flex flex-col items-center justify-center py-8 gap-5 px-4 text-center">
        <Icon name="nd-upload" size=28 className="text-nd_gray-400" />
        <div className="flex flex-col gap-1 items-center">
          <div className={`${body.lg.semibold} text-nd_gray-700`}>
            {React.string("Choose a file or drag & drop it here")}
          </div>
          <div className={`${body.md.medium} text-nd_gray-500`}>
            {React.string(".dfr or .dfr_resp | Chase Paymentech Delimited File Reports")}
          </div>
        </div>
        <div
          className={`px-3 py-2 border border-nd_gray-200 text-nd_gray-600 ${body.sm.semibold} rounded-lg hover:bg-nd_gray-100`}>
          {React.string("Browse Files")}
        </div>
      </div>
    </label>
    {switch error {
    | Some(message) => <Helpers.InfoBox tone=Error message />
    | None => React.null
    }}
    <div className="flex flex-wrap items-center gap-2">
      <span className={`${body.sm.medium} text-nd_gray-500 mr-1`}> {React.string("Or try a sample")} </span>
      {Engine.samples
      ->Array.mapWithIndex((sample, i) =>
        <Button
          key=sample.file
          id={`sample-${Int.toString(i)}`}
          buttonType=Secondary
          buttonSize=Small
          text=sample.label
          leftIcon={<Icon name="nd-file" size=14 className="text-nd_gray-500" />}
          onClick={_ => loadSample(sample)}
        />
      )
      ->React.array}
    </div>
  </div>
}
