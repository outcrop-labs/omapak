import fs from "node:fs";
import path from "node:path";

// Prerender entries come from the generated catalog so the static build
// emits one page per published app.
export function entries() {
  try {
    const catalog = JSON.parse(
      fs.readFileSync(path.resolve("static/data/catalog.json"), "utf8"),
    );
    return catalog.entries
      .filter((e: { report_available: boolean }) => e.report_available)
      .map((e: { app_id: string }) => ({ id: e.app_id }));
  } catch {
    return [];
  }
}

export const prerender = true;
