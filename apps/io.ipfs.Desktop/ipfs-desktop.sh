#!/usr/bin/env bash

# Choose Wayland natively if in a Wayland session
if [ -n "${WAYLAND_DISPLAY}" ]; then
    export ELECTRON_OZONE_PLATFORM_HINT="auto"
fi

# Ensure IPFS_DESKTOP_EXEC points to this wrapper
export IPFS_DESKTOP_EXEC="/app/bin/ipfs-desktop"

# Set path to bundled Kubo (go-ipfs) executable
export IPFS_GO_EXEC="/app/ipfs-desktop/resources/app.asar.unpacked/node_modules/kubo/kubo/ipfs"

# Ensure /app/bin is in PATH
export PATH="/app/bin:${PATH}"

# Launch application via zypak-wrapper
exec zypak-wrapper /app/ipfs-desktop/ipfs-desktop "$@"
