#!/usr/bin/env bash
# Which binary is Godot, on this machine.
#
# Shared by days/justfile's godot-demo recipe and scripts/self-check.sh's
# probe_godot, so the answer is written once. Prints the name (or path) of a
# Godot 4 binary on stdout and exits 0; prints nothing and exits 1 when there
# isn't one.
#
# There is no single answer, which is the whole reason this file exists:
#
#   nixpkgs          installs godot, godot4 and godot4.7 — all one binary
#   Debian, Fedora   ship godot4 only; `godot` there is the 3.x package
#   Homebrew         installs godot as an .app; the CLI is godot-mono/godot
#   godotengine.org  hands you Godot_v4.7.2-stable_linux.x86_64 and expects
#                    you to name it yourself
#
# godot4 first, deliberately: a machine with both has 3.x under `godot`, and
# nothing in this repo works against 3.x. $GODOT beats everything — that is
# the escape hatch for the official download and for anyone testing a second
# engine version.
#
# Usage: godot="$(scripts/godot-bin.sh)" || { echo "no godot"; exit 1; }

set -u

if [ -n "${GODOT:-}" ]; then
  # An explicit override is not second-guessed beyond "does it run".
  if command -v "$GODOT" >/dev/null 2>&1; then
    printf '%s\n' "$GODOT"
    exit 0
  fi
  echo "\$GODOT is set to '$GODOT', which is not on PATH" >&2
  exit 1
fi

for candidate in godot4 godot; do
  if command -v "$candidate" >/dev/null 2>&1; then
    printf '%s\n' "$candidate"
    exit 0
  fi
done

exit 1
