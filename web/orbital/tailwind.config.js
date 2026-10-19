/** Token subset copied from hyperswitch-control-center/tailwind.config.js and config/theme.json.
 *
 * Every color is a CSS variable with a light value (:root) and a dark value (.dark), so dark mode
 * is one class on <html>, set in index.html and toggled from the navbar. */
import plugin from "tailwindcss/plugin";

const light = {
  primary: "#006DF9", background: "#FFFFFF", white: "#FFFFFF", border_gray: "#E1E4EA",
  "sidebar-primary": "#FCFCFD", "sidebar-textColor": "#525866", "sidebar-textColorPrimary": "#1C6DEA",
  "sidebar-hoverColor": "#EFF1F5", "sidebar-borderColor": "#E6E8ED",
  "button-secondary-bg": "#f3f3f3", "button-secondary-text": "#626168", "button-secondary-hoverbg": "#fcfcfd",
  "nd_gray-0": "#FFFFFF", "nd_gray-25": "#FCFCFD", "nd_gray-50": "#F5F7FA", "nd_gray-100": "#FBFBFB",
  "nd_gray-150": "#ECEFF3", "nd_gray-200": "#E1E4EA", "nd_gray-300": "#CACFD8", "nd_gray-400": "#99A0AE",
  "nd_gray-500": "#606B85", "nd_gray-600": "#525866", "nd_gray-700": "#2B303B", "nd_gray-800": "#222530",
  "nd_gray-950": "#0E121B",
  "nd_primary_blue-25": "#F6FAFE", "nd_primary_blue-50": "#E4F1FD", "nd_primary_blue-100": "#BCD7FA",
  "nd_primary_blue-200": "#93BCF6", "nd_primary_blue-300": "#6AA1F2", "nd_primary_blue-400": "#4287EF",
  "nd_primary_blue-450": "#247DF9", "nd_primary_blue-500": "#1C6DEA", "nd_primary_blue-600": "#0561E2",
  "nd_br_gray-150": "#ECEFF3", "nd_br_gray-200": "#E1E4EA", "nd_br_gray-400": "#E1E1E1", "nd_br_gray-500": "#E1E3EA",
  "nd_green-50": "#ECF4EE", "nd_green-100": "#E8F8F5", "nd_green-150": "#F0FDF4", "nd_green-200": "#52B87A",
  "nd_green-300": "#69AF7D", "nd_green-400": "#2DA160", "nd_green-500": "#11927E", "nd_green-600": "#25744C",
  "nd_red-50": "#FCF1EF", "nd_red-100": "#FDD4CD", "nd_red-200": "#FCB5AA", "nd_red-400": "#EC5941",
  "nd_red-500": "#DD2B0E", "nd_red-600": "#C91C00",
  "nd_orange-50": "#FDF1DD", "nd_orange-100": "#FFE6CC", "nd_orange-150": "#FFF7ED", "nd_orange-200": "#FFD6A8",
  "nd_orange-300": "#FF6900", "nd_orange-600": "#9E5400",
  "nd_yellow-50": "#FEFCE8", "nd_yellow-100": "#FFFBEE", "nd_yellow-300": "#FFDF20", "nd_yellow-500": "#EFB100",
  "nd_yellow-600": "#D08700",
};

// Dark: the gray scale runs the other way (surfaces dark, text light), tints become deep tones,
// and the text shades of each color get lighter so they stay readable.
const dark = {
  primary: "#4C9AFF", background: "#0D1117", white: "#161B22", border_gray: "#2D3542",
  "sidebar-primary": "#11151C", "sidebar-textColor": "#A3ACBA", "sidebar-textColorPrimary": "#6AA8FF",
  "sidebar-hoverColor": "#1C232D", "sidebar-borderColor": "#222A35",
  "button-secondary-bg": "#1C232D", "button-secondary-text": "#C3CAD5", "button-secondary-hoverbg": "#242C38",
  "nd_gray-0": "#161B22", "nd_gray-25": "#1A2029", "nd_gray-50": "#1C232D", "nd_gray-100": "#1A1F27",
  "nd_gray-150": "#252D38", "nd_gray-200": "#2D3542", "nd_gray-300": "#3B4555", "nd_gray-400": "#7E8899",
  "nd_gray-500": "#9AA4B4", "nd_gray-600": "#B0B8C5", "nd_gray-700": "#D2D7DF", "nd_gray-800": "#E8EBF0",
  "nd_gray-950": "#F5F7FA",
  "nd_primary_blue-25": "#111D30", "nd_primary_blue-50": "#14284A", "nd_primary_blue-100": "#1B3666",
  "nd_primary_blue-200": "#28508F", "nd_primary_blue-300": "#3D6EB8", "nd_primary_blue-400": "#5B9BF5",
  "nd_primary_blue-450": "#4C9AFF", "nd_primary_blue-500": "#4C9AFF", "nd_primary_blue-600": "#86BBFF",
  "nd_br_gray-150": "#252D38", "nd_br_gray-200": "#2D3542", "nd_br_gray-400": "#2D3542", "nd_br_gray-500": "#2D3542",
  "nd_green-50": "#11281B", "nd_green-100": "#122E22", "nd_green-150": "#10251A", "nd_green-200": "#4FB878",
  "nd_green-300": "#4FB373", "nd_green-400": "#43C47A", "nd_green-500": "#2DB8A2", "nd_green-600": "#74D6A0",
  "nd_red-50": "#341612", "nd_red-100": "#5C2318", "nd_red-200": "#7A2C1E", "nd_red-400": "#F2735E",
  "nd_red-500": "#FF6F57", "nd_red-600": "#FF9280",
  "nd_orange-50": "#33240F", "nd_orange-100": "#3D2B13", "nd_orange-150": "#2C200F", "nd_orange-200": "#5C401B",
  "nd_orange-300": "#FF8C38", "nd_orange-600": "#F4A64C",
  "nd_yellow-50": "#2E2A10", "nd_yellow-100": "#332E12", "nd_yellow-300": "#FFE04D", "nd_yellow-500": "#F2C230",
  "nd_yellow-600": "#E3A82B",
};

const channels = (hex) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)).join(" ");
const variables = (palette) =>
  Object.fromEntries(Object.entries(palette).map(([key, hex]) => [`--c-${key}`, channels(hex)]));
const c = (key) => `rgb(var(--c-${key}) / <alpha-value>)`;
const scale = (name, steps) => Object.fromEntries(steps.map((step) => [step, c(`${name}-${step}`)]));

export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.res", "./src/**/*.mjs"],
  theme: {
    fontFamily: {
      "inter-style": ['"Inter"', "-apple-system", "BlinkMacSystemFont", '"Segoe UI"', "sans-serif"],
      "jetbrains-mono": ['"JetBrains Mono"', "ui-monospace", "monospace"],
    },
    extend: {
      screens: { mobile: "28.125rem" },
      maxWidth: { fixedPageWidth: "120rem" },
      fontSize: {
        "fs-10": "10px", "fs-11": "11px", "fs-12": "12px", "fs-13": "13px", "fs-14": "14px",
        "fs-16": "16px", "fs-18": "18px", "fs-20": "20px", "fs-24": "24px",
      },
      lineHeight: { 14: "14px", 16: "16px", 18: "18px", 20: "20px", 24: "24px", 26: "26px", 32: "32px" },
      boxShadow: {
        xs: "0px 1px 2px 0px rgba(0, 0, 0, 0.05)",
        cardShadow: "0px 2px 2px 0px rgba(0, 0, 0, 0.04)",
        hyperswitch_box_shadow: "0 2px 8px 0px rgba(0,0,0,0.08)",
        generic_shadow: "0 2px 5px 0 rgba(0, 0, 0, 0.12)",
        focusBoxShadow: "0px 1px 2px 0px rgba(0, 0, 0, 0.05), 0px 0px 0px 4px rgba(232, 243, 255, 1)",
      },
      colors: {
        primary: { DEFAULT: c("primary"), custom: c("primary") },
        white: c("white"),
        hyperswitch_background: c("background"),
        sidebar: {
          primary: c("sidebar-primary"), textColor: c("sidebar-textColor"),
          textColorPrimary: c("sidebar-textColorPrimary"), hoverColor: c("sidebar-hoverColor"),
          borderColor: c("sidebar-borderColor"),
        },
        button: {
          // The blue primary button looks the same in both themes.
          primary: { bg: "#1272f9", text: "#ffffff", hoverbg: "#0860dd" },
          secondary: { bg: c("button-secondary-bg"), text: c("button-secondary-text"), hoverbg: c("button-secondary-hoverbg") },
        },
        border_gray: c("border_gray"),
        nd_gray: scale("nd_gray", [0, 25, 50, 100, 150, 200, 300, 400, 500, 600, 700, 800, 950]),
        nd_primary_blue: scale("nd_primary_blue", [25, 50, 100, 200, 300, 400, 450, 500, 600]),
        nd_br_gray: scale("nd_br_gray", [150, 200, 400, 500]),
        nd_green: scale("nd_green", [50, 100, 150, 200, 300, 400, 500, 600]),
        nd_red: scale("nd_red", [50, 100, 200, 400, 500, 600]),
        nd_orange: scale("nd_orange", [50, 100, 150, 200, 300, 600]),
        nd_yellow: scale("nd_yellow", [50, 100, 300, 500, 600]),
      },
    },
  },
  plugins: [
    plugin(({ addBase }) =>
      addBase({
        ":root": variables(light),
        ".dark": { ...variables(dark), colorScheme: "dark" },
      }),
    ),
  ],
};
