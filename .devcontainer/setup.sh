#!/usr/bin/env bash
set -euo pipefail

# Install locales only if they aren't already present
if ! dpkg -s locales >/dev/null 2>&1; then
  apt-get update
  apt-get install -y --no-install-recommends locales
fi

# Enable and generate en_US.UTF-8 (idempotent)
if ! grep -q '^en_US.UTF-8 UTF-8' /etc/locale.gen; then
  echo 'en_US.UTF-8 UTF-8' >> /etc/locale.gen
fi
locale-gen en_US.UTF-8
update-locale LANG=en_US.UTF-8

# Export for the current shell (helps the script itself)
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Set USER if not already set (common in containers)
export USER=${USER:-root}
export HOME=${HOME:-/root}

# Get the directory where this script lives
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(dirname "$SCRIPT_DIR")"

# Add Nix channels
nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
nix-channel --update

# Install home-manager
nix-shell '<home-manager>' -A install

# Apply the configuration from this repo. Variant devcontainers (jj, kotlin,
# flutter) set WORKSHOP_HOME_NIX via containerEnv to their own home.nix, which
# imports the shared one below and adds packages.
home-manager switch -b backup -f "${WORKSHOP_HOME_NIX:-${SCRIPT_DIR}/home.nix}"

# Allow direnv for this template repo (if it has .envrc)
if [ -f "${WORKSPACE_DIR}/.envrc" ]; then
    cd "${WORKSPACE_DIR}"
    direnv allow
fi