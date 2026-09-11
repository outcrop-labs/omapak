#!/bin/sh
# omapak setup: one remote for everything.
#
#   curl -fsSL https://omapak.org/omapak.sh | sh
#
# What it does:
#   1. adds the omapak remote (omapak apps + the whole flathub catalog,
#      cached and served by us, updates included) and refreshes its
#      signing key — remote-add --if-not-exists never updates the keyring
#      of an existing remote, so re-running this after a key rollover
#      heals the install
#   2. if you have a flathub remote, moves your installed apps to omapak
#      origin (no re-downloads) and removes the flathub remote
#   3. prints where you stand
#
# Flags:
#   --keep-flathub   add omapak, leave flathub alone
#   --dry-run        show what would happen, change nothing
set -eu

FLATPAKREPO=https://repo.omapak.org/omapak.flatpakrepo
REMOTE=omapak
KEEP_FLATHUB=0
DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --keep-flathub) KEEP_FLATHUB=1 ;;
    --dry-run) DRY_RUN=1 ;;
    *) echo "unknown flag: $arg"; exit 1 ;;
  esac
done

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }

command -v flatpak >/dev/null 2>&1 || {
  echo "flatpak not found. install it first: https://flatpak.org/setup/"
  exit 1
}

run() {
  if [ "$DRY_RUN" = 1 ]; then
    printf '  (dry-run) %s\n' "$*"
  else
    "$@" >/dev/null 2>&1 || printf '  (skipped: %s)\n' "$*"
  fi
}

have_sudo() { command -v sudo >/dev/null 2>&1; }

# --- 1. add the omapak remote -------------------------------------------------
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
curl -fsSL "$FLATPAKREPO" -o "$tmpdir/omapak.flatpakrepo"
# -f2- keeps any '=' padding inside the base64 blob intact
grep '^GPGKey=' "$tmpdir/omapak.flatpakrepo" | cut -d= -f2- | base64 -d > "$tmpdir/key.gpg"
REMOTE_URL=$(grep '^Url=' "$tmpdir/omapak.flatpakrepo" | cut -d= -f2-)

say "adding the $REMOTE remote"
run flatpak remote-add --user --if-not-exists "$REMOTE" "$tmpdir/omapak.flatpakrepo"
# Key refresh for remotes added before a signing-key rollover, and URL
# correction for remotes pointing somewhere else (e.g. dl.flathub.org).
run flatpak remote-modify --user --gpg-import="$tmpdir/key.gpg" --url="$REMOTE_URL" "$REMOTE"
if [ "$(id -u)" = 0 ]; then
  run flatpak remote-add --system --if-not-exists "$REMOTE" "$tmpdir/omapak.flatpakrepo"
  run flatpak remote-modify --system --gpg-import="$tmpdir/key.gpg" --url="$REMOTE_URL" "$REMOTE"
elif have_sudo; then
  run sudo flatpak remote-add --system --if-not-exists "$REMOTE" "$tmpdir/omapak.flatpakrepo"
  run sudo flatpak remote-modify --system --gpg-import="$tmpdir/key.gpg" --url="$REMOTE_URL" "$REMOTE"
else
  echo "  (no sudo: added for your user only; system installs will need it later)"
fi

# --- 2. migrate flathub-origin installs to omapak -----------------------------
# Full refs from --all: bare names match multiple branches ("no ref chosen"),
# and Locale/GL sub-refs only surface with --all.
if [ "$KEEP_FLATHUB" = 0 ]; then
  for inst in --user --system; do
    refs=$(flatpak "$inst" list --all --columns=ref,origin 2>/dev/null | awk -F'\t' '$2=="flathub"{print $1}')
    [ -n "$refs" ] || continue
    say "moving $inst installs from flathub to $REMOTE origin"
    for ref in $refs; do
      run flatpak "$inst" install -y --reinstall --or-update "$REMOTE" "$ref"
    done
  done

  for inst in --user --system; do
    if flatpak "$inst" remotes 2>/dev/null | grep -qx flathub; then
      say "removing the flathub remote ($inst)"
      case "$inst" in
        --user) run flatpak remote-delete --user flathub ;;
        --system)
          if [ "$(id -u)" = 0 ]; then run flatpak remote-delete --system flathub
          elif have_sudo; then run sudo flatpak remote-delete --system flathub
          else echo "  (no sudo: remove it yourself with: sudo flatpak remote-delete --system flathub)"
          fi ;;
      esac
    fi
  done
fi

# --- 3. status ----------------------------------------------------------------
say "you're on omapak"
flatpak remotes 2>/dev/null | sed 's/^/  /'
count=$(flatpak --system remote-ls "$REMOTE" 2>/dev/null | wc -l | tr -d ' ')
[ "$count" -gt 0 ] 2>/dev/null || count=$(flatpak --user remote-ls "$REMOTE" 2>/dev/null | wc -l | tr -d ' ')
echo "  $REMOTE serves $count refs: omapak apps plus the flathub catalog"
echo "  try: flatpak install $REMOTE org.blender.Blender"
