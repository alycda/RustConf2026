#!/usr/bin/env bash
# Print the path of a verified JNA jar, fetching it once if needed.
#
# The single home for the JNA pin. Every consumer — days/justfile's
# kotlin-demo, the exercises justfile's kotlin recipe, and the Verify
# workflow — calls this instead of carrying its own copy of the version,
# the hash, and the download.
#
# Order: $JNA_JAR when set and present (the kotlin devcontainer exports the
# nix-store jar); otherwise one cached copy under $XDG_CACHE_HOME, shared by
# every day and exercise. The download lands in a .tmp beside the cache —
# outside the repo, so an interrupted fetch leaves nothing for jj to
# snapshot — and is verified before it is moved into place, so a tampered
# or partial jar never becomes a cached one that later runs.
set -euo pipefail

version=5.17.0
sha256=b3a9408e7c51e08ef0e3bfcc08f443f6ec0f6191ba8cd7c18d53d2b22e5bdbc0

if [ -n "${JNA_JAR:-}" ] && [ -f "$JNA_JAR" ]; then
  printf '%s\n' "$JNA_JAR"
  exit 0
fi

cache="${XDG_CACHE_HOME:-$HOME/.cache}/ffi-playground"
jar="$cache/jna-${version}.jar"
if [ ! -f "$jar" ]; then
  mkdir -p "$cache"
  echo "fetching jna-${version}.jar from Maven Central into $cache" >&2
  curl -fsSL -o "$jar.tmp" \
    "https://repo1.maven.org/maven2/net/java/dev/jna/jna/${version}/jna-${version}.jar"
  # GNU coreutils has sha256sum (linux, Git-for-Windows bash, GitHub's macos
  # images); a bare Mac has only shasum.
  if command -v sha256sum >/dev/null 2>&1; then
    echo "${sha256}  ${jar}.tmp" | sha256sum -c - >/dev/null
  else
    echo "${sha256}  ${jar}.tmp" | shasum -a 256 -c - >/dev/null
  fi
  mv "$jar.tmp" "$jar"
fi
printf '%s\n' "$jar"
