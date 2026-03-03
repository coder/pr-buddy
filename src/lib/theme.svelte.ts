export type ThemePreference = "system" | "light" | "dark";

let preference: ThemePreference = $state(
  (localStorage.getItem("theme") as ThemePreference) ?? "system",
);

function apply() {
  const isDark =
    preference === "dark" ||
    (preference === "system" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);

  document.documentElement.classList.toggle("dark", isDark);
}

window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", apply);

apply();

export function getTheme(): ThemePreference {
  return preference;
}

export function setTheme(t: ThemePreference) {
  preference = t;
  localStorage.setItem("theme", t);
  apply();
}
