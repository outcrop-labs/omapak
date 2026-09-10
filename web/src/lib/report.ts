// Mirrors crates/omapak-core/src/schema.rs — REPORT_SCHEMA_VERSION 1.
// Drift here is caught by the catalog build script, which validates each
// report against these types at build time.

export type Verdict = "published" | "build_failed";

export type Severity = "info" | "warning" | "critical";

export interface RubricScore {
  score: number;
  rationale: string;
}

export interface Differentiation extends RubricScore {
  better_alternatives?: string[];
}

export interface SecurityFlag {
  severity: Severity;
  detail: string;
}

export interface Rubric {
  problem_clarity: RubricScore;
  differentiation: Differentiation;
  architecture: RubricScore;
  code_quality: RubricScore;
  ui_ux: RubricScore;
  packaging_hygiene: RubricScore;
  security_flags?: SecurityFlag[];
}

export interface LinterRun {
  tool: string;
  status: "pass" | "failed" | "not_found" | "skipped";
  findings?: string[];
}

export interface StaticReport {
  manifest?: {
    app_id: string;
    runtime?: string;
    sdk?: string;
    finish_args?: string[];
    modules?: { name: string; sources?: { url?: string; pinned?: string }[] }[];
    command?: string;
  };
  metadata_present: boolean;
  appstream_present: boolean;
  linters?: LinterRun[];
  advisories?: { kind: string; detail: string }[];
  source_stats?: {
    files: number;
    bytes: number;
    largest: { path: string; bytes: number }[];
    commit_count?: number;
    last_commit_date?: string;
  };
}

export interface Report {
  schema_version: number;
  app_id: string;
  created_at: string;
  static: StaticReport;
  build: { ok: boolean; duration_secs: number; log_tail?: string[] };
  dynamic?: { launched: boolean; screenshots: string[]; log_tail?: string[] };
  rubric?: Rubric;
  verdict: Verdict;
  judge?: { model: string; base_url: string; prompt_version: string; duration_secs: number };
  legitimacy?: {
    model: string;
    summary: string;
    confidence: number;
    findings: { severity: "info" | "warning" | "critical"; detail: string; source?: string }[];
  };
}

export type SourceAccess = "public" | "proprietary";

export interface CatalogEntry {
  app_id: string;
  name?: string;
  summary: string;
  description?: string;
  submitter: string;
  source_repo: string;
  source_access?: SourceAccess;
  license?: string;
  homepage?: string;
  tags: string[];
  verdict: Verdict | "unpublished";
  certified: boolean;
  advisory_average?: number;
  last_commit_date?: string;
  report_available: boolean;
}

export interface Catalog {
  generated_at: string;
  entries: CatalogEntry[];
}

// Flathub catalog entries, served through the omapak caching proxy.
export interface FlathubEntry {
  app_id: string;
  name: string;
  summary: string;
  icon: string | null;
  license: string | null;
}

export interface FlathubIndex {
  generated_at: string;
  apps: FlathubEntry[];
}

export const VERDICT_LABEL: Record<Verdict, string> = {
  published: "published",
  build_failed: "build failed",
};

export function advisoryAverage(rubric: Rubric): number {
  const dims = [
    rubric.problem_clarity.score,
    rubric.differentiation.score,
    rubric.architecture.score,
    rubric.code_quality.score,
    rubric.ui_ux.score,
  ];
  return dims.reduce((a, b) => a + b, 0) / dims.length;
}
