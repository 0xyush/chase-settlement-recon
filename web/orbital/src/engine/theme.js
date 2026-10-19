// Light / dark theme: one class on <html>, remembered in this browser. Light is the default.
const KEY = "orbital-theme-mode";
export const isDark = () => document.documentElement.classList.contains("dark");
export const setDark = (dark) => {
  document.documentElement.classList.toggle("dark", dark);
  try {
    localStorage.setItem(KEY, dark ? "dark" : "light");
  } catch {}
};
