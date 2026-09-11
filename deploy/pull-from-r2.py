#!/usr/bin/env python3
"""Restore repo refs + summary from R2 via the S3 API.

rclone's listing of the grown bucket (~30k objects) fails silently, which
once wiped the omapak apps from a re-signed summary. This mirrors
push-to-r2.py: boto3 paginators for listing, a small thread pool for the
many tiny ref files.

Usage: pull-from-r2.py [repo-dir]
"""
import os
import re
import sys
from concurrent.futures import ThreadPoolExecutor

import boto3
from botocore.config import Config

REPO = sys.argv[1] if len(sys.argv) > 1 else "repo"
BUCKET = "omapak-repo"
KEEP = ("refs/", "summary", "config")
SHA256 = re.compile(r"[0-9a-f]{64}")

s3 = boto3.client(
    "s3",
    endpoint_url=f"https://{os.environ.get('R2_ACCOUNT_ID', '7396d8475acc6c87ef13e97a617712f1')}.r2.cloudflarestorage.com",
    aws_access_key_id=os.environ["R2_ACCESS_KEY_ID"],
    aws_secret_access_key=os.environ["R2_SECRET_ACCESS_KEY"],
    region_name="auto",
    config=Config(retries={"max_attempts": 3, "mode": "adaptive"}),
)

keys = []
for page in s3.get_paginator("list_objects_v2").paginate(Bucket=BUCKET):
    for obj in page.get("Contents", []):
        if obj["Key"].startswith(KEEP):
            keys.append(obj["Key"])
print(f"listing: {len(keys)} repo files in bucket", flush=True)


def fetch(key):
    local = os.path.join(REPO, key)
    os.makedirs(os.path.dirname(local), exist_ok=True)
    s3.download_file(BUCKET, key, local)


failed = 0
with ThreadPoolExecutor(max_workers=16) as pool:
    for key, exc in zip(keys, pool.map(fetch, keys)):
        if exc is not None:
            print(f"  FAILED: {key}: {exc}", file=sys.stderr)
            failed += 1

print(f"restored {len(keys) - failed}/{len(keys)} files ({failed} failed)")

# ostree summary --update reads each ref's commit object, but objects/
# isn't restored wholesale (it holds the big filez payloads). Commit
# objects are ~1KB — fetch the ones refs point at that we don't have.
commits = []
for root, _, files in os.walk(os.path.join(REPO, "refs", "heads")):
    for f in files:
        try:
            with open(os.path.join(root, f)) as fh:
                c = fh.read().strip()
        except OSError:
            continue
        if not SHA256.fullmatch(c):
            continue
        obj = os.path.join(REPO, "objects", c[:2], f"{c[2:]}.commit")
        if not os.path.exists(obj):
            commits.append((c, obj))

def fetch_commit(pair):
    c, obj = pair
    os.makedirs(os.path.dirname(obj), exist_ok=True)
    s3.download_file(BUCKET, f"objects/{c[:2]}/{c[2:]}.commit", obj)

cfailed = 0
with ThreadPoolExecutor(max_workers=16) as pool:
    for (c, _), exc in zip(commits, pool.map(fetch_commit, commits)):
        if exc is not None:
            print(f"  FAILED commit object: {c}: {exc}", file=sys.stderr)
            cfailed += 1
print(f"commit objects: {len(commits) - cfailed} fetched ({cfailed} failed)")
sys.exit(1 if (failed or cfailed) else 0)
