#!/usr/bin/env python3
"""Clear the update bot's PR queue: supersede stale updates, merge standard ones.

update-apps.yml runs flatpak-external-data-checker daily. The checker names
each branch after the tree hash it produced, so whenever main moves it opens
a fresh PR for an update that is still pending (the same Vish bump sat open
four times). Per app (the apps/<id> dirs a PR touches), this sweep:

  1. keeps the newest open update PR and closes the rest as superseded,
     deleting their branches;
  2. closes that newest PR too when main has changed any file it touches
     since it was computed (a newer update merged, or a hand edit). The
     checker reopens it against current main if it still applies;
  3. otherwise merges it when it is a standard update:
       - its branch is the checker's (update-<base>-<hash>) in this repo,
         never a fork;
       - the judge passed on its head commit: the omapak/judge status that
         judge-report.yml sets (built, no critical security flag);
       - the diff only moves checker-driven pins (url, sha256, sha512, size,
         tag, commit on a source that carries x-checker-data) and adds bare
         <release> entries to the metainfo. Manifests are compared parsed,
         so the checker's re-serialisation (quoting, line wraps) is no
         change;
       - every moved url still points at the same upstream: same host and
         port, https or the old scheme, and on a forge the same project
         (owner/repo).
     Anything else stays open with a comment saying why.

It approves as github-actions[bot] with GITHUB_TOKEN and merges with
MERGE_TOKEN (UPDATE_BOT_TOKEN). A merge pushed with GITHUB_TOKEN triggers
no workflows, so publish.yml would never ship it.

Requires: GITHUB_TOKEN; MERGE_TOKEN unless --dry-run (report, write nothing).
"""
import argparse
import functools
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

import yaml

REPO = os.environ.get("GITHUB_REPOSITORY", "outcrop-labs/omapak")
API = "https://api.github.com"
JUDGE_CONTEXT = "omapak/judge"
BOT = "github-actions[bot]"
MARKER = "<!-- omapak:update-sweep -->"

# The pins the checker moves on a source it owns (one with x-checker-data).
PIN_KEYS = {"url", "sha256", "sha512", "size", "tag", "commit"}

# Hosts whose first two path segments name the project (owner/repo,
# project/<name>, ~user/repo). A moved url must keep them.
FORGES = {
    "github.com", "codeload.github.com", "raw.githubusercontent.com",
    "gitlab.com", "gitlab.gnome.org", "gitlab.freedesktop.org",
    "invent.kde.org", "salsa.debian.org", "codeberg.org", "gitea.com",
    "bitbucket.org", "git.sr.ht", "sourceforge.net",
    "downloads.sourceforge.net", "download.sourceforge.net",
}

RELEASE_ATTRS = {"version", "date", "type", "urgency", "timestamp"}

DRY_RUN = False
SUMMARY = []


def request(method, path, token, body=None, accept="application/vnd.github+json", tries=3):
    url = path if path.startswith("https://") else API + path
    data = json.dumps(body).encode() if body is not None else None
    for attempt in range(1, tries + 1):
        req = urllib.request.Request(url, data=data, method=method, headers={
            "Authorization": f"Bearer {token}",
            "Accept": accept,
            "User-Agent": "omapak-update-sweep",
            "X-GitHub-Api-Version": "2022-11-28",
        })
        if data is not None:
            req.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(req) as r:
                raw = r.read()
                if "raw" in accept:
                    return r.status, raw
                return r.status, json.loads(raw) if raw else {}
        except urllib.error.HTTPError as e:
            if method == "GET" and e.code in (429, 502, 503) and attempt < tries:
                time.sleep(10 * attempt)
                continue
            try:
                detail = json.load(e)
            except ValueError:
                detail = {}
            return e.code, detail
        except urllib.error.URLError:
            if attempt < tries:
                time.sleep(10)
                continue
            raise


def gh(method, path, body=None, **kw):
    return request(method, path, os.environ["GITHUB_TOKEN"], body, **kw)


def get_all(path):
    items, page = [], 1
    sep = "&" if "?" in path else "?"
    while True:
        status, batch = gh("GET", f"{path}{sep}per_page=100&page={page}")
        if status != 200:
            sys.exit(f"GET {path}: HTTP {status} {batch.get('message', '')}")
        items += batch
        if len(batch) < 100:
            return items
        page += 1


def write(desc, method, path, body=None, token=None):
    """A mutating call, skipped under --dry-run. Returns (ok, detail)."""
    if DRY_RUN:
        print(f"  [dry-run] would {desc}")
        return True, {}
    status, detail = request(method, path, token or os.environ["GITHUB_TOKEN"], body)
    if status >= 300:
        print(f"::warning::{desc} failed: HTTP {status} {detail.get('message', '')}")
        return False, detail
    return True, detail


@functools.lru_cache(maxsize=None)
def file_at(path, ref):
    status, raw = gh(
        "GET", f"/repos/{REPO}/contents/{urllib.parse.quote(path)}?ref={ref}",
        accept="application/vnd.github.raw+json")
    return raw if status == 200 else None


# ---- policy -----------------------------------------------------------------

def project(parts):
    segs = [s.lower() for s in parts.path.split("/") if s][:2]
    if len(segs) == 2:
        segs[1] = segs[1].removesuffix(".git")
    return segs


def moved_url_problems(old, new):
    if not isinstance(old, str) or not isinstance(new, str):
        return ["url is not a string"]
    o, n = urllib.parse.urlsplit(old), urllib.parse.urlsplit(new)
    problems = []
    if n.scheme != o.scheme and n.scheme != "https":
        problems.append(f"scheme changed {o.scheme} -> {n.scheme}")
    if o.netloc.lower() != n.netloc.lower():
        problems.append(f"host changed {o.netloc} -> {n.netloc}")
    elif (o.hostname or "") in FORGES and project(o) != project(n):
        problems.append(
            f"project changed {'/'.join(project(o))} -> {'/'.join(project(n))} on {o.hostname}")
    return problems


def compare(old, new, where, problems):
    if isinstance(old, dict) and isinstance(new, dict):
        checked = "x-checker-data" in old and "x-checker-data" in new
        for key in sorted(old.keys() | new.keys(), key=str):
            at = f"{where}.{key}" if where else str(key)
            if checked and key in PIN_KEYS:
                if key not in new:
                    problems.append(f"`{at}` removed")
                elif key not in old:
                    # A new checksum or commit pin is fine; a url appearing
                    # has no previous upstream to hold it to.
                    if key == "url":
                        problems.append(f"`{at}` added")
                elif key == "url" and old[key] != new[key]:
                    problems += [f"`{at}`: {p}" for p in moved_url_problems(old[key], new[key])]
                continue
            if key not in old or key not in new:
                problems.append(f"`{at}` {'added' if key in new else 'removed'}")
            else:
                compare(old[key], new[key], at, problems)
    elif isinstance(old, list) and isinstance(new, list) and len(old) == len(new):
        for i, (o, n) in enumerate(zip(old, new)):
            compare(o, n, f"{where}[{i}]", problems)
    elif old != new:
        problems.append(f"`{where or '<root>'}` changed")


def manifest_problems(name, old_raw, new_raw):
    load = json.loads if name.endswith(".json") else yaml.safe_load
    try:
        old, new = load(old_raw), load(new_raw)
    except (ValueError, yaml.YAMLError) as e:
        return [f"cannot parse: {e}"]
    problems = []
    compare(old, new, "", problems)
    return problems


def shape(el):
    return (
        el.tag,
        tuple(sorted(el.attrib.items())),
        (el.text or "").strip(),
        tuple((shape(c), (c.tail or "").strip()) for c in el),
    )


def pop_releases(root):
    releases = root.find("releases")
    if releases is None:
        return {}, []
    root.remove(releases)
    return dict(releases.attrib), list(releases)


def metainfo_problems(old_raw, new_raw):
    try:
        old, new = ET.fromstring(old_raw), ET.fromstring(new_raw)
    except ET.ParseError as e:
        return [f"cannot parse: {e}"]
    (old_attrs, old_rels), (new_attrs, new_rels) = pop_releases(old), pop_releases(new)
    problems = []
    if shape(old) != shape(new):
        problems.append("changed outside `<releases>`")
    if old_attrs != new_attrs:
        problems.append("`<releases>` attributes changed")
    new_by_version = {r.get("version"): r for r in new_rels}
    for r in old_rels:
        v = r.get("version")
        if v not in new_by_version:
            problems.append(f"release {v} removed")
        elif shape(new_by_version[v]) != shape(r):
            problems.append(f"release {v} changed")
    old_versions = {r.get("version") for r in old_rels}
    for v, r in new_by_version.items():
        if v in old_versions:
            continue
        if r.tag != "release" or set(r.attrib) - RELEASE_ATTRS:
            problems.append(f"new release {v} carries unexpected attributes")
        # The checker writes an empty <description/>; notes mean a human.
        if any(c.tag != "description" or len(c) or (c.text or "").strip() for c in r):
            problems.append(f"new release {v} carries content")
    return problems


def merge_base(pr):
    status, cmp = gh("GET", f"/repos/{REPO}/compare/{pr['base']['ref']}...{pr['head']['sha']}")
    return cmp["merge_base_commit"]["sha"] if status == 200 else None


def policy_problems(pr, files, base):
    problems = []
    for f in files:
        name = f["filename"]
        if f["status"] != "modified":
            problems.append(f"`{name}` {f['status']}: an update only edits existing files")
            continue
        old_raw, new_raw = file_at(name, base), file_at(name, pr["head"]["sha"])
        if old_raw is None or new_raw is None:
            problems.append(f"`{name}`: could not fetch both versions")
        elif name.endswith((".yml", ".yaml", ".json")):
            problems += [f"`{name}`: {p}" for p in manifest_problems(name, old_raw, new_raw)]
        elif re.search(r"\.(metainfo|appdata)\.xml$", name):
            problems += [f"`{name}`: {p}" for p in metainfo_problems(old_raw, new_raw)]
        else:
            problems.append(f"`{name}`: not a manifest or metainfo file")
    return problems


# ---- actions ----------------------------------------------------------------

def judge_status(sha):
    status, statuses = gh("GET", f"/repos/{REPO}/commits/{sha}/statuses?per_page=100")
    if status != 200:
        return None, None
    for s in statuses:  # newest first
        if s["context"] == JUDGE_CONTEXT and (s.get("creator") or {}).get("login") == BOT:
            return s["state"], s.get("description") or ""
    return None, None


def upsert_comment(pr, body):
    body = f"{MARKER}\n{body}"
    comments = get_all(f"/repos/{REPO}/issues/{pr['number']}/comments")
    mine = next((c for c in comments
                 if c["user"]["login"] == BOT and c["body"].startswith(MARKER)), None)
    if mine and mine["body"] == body:
        return
    if mine:
        write(f"update comment on #{pr['number']}", "PATCH",
              f"/repos/{REPO}/issues/comments/{mine['id']}", {"body": body})
    else:
        write(f"comment on #{pr['number']}", "POST",
              f"/repos/{REPO}/issues/{pr['number']}/comments", {"body": body})


def delete_branch(pr):
    write(f"delete branch {pr['head']['ref']}", "DELETE",
          f"/repos/{REPO}/git/refs/heads/{urllib.parse.quote(pr['head']['ref'])}")


def close(pr, why):
    print(f"#{pr['number']}: closing, {why}")
    upsert_comment(pr, f"Closing: {why}.")
    ok, _ = write(f"close #{pr['number']}", "PATCH",
                  f"/repos/{REPO}/pulls/{pr['number']}", {"state": "closed"})
    if ok:
        delete_branch(pr)
        SUMMARY.append(f"- closed #{pr['number']}: {why}")


def leave_for_review(pr, reasons):
    print(f"#{pr['number']}: left for review")
    for r in reasons:
        print(f"    {r}")
    upsert_comment(pr, "**Left for review.** The update sweep only auto-merges standard "
                       "updates, and this one isn't:\n\n"
                   + "\n".join(f"- {r}" for r in reasons)
                   + "\n\nMerge by hand if it's fine. A newer update for this app closes "
                     "this PR in its favour.")
    SUMMARY.append(f"- left #{pr['number']} for review: {'; '.join(reasons)}")


def merge(pr):
    sha = pr["head"]["sha"]
    reviews = get_all(f"/repos/{REPO}/pulls/{pr['number']}/reviews")
    if not any(r["user"]["login"] == BOT and r["state"] == "APPROVED"
               and r["commit_id"] == sha for r in reviews):
        ok, _ = write(f"approve #{pr['number']}", "POST",
                      f"/repos/{REPO}/pulls/{pr['number']}/reviews",
                      {"event": "APPROVE", "commit_id": sha,
                       "body": "Standard update: judge passed, only checker-driven "
                               "source pins moved, same upstream."})
        if not ok:
            return
    # sha pins the merge to the commit that was judged and checked.
    ok, detail = write(f"merge #{pr['number']}", "PUT",
                       f"/repos/{REPO}/pulls/{pr['number']}/merge",
                       {"sha": sha, "merge_method": "merge"},
                       token=os.environ.get("MERGE_TOKEN"))
    if not ok:
        leave_for_review(pr, [f"merge refused: {detail.get('message', 'unknown error')}"])
        return
    print(f"#{pr['number']}: merged")
    SUMMARY.append(f"- merged #{pr['number']}: {pr['title']}")
    delete_branch(pr)


def consider(pr, files):
    dirs = {"/".join(f["filename"].split("/")[:2]) for f in files}
    if len(dirs) != 1 or not re.fullmatch(r"apps/[^/]+", next(iter(dirs))):
        leave_for_review(pr, ["touches files outside a single `apps/<id>` directory"])
        return
    base = merge_base(pr)
    if base is None:
        leave_for_review(pr, [f"could not compare with `{pr['base']['ref']}`"])
        return
    moved = [f["filename"] for f in files
             if file_at(f["filename"], pr["base"]["ref"]) != file_at(f["filename"], base)]
    if moved:
        close(pr, f"`{pr['base']['ref']}` has changed {', '.join(f'`{m}`' for m in moved)} "
                  "since this update was computed (a newer update merged, or a hand edit). "
                  "The daily check reopens it against current "
                  f"`{pr['base']['ref']}` if it still applies")
        return
    state, description = judge_status(pr["head"]["sha"])
    if state in (None, "pending"):
        print(f"#{pr['number']}: waiting for the judge")
        SUMMARY.append(f"- #{pr['number']} waiting for the judge")
        return
    reasons = [] if state == "success" else [f"judge: {description or state}"]
    reasons += policy_problems(pr, files, base)
    if reasons:
        leave_for_review(pr, reasons)
    else:
        merge(pr)


def main():
    global DRY_RUN
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--dry-run", action="store_true", help="report decisions, write nothing")
    args = ap.parse_args()
    DRY_RUN = args.dry_run
    if not DRY_RUN and not os.environ.get("MERGE_TOKEN"):
        sys.exit("MERGE_TOKEN is unset: merges pushed with GITHUB_TOKEN never publish")

    prs = [p for p in get_all(f"/repos/{REPO}/pulls?state=open")
           if (p["head"]["repo"] or {}).get("full_name") == REPO
           and re.fullmatch(rf"update-{re.escape(p['base']['ref'])}-[0-9a-f]{{7,}}", p["head"]["ref"])
           and not p["draft"]]
    print(f"{len(prs)} open update PRs")

    groups = {}
    for pr in prs:
        files = get_all(f"/repos/{REPO}/pulls/{pr['number']}/files")
        key = tuple(sorted({"/".join(f["filename"].split("/")[:2]) for f in files}))
        groups.setdefault(key, []).append((pr, files))

    for key, group in sorted(groups.items()):
        group.sort(key=lambda g: g[0]["number"])
        newest, files = group[-1]
        print(f"== {', '.join(key)}")
        for pr, _ in group[:-1]:
            close(pr, f"superseded by #{newest['number']}. The update sweep keeps only "
                      "the newest open update PR per app")
        consider(newest, files)

    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a") as f:
            f.write("### update-sweep\n" + ("\n".join(SUMMARY) or "nothing to do") + "\n")


if __name__ == "__main__":
    main()
