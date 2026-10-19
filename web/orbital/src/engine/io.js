// Browser I/O the ReScript components call through bindings in Engine.res.
export const fileFromInput = (event) => event.target.files?.[0] ?? null;
export const fileFromDrop = (event) => event.dataTransfer?.files?.[0] ?? null;
export const fileName = (file) => file.name;
// DFR files usually end in .dfr (.dfr_resp when fetched from Chase); plain-text copies are fine
// too. The engine rejects anything else.
export const isAccepted = (file) => /\.(dfr|dfr_resp|txt|dat|csv)$/i.test(file.name) || !file.name.includes(".");
export const readFileText = (file) => file.text();
export const fetchSample = (name) => fetch(`samples/${name}`).then((r) => r.text());

// Stop the browser from opening a file dropped outside the drop zone.
export const preventWindowDrop = () => {
  const stop = (e) => e.preventDefault();
  window.addEventListener("dragover", stop);
  window.addEventListener("drop", stop);
  return () => {
    window.removeEventListener("dragover", stop);
    window.removeEventListener("drop", stop);
  };
};
