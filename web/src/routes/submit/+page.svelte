<svelte:head>
  <title>how to submit · Omapak</title>
  <meta
    name="description"
    content="Submit an app to Omapak: fork, add your manifest and metadata.yml, open a PR. The agent judge grades it, a human merges. No attestation about who wrote your code."
  />
</svelte:head>

<section class="py-14">
  <p class="font-mono text-xs uppercase tracking-[0.2em] text-ink-dim">how to submit</p>
  <h1 class="mt-3 max-w-3xl text-4xl leading-tight">Open a PR. Get graded. Merge is human.</h1>
  <p class="mt-5 max-w-[var(--read-width)] text-lg text-muted">
    No account on our side, no attestation about who or what wrote your code. That question
    doesn't get asked here. If it builds, installs, and isn't hostile to the user, a human
    reads the judge report and makes the call. Here's the whole tutorial; it's short.
  </p>
</section>

<section class="border-t border-line py-10">
  <div class="space-y-12">
    <div>
      <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
        step 0 · what you need
      </p>
      <p class="mt-3 max-w-[var(--read-width)] text-muted">
        A flatpak-builder manifest that actually builds your app. If you've got one that works
        with <code class="font-mono text-sm text-fg">flatpak-builder</code> locally, you're
        90% done. If you're starting from scratch, the
        <a
          href="https://docs.flatpak.org/"
          class="text-accent underline decoration-accent/40 underline-offset-4 hover:decoration-accent"
          >flatpak docs</a
        >
        walk through it.
      </p>
    </div>

    <div>
      <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">step 1 · branch and PR</p>
      <p class="mt-3 max-w-[var(--read-width)] text-muted">
        Collaborator? Push a branch straight to
        <code class="font-mono text-sm text-fg">outcrop-labs/omapak</code> and open the PR.
        Everyone else: fork first. No account on our side either way, and the fork path
        isn't busywork; it's why submitters never need write access to anything of ours.
      </p>
      <pre
        class="mt-3 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-4 font-mono text-xs leading-relaxed text-muted">git checkout -b <span class="text-fg">add-my-app</span>   <span class="text-ink-dim"># in-repo if you're a collaborator</span>
<span class="text-ink-dim"># or, from your fork:</span>
git clone git@github.com:<span class="text-fg">your-handle</span>/omapak.git && cd omapak
git checkout -b <span class="text-fg">add-my-app</span></pre>
    </div>

    <div>
      <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
        step 2 · add your app
      </p>
      <p class="mt-3 max-w-[var(--read-width)] text-muted">
        Create <code class="font-mono text-sm text-fg">apps/&lt;your.app-id&gt;/</code> with
        two files: your manifest (named after the app id, flathub-style) and a
        <code class="font-mono text-sm text-fg">metadata.yml</code>. That's the entire
        submission surface:
      </p>
      <pre
        class="mt-4 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-4 font-mono text-xs leading-relaxed text-muted">apps/com.yourname.yourapp/
├── com.yourname.yourapp.json   ← your flatpak-builder manifest
├── com.yourname.yourapp.metainfo.xml   ← optional but the judge likes it
└── metadata.yml</pre>
      <p class="mt-4 max-w-[var(--read-width)] text-muted">metadata.yml, every field:</p>
      <pre
        class="mt-3 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-4 font-mono text-xs leading-relaxed text-muted">submitter: your-handle
source_repo: https://github.com/you/your-app
summary: One line, plain words
license: MIT
tags: [utility, gnome]</pre>
      <p class="mt-4 max-w-[var(--read-width)] text-muted">
        Note what's <strong class="text-fg">not</strong> in there: any question about who or
        what wrote the code. We don't ask. Don't tell us; we don't care.
      </p>
      <div class="mt-6 max-w-[var(--read-width)] rounded-sm border border-warning/40 bg-card p-5">
        <p class="font-mono t          <p class="font-mono text-xs uppercase tracking-[0.15em] text-warning">
          private repo or closed source?
        </p>
        <p class="mt-3 text-sm text-muted">
          Allowed, Flathub-style. Set
          <code class="font-mono text-sm text-fg">source_access: proprietary</code> in
          metadata.yml and pin your manifest to your own release assets with sha256
          checksums. The judge grades packaging and provenance (it can't read the code,
          and it will say so), and the catalog shows a
          <span class="text-warning">proprietary</span> badge so users know what they're
          installing. Third-party submissions of someone else's closed app need the
          owner's okay on an omapak notification issue before merge; if it's your app,
          you ARE the okay.
        </p>
        <p class="mt-3 text-sm text-muted">
          Optionally: if you'd rather have the code actually read, grant a maintainer a
          scoped read-only token and we'll run the judge against the real source, publish
          only the verdict, and delete the token. Nice offer, never a requirement. Same
          as Flathub, minus the gatekeeping theater.
        </p>
      </div>
      <div class="mt-6 max-w-[var(--read-width)] rounded-sm border border-line bg-card p-5">
        <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
          required: valid appstream metainfo
        </p>
        <p class="mt-3 text-sm text-muted">
          Your <code class="font-mono text-sm text-fg">&lt;app-id&gt;.metainfo.xml</code> must
          exist and pass <code class="font-mono text-sm text-fg">appstreamcli validate</code>
          with no errors. This is a gate, same as the build: an app with broken or missing
          metadata renders as a blank tile in GNOME Software, Discover, and the omapak
          catalog, and blank tiles don't ship from here. Give it a real name, summary,
          description, and release notes. Screenshots aren't gated, but add them; an app with
          screenshots gets installed, an app without gets scrolled past.
        </p>
      </div>
    </div>

    <div>
      <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">step 3 · test locally</p>
      <p class="mt-3 max-w-[var(--read-width)] text-muted">
        Cheapest check: just build it.
      </p>
      <pre
        class="mt-3 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-4 font-mono text-xs leading-relaxed text-muted">flatpak-builder --force-clean --repo=/tmp/omapak-test \
  build apps/com.yourname.yourapp/com.yourname.yourapp.json</pre>
      <p class="mt-4 max-w-[var(--read-width)] text-muted">
        If you want the full treatment (what CI will run), use the judge itself. Any
        OpenAI-compatible endpoint works:
      </p>
      <pre
        class="mt-3 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-4 font-mono text-xs leading-relaxed text-muted">git clone --depth 100 https://github.com/you/your-app /tmp/src
cargo run -p omapak-judge -- apps/com.yourname.yourapp --source-dir /tmp/src</pre>
    </div>

    <div>
      <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
        step 4 · open the PR
      </p>
      <p class="mt-3 max-w-[var(--read-width)] text-muted">
        Push your branch, open a PR against
        <code class="font-mono text-sm text-fg">outcrop-labs/omapak</code>. CI runs the judge
        (static checks, a real build in a clean container, then the rubric) and posts the
        full report as a comment on your PR. Scores are advisory; the gates are: it builds,
        it's not hostile to the user, packaging isn't a mess.
      </p>
    </div>

    <div>
      <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
        step 5 · merge and publish
      </p>
      <p class="mt-3 max-w-[var(--read-width)] text-muted">
        A maintainer reads the report and merges, or doesn't, with written reasons, in
        public. On merge your app is built, signed, published to the repo, and its judge
        report goes live on this site. Users install it like any flatpak:
      </p>
      <pre
        class="mt-3 max-w-full overflow-x-auto rounded-sm border border-line bg-panel p-4 font-mono text-xs leading-relaxed text-muted">flatpak install omapak com.yourname.yourapp</pre>
    </div>

    <div class="rounded-sm border border-line bg-card p-5">
      <p class="font-mono text-xs uppercase tracking-[0.15em] text-ink-dim">
        the one ongoing rule · maintenance
      </p>
      <p class="mt-3 max-w-[var(--read-width)] text-sm text-muted">
        Published apps have to be alive or perma-stable. Quiet or broken past the window (TBD;
        we'll set it as a community, in the open) and your app gets archived to
        <code class="font-mono text-sm text-fg">omapak-unmaintained</code>: still installable,
        clearly labeled. Quiet after that and it drops. Pretending dead software is fine does
        users no favors.
      </p>
    </div>
  </div>
</section>
