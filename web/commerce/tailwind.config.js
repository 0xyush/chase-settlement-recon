/** Token subset copied from hyperswitch-control-center/tailwind.config.js and config/theme.json. */
export default {
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
        primary: { DEFAULT: "#006DF9", custom: "#006DF9" },
        hyperswitch_background: "#FFFFFF",
        sidebar: { primary: "#FCFCFD", textColor: "#525866", textColorPrimary: "#1C6DEA", hoverColor: "#EFF1F5", borderColor: "#E6E8ED" },
        button: {
          primary: { bg: "#1272f9", text: "#ffffff", hoverbg: "#0860dd" },
          secondary: { bg: "#f3f3f3", text: "#626168", hoverbg: "#fcfcfd" },
        },
        border_gray: "#E1E4EA",
        nd_gray: {
          0: "#FFFFFF", 25: "#FCFCFD", 50: "#F5F7FA", 100: "#FBFBFB", 150: "#ECEFF3", 200: "#E1E4EA",
          300: "#CACFD8", 400: "#99A0AE", 500: "#606B85", 600: "#525866", 700: "#2B303B", 800: "#222530", 950: "#0E121B",
        },
        nd_primary_blue: {
          25: "#F6FAFE", 50: "#E4F1FD", 100: "#BCD7FA", 200: "#93BCF6", 300: "#6AA1F2", 400: "#4287EF",
          450: "#247DF9", 500: "#1C6DEA", 600: "#0561E2",
        },
        nd_br_gray: { 150: "#ECEFF3", 200: "#E1E4EA", 400: "#E1E1E1", 500: "#E1E3EA" },
        nd_green: { 50: "#ECF4EE", 100: "#E8F8F5", 150: "#F0FDF4", 200: "#52B87A", 300: "#69AF7D", 400: "#2DA160", 500: "#11927E", 600: "#25744C" },
        nd_red: { 50: "#FCF1EF", 100: "#FDD4CD", 200: "#FCB5AA", 400: "#EC5941", 500: "#DD2B0E", 600: "#C91C00" },
        nd_orange: { 50: "#FDF1DD", 100: "#FFE6CC", 150: "#FFF7ED", 200: "#FFD6A8", 300: "#FF6900", 600: "#9E5400" },
        nd_yellow: { 50: "#FEFCE8", 100: "#FFFBEE", 300: "#FFDF20", 500: "#EFB100", 600: "#D08700" },
      },
    },
  },
  plugins: [],
};
