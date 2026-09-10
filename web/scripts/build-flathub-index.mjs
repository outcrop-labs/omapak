#!/usr/bin/env node
// Builds static/data/flathub.json: flathub's catalog as pass-through entries.
// We do not mirror or proxy their bytes; the site lists their apps, badged
// "flathub managed", with installs going through their remote. If they ever
// cut us off, the fallback is a full mirror overnight (see MISSION.md).
//
// Flags: --force (ignore cache age) · --limit N (dev) · --max-age seconds
import { existsSync, statSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const OUT = resolve(here, "../static/data/flathub.json");
const MAX_AGE = 24 * 60 * 60;
const CONCURRENCY = 12;

const args = process.argv.slice(2);
const force = args.includes("--force");
const limit = args.includes("--limit") ? Number(args[args.indexOf("--limit") + 1]) : 0;

if (!force && existsSync(OUT)) {
  const age = (Date.now() - statSync(OUT).mtimeMs) / 1000;
  if (age < MAX_AGE) {
    console.log(`flathub index fresh (${Math.round(age / 3600)}h old), reusing`);
    process.exit(0);
  }
}

const listRes = await fetch("https://flathub.org/api/v2/appstream");
if (!listRes.ok) throw new Error(`flathub list: ${listRes.status}`);
const ids = await listRes.json();
const slice = limit ? ids.slice(0, limit) : ids;
console.log(`flathub: fetching details for ${slice.length} apps`);

async function pool(items, worker) {
  const queue = [...items];
  const runners = Array.from({ length: CONCURRENCY }, async () => {
    while (queue.length) {
      const item = queue.shift();
      try {
        await worker(item);
      } catch {
        // individual failures just drop the entry
      }
    }
  });
  await Promise.all(runners);
}

const apps = [];
await pool(slice, async (id) => {
  const res = await fetch(`https://flathub.org/api/v2/appstream/${id}`);
  if (!res.ok) return;
  const d = await res.json();
  if (d.is_eol) return;
  apps.push({
    app_id: id,
    name: d.name ?? id,
    summary: d.summary ?? "",
    icon: d.icon ?? null,
    license: d.is_free_license === false ? "proprietary" : (d.project_license ?? null),
  });
});

apps.sort((a, b) => a.app_id.localeCompare(b.app_id));
writeFileSync(OUT, JSON.stringify({ generated_at: new Date().toISOString(), apps }));
console.log(`flathub index: ${apps.length} entries written`);
