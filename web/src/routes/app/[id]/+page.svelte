<script lang="ts">
  import { page } from "$app/state";
  import { useReport, useCatalog } from "$lib/queries";
  import { advisoryAverage } from "$lib/report";
  import ScoreBar from "$lib/components/ScoreBar.svelte";
  import VerdictBadge from "$lib/components/VerdictBadge.svelte";
  import Copy from "@lucide/svelte/icons/copy";
  import Check from "@lucide/svelte/icons/check";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  

  const id = $derived(page.params.id ?? "");
  const report = useReport(id);
  const catalog = useCatalog();
  const entry = $derived(
    ($catalog.data?.entries ?? []).find((e) => e.app_id === id),
  );

  let copied = $state(false);
  function copyInstall() {
    navigator.clipboard.writeText(`flatpak install omapak ${id}`);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  const dims = [
    { key: "problem_clarity", label: "problem clarity" },
    { key: "differentiation", label: "differentiation" },
    { key: "architecture", label: "architecture" },
    { key: "code_quality", label: "code quality" },
    { key: "ui_ux", label: "ui/ux" },
    { key: "packaging_hygiene", label: "packaging" },
  ];

  function score(key: string): number | null {
    const rubric = $report.data?.rubric as Record<string, { score: number }> | undefined;
    return rubric?.[key]?.score ?? null;
  }
</script>

{#if $report.isPending && $catalog.isPending}
  <div class="py-20 text-center font-mono text-sm text-muted">loading…</div>
{:else if entry || $report.data}
  {@const r = $report.data}
  {@const name = entry?.name || id.split(".").pop() || id}
  <div class="pb-16">
    <!-- Hero -->
    <section class="border-b border-line pb-8 pt-10">
      <div class="flex flex-wrap items-start gap-6">
        <div class="flex h-20 w-20 shrink-0 items-center justify-center rounded-xl border border-line bg-raised shadow-[var(--theme-shadow-1)]">
          {#if entry?.icon && entry.icon.startsWith("http")}
            <img src={entry.icon} alt={name} class="h-14 w-14 rounded-lg" />
          {:else}
            <span class="font-mono text-2xl font-bold text-accent">{name[0]}</span>
          {/if}
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-3">
            <h1 class="text-3xl font-semibold text-fg">{name}</h1>
            {#if entry?.certified}
              <span class="rounded-sm border border-accent-border bg-accent-soft px-2.5 py-1 font-mono text-xs text-accent">✓ certified</span>
            {/if}
          </div>
          <p class="mt-1 font-mono text-sm text-ink-dim">{id}</p>
          <p class="mt-3 max-w-2xl text-lg text-muted">{entry?.summary || "No summary"}</p>
          <div class="mt-4 flex flex-wrap gap-4 font-mono text-sm">
            {#if entry?.source_repo}
              <a href={entry.source_repo} target="_blank" rel="noreferrer" class="inline-flex items-center gap-1.5 text-accent hover:underline">
                <ExternalLink size={15} /> source
              </a>
            {/if}
            {#if entry?.homepage}
              <a href={entry.homepage} target="_blank" rel="noreferrer" class="inline-flex items-center gap-1.5 text-accent hover:underline">
                <ExternalLink size={15} /> website
              </a>
            {/if}
            {#if entry?.license}
              <span class="text-muted">{entry.license}</span>
            {/if}
            {#if entry?.source_access === "proprietary"}
              <span class="rounded-sm border border-warning/40 px-1.5 py-0.5 text-xs text-warning">proprietary</span>
            {/if}
          </div>
        </div>
      </div>

      <!-- Install -->
      <div class="mt-6 flex flex-wrap items-center gap-3">
        <button
          onclick={copyInstall}
          class="group flex items-center gap-3 rounded-sm border border-accent-border bg-accent-soft px-5 py-3 font-mono text-sm text-accent transition-colors hover:bg-accent hover:text-surface"
        >
          {#if copied}
            <Check size={16} /> copied!
          {:else}
            <Copy size={16} /> flatpak install omapak {id}
          {/if}
        </button>
        <span class="font-mono text-xs text-ink-dim">one remote gets you everything</span>
      </div>
    </section>

    <!-- Judge Report -->
    {#if r?.rubric}
      <section class="mt-8">
        <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">judge report</h2>
        <div class="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
          {#each dims as d (d.key)}
            {@const s = score(d.key)}
            <div class="rounded-sm border border-line bg-card p-4 text-center">
              <ScoreBar score={s ?? 0} />
              <p class="mt-2 font-mono text-xs text-muted">{d.label}</p>
            </div>
          {/each}
        </div>

        {#if r.rubric.differentiation.better_alternatives?.length}
          <div class="mt-6 rounded-sm border border-line bg-panel p-5">
            <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">existing solutions (informational)</p>
            <p class="mt-2 text-sm text-muted">{r.rubric.differentiation.rationale}</p>
          </div>
        {/if}

        {#if r.rubric.security_flags?.length}
          <div class="mt-4 rounded-sm border border-warning/40 bg-card p-5">
            <p class="font-mono text-xs uppercase tracking-[0.15em] text-warning">security notes</p>
            <ul class="mt-3 space-y-2">
              {#each r.rubric.security_flags as flag}
                <li class="text-sm text-muted">
                  <span class="mr-2 font-mono text-xs uppercase {flag.severity === 'critical' ? 'text-danger' : 'text-warning'}">{flag.severity}</span>
                  {flag.detail}
                </li>
              {/each}
            </ul>
          </div>
        {/if}

        {#if r.judge}
          <p class="mt-6 font-mono text-xs text-ink-dim">
            scored by {r.judge.model} · prompt v{r.judge.prompt_version} · {r.judge.duration_secs}s
          </p>
        {/if}
      </section>
    {:else}
      <section class="mt-8 rounded-sm border border-line bg-panel p-6 text-center">
        <p class="font-mono text-sm text-muted">judge report pending — the pipeline runs on merge</p>
      </section>
    {/if}

    <!-- Back -->
    <a href="/" class="mt-8 inline-block font-mono text-sm text-accent hover:underline">← all apps</a>
  </div>
{:else}
  <div class="py-20 text-center">
    <h1 class="text-2xl text-fg">{id}</h1>
    <p class="mt-3 text-muted">This app isn't on omapak (yet).</p>
    <a href="/" class="mt-4 inline-block font-mono text-sm text-accent hover:underline">← back</a>
  </div>
{/if}
