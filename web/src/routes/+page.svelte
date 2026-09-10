<script lang="ts">
  import { useCatalog, useFlathub } from "$lib/queries";
  import VerdictBadge from "$lib/components/VerdictBadge.svelte";

  const catalog = useCatalog();
  const flathub = useFlathub();
  const omapakEntries = $derived($catalog.data?.entries ?? []);
  const hosted = $derived(new Set(omapakEntries.map((e) => e.app_id)));

  let filter = $state<"all" | "omapak" | "flathub">("all");
  let search = $state("");

  const flathubEntries = $derived(
    ($flathub.data?.apps ?? []).filter((a) => !hosted.has(a.app_id)),
  );
  const flathubFiltered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return flathubEntries.filter(
      (a) =>
        !q ||
        a.app_id.toLowerCase().includes(q) ||
        a.name.toLowerCase().includes(q) ||
        a.summary.toLowerCase().includes(q),
    );
  });
  const omapakFiltered = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return omapakEntries.filter(
      (e) => !q || e.app_id.toLowerCase().includes(q) || e.summary.toLowerCase().includes(q),
    );
  });
  const total = $derived(omapakEntries.length + flathubEntries.length);
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
      curl -O https://repo.omapak.org/omapak.flatpakrepo &&<br />
      flatpak remote-add --if-not-exists omapak omapak.flatpakrepo</code
    >
  </div>
</section>

<section class="border-t border-line py-10">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">catalog</h2>
    <div class="flex items-center gap-3">
      <input
        type="search"
        placeholder="search {total} apps…"
        bind:value={search}
        class="w-56 rounded-sm border border-line bg-input px-3 py-1.5 font-mono text-sm text-fg placeholder:text-ink-dim focus:border-line-strong focus:outline-none"
      />
      <div class="flex overflow-hidden rounded-sm border border-line font-mono text-xs">
        {#each ["all", "omapak", "flathub"] as f (f)}
          <button
            onclick={() => (filter = f as typeof filter)}
            class="px-3 py-1.5 transition-colors {filter === f
              ? 'bg-accent-soft text-accent'
              : 'text-muted hover:text-fg'}"
          >
            {f}
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if $catalog.isPending}
    <p class="mt-8 font-mono text-sm text-muted">loading catalog…</p>
  {:else if $catalog.isError}
    <p class="mt-8 font-mono text-sm text-danger">catalog failed to load</p>
  {:else if filter !== "flathub" && omapakFiltered.length === 0}
    <p class="mt-8 max-w-[var(--read-width)] text-muted">
      Nothing hosted here yet{search ? " matches that search" : ""}. The queue is open.
      <a href="/submit" class="text-accent">Be the first</a>.
    </p>
  {/if}

  {#if filter !== "flathub" && omapakFiltered.length > 0}
    <p class="mt-8 font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
      omapak hosted · judged
    </p>
    <div class="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each omapakFiltered as entry (entry.app_id)}
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
              {#if entry.source_access === "proprietary"}
                <span
                  class="rounded-sm border border-warning/40 px-1.5 py-0.5 text-warning"
                  title="Closed source, distributed with owner permission; code not audited"
                  >proprietary</span
                >
              {/if}
            </span>
            {#if entry.advisory_average !== undefined}
              <span class="text-accent">{entry.advisory_average.toFixed(1)} avg</span>
            {/if}
          </div>
        </a>
      {/each}
    </div>
  {/if}

  {#if filter !== "omapak"}
    <p class="mt-10 font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
      flathub managed · pass-through, not mirrored
    </p>
    {#if $flathub.isPending}
      <p class="mt-4 font-mono text-sm text-muted">loading flathub catalog…</p>
    {:else if $flathub.isError}
      <p class="mt-4 font-mono text-xs text-ink-dim">
        flathub catalog unavailable right now; omapak-hosted apps are unaffected.
      </p>
    {:else if flathubFiltered.length === 0}
      <p class="mt-4 font-mono text-sm text-muted">no matches.</p>
    {:else}
      <div class="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {#each flathubFiltered.slice(0, 60) as app (app.app_id)}
          <a
            href="https://flathub.org/apps/{app.app_id}"
            class="group flex items-center gap-3 rounded-sm border border-line-subtle bg-panel p-3 transition-colors hover:border-line hover:bg-hover"
          >
            {#if app.icon}
              <img src={app.icon} alt="" loading="lazy" class="h-9 w-9 shrink-0 rounded-sm" />
            {/if}
            <span class="min-w-0">
              <span class="block truncate font-mono text-xs text-fg group-hover:text-accent"
                >{app.name}</span
              >
              <span class="block truncate text-xs text-muted">{app.summary}</span>
            </span>
            <span
              class="ml-auto shrink-0 rounded-sm border border-line-strong px-1.5 py-0.5 font-mono text-[10px] text-muted"
              >flathub</span
            >
          </a>
        {/each}
      </div>
      <p class="mt-4 font-mono text-xs text-ink-dim">
        showing 60 of {flathubFiltered.length}. these install from the flathub remote
        (<code class="text-fg">flatpak install flathub &lt;app-id&gt;</code>); omapak never
        mirrors or proxies their bytes. apps we host ourselves always win the ID collision.
      </p>
    {/if}
  {/if}
</section>
