# omapak

The open Flatpak repository. **Grade the artifact, not the authorship.**

omapak accepts apps on what they are — does it build, does it install, is it safe, is it any
good — and never on who or what wrote the code. Every submission is scored by a published
rubric, every judge report is public, and merging is a human decision. Read
[MISSION.md](MISSION.md) for why this exists.

Born in the [Omarchy](https://omarchy.org) community, open to every distro — it's just
flatpak.

## Layout

```
crates/omapak-core/     shared schema: rubric, reports, manifest/metadata parsing
crates/omapak-judge/    the judge pipeline (static → build → [dynamic] → LLM rubric → report)
apps/<app-id>/          submissions: manifest + metadata.yml (+ report.json once merged)
fixtures/testapp/       end-to-end fixture for the judge pipeline
web/                    SvelteKit + Mercury catalog & manifesto site
.github/workflows/      judge (two-phase, secrets-safe) + publish (GPG-signed OSTree → R2)
deploy/r2.sh            rclone sync to Cloudflare R2
omapak.flatpakrepo      the remote users add
```

## The judge

```
cargo run -p omapak-judge -- apps/my.app --source-dir ~/src/my.app
```

Stages:

1. **static** — flatpak-builder-lint, appstream validation, deterministic signals
   (risky finish-args, unpinned sources, tree stats, last commit date)
2. **build** — a real `flatpak-builder` build. Failing here is a hard reject.
3. **dynamic** *(optional, `--with-dynamic`)* — install, launch under a headless compositor,
   screenshot at t+2s/t+10s. Best-effort; degrades to a note.
4. **judge** — OpenAI-compatible LLM scores the rubric. Configured via
   `OMAPAK_LLM_BASE_URL`, `OMAPAK_LLM_KEY`, `OMAPAK_LLM_MODEL` (any provider works).
   The prompt lives in `crates/omapak-judge/src/prompt.rs` and is published on the site.
5. **report** — versioned JSON + markdown PR comment.

Gates: build success, no critical security flag, packaging hygiene ≥ 2/5. Everything else —
including "better apps already exist" — is advisory and always will be.

## Submitting

PR against `apps/` with your manifest + `metadata.yml`. CI judges, posts the report on the
PR, a human merges. See `/submit` on the site.

## Repo maintenance lifecycle

Apps must be actively maintained. Past the window (TBD by community consensus): archived to
`omapak-unmaintained`. Still quiet after a second window: dropped from repo and servers.

## Signing + publishing

Generate the repo signing key once:

```
gpg --quick-generate-key "omapak repo <repo@omapak.outcrop.labs>" default sign never
gpg --armor --export-secret-keys <keyid>   # → GitHub secret OMAPAK_GPG_KEY
gpg --armor --export <keyid>               # → omapak.flatpakrepo GPGKey= (base64 of this)
```

`publish.yml` builds all apps into the OSTree repo, signs the summary, keeps repo state on
the `repo` branch, and syncs to R2 (`R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`,
`R2_ENDPOINT`).

## Web

```
cd web
bun install
bun run sync-rubric && bun run build-catalog --with-fixtures
bun run dev
```

## License

MIT.
