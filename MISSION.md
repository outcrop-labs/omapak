# Why Omapak exists.

In May 2026 Flathub decided new apps are banned if AI had anything to do with them. 
Even one AI-touched commit disqualifies a submission. Their house, their rules... 
but it's a dumb rule, and I'd rather build the alternative than argue with people
who are unwilling to have a fair conversation about it.

Here's what their policy completely ignores: **Normal people don't give a shit how
something was built if it works well and it fits their needs.** A useless app 
hand-typed by Linus Torvalds himself is still useless. A great app vibe-coded over 
a weekend is still a great app. Nobody installs a commit history. "Who wrote it" is 
identity politics for package managers; "what does it do, does it work, is it honest" 
is all we should care about. If you disagree, I honestly don't care.

Omapak flips it: **grade the app on its own merits.** We don't care as much about how 
the sausage got made as we do about what the intent is, and what its state is.

1. **It builds and installs.** Hard gate. Non-negotiable.
2. **It's not hostile to the user.** Obfuscated payloads, mystery endpoints,
   undisclosed telemetry, miner-shaped code: hard pass, pound sand.
3. **The packaging isn't a mess.** Below 2/5 on packaging hygiene, come back
   when it installs clean.

Everything else gets **scored, published, and never gatekept.** If your app is a clone, 
the report says so and the user decides. Not us. That's the difference between a store 
and a gatekeeper.

Omapak will be governed in the open: every agent judge report is public, every prompt is
[public and versioned](https://omapak.org/rubric), every accept or reject comes with 
written reasons. An agent that we build in the open does the triage against a rubric
you can read (and the community guides); a human makes the call. The rubric does the 
boring work so a small team of maintainers can reasonably run this repo.

To be clear, I'm not anti-Flathub and I'm not anti-human-review. I'm anti *gatekeeping*, 
and anti *treating "used an LLM" as a character flaw*. That said, Omapak is not a slop repo 
either. That's the whole point. "Built it using AI/agents" and "slop" are different things.

## The other rules

- **Maintenance.** Apps here have to be alive or perma-stable. Quiet or broken past the window (TBD —
  we'll set it as a community, in the open) and your app gets archived to
  `omapak-unmaintained`, still installable, clearly labeled. Quiet after that
  and it drops from the repo and the servers. Pretending dead software is fine
  does users no favors.
- **Every distro.** Born in Omarchy, but it's a standard flatpak repo, so it will do flatpak
  things. If another repo deplatforms a good app, we'll happily host the continuation.
- **Flathub's apps.** We list their whole catalog and serve it: fetched from
  them once, cached on our infra, delivered from repo.omapak.org. Not a
  pre-mirror, and I'm not going to pretend we don't hold bytes we hold. If
  they ever block us, users keep updating from what we've cached and we
  backfill the rest overnight. Your app store shouldn't depend on anyone's
  permission or opinions.

Let's just support people who want to make things, and do it in the open.

Jon, September 2026
