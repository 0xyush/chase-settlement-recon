// Transformation configs for each DFR report: copies of engines/orbital/configs
// (refresh with `npm run sync-configs`).
import act0010 from "./presets/act0010_deposit_detail.json";
import act0010Live from "./presets/act0010_deposit_detail_live.json";
import pde0017 from "./presets/pde0017_chargebacks.json";

export const PRESETS = {
  act0010_deposit_detail: act0010,
  act0010_deposit_detail_live: act0010Live,
  pde0017_chargebacks: pde0017,
};

// Synthetic sample files (orbital_recon samples), served from public/samples.
export const SAMPLES = [
  { file: "0000999999.261001.d.B101.dfr", label: "Day 1 · deposits (B file)" },
  { file: "0000999999.261001.d.A101.dfr", label: "Day 1 · chargebacks (A file)" },
  { file: "0000999999.261002.d.B102.dfr", label: "Day 2 · pipe + interchange" },
  { file: "0000999999.261003.d.B103.dfr", label: "Day 3 · no activity" },
  { file: "0000999999.261004.d.B104.dfr", label: "Day 4 · cut off in transfer" },
  { file: "0000999999.261005.d.B105.dfr", label: "Day 5 · live layout (tab)" },
];
