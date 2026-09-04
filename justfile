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
    @echo "Install the Dart SDK (3.0+): https://dart.dev/get-dart"
    @echo "Or skip that: reopen the repo in the 'Flutter/Dart track' devcontainer."

# devcontainer only: rebuild the home-manager profile (WORKSHOP_HOME_NIX is
# set by the variant devcontainers so their extra packages survive a rebuild)
# Goes through setup.sh rather than calling `home-manager switch` directly,
# so a rebuild that changes nothing skips the switch (see the guard there)
# instead of emptying the profile under a running rust-analyzer.
_rebuild:
    bash .devcontainer/setup.sh
