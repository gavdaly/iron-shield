type ThemeMode = "dark" | "light";

const THEME_STORAGE_KEY = "ironShieldTheme";
let activeTheme: ThemeMode = "dark";
let prefersLightQuery: MediaQueryList | null = null;
let usingSystemPreference = true;

export function initThemeManager(): void {
  activeTheme = detectInitialTheme();
  applyTheme(activeTheme);

  if (usingSystemPreference) {
    observeSystemPreference();
  }

  const select = document.getElementById("theme-mode-select") as HTMLSelectElement | null;
  if (select) {
    select.value = activeTheme;
    select.addEventListener("change", () => {
      const nextTheme = select.value === "light" ? "light" : "dark";
      setTheme(nextTheme, { persist: true });
    });
  }
}

function detectInitialTheme(): ThemeMode {
  if (typeof window === "undefined") {
    return "dark";
  }

  const stored = getStoredTheme();
  if (stored) {
    usingSystemPreference = false;
    return stored;
  }

  usingSystemPreference = true;
  return window.matchMedia?.("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function applyTheme(theme: ThemeMode): void {
  document.documentElement.setAttribute("data-theme", theme);
}

function setTheme(theme: ThemeMode, options?: { persist?: boolean }): void {
  activeTheme = theme;
  applyTheme(theme);

  if (options?.persist) {
    usingSystemPreference = false;
    try {
      localStorage.setItem(THEME_STORAGE_KEY, theme);
    } catch {
      // Ignore persistence errors.
    }
  }

  syncSelect(theme);
}

function getStoredTheme(): ThemeMode | null {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark") {
      return stored;
    }
  } catch {
    // Ignore storage access issues.
  }

  return null;
}

function observeSystemPreference(): void {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return;
  }

  prefersLightQuery = window.matchMedia("(prefers-color-scheme: light)");
  const handler = (event: MediaQueryList | MediaQueryListEvent): void => {
    if (!usingSystemPreference) {
      return;
    }

    const theme: ThemeMode = event.matches ? "light" : "dark";
    setTheme(theme);
  };

  if (typeof prefersLightQuery.addEventListener === "function") {
    prefersLightQuery.addEventListener("change", handler);
  } else if (typeof prefersLightQuery.addListener === "function") {
    prefersLightQuery.addListener(handler);
  }
}

function syncSelect(theme: ThemeMode): void {
  const select = document.getElementById("theme-mode-select") as HTMLSelectElement | null;
  if (select && select.value !== theme) {
    select.value = theme;
  }
}
