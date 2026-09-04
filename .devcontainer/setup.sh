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

# Add the home-manager channel once. An unconditional `nix-channel --update`
# here pulled a fresh master on every rebuild, so the generation always
# differed and the skip-if-unchanged guard below could never fire. Updating
# home-manager is now deliberate: nix-channel --update home-manager, then
# `just _rebuild`.
if ! nix-channel --list | grep -q '^home-manager '; then
  nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
  nix-channel --update home-manager
fi

# Install home-manager
nix-shell '<home-manager>' -A install

# Apply the configuration from this repo. Variant devcontainers (jj, kotlin,
# flutter) set WORKSHOP_HOME_NIX via containerEnv to their own home.nix, which
# imports the shared one below and adds packages.
HOME_NIX="${WORKSHOP_HOME_NIX:-${SCRIPT_DIR}/home.nix}"

# Only switch if the generation would actually change. A switch is not atomic
# from the outside: it unlinks the old home-manager path before adding the new
# one, so for a few seconds ~/.nix-profile/bin holds neither rustc nor cargo.
# Anything that probes for a toolchain in that window sees it missing and, in
# rust-analyzer's case, gives up on the workspace for good — it does not
# re-probe, so days/ stays unloaded until someone restarts the server by hand.
# That is why every devcontainer.json runs this script as onCreateCommand,
# which completes before the extension host starts, rather than as
# postCreateCommand, which runs beside it. A no-op switch opens that window
# for nothing, and this script re-runs whenever the container is rebuilt and
# whenever `just _rebuild` is invoked by hand.
#
# `home-manager build` is a plain nix-build: it evaluates the same generation
# and writes a result symlink without touching the profile. If that path is
# already the live one, there is nothing to install.
hm_profile=""
for candidate in \
    "${XDG_STATE_HOME:-${HOME}/.local/state}/nix/profiles/home-manager" \
    "/nix/var/nix/profiles/per-user/${USER}/home-manager"; do
  if [ -e "$candidate" ]; then hm_profile="$candidate"; break; fi
done

hm_wanted=""
build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT
# A build failure here is not fatal: fall through to the switch and let it
# report the real error, rather than swallowing a broken home.nix.
if (cd "$build_dir" && home-manager build -f "$HOME_NIX") >/dev/null 2>&1; then
  hm_wanted="$(readlink -f "${build_dir}/result" 2>/dev/null || true)"
fi
hm_current=""
if [ -n "$hm_profile" ]; then
  hm_current="$(readlink -f "$hm_profile" 2>/dev/null || true)"
fi

if [ -n "$hm_wanted" ] && [ "$hm_wanted" = "$hm_current" ]; then
  echo "home-manager: generation unchanged, skipping switch"
else
  home-manager switch -b backup -f "$HOME_NIX"
fi

# Allow direnv for this template repo (if it has .envrc)
if [ -f "${WORKSPACE_DIR}/.envrc" ]; then
    cd "${WORKSPACE_DIR}"
    direnv allow
fi
