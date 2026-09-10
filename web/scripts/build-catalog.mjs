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

const entries = [];
mkdirSync(resolve(here, "../static/data/reports"), { recursive: true });

for (const { dir, published } of sources) {
  if (!existsSync(dir)) continue;
  for (const name of readdirSync(dir)) {
    const appDir = join(dir, name);
    const metaPath = join(appDir, "metadata.yml");
    const reportPath = join(appDir, "report.json");
    if (!existsSync(metaPath)) continue;

    let meta;
    try {
      meta = parseYaml(readFileSync(metaPath, "utf8"));
      // Extract nice name from the metainfo XML
      const metainfoPath = join(appDir, `${appId}.metainfo.xml`);
      if (existsSync(metainfoPath)) {
        const xml = readFileSync(metainfoPath, "utf8");
        const nameMatch = xml.match(/<name>([^<]+)<\/name>/);
        if (nameMatch) meta.name = nameMatch[1].trim();
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
      summary: meta.summary,
      description: meta.description,
      submitter: meta.submitter,
      source_repo: meta.source_repo,
      license: meta.license,
      homepage: meta.homepage,
      tags: meta.tags ?? [],
      source_access: meta.source_access ?? "public",
      verdict: report?.verdict ?? "unpublished",
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
