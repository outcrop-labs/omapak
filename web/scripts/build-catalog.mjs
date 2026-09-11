#!/usr/bin/env node
// Builds static/data/catalog.json + per-app report JSON from apps/ (accepted,
// published) and optionally fixtures/ (for local dev/demo).
import { existsSync, mkdirSync, readFileSync, writeFileSync, copyFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../..");
const withFixtures = process.argv.includes("--with-fixtures");

const sources = [
  { dir: join(root, "apps"), published: true },
  ...(withFixtures ? [{ dir: join(root, "fixtures"), published: false }] : []),
];


// Resolve an icon URL for an app: local file in the app dir → copied
// to static/icons/, else GitHub owner avatar → fetched and cached.
async function getIcon(appId, sourceRepo, appDir) {
  const iconsDir = join(here, "../static/icons");
  mkdirSync(iconsDir, { recursive: true });
  const dest = join(iconsDir, `${appId}.png`);

  // Already cached
  if (existsSync(dest)) return `https://repo.omapak.org/icons/${appId}.png`;

  // Look for icon files in the app dir
  const candidates = [
    join(appDir, `${appId}.png`),
    join(appDir, `${appId}.svg`),
    join(appDir, "icon.png"),
    join(appDir, "icon.svg"),
    ...["512", "256", "128", "64"].map(s => join(appDir, "icons", `icon-${s}.png`)),
  ];
  for (const c of candidates) {
    if (existsSync(c)) {
      copyFileSync(c, dest);
      return `https://repo.omapak.org/icons/${appId}.png`;
    }
  }

  // Fallback: GitHub owner avatar
  const owner = (sourceRepo || "").replace(/.*github\.com\//, "").split("/")[0];
  if (owner) {
    try {
      const res = await fetch(`https://github.com/${owner}.png?size=128`);
      if (res.ok) {
        const buf = Buffer.from(await res.arrayBuffer());
        writeFileSync(dest, buf);
        return `https://repo.omapak.org/icons/${appId}.png`;
      }
    } catch {}
  }

  return null;
}

const entries = [];
mkdirSync(resolve(here, "../static/data/reports"), { recursive: true });

for (const { dir, published } of sources) {
  if (!existsSync(dir)) continue;
  for (const name of readdirSync(dir)) {
    // (async handled via await in getIcon)
    const appDir = join(dir, name);
    const metaPath = join(appDir, "metadata.yml");
    const reportPath = join(appDir, "report.json");
    if (!existsSync(metaPath)) continue;

    let meta;
    try {
      meta = parseYaml(readFileSync(metaPath, "utf8"));
      // Parse metainfo XML for real store data (description, dev, urls, screenshots)
      const appDirFiles = readdirSync(appDir);
      const xmlFile = appDirFiles.find(f => f.endsWith(".metainfo.xml") || f.endsWith(".appdata.xml"));
      if (xmlFile) {
        const xml = readFileSync(join(appDir, xmlFile), "utf8");
        const pick = (re) => { const m = xml.match(re); return m ? m[1].trim() : null; };
        meta.name = pick(/<name>([^<]+)<\/name>/) || meta.name;
        meta.summary = pick(/<summary>([^<]+)<\/summary>/) || meta.summary;
        meta.developer = pick(/<developer[^>]*>[\s\S]*?<name>([^<]+)<\/name>/) || null;
        // Full description: paragraphs + lists as structured data
        const descMatch = xml.match(/<description>([\s\S]*?)<\/description>/);
        if (descMatch) {
          const raw = descMatch[1];
          const blocks = [];
          const parts = raw.split(/(<\/?(?:p|ul|li)>)/);
          let list = null;
          let para = null;
          for (const part of parts) {
            const t = part.trim();
            if (t === "<p>") { para = ""; }
            else if (t === "</p>") { if (para) blocks.push(para); para = null; }
            else if (t === "<ul>") { list = []; }
            else if (t === "</ul>") { if (list) blocks.push(list); list = null; }
            else if (t === "<li>") { if (list) list.push(""); }
            else if (t === "</li>") { continue; }
            else if (t && !t.startsWith("<")) {
              const text = t.replace(/<[^>]+>/g, "").trim();
              if (!text) continue;
              if (list) list[list.length - 1] = text;
              else if (para !== null) para = text;
            }
          }
          meta.description = blocks.length ? blocks : meta.description;
        }
        // URLs
        meta.urls = {};
        for (const m of xml.matchAll(/<url type="([^"]+)">([^<]+)<\/url>/g)) meta.urls[m[1]] = m[2].trim();
        meta.homepage = meta.urls.homepage || meta.homepage;
        meta.bugtracker = meta.urls.bugtracker || null;
        // Screenshots
        meta.screenshots = [...xml.matchAll(/<image[^>]*>([^<]+)<\/image>/g)].map(m => m[1].trim());
        meta.icon = pick(/<icon[^>]*>([^<]+)<\/icon>/) || null;
      }
    } catch (e) {
      console.warn(`! ${name}: metadata.yml unreadable: ${e.message}`);
      continue;
    }
    if (!meta?.submitter || !meta?.source_repo || !meta?.summary) {
      console.warn(`! ${name}: metadata.yml missing submitter/source_repo/summary`);
      continue;
    }

    let report = null;
    if (existsSync(reportPath)) {
      try {
        report = JSON.parse(readFileSync(reportPath, "utf8"));
      } catch (e) {
        console.warn(`! ${name}: report.json unreadable: ${e.message}`);
      }
    }

    const appId = report?.app_id ?? name;
    const advisory = report?.rubric
      ? (
          [
            report.rubric.problem_clarity.score,
            report.rubric.differentiation.score,
            report.rubric.architecture.score,
            report.rubric.code_quality.score,
            report.rubric.ui_ux.score,
          ].reduce((a, b) => a + b, 0) / 5
        ).toFixed(1)
      : undefined;

    entries.push({
      app_id: appId,
      name: meta.name || null,
      icon: await getIcon(appId, meta.source_repo, appDir),
      developer: meta.developer || null,
      description: meta.description || null,
      urls: meta.urls || {},
      bugtracker: meta.bugtracker || null,
      help: meta.help || null,
      screenshots: meta.screenshots || [],
      summary: meta.summary,
      submitter: meta.submitter,
      source_repo: meta.source_repo,
      license: meta.license,
      homepage: meta.homepage,
      tags: meta.tags ?? [],
      source_access: meta.source_access ?? "public",
      verdict: report?.verdict ?? "published",  // being on main = merged = published
      certified: report?.certified ?? false,
      advisory_average: advisory ? Number(advisory) : undefined,
      last_commit_date: report?.static?.source_stats?.last_commit_date,
      report_available: report !== null,
    });

    if (report) {
      copyFileSync(reportPath, resolve(here, `../static/data/reports/${appId}.json`));
    }
  }
}

const catalog = {
  generated_at: new Date().toISOString(),
  entries: entries.sort((a, b) => a.app_id.localeCompare(b.app_id)),
};

writeFileSync(resolve(here, "../static/data/catalog.json"), JSON.stringify(catalog, null, 2));
console.log(`catalog: ${catalog.entries.length} entries (${withFixtures ? "with fixtures" : "apps only"})`);
