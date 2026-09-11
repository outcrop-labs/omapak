#!/bin/bash
# Push the repo directory to R2 via the Cloudflare API.
# R2's S3 API doesn't support CopyObject, so rclone fails with 501.
# This uses the Cloudflare API directly which is fully supported.
set -u

DIR="${1:-repo}"
BUCKET="omapak-repo"
APIBase="https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT}/r2/buckets/${BUCKET}/objects"

pushed=0
failed=0

find "$DIR" -type f | while read -r f; do
  rel="${f#"$DIR"/}"
  # URL-encode the path (slashes become %2F for the API path)
  encoded=$(echo "$rel" | sed 's/\//%2F/g')
  
  result=$(curl -s -o /dev/null -w "%{http_code}" -X PUT "${ApiBase}/${encoded}" \
    -H "Authorization: Bearer ${CF_TOKEN}" \
    --data-binary "@$f")
  
  if [ "$result" = "200" ]; then
    pushed=$((pushed+1))
  else
    echo "  FAILED ($result): $rel"
    failed=$((failed+1))
  fi
done

echo "push complete"
