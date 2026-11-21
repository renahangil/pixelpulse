import catalog from "./locales.json";

export type Language = keyof typeof catalog;
export type TextKey = keyof typeof catalog["en-US"];

let active: Language = "en-US";
export function useLanguage(language: Language): void {
  active = language;
  document.documentElement.lang = language;
}

export function t(key: TextKey): string {
  return catalog[active][key];
}
