#!/bin/bash
set -u
sudo flatpak remote-add --if-not-exists --system flathub https://dl.flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true
for m in apps/*/*.yml apps/*/*.yaml apps/*/*.json; do
  [ -f "$m" ] || continue
  rt=$(grep -m1 "^runtime:" "$m" | sed "s/runtime:[[:space:]]*//; s/[\"']//g" | tr -d ' ')
  rv=$(grep -m1 "^runtime-version:" "$m" | sed "s/runtime-version:[[:space:]]*//; s/[\"']//g" | tr -d ' ')
  if [ -n "$rt" ] && [ -n "$rv" ]; then
    echo "Installing $rt//$rv"
    sudo flatpak install --system -y --noninteractive flathub "$rt//$rv" "$rt.Sdk//$rv" 2>&1 | tail -1
  fi
done
sudo flatpak install --system -y --noninteractive flathub org.freedesktop.Platform//24.08 org.freedesktop.Sdk//24.08 2>/dev/null | tail -1
