#!/usr/bin/env python3
"""Push the repo directory to R2 via the S3 API.

R2's direct REST API doesn't handle multi-segment object keys (slashes),
and rclone's CopyObject isn't implemented by R2. boto3's upload_file
sends a simple PUT which R2 fully supports.

Non-blocking: individual failures are warnings, never kill the process.
"""
import os
import sys

import boto3
from botocore.config import Config

REPO = sys.argv[1] if len(sys.argv) > 1 else "repo"
BUCKET = "omapak-repo"

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
total = sum(len(files) for _, _, files in os.walk(REPO))

for root, _, files in os.walk(REPO):
    for f in files:
        local = os.path.join(root, f)
        key = os.path.relpath(local, REPO)
        try:
            s3.upload_file(local, BUCKET, key)
            pushed += 1
        except Exception as e:
            print(f"  FAILED: {key}: {e}", file=sys.stderr)
            failed += 1
        if (pushed + failed) % 500 == 0:
            print(f"  progress: {pushed + failed} / {total}", flush=True)

print(f"pushed {pushed}/{total} files ({failed} failed)")
