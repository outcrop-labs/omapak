#!/bin/bash
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true

strip() {
  echo "$1" | sed 's/[^a-zA-Z0-9._-]//g'
}

# dl.flathub.org throws transient HTTP/2 framing errors under load; a
# runtime that fails to install here means every app needing it silently
# drops from the repo, so retry before giving up.
failed=""
install_retry() {
  echo "Installing $*"
  for attempt in 1 2 3; do
    if flatpak install --user -y --noninteractive flathub "$@" >/tmp/rt-install.log 2>&1; then
      grep -m3 . /tmp/rt-install.log | tail -3
      return 0
    fi
    [ "$attempt" = 3 ] || { echo "  attempt $attempt failed, retrying"; sleep $((attempt * 5)); }
  done
  echo "  FAILED to install: $*"
  tail -5 /tmp/rt-install.log
  failed="$failed $*"
  return 1
}

# sdk-extensions entries, one per line: handles both YAML styles —
#   sdk-extensions: [org.freedesktop.Sdk.Extension.node24]
#   sdk-extensions:
#     - org.freedesktop.Sdk.Extension.rust-stable
sdk_exts() {
  awk '/^sdk-extensions:/ {
         if ($0 ~ /\[/) {
           sub(/^sdk-extensions:[[:space:]]*/, ""); gsub(/[\[\],]/, " "); print; exit
         }
         f=1; next
       }
       f && /^[[:space:]]*-/ {print; next}
       f {exit}' "$1" \
    | sed -e 's/^[[:space:]]*-[[:space:]]*//' -e 's/["'\'']//g'
}

# Args: app dirs to provision for (PR checks judge only the changed
# app). No args: every app in apps/ (publish builds everything).
if [ "$#" -gt 0 ]; then
  paths=""
  for d in "$@"; do paths="$paths $d/*.yml $d/*.yaml $d/*.json"; done
else
  paths="apps/*/*.yml apps/*/*.yaml apps/*/*.json"
fi

for m in $paths; do
  [ -f "$m" ] || continue
  rt=$(strip "$(grep -m1 '^runtime:' "$m" | cut -d: -f2-)")
  rv=$(strip "$(grep -m1 '^runtime-version:' "$m" | cut -d: -f2-)")
  if [ -n "$rt" ] && [ -n "$rv" ]; then
    sdk=$(echo "$rt" | sed 's/Platform/Sdk/')
    install_retry "$rt//$rv" "$sdk//$rv"
    # Extensions are versioned alongside the SDK; flatpak-builder fails
    # outright ("Requested extension ... not installed") without them.
    for ext in $(sdk_exts "$m"); do
      install_retry "$ext//$rv"
    done
  fi
  bt=$(strip "$(grep -m1 '^base:' "$m" | cut -d: -f2-)")
  bv=$(strip "$(grep -m1 '^base-version:' "$m" | cut -d: -f2-)")
  if [ -n "$bt" ] && [ -n "$bv" ]; then
    install_retry "$bt//$bv"
  fi
done

install_retry org.freedesktop.Platform//24.08 org.freedesktop.Sdk//24.08

echo '=== installed runtimes:'
flatpak list --user --runtime 2>/dev/null | head -20
# Record failures for the end-of-publish gate; apps needing a missing
# runtime fail (loudly) in the build step, but must not block the push —
# one broken upstream object can't hold the whole repo hostage.
[ -z "$failed" ] || echo "::error::runtime install failures:$failed"
echo "$failed" > /tmp/rt-failed.txt
