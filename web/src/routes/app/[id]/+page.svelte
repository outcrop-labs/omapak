<script lang="ts">
  import { page } from "$app/state";
  import { useReport, useCatalog } from "$lib/queries";
  import ScoreBar from "$lib/components/ScoreBar.svelte";
  import VerdictBadge from "$lib/components/VerdictBadge.svelte";
  import Copy from "@lucide/svelte/icons/copy";
  import Check from "@lucide/svelte/icons/check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ExternalLink from "@lucide/svelte/icons/external-link";

  const id = $derived(page.params.id ?? "");
  const report = useReport(id);
  const catalog = useCatalog();
  const entry = $derived(
    ($catalog.data?.entries ?? []).find((e) => e.app_id === id),
  );

  let copied = $state(false);
  let showReport = $state(false);

  function copyInstall() {
    navigator.clipboard.writeText(`flatpak install omapak ${id}`);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  const dims = [
    { key: "problem_clarity", label: "clarity" },
    { key: "differentiation", label: "uniqueness" },
    { key: "architecture", label: "architecture" },
    { key: "code_quality", label: "code" },
    { key: "ui_ux", label: "ui/ux" },
    { key: "packaging_hygiene", label: "packaging" },
  ];

  function score(key: string): number {
    const rubric = $report.data?.rubric as Record<string, { score: number }> | undefined;
    return rubric?.[key]?.score ?? 0;
  }
</script>

{#if $catalog.isPending}
  <div class="py-20 text-center font-mono text-sm text-muted">loading…</div>
{:else if entry}
  {@const name = entry.name || id.split(".").pop() || id}
  <div class="pb-16">
    <!-- Hero -->
    <section class="border-b border-line pb-8 pt-10">
      <div class="flex flex-wrap items-start gap-6">
        <div class="flex h-20 w-20 shrink-0 items-center justify-center rounded-xl border border-line bg-raised shadow-[var(--theme-shadow-1)]">
          <span class="font-mono text-2xl font-bold text-accent">{name[0]}</span>
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-3">
            <h1 class="text-3xl font-semibold text-fg">{name}</h1>
            {#if entry.certified}
              <span class="rounded-sm border border-accent-border bg-accent-soft px-2.5 py-1 font-mono text-xs text-accent">✓ certified</span>
            {/if}
          </div>
          <p class="mt-3 max-w-2xl text-lg text-muted">{entry.summary}</p>
          {#if entry.tags?.length}
            <div class="mt-3 flex flex-wrap gap-2">
              {#each entry.tags as tag}
                <span class="rounded-sm border border-line-subtle bg-panel px-2 py-0.5 font-mono text-xs text-muted">{tag}</span>
              {/each}
            </div>
          {/if}
        </div>
        <button
          onclick={copyInstall}
          class="flex shrink-0 items-center gap-2 rounded-sm border border-accent-border bg-accent-soft px-5 py-3 font-mono text-sm text-accent transition-colors hover:bg-accent hover:text-surface"
        >
          {#if copied}<Check size={16} /> copied{:else}<Copy size={16} /> install{/if}
        </button>
      </div>
      <div class="mt-4 flex flex-wrap gap-4 font-mono text-sm">
        {#if entry.source_repo}
          <a href={entry.source_repo} target="_blank" rel="noreferrer" class="inline-flex items-center gap-1.5 text-accent hover:underline">
            <ExternalLink size={14} /> source
          </a>
        {/if}
        {#if entry.homepage}
          <a href={entry.homepage} target="_blank" rel="noreferrer" class="inline-flex items-center gap-1.5 text-accent hover:underline">
            <ExternalLink size={14} /> website
          </a>
        {/if}
        {#if entry.license}
          <span class="text-muted">{entry.license}</span>
        {/if}
        {#if entry.source_access === "proprietary"}
          <span class="rounded-sm border border-warning/40 px-1.5 py-0.5 text-xs text-warning">proprietary</span>
        {/if}
      </div>
    </section>

    <!-- Description -->
    {#if entry.description}
      <section class="mt-8">
        <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">about</h2>
        <div class="mt-4 max-w-3xl space-y-3">
          {#each Array.isArray(entry.description) ? entry.description : [entry.description] as block}
            {#if typeof block === "string"}
              <p class="leading-relaxed text-muted">{block}</p>
            {:else if Array.isArray(block)}
              <ul class="ml-4 list-disc space-y-1 text-muted">
                {#each block as item}<li>{item}</li>{/each}
              </ul>
            {/if}
          {/each}
        </div>
      </section>
    {/if}

    <!-- Screenshots -->
    {#if entry.screenshots?.length}
      <section class="mt-8">
        <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">screenshots</h2>
        <div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2">
          {#each entry.screenshots.slice(0, 4) as src}
            <img src={src} alt={name} loading="lazy" class="rounded-sm border border-line shadow-[var(--theme-shadow-1)]" />
          {/each}
        </div>
      </section>
    {/if}

    <!-- Install -->
    <section class="mt-8 rounded-sm border border-line bg-panel p-5">
      <p class="font-mono text-sm text-fg">flatpak install omapak {id}</p>
      <p class="mt-1 font-mono text-xs text-ink-dim">from the omapak remote</p>
    </section>

    <!-- Judge Report (accordion) -->
    <section class="mt-8">
      <button
        onclick={() => (showReport = !showReport)}
        class="flex w-full items-center justify-between rounded-sm border border-line bg-card px-5 py-4 text-left transition-colors hover:border-line-strong"
      >
        <span class="font-mono text-sm text-fg">judge report</span>
        <span class="flex items-center gap-3">
          {#if $report.data?.rubric}
            {@const avg = dims.reduce((sum, d) => sum + score(d.key), 0) / dims.length}
            <span class="font-mono text-xs text-accent">{avg.toFixed(1)}/5</span>
          {/if}
          <ChevronDown size={16} class="text-muted transition-transform {showReport ? 'rotate-180' : ''}" />
        </span>
      </button>

      {#if showReport}
        <div class="mt-2 rounded-sm border border-line bg-card p-6">
          {#if $report.data?.rubric}
            <div class="grid grid-cols-3 gap-4 sm:grid-cols-6">
              {#each dims as d (d.key)}
                <div class="text-center">
                  <ScoreBar score={score(d.key)} />
                  <p class="mt-2 font-mono text-xs text-muted">{d.label}</p>
                </div>
              {/each}
            </div>

            {#if $report.data.rubric.security_flags?.length}
              <div class="mt-6">
                <p class="font-mono text-xs uppercase tracking-[0.15em] text-warning">security notes</p>
                <ul class="mt-3 space-y-2">
                  {#each $report.data.rubric.security_flags as flag}
                    <li class="text-sm text-muted">
                      <span class="mr-2 font-mono text-xs uppercase {flag.severity === 'critical' ? 'text-danger' : 'text-warning'}">{flag.severity}</span>
                      {flag.detail}
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}

            {#if $report.data.rubric.differentiation.better_alternatives?.length}
              <div class="mt-6">
                <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">similar apps</p>
                <p class="mt-2 text-sm text-muted">{$report.data.rubric.differentiation.rationale}</p>
              </div>
            {/if}

            {#if $report.data.judge}
              <p class="mt-6 font-mono text-xs text-ink-dim">
                {$report.data.judge.model} · prompt v{$report.data.judge.prompt_version}
              </p>
            {/if}
          {:else}
            <p class="text-sm text-muted">No judge report data available.</p>
          {/if}
        </div>
      {/if}
    </section>

    <a href="/" class="mt-8 inline-block font-mono text-sm text-accent hover:underline">← all apps</a>
  </div>
{:else}
  <div class="py-20 text-center">
    <h1 class="text-2xl text-fg">{id}</h1>
    <p class="mt-3 text-muted">Not found.</p>
    <a href="/" class="mt-4 inline-block font-mono text-sm text-accent hover:underline">← back</a>
  </div>
{/if}
