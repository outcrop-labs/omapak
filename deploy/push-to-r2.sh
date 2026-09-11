#!/bin/bash
# Push repo files to R2 via Cloudflare API.
# R2's S3 API doesn't support CopyObject, so rclone fails.
# Object keys keep their slashes — no URL encoding needed.
set -u

DIR="${1:-repo}"
API="https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT}/r2/buckets/omapak-repo/objects"

total=$(find "$DIR" -type f | wc -l)
count=0

while IFS= read -r f; do
  rel="${f#"$DIR"/}"
  count=$((count + 1))
  code=$(curl -s -o /dev/null -w "%{http_code}" -X PUT "${API}/${rel}" \
    -H "Authorization: Bearer ${CF_TOKEN}" \
    --data-binary "@$f")
  if [ "$code" != "200" ]; then
    echo "  FAILED ($code): $rel"
  fi
  [ $((count % 500)) -eq 0 ] && echo "  progress: $count / $total"
done < <(find "$DIR" -type f)

echo "pushed $count files"
