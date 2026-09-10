<script lang="ts">
  import { marked } from "marked";
  import { MISSION_MD } from "$lib/generated/rubric";

  const html = marked.parse(MISSION_MD, { async: false }) as string;
</script>

<svelte:head>
  <title>mission — Omapak</title>
  <meta
    name="description"
    content="Why Omapak exists: grade the app on its own merits. Scored, published, never gatekept."
  />
</svelte:head>

<section class="py-14">
  <p class="font-mono text-xs uppercase tracking-[0.2em] text-ink-dim">the mission</p>
  <div class="mission-prose mt-8">
    {@html html}
  </div>
  <div class="mt-12 flex flex-wrap items-center gap-3 border-t border-line pt-8">
    <a
      href="/submit"
      class="rounded-sm border border-accent-border bg-accent-soft px-4 py-2.5 font-mono text-sm text-accent transition-colors hover:bg-accent hover:text-surface"
    >
      submit an app →</a
    >
    <a
      href="/rubric"
      class="rounded-sm border border-line px-4 py-2.5 font-mono text-sm text-muted transition-colors hover:border-line-strong hover:text-fg"
    >
      read the rubric</a
    >
    <a
      href="https://github.com/outcrop-labs/omapak"
      class="font-mono text-sm text-muted underline decoration-accent/40 underline-offset-4 hover:text-accent"
    >
      the repo</a
    >
  </div>
</section>
