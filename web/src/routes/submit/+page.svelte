<svelte:head>
  <title>submit — omapak</title>
</svelte:head>

<section class="py-14">
  <p class="font-mono text-xs uppercase tracking-[0.2em] text-ink-dim">submit an app</p>
  <h1 class="mt-3 max-w-3xl text-4xl leading-tight">Open a PR. Get graded. Merge is human.</h1>
  <p class="mt-5 max-w-[var(--read-width)] text-lg text-muted">
    No account on our side, no attestation about who wrote your code. If it builds, installs,
    and doesn't trip the security gate, a human looks at the judge report and decides.
  </p>
</section>

<section class="border-t border-line py-10">
  <h2 class="font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">how it works</h2>
  <ol class="mt-6 max-w-[var(--read-width)] space-y-4 text-muted">
    <li>
      <span class="font-mono text-accent">01</span>
      Fork <code class="font-mono text-sm text-fg">outcrop-labs/omapak</code> and create
      <code class="font-mono text-sm text-fg">apps/&lt;your.app-id&gt;/</code> with your
      flatpak-builder manifest and a
      <code class="font-mono text-sm text-fg">metadata.yml</code> (submitter, source_repo,
      summary, license).
    </li>
    <li>
      <span class="font-mono text-accent">02</span>
      Open a PR touching <code class="font-mono text-sm text-fg">apps/</code>. CI runs the
      judge — static checks, a real flatpak build, then the LLM rubric — and posts the full
      report as a PR comment.
    </li>
    <li>
      <span class="font-mono text-accent">03</span>
      A maintainer reads the report and merges (or doesn't — with reasons, in public). Merge
      publishes the app to the omapak repo and its judge report to this site.
    </li>
  </ol>

  <h2 class="mt-12 font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">
    metadata.yml
  </h2>
  <pre
    class="mt-4 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-5 font-mono text-xs leading-relaxed text-muted">submitter: your-handle
source_repo: https://github.com/you/your-app
summary: One line, plain words
license: MIT
tags: [utility, gnome]</pre>

  <h2 class="mt-12 font-mono text-sm uppercase tracking-[0.15em] text-ink-dim">maintenance</h2>
  <p class="mt-4 max-w-[var(--read-width)] text-muted">
    Published apps must be actively maintained. If an app goes quiet past the community-decided
    window, it's archived to <code class="font-mono text-sm text-fg">omapak-unmaintained</code>
    — still installable, clearly labeled. If it stays quiet after that, it drops from the repo
    and the servers. The window is TBD by community consensus, and when it's set it will be
    published here with the rest of the rules.
  </p>
</section>
