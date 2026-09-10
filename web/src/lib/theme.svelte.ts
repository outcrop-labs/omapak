/**
 * MERCURY'S TWO MODES — theme switch, ported from the skal site
 * (skal-website/src/lib/theme.svelte.ts, originally Talaria). mercury (dark)
 * and mercury-light (paper-white) as explicit choices plus `system`, which
 * follows prefers-color-scheme and is the default.
 *
 * The resolved mode is one attribute on <html>; everything token-bound
 * follows it for free.
 */

export type ThemeId = "mercury" | "mercury-light";
export type ThemePref = ThemeId | "system";

const STORAGE_KEY = "omapak-theme";
export const DEFAULT_PREF: ThemePref = "system";

const META_COLOR: Record<ThemeId, string> = {
  mercury: "#090a09",
  "mercury-light": "#f2f0eb",
};

const CYCLE: Record<ThemePref, ThemePref> = {
  system: "mercury-light",
  "mercury-light": "mercury",
  mercury: "system",
};

let pref = $state<ThemePref>(DEFAULT_PREF);
let ready = $state(false);

export function getPref(): ThemePref {
  return pref;
}

export function isReady(): boolean {
  return ready;
}

function readStoredPref(): ThemePref {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "mercury" || stored === "mercury-light" || stored === "system") return stored;
  } catch {
    // Private mode et al. — the preference is simply not remembered.
  }
  return DEFAULT_PREF;
}

function resolve(pref: ThemePref): ThemeId {
  if (pref !== "system") return pref;
  if (typeof window === "undefined") return "mercury";
  return window.matchMedia("(prefers-color-scheme: light)").matches
    ? "mercury-light"
    : "mercury";
}

function applyMode(mode: ThemeId): void {
  document.documentElement.setAttribute("data-theme", mode);
  document
    .querySelector('meta[name="theme-color"]')
    ?.setAttribute("content", META_COLOR[mode]);
}

export function setPref(next: ThemePref): void {
  pref = next;
  if (typeof document === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Unwritable storage still flips the pageview.
  }
  applyMode(resolve(next));
}

export function cyclePref(): ThemePref {
  setPref(CYCLE[pref]);
  return pref;
}

export function initTheme(): void {
  if (typeof window === "undefined") return;
  pref = readStoredPref();
  ready = true;
  applyMode(resolve(pref));

  const mq = window.matchMedia("(prefers-color-scheme: light)");
  mq.addEventListener("change", () => {
    if (pref === "system") applyMode(resolve(pref));
  });
}
