#!/usr/bin/env python3
"""Restore the repo from R2 via the S3 API.

rclone's listing of the grown bucket (~30k objects) fails silently, which
once wiped the omapak apps from a re-signed summary. This mirrors
push-to-r2.py: boto3 paginators for listing, a small thread pool for the
many tiny files.

Restores refs/, summary*, config — and the small half of objects/.
Building and re-signing only need metadata objects (commit/dirtree/
dirmeta, all tiny); the big filez payloads are served straight from R2
by the worker and never need to exist on a CI runner.

Usage: pull-from-r2.py [repo-dir]
"""
import os
import sys
from concurrent.futures import ThreadPoolExecutor

import boto3
from botocore.config import Config

REPO = sys.argv[1] if len(sys.argv) > 1 else "repo"
BUCKET = "omapak-repo"
OBJECT_MAX = 512 * 1024

s3 = boto3.client(
    "s3",
    endpoint_url=f"https://{os.environ.get('R2_ACCOUNT_ID', '7396d8475acc6c87ef13e97a617712f1')}.r2.cloudflarestorage.com",
    aws_access_key_id=os.environ["R2_ACCESS_KEY_ID"],
    aws_secret_access_key=os.environ["R2_SECRET_ACCESS_KEY"],
    region_name="auto",
    config=Config(retries={"max_attempts": 3, "mode": "adaptive"}),
)

keys = []
skipped_big = 0
for page in s3.get_paginator("list_objects_v2").paginate(Bucket=BUCKET):
    for obj in page.get("Contents", []):
        key, size = obj["Key"], obj["Size"]
        if key.startswith("refs/") or key.startswith("summary") or key == "config":
            keys.append((key, size))
        elif key.startswith("objects/") and size <= OBJECT_MAX:
            keys.append((key, size))
        elif key.startswith("objects/"):
            skipped_big += 1
print(f"listing: {len(keys)} files to restore ({skipped_big} big objects skipped)", flush=True)


def fetch(pair):
    key, _ = pair
    try:
        local = os.path.join(REPO, key)
        os.makedirs(os.path.dirname(local), exist_ok=True)
        s3.download_file(BUCKET, key, local)
        return None
    except Exception as e:
        return (key, e)


failed = 0
with ThreadPoolExecutor(max_workers=16) as pool:
    for result in pool.map(fetch, keys):
        if result is not None:
            key, e = result
            print(f"  FAILED: {key}: {e}", file=sys.stderr)
            failed += 1
        if failed and failed % 100 == 0:
            print(f"  {failed} failures so far", flush=True)

print(f"restored {len(keys) - failed}/{len(keys)} files ({failed} failed)")
sys.exit(1 if failed else 0)
