# Flake handoff: what landed, and what a full port still needs

This branch pins the environment and adds per-track dev shells. It
deliberately stops short of a full migration: CI, the devcontainers and the
`.vscode` tasks all still call `nix-shell`, and they still work, because
`shell.nix` was kept as a shim onto the same definitions rather than deleted.

That leaves a seam. This document is the seam, written down.

## What landed

`nix/shells.nix` is now the single definition of every shell. `flake.nix`
and `shell.nix` are two entry points onto it, and `shell.nix` reads the
pinned revision out of `flake.lock`, so neither path can drift from the
other.

| shell | `nix develop` | `nix-shell` | was |
|---|---|---|---|
| workshop toolchain | `nix develop` | `nix-shell` | `nix-shell` |
| \+ the nine C libraries | `nix develop .#full` | `nix-shell --arg full true` | `nix-shell --arg full true` |
| \+ Python track | `nix develop .#python` | `nix-shell -A python` | `just setup-python` |
| \+ Kotlin track | `nix develop .#kotlin` | `nix-shell -A kotlin` | `just setup-kotlin` |
| \+ Swift track | `nix develop .#swift` | `nix-shell -A swift` | `just setup-swift` |
| \+ Dart track | `nix develop .#dart` | `nix-shell -A dart` | `just setup-dart` |

`--arg full true` is preserved verbatim and maps to the `full` shell. This
was checked rather than assumed: `nix-instantiate --arg full true` and
`nix-instantiate -A full` produce the **same derivation hash**, so
`.github/workflows/rust.yml`'s `ffi` job needed no edit and is running the
same environment it was.

The `just setup-*` recipes are untouched and remain the answer off Nix.

## Verified on this branch

Against the pinned nixpkgs (`nixos-25.05`, `ac62194c`), on x86_64-linux:

- All six shells evaluate; `nix flake show` yields six on each of
  x86_64-linux, aarch64-linux, x86_64-darwin and aarch64-darwin.
- `nix-shell`, `nix-shell --arg full true` and `nix-shell -A <track>` all
  work; a bare `nix-shell` still returns a derivation (see the note in
  `shell.nix` — an attribute set fails with "requires a single derivation"
  and would have broken CI and `.vscode/tasks.json`).
- The `ffi` job's pkg-config roll call passes in the pinned full shell —
  caca 0.99.beta20, libtcc 0.9.27, libhs 5.4.11, icu 76.1, chipmunk 7.0.3,
  duckdb 1.2.2, yara 4.5.2, espeak-ng 1.51.1 — including both `.pc` files
  `nix/shells.nix` synthesizes.
- The chipmunk darwin tripwire still fires correctly: glfw is still in
  `buildInputs` at the pinned revision, so the override is still needed and
  the `warnIf` stays silent.
- Track floors land at Python 3.12.12 + cffi 1.17.1, JDK 17.0.17, Dart
  3.7.3, Swift 5.8; rustc is 1.86.0 against the 1.85 floor.

Not verified here: macOS (everything above is Linux), and the Darwin
`tinycc` override, which by construction only runs there.

## What a full port still needs

### 1. Decide whether CI should say `nix develop`

`.github/workflows/rust.yml`'s `ffi` job currently runs:

```yaml
- uses: cachix/install-nix-action@v31
  with:
    nix_path: nixpkgs=channel:nixpkgs-unstable
- run: nix-shell ../shell.nix --arg full true --run '…'
```

That still works and is now pinned, which is an improvement nobody asked
for: the job used to track `nixpkgs-unstable` and now tracks `flake.lock`.
Two loose ends follow from it.

The `nix_path:` line is dead weight — `shell.nix` no longer reads
`<nixpkgs>`. It is harmless, but it reads as though the channel still
matters.

More substantially, the job's comment says it proves "the exact environment
an attendee gets". That is now more true than it was, and the honest version
of the job would say `nix develop .#full` so the flake path is the one under
test. Switching means `install-nix-action` needs
`extra_conf: experimental-features = nix-command flakes` (or the
`nix_path` line swapped for it).

### 2. `.github/workflows/env-check.yml` still provisions tracks by hand

The 17-cell verify matrix installs Dart via `dart-lang/setup-dart`, builds a
venv for Python, and relies on the stock runner images shipping `kotlinc`
and Swift — with the provisioning blocks for those two commented out and
marked ON ICE against the day an image drops them.

Every one of those cells has a pinned shell available now. `nix develop
.#kotlin` cannot go red because GitHub changed a runner image. Converting
the matrix is the single biggest remaining win and the single biggest diff;
it was left out of this branch because it is a CI rewrite, not a flake.

Note the constraint if you do it: that workflow's whole purpose is mapping
what *stock* machines have, so at least one row should keep probing the
bare image. Pinning every cell would answer a different question than the
one the workflow was built to ask.

### 3. The devcontainers still install Nix + home-manager

`.devcontainer/setup.sh` runs `nix-shell '<home-manager>' -A install` and
`.devcontainer/home.nix` builds the profile. Those are independent of this
change and still work. The Kotlin, Swift and Flutter/Dart container variants
now duplicate what `nix develop .#<track>` does — a container variant could
become a one-line `nix develop` instead of its own package list, which would
delete a fair amount of `.devcontainer/`.

### 4. `full` and a language track cannot be combined

The track shells build on the workshop shell, not on `full`, because
Exercise 3 calls a day's cbindgen C API and needs no system library. If
somebody genuinely needs both, the shells are a `genAttrs` away from a
`full-python`/`full-kotlin`/… cross product in `nix/shells.nix`. It was left
out because nothing in the workshop wants it and it doubles what
`nix flake show` prints.

### 5. Re-measure the download split

The size figures in `nix/shells.nix`'s header (726.5 MiB default against
1.4 GiB full, espeak-ng being ~600 MiB of the difference) were measured on
`nixpkgs-unstable` on 2026-09-06, *before* this pin existed. They are the
reason the default/full split exists at all, so they are worth re-taking
against the pinned revision on a cold store — the numbers here could not be
re-measured because this machine's store already held most of the default
shell.

### 6. The version floors Java and Python still lack

`scripts/self-check.sh` checks floors for rustc, `cbindgen`, `just` and Dart
(the Dart one reads its number out of `exercises/ex3-bindings/dart/pubspec.yaml`,
which is the right pattern). Java and Python have no floor check, which is
what makes `command -v java` report ready against a Java 8.

The pin fixes this on the Nix path only, and the script now says which of
the two situations you are in — "pinned by the … dev shell" versus "found on
PATH — version not checked". Adding two more `check_floor` calls would close
it off Nix as well. That is a deliberate open choice, not an oversight: the
argument for pinning was partly *not* to write those parsers.
