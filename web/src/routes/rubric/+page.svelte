<script lang="ts">
  import { JUDGE_PROMPT, PROMPT_VERSION } from "$lib/generated/rubric";

  const dims = [
    {
      name: "problem clarity",
      gates: false,
      text: "Is it obvious what problem this solves and for whom? Common problems are fine — a to-do app can score 5. Incoherent ones cannot.",
    },
    {
      name: "differentiation",
      gates: false,
      text: "Does this exist already, better? The judge names the alternatives — as information for the reviewer and the user. This dimension NEVER gates. Clones are allowed; hiding that they're clones is not.",
    },
    {
      name: "architecture",
      gates: false,
      text: "Sensible structure, state handling, error paths, no gratuitous dependencies — judged at the app's scale.",
    },
    {
      name: "code quality",
      gates: false,
      text: "Readability, consistency, dead code, error handling. Style differences don't score. And 'built with AI' is not a signal — that's the entire reason this repo exists. We judge slop, and slop is judgeable on its own.",
    },
    {
      name: "ui / ux",
      gates: false,
      text: "From screenshots when the dynamic stage runs, else from appstream metadata and command structure. Usable, labeled, respects the desktop.",
    },
    {
      name: "packaging hygiene",
      gates: true,
      text: "Manifest sanity, pinned sources, runtime fit, sane finish-args, truthful appstream. Below 2/5 is a hard reject — come back when it installs clean.",
    },
    {
      name: "security flags",
      gates: true,
      text: "Obfuscated payloads, mystery endpoints, undisclosed telemetry, miner-shaped code, harvesting beyond the stated purpose. Hard pass, pound sand. An empty list is the common case — the judge is told not to invent flags to seem thorough.",
    },
  ];
</script>

<svelte:head>
  <title>rubric — omapak</title>
</svelte:head>

<section class="py-14">
  <p class="font-mono text-xs uppercase tracking-[0.2em] text-ink-dim">the rubric · v{PROMPT_VERSION}</p>
  <h1 class="mt-3 max-w-3xl text-4xl leading-tight">
    Every submission graded in the open.
  </h1>
  <p class="mt-5 max-w-[var(--read-width)] text-lg text-muted">
    This is the exact prompt sent with every submission — published, versioned, in the repo.
    The gates are honest and boring: it builds, it's not hostile to the user, the packaging
    isn't a mess. Everything else is scored, published, and never gatekept. No vibes, no
    overzealous mods denying on a whim, everything written down. A human makes the call.
  </p>
</section>

<section class="border-t border-line py-10">
  <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">dimensions</h2>
  <div class="mt-6 grid grid-cols-1 gap-4 md:grid-cols-2">
    {#each dims as d (d.name)}
      <div class="rounded-sm border border-line bg-card p-5">
        <div class="flex items-center justify-between">
          <h3 class="font-mono text-sm text-fg">{d.name}</h3>
          <span
            class="rounded-sm border px-2 py-0.5 font-mono text-xs
              {d.gates ? 'border-danger/40 text-danger' : 'border-line-strong text-muted'}"
          >
            {d.gates ? "gate" : "advisory"}
          </span>
        </div>
        <p class="mt-3 text-sm text-muted">{d.text}</p>
      </div>
    {/each}
  </div>
</section>

<section class="border-t border-line py-10">
  <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">
    the judge prompt · source of truth: crates/omapak-judge/src/prompt.rs
  </h2>
  <pre
    class="mt-6 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-5 font-mono text-xs leading-relaxed text-muted whitespace-pre-wrap">{JUDGE_PROMPT}</pre>
</section>
