<script lang="ts">
  import { useCatalog } from "$lib/queries";
  import VerdictBadge from "$lib/components/VerdictBadge.svelte";

  const catalog = useCatalog();
  const entries = $derived($catalog.data?.entries ?? []);
</script>

<section class="py-14">
  <p class="font-mono text-xs uppercase tracking-[0.2em] text-ink-dim">
    the open flatpak repository
  </p>
  <h1 class="mt-3 max-w-3xl text-4xl leading-tight sm:text-5xl">
    Grade the app on its own <span class="text-accent">merits</span>.
  </h1>
  <p class="mt-5 max-w-[var(--read-width)] text-lg text-muted">
    Normal people don't give a shit how an app was built if it works well and fits their
    needs. Omapak takes that seriously. Every submission gets scored by an
    <a href="/rubric" class="text-accent underline decoration-accent/40 underline-offset-4 hover:decoration-accent">agent judge</a>
    we build in the open, every report is
    <a href="/rubric" class="text-accent underline decoration-accent/40 underline-offset-4 hover:decoration-accent">public</a>,
    and a human makes the merge call. Scored, published, never gatekept. That's
    the difference between a store and a gatekeeper.
  </p>
  <div class="mt-8 flex flex-wrap items-center gap-3">
    <a
      href="/mission"
      class="rounded-sm border border-accent-border bg-accent-soft px-4 py-2.5 font-mono text-sm text-accent transition-colors hover:bg-accent hover:text-surface"
    >
      read the mission →</a
    >
    <a
      href="/submit"
      class="rounded-sm border border-line px-4 py-2.5 font-mono text-sm text-muted transition-colors hover:border-line-strong hover:text-fg"
    >
      submit an app</a
    >
    <code
      class="rounded-sm border border-line bg-raised px-4 py-2.5 font-mono text-sm text-fg shadow-[var(--theme-shadow-1)]"
    >
      flatpak remote-add --if-not-exists omapak omapak.flatpakrepo</code
    >
  </div>
</section>

<section class="border-t border-line py-10">
  <div class="flex items-baseline justify-between">
    <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">catalog</h2>
    <span class="font-mono text-xs text-ink-dim">{entries.length} apps</span>
  </div>

  {#if $catalog.isPending}
    <p class="mt-8 font-mono text-sm text-muted">loading catalog…</p>
  {:else if $catalog.isError}
    <p class="mt-8 font-mono text-sm text-danger">catalog failed to load</p>
  {:else if entries.length === 0}
    <p class="mt-8 max-w-[var(--read-width)] text-muted">
      Nothing published yet. The queue is open. <a href="/submit" class="text-accent">Be the first</a>.
    </p>
  {:else}
    <div class="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each entries as entry (entry.app_id)}
        <a
          href="/app/{entry.app_id}"
          class="group rounded-sm border border-line bg-card p-5 shadow-[var(--theme-shadow-1)] transition-colors hover:border-line-strong hover:bg-hover"
        >
          <div class="flex items-start justify-between gap-3">
            <code class="font-mono text-sm text-fg group-hover:text-accent"
              >{entry.app_id}</code
            >
            <VerdictBadge verdict={entry.verdict} />
          </div>
          <p class="mt-2 line-clamp-2 text-sm text-muted">{entry.summary}</p>
          <div class="mt-4 flex items-center justify-between font-mono text-xs text-ink-dim">
            <span class="flex flex-wrap gap-1.5">
              {#each entry.tags.slice(0, 3) as tag}
                <span class="rounded-sm border border-line-subtle px-1.5 py-0.5">{tag}</span>
              {/each}
            </span>
            {#if entry.advisory_average !== undefined}
              <span class="text-accent">{entry.advisory_average.toFixed(1)} avg</span>
            {/if}
          </div>
        </a>
      {/each}
    </div>
  {/if}
</section>
