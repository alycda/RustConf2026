# just >= 1.31 required: `mod? days` (1.31) and `shell()` (1.27) below both
# predate what apt/dnf ship — the README's manual path says so too.

# Containers start with USER unset — .devcontainer/setup.sh defaults it to
# root for the same reason. Recipes below shell out to tools that read it,
# so derive it here and they behave the same in and out of the container.
export USER := shell("whoami")

# the first recipe is the default
_default:
    @just --list

# list the cheatsheets for the FFI patterns we'll hit
cheats:
    cheat -l

# the AoC day library: scaffold and run days, e.g. `just days new 2022-12-01`
# (`mod?`: the module activates when days/justfile lands, so this file is
# valid in trees that predate the day library)
mod? days

# (`mod?` for the same reason as days above)
# the workshop exercises: `just exercises ex2`, `just exercises python`, …
mod? exercises

# verify required toolchain + optional tracks (always exits 0; CI: run scripts/self-check.sh)
check:
    -@./scripts/self-check.sh

# asciinema is deliberately not in shell.nix (my rig, not the workshop's
# contract), so the recipe borrows it when absent.

# Presenter only: replay the recorded Module 2 demo — the demo-gods fallback for the Live slide
demo:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v asciinema >/dev/null; then
        exec nix-shell -p asciinema --run "asciinema play -i 2 docs/demo/module2.cast"
    fi
    asciinema play -i 2 docs/demo/module2.cast

# Language-track setup (Exercise 3 — pick ONE track; see `just check`).
# Required Rust/C toolchain comes from shell.nix, not from these recipes.

# Python track: repo-local venv with cffi
setup-python:
    python3 -m venv .venv
    ./.venv/bin/python -m pip install --upgrade pip cffi
    @echo "Done. Activate with: source .venv/bin/activate — then re-run: just check"

# Run test, not `command -v`: the OS-image xcrun stub at /usr/bin/swiftc
# exists even without the CLT.

# Swift track: toolchain via Xcode CLT
[macos]
setup-swift:
    @swiftc --version >/dev/null 2>&1 && echo "swiftc already installed" || xcode-select --install

# Swift track: no unattended installer on Linux — points at swift.org
[linux]
setup-swift:
    @echo "Install the Swift toolchain from https://www.swift.org/install/"

# Kotlin/JNA track: JDK + kotlinc via brew (keg-only JDK needs the symlink)
[macos]
setup-kotlin:
    brew install openjdk kotlin
    @echo "brew's openjdk is keg-only; link it so 'java' resolves:"
    @echo "  sudo ln -sfn $(brew --prefix)/opt/openjdk/libexec/openjdk.jdk /Library/Java/JavaVirtualMachines/openjdk.jdk"
    @echo "then re-run: just check"

# Kotlin/JNA track: sdkman is the recommended path on Linux
[linux]
setup-kotlin:
    @echo "Recommended: sdkman — https://sdkman.io/install then:"
    @echo "  sdk install java 17-tem && sdk install kotlin"
    @echo "Or skip all that: reopen the repo in the 'Kotlin/JNA track' devcontainer."

# Homebrew ≥6 refuses formulae from untrusted third-party taps, hence the
# `brew trust`; its `-` prefix keeps older brews (no trust subcommand) working.

# Dart track: SDK via the official brew tap
[macos]
setup-dart:
    brew tap dart-lang/dart
    -brew trust dart-lang/dart
    brew install dart

# Dart track: distro installs vary — points at dart.dev
[linux]
setup-dart:
    @echo "Install the Dart SDK: https://dart.dev/get-dart (then: just check — it verifies the version floor)"
    @echo "Or skip that: reopen the repo in the 'Flutter/Dart track' devcontainer."

# wasm track (Exercise 4): the wasm32 target for a rustup toolchain, and Node.
# Two halves because they come from two places. The target's std is rustc's
# to install — `rustup target add` on the manual path; the nix shell's rustc
# has it built in, so under nix that half is a no-op and says so. Node is the
# track's own install, kept out of shell.nix on purpose (45 MiB nobody on
# another track needs); the wasm devcontainer carries it, and so does brew.
# wasm-bindgen-cli is only for the generated lap (days/2015-12-01) and the
# recipe there names the exact version to install if you want it.
#
# The target half is one recipe both OS variants depend on; only the Node
# line differs per OS. The floor is 22 (what CI and the devcontainer run):
# an older node on PATH is not "done", it is the case the self-check will
# fail next, so the recipe reads the major version rather than the presence.

# the wasm32 target's std: rustup installs it; the nix shell's rustc has it
_setup-wasm-target:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v rustup >/dev/null 2>&1; then
        rustup target add wasm32-unknown-unknown
    else
        echo "no rustup — assuming the nix shell's rustc, which has wasm32-unknown-unknown built in"
    fi

# wasm track: rustup's wasm32 target (no-op under nix) + Node 22 via brew (keg-only: prints the link step)
[macos]
setup-wasm: _setup-wasm-target
    #!/usr/bin/env bash
    set -euo pipefail
    major="$(node --version 2>/dev/null | sed -nE 's/^v([0-9]+)\..*/\1/p')"
    if [ -n "$major" ] && [ "$major" -ge 22 ]; then
        echo "node $(node --version) meets the 22 floor"
    else
        brew install node@22
        # A versioned formula: Homebrew installs it unlinked, so `node` is
        # still the old one (or nothing) until it is put on PATH.
        echo "brew's node@22 is keg-only; link it so 'node' resolves:"
        echo "  brew link --overwrite node@22"
        echo "or put it first on PATH: export PATH=\"$(brew --prefix)/opt/node@22/bin:\$PATH\""
    fi
    echo "then re-run: just check"

# wasm track: rustup's wasm32 target (no-op under nix) + a Node 22 pointer
[linux]
setup-wasm: _setup-wasm-target
    #!/usr/bin/env bash
    set -euo pipefail
    major="$(node --version 2>/dev/null | sed -nE 's/^v([0-9]+)\..*/\1/p')"
    if [ -n "$major" ] && [ "$major" -ge 22 ]; then
        echo "node $(node --version) meets the 22 floor"
    else
        echo "Install Node 22 LTS: https://nodejs.org (or your distro's nodejs package, if it is 22+)"
        echo "Or skip that: reopen the repo in the 'wasm track' devcontainer."
    fi
    echo "then re-run: just check"

# devcontainer only: rebuild the home-manager profile (WORKSHOP_HOME_NIX is
# set by the variant devcontainers so their extra packages survive a rebuild)
# Goes through setup.sh rather than calling `home-manager switch` directly,
# so a rebuild that changes nothing skips the switch (see the guard there)
# instead of emptying the profile under a running rust-analyzer.
_rebuild:
    bash .devcontainer/setup.sh
