#!/usr/bin/env bash
# Workshop environment self-check.
#
# Verifies the REQUIRED toolchain (Rust + C + cbindgen) and reports on
# OPTIONAL language tracks (Swift, Kotlin/JNA, Python/cffi, Dart).
# Exit code is non-zero only when a REQUIRED tool is missing or broken —
# pick ONE optional track; you do not need them all.
#
# Usage: ./scripts/self-check.sh            (or: just check)
#        ./scripts/self-check.sh --track <swift|kotlin|python|dart>
#
# --track probes ONE optional track and says nothing else: exit 0 ready,
# exit 1 not. It exists for CI (.github/workflows/env-check.yml), which
# used to grep this script's human-facing labels — a reworded status line
# would flip a CI cell red with no real change. The exit code is the
# machine contract; the printed rows stay free to change.

set -u

GREEN=$'\033[0;32m'; RED=$'\033[0;31m'; YELLOW=$'\033[0;33m'; DIM=$'\033[2m'; NC=$'\033[0m'
PASS="${GREEN}✅${NC}"; FAIL="${RED}❌${NC}"; SKIP="${YELLOW}○${NC}"

required_failures=0

check_required() { # name, command, fix hint
  local name="$1" cmd="$2" hint="$3"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    printf ' %s %-10s missing — %s\n' "$FAIL" "$name" "$hint"
    required_failures=$((required_failures + 1))
    return
  fi
  # Capture before piping: a `| head` on the same line would report head's
  # exit status, not the tool's. A rustup shim with no default toolchain is
  # the classic "installed but not runnable" case this catches.
  local version
  if version=$("$cmd" --version 2>&1); then
    printf ' %s %-10s %s\n' "$PASS" "$name" "${DIM}$(printf '%s' "$version" | head -1)${NC}"
  else
    printf ' %s %-10s present but not runnable — %s\n' "$FAIL" "$name" "$hint"
    printf '%s\n' "$version" | head -3 | sed 's/^/      /'
    required_failures=$((required_failures + 1))
  fi
}

check_optional() { # track, command, install hint
  local track="$1" cmd="$2" hint="$3"
  if command -v "$cmd" >/dev/null 2>&1; then
    printf ' %s %-12s ready\n' "$PASS" "$track"
    return 0
  else
    printf ' %s %-12s not installed %s(only needed for this track — %s)%s\n' "$SKIP" "$track" "$DIM" "$hint" "$NC"
    return 1
  fi
}

# Each probe prints its own row and returns 0 ready / 1 not — the single
# source of truth for "is this track ready", shared by the full run below
# and by --track. Keep the printf rows free-form; only the return codes
# are contract.

# check_floor <required|optional> <label> <command> <sed-expr> <prefix> <floor> <what> <hint>
# Parses `<command> --version` with <sed-expr> (which must print the minor
# version), compares it to <floor>, and prints the verdict with <prefix> in
# front of the minor (1. for rustc, 0. for cbindgen). Merely existing is not
# enough for these tools: an old one passes the check above and then fails
# at the first real command. Parse failures skip silently — a tool that can't
# report a version was already flagged as broken by its existence check. A
# miss counts as a required failure, or returns 1 for an optional track.
check_floor() {
  local kind=$1 label=$2 cmd=$3 expr=$4 prefix=$5 floor=$6 what=$7 hint=$8 minor
  command -v "$cmd" >/dev/null 2>&1 || return 0
  minor=$("$cmd" --version 2>/dev/null | sed -nE "$expr")
  [ -n "$minor" ] || return 0
  if [ "$minor" -ge "$floor" ]; then
    printf ' %s %-10s %s%s meets the %s%s floor (%s)\n' "$PASS" "$label" "$prefix" "$minor" "$prefix" "$floor" "$what"
    return 0
  fi
  printf ' %s %-10s %s %s%s is older than %s%s (%s) — %s\n' "$FAIL" "$label" "$cmd" "$prefix" "$minor" "$prefix" "$floor" "$what" "$hint"
  if [ "$kind" = required ]; then
    required_failures=$((required_failures + 1))
  fi
  return 1
}

# `swiftc --version` instead of `command -v swiftc`: /usr/bin/swiftc is the
# same OS-image xcrun stub as /usr/bin/java — present even without the CLT,
# runnable only once a real toolchain is installed.
probe_swift() {
  if command -v swiftc >/dev/null 2>&1 && swiftc --version >/dev/null 2>&1; then
    printf ' %s %-12s ready\n' "$PASS" "Swift"
    return 0
  elif command -v swiftc >/dev/null 2>&1; then
    printf ' %s %-12s swiftc found, not runnable %s(macOS: install the CLT — run: just setup-swift)%s\n' "$SKIP" "Swift" "$DIM" "$NC"
  else
    printf ' %s %-12s not installed %s(only needed for this track — run: just setup-swift)%s\n' "$SKIP" "Swift" "$DIM" "$NC"
  fi
  return 1
}

# `java -version` instead of `command -v java`: macOS ships a /usr/bin/java
# stub that exists but exits 1 ("Unable to locate a Java Runtime") until a
# real JDK is linked — the keg-only case setup-kotlin's symlink hint covers.
probe_kotlin() {
  if command -v kotlinc >/dev/null 2>&1 && java -version >/dev/null 2>&1; then
    printf ' %s %-12s ready\n' "$PASS" "Kotlin/JNA"
    return 0
  elif command -v kotlinc >/dev/null 2>&1; then
    printf ' %s %-12s kotlinc found, java not runnable %s(macOS: link the keg-only JDK — re-run: just setup-kotlin for the command)%s\n' "$SKIP" "Kotlin/JNA" "$DIM" "$NC"
  else
    printf ' %s %-12s not installed %s(needs JDK 17+ and kotlinc — run: just setup-kotlin)%s\n' "$SKIP" "Kotlin/JNA" "$DIM" "$NC"
  fi
  return 1
}

# Prefer the repo-local venv even when it isn't activated: nix-shell prepends
# its own python3 to the inherited PATH, shadowing an activated .venv — the
# probe must not turn a completed `just setup-python` into a false "not ready".
probe_python() {
  local py venv_py activate
  # scripts/venv-python.sh owns the layout rule (bin/ vs Scripts/, .exe or
  # not) and prints python3 when there is no venv; the justfiles and the
  # Verify workflow ask it the same question.
  py="$("$(dirname "$0")/venv-python.sh")"
  venv_py=""
  [ "$py" != python3 ] && venv_py="$py"
  if command -v "$py" >/dev/null 2>&1; then
    if "$py" -c 'import cffi' 2>/dev/null; then
      if [ -n "$venv_py" ] && [ "$py" = "$venv_py" ] && [ -z "${VIRTUAL_ENV:-}" ]; then
        case "$venv_py" in
          */Scripts/*) activate=".venv/Scripts/activate" ;;
          *)           activate=".venv/bin/activate" ;;
        esac
        printf ' %s %-12s ready %s(cffi in .venv — activate: source %s)%s\n' "$PASS" "Python" "$DIM" "$activate" "$NC"
      else
        printf ' %s %-12s ready (cffi installed)\n' "$PASS" "Python"
      fi
      return 0
    fi
    printf ' %s %-12s python3 found, cffi missing %s(run: just setup-python, then: source .venv/bin/activate)%s\n' "$SKIP" "Python" "$DIM" "$NC"
  else
    printf ' %s %-12s not installed %s(python.org, 3.10+, then: just setup-python)%s\n' "$SKIP" "Python" "$DIM" "$NC"
  fi
  return 1
}

# The Dart floor is whatever the exercise pubspec pins (`sdk: ^3.x.0`) —
# read from there rather than copied here, so the number lives in one place.
probe_dart() {
  local pubspec floor
  check_optional "Dart" "dart" "run: just setup-dart" || return 1
  pubspec="$(dirname "$0")/../exercises/ex3-bindings/dart/pubspec.yaml"
  floor=$(sed -nE 's/^[[:space:]]*sdk: \^3\.([0-9]+)\..*/\1/p' "$pubspec" 2>/dev/null)
  [ -n "$floor" ] || return 0
  check_floor optional "Dart floor" dart 's/^Dart SDK version: 3\.([0-9]+)\..*/\1/p' "3." "$floor" "exercises/ex3-bindings/dart/pubspec.yaml" "https://dart.dev/get-dart"
}

# --track <name>: probe one optional track, exit with its status. Handled
# before any required checks so CI track cells cost one probe, not a full run.
if [ "${1:-}" = "--track" ]; then
  case "${2:-}" in
    swift)  probe_swift;  exit $? ;;
    kotlin) probe_kotlin; exit $? ;;
    python) probe_python; exit $? ;;
    dart)   probe_dart;   exit $? ;;
    *) echo "unknown track '${2:-}' — one of: swift kotlin python dart" >&2; exit 2 ;;
  esac
fi


echo "Workshop self-check — Using Advent of Code as an FFI Playground"
echo
echo "Required toolchain:"
check_required "rustc"    "rustc"    "nix shell provides it: direnv allow (no nix: https://rustup.rs)"
check_required "cargo"    "cargo"    "nix shell provides it: direnv allow (no nix: comes with rustup)"
check_required "cbindgen" "cbindgen" "nix shell provides it: direnv allow (no nix: cargo install cbindgen)"

# The day crates set a hard floor (edition 2024 needs rustc 1.85, resolver "3"
# needs cargo 1.84 — see days/): merely existing isn't enough, since an old
# channel's rustc passes the check above and then every cargo command in days/
# fails at manifest parse. Parse failures here skip silently — a rustc that
# can't even report a 1.x version was already flagged as broken above.
check_floor required "rust floor" rustc 's/^rustc 1\.([0-9]+)\..*/\1/p' "1." 85 "edition 2024" "rustup update stable (nix: newer channel)" || true

# cbindgen floor: every export in days/*/src/c_api.rs and Exercise 2's
# lib.rs is spelled #[unsafe(no_mangle)], the edition-2024 form, and cbindgen
# parses it only from 0.28 (0.26: "expected path"; 0.27: "expected
# identifier, found keyword unsafe"). Ubuntu 24.04 LTS packages 0.26, so a
# distro cbindgen passes the existence check above and then Exercise 2's
# build script dies at the header step. Same shape as the rust floor.
check_floor required "cbindgen floor" cbindgen 's/^cbindgen 0\.([0-9]+)(\..*)?$/\1/p' "0." 28 "#[unsafe(no_mangle)]" "cargo install cbindgen --locked (nix: newer channel)" || true

# just floor: the root justfile's `mod?` needs just 1.31 (README says so), and
# an older distro `just` cannot even parse it — the one required tool this
# script otherwise never looks at, because `just check` is how it is run.
check_floor required "just floor" just 's/^just 1\.([0-9]+)(\..*)?$/\1/p' "1." 31 "mod? in the justfile" "https://just.systems/man/en/installation.html (nix: newer channel)" || true

# C compiler: accept cc, clang, or gcc.
c_compiler=""
for candidate in cc clang gcc; do
  if command -v "$candidate" >/dev/null 2>&1; then c_compiler="$candidate"; break; fi
done
if [ -n "$c_compiler" ]; then
  printf ' %s %-10s %s\n' "$PASS" "C compiler" "${DIM}$($c_compiler --version 2>/dev/null | head -1)${NC}"
else
  printf ' %s %-10s missing — macOS: xcode-select --install · Linux: apt/dnf install gcc · Windows: use WSL2\n' "$FAIL" "C compiler"
  required_failures=$((required_failures + 1))
fi

# Functional smoke test: compile AND link a real executable, since a broken
# SDK path (common after macOS upgrades) passes `command -v` but fails here —
# and this workshop is about linking, so the linker is the part worth proving.
if [ -n "$c_compiler" ]; then
  if tmpdir=$(mktemp -d 2>/dev/null) && [ -n "$tmpdir" ]; then
    trap 'rm -rf "$tmpdir"' EXIT
    cat > "$tmpdir/smoke.c" <<'EOF'
#include <stdio.h>
int main(void) { printf("%d\n", 1 + 1); return 0; }
EOF
    if "$c_compiler" "$tmpdir/smoke.c" -o "$tmpdir/smoke" 2>"$tmpdir/err"; then
      printf ' %s %-10s compiles and links a C executable\n' "$PASS" "linker"
    else
      printf ' %s %-10s C compiler present but cannot build an executable:\n' "$FAIL" "linker"
      sed 's/^/      /' "$tmpdir/err" | head -3
      echo "      macOS fix: xcode-select --install (or: sudo xcode-select -r)"
      required_failures=$((required_failures + 1))
    fi
  else
    printf ' %s %-10s cannot create a temp dir — check TMPDIR and disk space (smoke test not run)\n' "$FAIL" "linker"
    required_failures=$((required_failures + 1))
  fi
fi

tracks_ready=0

echo
echo "Optional language tracks (pick ONE for Exercise 3):"
if probe_swift;  then tracks_ready=$((tracks_ready + 1)); fi
if probe_kotlin; then tracks_ready=$((tracks_ready + 1)); fi
if probe_python; then tracks_ready=$((tracks_ready + 1)); fi
if probe_dart;   then tracks_ready=$((tracks_ready + 1)); fi

# Track readiness shapes the banner only, never the exit code: one ready
# track is plenty, and an attendee with one track must never be blocked.
echo
if [ "$required_failures" -eq 0 ]; then
  if [ "$tracks_ready" -gt 0 ]; then
    echo "${GREEN}✅ You're ready for the workshop!${NC} (One ready track is plenty — the other ○ rows can stay grey.)"
  else
    echo "${GREEN}✅ Required toolchain ready (step -1 done).${NC} Step 0: pick ONE language track above and run its setup recipe, e.g. just setup-python"
  fi
  exit 0
else
  echo "${RED}❌ $required_failures required tool(s) missing.${NC} Fix the items above, then re-run: ./scripts/self-check.sh"
  exit 1
fi
