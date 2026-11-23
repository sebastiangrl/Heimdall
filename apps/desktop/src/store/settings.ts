import { create } from "zustand";

const LOCALE_KEY = "heimdall.locale";
const GLASS_KEY = "heimdall.reducedGlass";

function readLocale(): "en" | "es" {
  try {
    const v = localStorage.getItem(LOCALE_KEY);
    return v === "en" || v === "es" ? v : "es";
  } catch {
    return "es";
  }
}

function readReducedGlass(): boolean {
  try {
    return localStorage.getItem(GLASS_KEY) === "1";
  } catch {
    return false;
  }
}

interface SettingsState {
  locale: "en" | "es";
  reducedGlass: boolean;
  setLocale: (locale: "en" | "es") => void;
  setReducedGlass: (value: boolean) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  locale: readLocale(),
  reducedGlass: readReducedGlass(),
  setLocale: (locale) => {
    try {
      localStorage.setItem(LOCALE_KEY, locale);
    } catch {
      /* ignore */
    }
    set({ locale });
  },
  setReducedGlass: (reducedGlass) => {
    try {
      localStorage.setItem(GLASS_KEY, reducedGlass ? "1" : "0");
    } catch {
      /* ignore */
    }
    set({ reducedGlass });
  },
}));