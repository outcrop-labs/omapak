# Why omapak exists

In May 2026 Flathub decided new apps are banned if AI had anything to do with
them — a commit, some docs, whatever. Even one AI-touched commit disqualifies a
submission. Their house, their rules. But it's a dumb rule, and I'd rather build
the alternative than argue about it.

Here's what that policy gets backwards: **authorship was never the signal. The
artifact is.** A crap app hand-typed by a saint is still a crap app. A great app
vibe-coded over a weekend is still a great app. Nobody installs a commit
history. "Who wrote it" is identity politics for package managers; "what does
it do, does it work, is it honest" is the whole job.

So omapak flips it: **grade the artifact, not the authorship.** We don't ask how
the sausage got made. We check exactly three things as gates:

1. **It builds and installs.** Hard gate. Non-negotiable.
2. **It's not hostile to the user.** Obfuscated payloads, mystery endpoints,
   undisclosed telemetry, miner-shaped code — critical flags are a hard reject.
3. **The packaging isn't a mess.** Below 2/5 on packaging hygiene, come back
   when it installs clean.

Everything else — problem clarity, architecture, code quality, UI/UX, and yes,
"is this just a worse clone of something that already exists" — gets **scored,
published, and never gatekept.** If your app is a clone, the report says so and
the user decides. Not us. That's the difference between a store and a
gatekeeper.

And it's all in the open: every judge report is public, every prompt is
[public and versioned](https://omapak.org/rubric), every accept or reject comes
with written reasons. No vibes-based moderation queue, no interrogating
developers about their tools. An LLM does the triage against a rubric you can
read; a human makes the call. The rubric does the boring work so a couple of
maintainers can run this honestly.

To be clear, I'm not anti-Flathub and I'm not anti-human-review. I'm anti
*scaling moderation on vibes*, and anti *treating "used an LLM" as a character
flaw*. omapak is not a slop repository either — that's the whole point. "AI-made"
and "slop" are different axes, and slop is the one we judge. By what the app
does. Not by which tools made it.

## The other rules

- **Maintenance.** Apps here have to be alive. Quiet past the window (TBD —
  we'll set it as a community, in the open) and your app gets archived to
  `omapak-unmaintained`, still installable, clearly labeled. Quiet after that
  and it drops from the repo and the servers. Pretending dead software is fine
  does users no favors.
- **Every distro.** Born in Omarchy, but it's a standard flatpak repo — if
  you're on Fedora or Nix or whatever, come on in. And if another repo
  deplatforms an app ID, we'll happily host the continuation.
- **Flathub's apps.** We don't mirror or proxy them; the catalog links through.
  If that ever gets blocked, we'll mirror overnight and keep going. Not because
  I want to — because your app store shouldn't depend on anyone's permission,
  and neither does this architecture.

— jon, Outcrop Labs, September 2026
