#!/bin/bash
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true

strip() { local s=\$1; s=\${s#\"}; s=\${s%\"}; s=\${s#\'}; s=\${s%\'}; echo "\$s" | tr -d ' '; }

for m in apps/*/*.yml apps/*/*.yaml apps/*/*.json; do
  [ -f "$m" ] || continue
  rt=$(strip "$(grep -m1 "^runtime:" "$m" | cut -d: -f2-)")
  rv=$(strip "$(grep -m1 "^runtime-version:" "$m" | cut -d: -f2-)")
  if [ -n "$rt" ] && [ -n "$rv" ]; then
    sdk=$(echo "$rt" | sed "s/Platform/Sdk/")
    echo "Installing $rt//$rv + $sdk//$rv"
    flatpak install --user -y --noninteractive flathub "$rt//$rv" "$sdk//$rv" 2>&1 | tail -3
  fi
  bt=$(strip "$(grep -m1 "^base:" "$m" | cut -d: -f2-)")
  bv=$(strip "$(grep -m1 "^base-version:" "$m" | cut -d: -f2-)")
  if [ -n "$bt" ] && [ -n "$bv" ]; then
    echo "Installing base $bt//$bv"
    flatpak install --user -y --noninteractive flathub "$bt//$bv" 2>&1 | tail -3
  fi
done

flatpak install --user -y --noninteractive flathub org.freedesktop.Platform//24.08 org.freedesktop.Sdk//24.08 2>&1 | tail -1

echo '=== installed runtimes:'
flatpak list --user --runtime 2>/dev/null | head -20
exit 0
