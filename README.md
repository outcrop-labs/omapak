# Omapak

The open Flatpak repo. **Grade the app on its own merits.**

I built this because Flathub decided in May 2026 that new apps get banned if AI
had anything to do with them — even one commit. That's a dumb rule, and I'd
rather build the alternative than argue with people who won't have a fair
conversation about it. Normal people don't give a shit how something was built
if it works well and fits their needs. Neither do we.

So: every submission gets scored by an [agent judge](https://omapak.org/rubric)
we build in the open, every report is public, and a human makes the merge call.
The gates are honest and boring — it builds, it's not hostile to the user, the
packaging isn't a mess. Everything else is scored, published, and never
gatekept. If your app is a clone, the report says so and the user decides. Not us.

**Read [MISSION.md](MISSION.md) before anything else** — it's short, it's the
whole point, and it's the tone to expect around here. If you disagree with it,
that's fine; this repo probably isn't for you.

Born in [Omarchy](https://omarchy.org), works on any distro, because it's just
flatpak. If another repo deplatforms a good app, we'll happily host the
continuation.

## What's in here

```
crates/omapak-core/     the schema: rubric, reports, manifest parsing
crates/omapak-judge/    the judge — static checks → real build → agent rubric → public report
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
sandbox holes, unpinned sources), does a real `flatpak-builder` build, then the
judge agent scores it against the rubric. Point it at any OpenAI-compatible
endpoint:

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
