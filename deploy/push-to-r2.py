#!/usr/bin/env python3
"""Push the repo directory to R2 via the S3 API.

R2's direct REST API doesn't handle multi-segment object keys (slashes),
and rclone's CopyObject isn't implemented by R2. boto3's upload_file
sends a simple PUT which R2 fully supports.

Non-blocking: individual failures are warnings, never kill the process.
"""
import os
import sys
from pathlib import Path

import boto3
from botocore.config import Config

REPO = sys.argv[1] if len(sys.argv) > 1 else "repo"
BUCKET = "omapak-repo"
ROOT = Path(__file__).resolve().parents[1]

s3 = boto3.client(
    "s3",
    endpoint_url=f"https://{os.environ.get('R2_ACCOUNT_ID', '7396d8475acc6c87ef13e97a617712f1')}.r2.cloudflarestorage.com",
    aws_access_key_id=os.environ["R2_ACCESS_KEY_ID"],
    aws_secret_access_key=os.environ["R2_SECRET_ACCESS_KEY"],
    region_name="auto",
    config=Config(retries={"max_attempts": 3, "mode": "adaptive"}),
)

pushed = 0
failed = 0
skipped = 0
total = sum(len(files) for _, _, files in os.walk(REPO))

# Content-addressed ostree objects are immutable; skip re-uploading the
# ~12k flathub ref/commit files that are already in the bucket. The
# summary, signatures and flatpakrepo always re-upload.
ALWAYS_PUSH = {"summary", "summary.sig", "omapak.flatpakrepo"}
existing = {}
for page in s3.get_paginator("list_objects_v2").paginate(Bucket=BUCKET):
    for obj in page.get("Contents", []):
        existing[obj["Key"]] = obj["Size"]

for root, _, files in os.walk(REPO):
    for f in files:
        local = os.path.join(root, f)
        key = os.path.relpath(local, REPO)
        if key not in ALWAYS_PUSH and existing.get(key) == os.path.getsize(local):
            skipped += 1
            continue
        try:
            s3.upload_file(local, BUCKET, key)
            pushed += 1
        except Exception as e:
            print(f"  FAILED: {key}: {e}", file=sys.stderr)
            failed += 1
        if (pushed + failed + skipped) % 500 == 0:
            print(f"  progress: {pushed + failed + skipped} / {total}", flush=True)

print(f"pushed {pushed}/{total} files ({skipped} unchanged skipped, {failed} failed)")

# Flatpak fetches summary.idx before summary, so a stale .idx/.idx.sig
# (old-key signed, left over from local test pushes — CI never generates
# them) fails gpg-verify-summary for every client even when summary and
# summary.sig are fresh. Same for a stale omapak.flatpakrepo: it would
# advertise an old keyring. Purge the leftovers, then republish the
# flatpakrepo so the advertised key always matches the signing key.
for key in ("summary.idx", "summary.idx.sig"):
    s3.delete_object(Bucket=BUCKET, Key=key)
    print(f"purged {key} (if present)")

stale = s3.list_objects_v2(Bucket=BUCKET, Prefix="summaries/")
for obj in stale.get("Contents", []):
    s3.delete_object(Bucket=BUCKET, Key=obj["Key"])
    print(f"purged {obj['Key']}")

s3.upload_file(str(ROOT / "omapak.flatpakrepo"), BUCKET, "omapak.flatpakrepo")
print("pushed omapak.flatpakrepo (current signing key)")
