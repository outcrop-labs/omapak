# Why omapak exists

In May 2026, Flathub decided that new apps are banned if any part of them — code, docs, a
commit, whatever — was written with AI assistance. Even one AI-touched commit disqualifies a
submission. Their house, their rules, fine. But it's a dumb rule, and I think we can do better
than dumb rules.

Here's the thing the policy gets exactly backwards: **authorship was never the signal. The
artifact is.** A crap app hand-typed by a saint is still a crap app. A great app vibe-coded in
a weekend by someone who barely knows Rust is still a great app. Users don't install a commit
history; they install software. "Who wrote it" is identity politics for package managers.
"What does it do, does it work, is it honest" is the whole job.

So omapak flips it: **grade the artifact, not the authorship.** We don't ask, we don't care,
and we don't want to know how the sausage was made. We care about:

1. **Does it build and install.** Hard gate. Non-negotiable.
2. **Is it safe to hand to users.** Obfuscated payloads, mystery endpoints, undisclosed
   telemetry, miner-shaped code — flagged, and critical flags are a hard reject.
3. **Is it any good.** Scored, in the open, by a published rubric — problem clarity,
   architecture, code quality, UI/UX, packaging. Advisory, not gatekeeping. If your app is a
   clone of something better, the report says so and the user decides. We don't decide for
   them. That's the difference between a store and a gatekeeper.

Every judge report is public. Every prompt is public, versioned, in the repo. You can read
exactly why any app got in or didn't. No vibes-based moderation, no interminable
"moderation team" queues, no asking you embarrassing questions about your git history. An LLM
does the triage against a rubric you can read; a human reads its report and merges — or
doesn't, with public reasons.

That last bit matters: I'm not anti-Flathub, and I'm not anti-human-review. I'm anti-*_scaling
moderation on vibes* and anti-*interrogating developers about their tools*. The rubric does
the triage so a small maintainer team can run a big repo honestly. Judge output is advisory;
humans decide; everything is written down.

## The other rules

- **Maintenance.** Published apps must be actively maintained. Go quiet past the window (TBD —
  we'll set it by community consensus, in the open), and your app gets archived to
  `omapak-unmaintained` — still installable, clearly labeled. Stay quiet after that and it
  drops from the repo and the servers. Software rot is real and pretending otherwise does
  users no favors.
- **Everything flatpak.** omapak is born in the Omarchy community, but it's a standard flatpak
  repo — every distro is welcome, and we'll happily host the app IDs other repos deplatform.
- **Flathub content.** We don't mirror or proxy Flathub's apps; our catalog links through.
  If that ever gets blocked, we'll mirror overnight and keep going. Not because we want to —
  because the architecture doesn't depend on anyone's permission, and neither should your
  app store.

## What omapak is not

Not a slop repository. The whole point is that "AI-made" and "slop" are different axes. Slop
is judged here — by what the app does, not by which tools made it. If that distinction is too
subtle for your policy, that's a you problem.

— jon, Outcrop Labs, September 2026
