# omapak

The open Flatpak repo. **Grade the artifact, not the authorship.**

I built this because Flathub decided in May 2026 that new apps get banned if any
part of them touched an LLM — even one commit. That's a dumb rule. Users don't
install your git history, they install software. So here, we judge the software:
does it build, does it install, is it safe, is it any good. Who or what typed
the code — human, AI, or a raccoon with a keyboard — is not our business.

Every submission gets scored by a [published rubric](https://omapak.org/rubric),
every judge report is public, and a human makes the merge call. That's the whole
gimmick. Read [MISSION.md](MISSION.md) if you want the long version — it's short.

Started for the [Omarchy](https://omarchy.org) crowd, works on any distro,
because it's just flatpak.

## What's in here

```
crates/omapak-core/     the schema: rubric, reports, manifest parsing
crates/omapak-judge/    the judge — static checks → real build → LLM rubric → public report
apps/<app-id>/          submissions: your manifest + metadata.yml
fixtures/testapp/       fixture app I use to beat on the pipeline
web/                    omapak.org — SvelteKit + Mercury
deploy/r2.sh            pushes the signed repo to Cloudflare R2
omapak.flatpakrepo      what users install with
```

## The judge

```
cargo run -p omapak-judge -- apps/my.app --source-dir ~/src/my.app
```

It runs the boring checks first (flatpak-builder-lint, appstream validation,
sandbox holes, unpinned sources), does a real `flatpak-builder` build, then an
LLM scores it against the rubric. Point it at any OpenAI-compatible endpoint:

```
OMAPAK_LLM_BASE_URL / OMAPAK_LLM_KEY / OMAPAK_LLM_MODEL
```

The prompt is [in the repo](crates/omapak-judge/src/prompt.rs) and on the site.
I'm not going to pretend it's un-gameable — it's advisory. The gates are dumb
and honest on purpose: **build must pass, no critical security flag, packaging
≥ 2/5.** Everything else — including "better apps already exist" — is advisory
and always will be. Clones are allowed. Hiding that they're clones is not.

## Submitting

Fork, add `apps/<your.app-id>/` with your manifest and a `metadata.yml`
(submitter, source_repo, summary, license — that's it, no attestation about who
wrote your code), open a PR. The judge posts its report on the PR, a human
reads it and merges or doesn't — with public reasons either way.

## The maintenance rule

Published apps have to be alive. Go quiet past the window (we'll set it by
community consensus, in the open) and your app moves to `omapak-unmaintained` —
still installable, clearly labeled. Stay quiet and it drops. Dead software in a
store is a lie to users.

## Repo ops

Signing key lives on my machine (`FAD5B6F0BD9E93E7`). CI (`.github/workflows/`)
builds every merged app, signs the summary, keeps repo state on the `repo`
branch, and syncs to R2. Secrets it wants: `OMAPAK_LLM_*`, `OMAPAK_GPG_KEY`,
`R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, plus vars `R2_ENDPOINT`/`R2_BUCKET`.

## The site

```
cd web
bun install
bun run sync-rubric && bun run build-catalog --with-fixtures
bun run dev
```

## License

MIT.
