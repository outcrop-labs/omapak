#!/usr/bin/env node
// Builds catalog.json from apps/ and pushes it to R2 so the site reads
// live data from the CDN — no redeploy needed for app changes.
import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync } from "node:fs";
import { execSync } from "node:child_process";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../..");
const tmp = "/tmp/omapak-catalog";

// Build the catalog (same logic as build-catalog but writes to tmp)
mkdirSync(join(tmp, "reports"), { recursive: true });
const entries = [];

for (const name of readdirSync(join(root, "apps"))) {
  const appDir = join(root, "apps", name);
  const metaPath = join(appDir, "metadata.yml");
  const reportPath = join(appDir, "report.json");
  if (!existsSync(metaPath)) continue;

  let meta;
  try { meta = parseYaml(readFileSync(metaPath, "utf8")); } catch { continue; }
  if (!meta?.submitter || !meta?.source_repo || !meta?.summary) continue;

  const files = readdirSync(appDir);
  const xmlFile = files.find(f => f.endsWith(".metainfo.xml") || f.endsWith(".appdata.xml"));
  if (xmlFile) {
    const xml = readFileSync(join(appDir, xmlFile), "utf8");
    const pick = (re) => { const m = xml.match(re); return m ? m[1].trim() : null; };
    meta.name = pick(/<name>([^<]+)<\/name>/) || meta.name;
    meta.summary = pick(/<summary>([^<]+)<\/summary>/) || meta.summary;
    meta.developer = pick(/<developer[^>]*>[\s\S]*?<name>([^<]+)<\/name>/) || null;
    const dm = xml.match(/<description>([\s\S]*?)<\/description>/);
    if (dm) {
      const blocks = [];
      const parts = dm[1].split(/(<\/?(?:p|ul|li)>)/);
      let list = null, para = null;
      for (const p of parts) {
        const t = p.trim();
        if (t === "<p>") para = "";
        else if (t === "</p>") { if (para) blocks.push(para); para = null; }
        else if (t === "<ul>") list = [];
        else if (t === "</ul>") { if (list) blocks.push(list); list = null; }
        else if (t === "<li>") { if (list) list.push(""); }
        else if (t === "</li>") continue;
        else if (t && !t.startsWith("<")) {
          const text = t.replace(/<[^>]+>/g, "").trim();
          if (!text) continue;
          if (list) list[list.length - 1] = text;
          else if (para !== null) para = text;
        }
      }
      meta.description = blocks.length ? blocks : meta.description;
    }
    meta.urls = {};
    for (const m of xml.matchAll(/<url type="([^"]+)">([^<]+)<\/url>/g)) meta.urls[m[1]] = m[2].trim();
    meta.homepage = meta.urls.homepage || meta.homepage;
    meta.screenshots = [...xml.matchAll(/<image[^>]*>([^<]+)<\/image>/g)].map(m => m[1].trim());
    meta.icon = pick(/<icon[^>]*>([^<]+)<\/icon>/) || null;
  }

  let report = null;
  if (existsSync(reportPath)) {
    try { report = JSON.parse(readFileSync(reportPath, "utf8")); } catch {}
  }

  const appId = report?.app_id ?? name;
  entries.push({
    app_id: appId,
    name: meta.name || null,
    icon: meta.icon || null,
    developer: meta.developer || null,
    description: meta.description || null,
    urls: meta.urls || {},
    screenshots: meta.screenshots || [],
    summary: meta.summary,
    submitter: meta.submitter,
    source_repo: meta.source_repo,
    license: meta.license,
    homepage: meta.homepage,
    tags: meta.tags ?? [],
    source_access: meta.source_access ?? "public",
    verdict: report?.verdict ?? "published",
    certified: report?.certified ?? false,
    report_available: report !== null,
  });
}

writeFileSync(join(tmp, "catalog.json"), JSON.stringify({
  generated_at: new Date().toISOString(),
  entries: entries.sort((a, b) => a.app_id.localeCompare(b.app_id),
}, null, 2));

// Push to R2
const { R2_ENDPOINT, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY } = process.env;
if (!R2_ENDPOINT || !R2_ACCESS_KEY_ID || !R2_SECRET_ACCESS_KEY) {
  console.error("R2 env not set, skipping push");
  process.exit(0);
}
const r2 = `:s3,provider=Cloudflare,endpoint="${R2_ENDPOINT}":omapak-repo`;
execSync(`rclone copyto ${tmp}/catalog.json "${r2}/data/catalog.json" --s3-access-key-id ${R2_ACCESS_KEY_ID} --s3-secret-access-key ${R2_SECRET_ACCESS_KEY} --s3-no-check-bucket`, { stdio: "inherit" });
console.log(`catalog pushed: ${entries.length} apps`);

// Push icons to R2 (non-blocking: failures warn, never kill)
const CF_TOKEN = process.env.CLOUDFLARE_API_TOKEN;
const CF_ACCOUNT = process.env.CF_ACCOUNT || "7396d8475acc6c87ef13e97a617712f1";
const iconsDir = resolve(here, "../static/icons");

if (CF_TOKEN && existsSync(iconsDir)) {
  for (const file of readdirSync(iconsDir)) {
    if (!file.endsWith(".png") && !file.endsWith(".svg")) continue;
    try {
      const buf = readFileSync(join(iconsDir, file));
      const res = await fetch(
        `https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT}/r2/buckets/omapak-repo/objects/icons/${file}`,
        {
          method: "PUT",
          headers: { Authorization: `Bearer ${CF_TOKEN}` },
          body: buf,
        }
      );
      if (res.ok) console.log(`  icon pushed: ${file}`);
      else console.warn(`  icon push failed (${res.status}): ${file}`);
    } catch (e) {
      console.warn(`  icon push error: ${file}: ${e.message}`);
    }
  }
} else {
  console.log("icons: skipped (no CLOUDFLARE_API_TOKEN)");
}
