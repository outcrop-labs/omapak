#!/usr/bin/env bash
# Sync the built OSTree repo to Cloudflare R2. Used by publish.yml and usable
# locally for manual pushes.
#
# Required environment:
#   R2_ACCESS_KEY_ID / R2_SECRET_ACCESS_KEY  — token with Object Write on the bucket
#   R2_ENDPOINT — e.g. https://<account_id>.r2.cloudflarestorage.com
#   R2_BUCKET — e.g. omapak-repo
# (In CI these come from repo secrets/vars of the same names.)
set -euo pipefail

repo_dir="${1:?usage: r2.sh <repo-dir>}"
: "${R2_ACCESS_KEY_ID:?R2_ACCESS_KEY_ID not set}"
: "${R2_SECRET_ACCESS_KEY:?R2_SECRET_ACCESS_KEY not set}"
: "${R2_ENDPOINT:?R2_ENDPOINT not set}"
: "${R2_BUCKET:=omapak-repo}"

command -v rclone >/dev/null || { echo "rclone not installed" >&2; exit 1; }

# Connection-string form: everything after the final ':' is bucket/path;
# the endpoint value must be quoted or its '://' colons split the parser.
remote=":s3,provider=Cloudflare,endpoint=\"${R2_ENDPOINT}\":${R2_BUCKET}"
rclone copy "$repo_dir/" "$remote" \
  --s3-access-key-id "$R2_ACCESS_KEY_ID" \
  --s3-secret-access-key "$R2_SECRET_ACCESS_KEY" \
  --s3-no-check-bucket \
  --checksum

echo "synced $repo_dir → $R2_ENDPOINT"
