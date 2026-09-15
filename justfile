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

# gfortran is deliberately not in shell.nix: ~102 MiB for one optional
# track, the same rule that keeps the C libraries behind `--arg full true`.
# Nor does any C toolchain an attendee already has bring it along — the
# Xcode CLT ships clang and no Fortran front end whatsoever, which is why
# the macOS recipe installs a compiler rather than pointing at one.

# Fortran track: gfortran, which arrives with brew's gcc (the CLT has none)
[macos]
setup-fortran:
    brew install gcc
    @echo "brew's gcc is what provides gfortran; then re-run: just check"

# Fortran track: distro package, or nix for a no-install shell
[linux]
setup-fortran:
    @echo "Debian/Ubuntu: sudo apt install gfortran · Fedora: sudo dnf install gcc-gfortran"
    @echo "Or install nothing system-wide: nix-shell -p gfortran (then run just from inside it)"
    @echo "then re-run: just check"
# R track: `Rscript` is the whole toolchain — no package manager step, no
# dependency to fetch. .C() is in base R.

# R track: R via brew (the formula is lowercase `r`, the command is `R`)
[macos]
setup-r:
    brew install r

# R track: CRAN publishes per-distro repositories — points at them
[linux]
setup-r:
    @echo "R from CRAN (distro repos, newer than what apt/dnf ship):"
    @echo "  https://cran.r-project.org/bin/linux/"
    @echo "Or borrow it for one command without installing anything:"
    @echo "  nix-shell -p R --run 'just days r-demo 2015-12-01'"

# Godot/GDExtension track: the editor binary, which is also the headless one
[macos]
setup-godot:
    brew install --cask godot
    @echo "Then re-run: just check (it verifies the 4.6 floor the .gdextension declares)"

# Godot track: distros disagree on the package name AND on which major it is
# — `godot` is 3.x on Debian and Fedora, where 4.x is `godot4` — so this
# points at the download rather than guessing. Nothing here needs the editor
# UI: the same binary runs the track headless.
[linux]
setup-godot:
    @echo "Install Godot 4.6+ from https://godotengine.org/download (the standard build, not .NET)"
    @echo "Your distro may package it as 'godot4'; scripts/godot-bin.sh finds either."
    @echo "Renamed it, or unzipped it somewhere off PATH? Point at it: export GODOT=/path/to/Godot_v4.7-stable_linux.x86_64"
    @echo "Or install nothing: reopen in the Godot devcontainer variant (.devcontainer/godot), which carries the engine."
    @echo "then: just check — it verifies the 4.6 floor the .gdextension declares."
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
# line differs per OS. The floor is engines.node in the exercise's package.json
# (22 today, what CI and the devcontainer run):
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
    floor="$(sed -nE 's/^[[:space:]]*"node":[[:space:]]*">=([0-9]+)".*/\1/p' exercises/ex4-wasm/wasm/package.json)"
    # `|| true` inside the substitution: with `set -eo pipefail` a missing
    # node exits the pipeline 127, an assignment takes the substitution's
    # status, and the recipe died here — on the one machine it is for —
    # before reaching the branch that says where to get Node.
    major="$({ node --version 2>/dev/null || true; } | sed -nE 's/^v([0-9]+)\..*/\1/p')"
    if [ -n "$major" ] && [ "$major" -ge "${floor:-22}" ]; then
        echo "node $(node --version) meets the ${floor:-22} floor (exercises/ex4-wasm/wasm/package.json)"
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
    floor="$(sed -nE 's/^[[:space:]]*"node":[[:space:]]*">=([0-9]+)".*/\1/p' exercises/ex4-wasm/wasm/package.json)"
    # `|| true` inside the substitution: with `set -eo pipefail` a missing
    # node exits the pipeline 127, an assignment takes the substitution's
    # status, and the recipe died here — on the one machine it is for —
    # before reaching the branch that says where to get Node.
    major="$({ node --version 2>/dev/null || true; } | sed -nE 's/^v([0-9]+)\..*/\1/p')"
    if [ -n "$major" ] && [ "$major" -ge "${floor:-22}" ]; then
        echo "node $(node --version) meets the ${floor:-22} floor (exercises/ex4-wasm/wasm/package.json)"
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
