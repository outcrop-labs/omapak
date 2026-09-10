<script lang="ts">
  import { useReport } from "$lib/queries";
  import { advisoryAverage, type RubricScore } from "$lib/report";
  import ScoreBar from "$lib/components/ScoreBar.svelte";
  import VerdictBadge from "$lib/components/VerdictBadge.svelte";

  let { appId }: { appId: string } = $props();

  // Remounted by the parent's {#key id}, so the initial value is the value.
  // svelte-ignore state_referenced_locally
  const report = useReport(appId);

  const dims: { key: string; label: string; gate?: string }[] = [
    { key: "problem_clarity", label: "problem clarity" },
    { key: "differentiation", label: "differentiation", gate: "advisory, never gates" },
    { key: "architecture", label: "architecture" },
    { key: "code_quality", label: "code quality" },
    { key: "ui_ux", label: "ui / ux" },
    { key: "packaging_hygiene", label: "packaging hygiene", gate: "gate: ≥ 2" },
  ];

  function dim(rubric: Record<string, unknown>, key: string): RubricScore {
    return rubric[key] as RubricScore;
  }
</script>

{#if $report.isPending}
  <p class="py-16 font-mono text-sm text-muted">loading report…</p>
{:else if $report.isError}
  <div class="py-16">
    <h1 class="font-mono text-xl text-fg">{appId}</h1>
    <p class="mt-3 text-muted">No judge report published for this app (yet).</p>
  </div>
{:else if $report.data}
  {@const r = $report.data}
  <section class="py-10">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="font-mono text-2xl text-fg">{r.app_id}</h1>
        <p class="mt-1 font-mono text-xs text-ink-dim">
          judged {new Date(r.created_at).toISOString().slice(0, 10)}
          {#if r.judge}· {r.judge.model} · prompt v{r.judge.prompt_version}{/if}
        </p>
      </div>
      <VerdictBadge verdict={r.verdict} />
    </div>

    {#if !r.build.ok}
      <div
        class="mt-6 rounded-sm border border-danger/40 bg-danger/10 p-4 font-mono text-sm text-danger"
      >
        packaging gate FAILED. This build does not install. Scores below are context for
        fixing it.
      </div>
    {/if}

    {#if r.rubric}
      <div class="mt-8 overflow-x-auto rounded-sm border border-line bg-card">
        <table class="w-full text-sm">
          <tbody>
            {#each dims as d (d.key)}
              {@const s = dim(r.rubric as unknown as Record<string, unknown>, d.key)}
              <tr class="border-b border-line-subtle last:border-0">
                <td class="w-56 px-5 py-4 align-top font-mono text-xs text-muted">
                  {d.label}
                  {#if d.gate}<span class="mt-1 block text-ink-dim">{d.gate}</span>{/if}
                </td>
                <td class="w-40 px-5 py-4 align-top"><ScoreBar score={s.score} /></td>
                <td class="px-5 py-4 text-muted">{s.rationale}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      {#if r.rubric.differentiation.better_alternatives?.length}
        <div class="mt-6 rounded-sm border border-line bg-panel p-5">
          <h2 class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
            better existing solutions: informational, never gates
          </h2>
          <ul class="mt-3 space-y-1.5">
            {#each r.rubric.differentiation.better_alternatives as alt}
              <li class="text-sm text-muted">· {alt}</li>
            {/each}
          </ul>
        </div>
      {/if}

      {#if r.rubric.security_flags?.length}
        <div class="mt-6 rounded-sm border border-warning/40 bg-warning/5 p-5">
          <h2 class="font-mono text-xs uppercase tracking-[0.15em] text-warning">
            security flags
          </h2>
          <ul class="mt-3 space-y-1.5">
            {#each r.rubric.security_flags as flag}
              <li class="text-sm text-muted"
                ><span
                  class="mr-2 font-mono text-xs uppercase {flag.severity === 'critical'
                    ? 'text-danger'
                    : 'text-warning'}">{flag.severity}</span
                >{flag.detail}</li
              >
            {/each}
          </ul>
        </div>
      {/if}
    {:else}
      <p class="mt-6 font-mono text-sm text-muted">No rubric for this report. Gates only.</p>
    {/if}

    {#if r.static.advisories?.length}
      <div class="mt-6 rounded-sm border border-line bg-panel p-5">
        <h2 class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
          manifest advisories
        </h2>
        <ul class="mt-3 space-y-1.5">
          {#each r.static.advisories as a}
            <li class="text-sm text-muted"
              ><code class="mr-2 font-mono text-xs text-muted">{a.kind}</code>{a.detail}</li
            >
          {/each}
        </ul>
      </div>
    {/if}

    <p class="mt-8 font-mono text-xs text-ink-dim">
      build {r.build.ok ? "passed" : "failed"} in {r.build.duration_secs}s
      {#if r.rubric}· advisory average {advisoryAverage(r.rubric).toFixed(1)}{/if}
      · merge is a human decision
    </p>
    <a href="/" class="mt-4 inline-block font-mono text-sm text-accent hover:underline"
      >← catalog</a
    >
  </section>
{/if}
