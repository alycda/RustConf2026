---
title: nix-shell fails on aarch64-darwin — chipmunk pulls glfw2, tinycc ships a broken .pc and dylib id, libtcc races on first use
date: 2026-09-07
category: build-errors
module: shell.nix
problem_type: build_error
component: tooling
symptoms:
  - "Refusing to evaluate package 'glfw-2.7.9' ... because it is not available on the requested hostPlatform: aarch64-darwin"
  - "clang: error: no such file or directory: '/0sra2y18lr3h6j58qjm0w46yv36h1wjmilb09n8aimdpivdymscx/lib'"
  - "dyld: Library not loaded: @rpath/libtcc.dylib"
  - "2015-12-01 tcc tests die with SIGSEGV or SIGTRAP on macos-latest, intermittently, only in the first cargo test after a build"
root_cause: config_error
resolution_type: config_change
severity: high
related_components: [testing_framework, development_workflow]
tags: [nix, nixpkgs, aarch64-darwin, macos, chipmunk, tinycc, pkg-config, ci-matrix]
---

# nix-shell fails on aarch64-darwin — chipmunk pulls glfw2, tinycc ships a broken .pc and dylib id, libtcc races on first use

## Problem

An attendee's first `nix-shell` on an Apple Silicon Mac died at evaluation
(GitHub issue #1). Fixing that exposed three more darwin defects behind it,
all in nixpkgs packages this shell depends on, plus the CI gap that let all
four reach an attendee. None of them are Nix-the-tool bugs; the reporter's
Determinate install was irrelevant.

## Symptoms

Peeling in order, each one visible only after the previous was fixed:

1. `nix-shell` refuses to evaluate at all: `Refusing to evaluate package
   'glfw-2.7.9' ... not available on the requested hostPlatform:
   aarch64-darwin`. The trace points at chipmunk's buildInputs.
2. The shell realises, `pkg-config --modversion` passes for every library,
   then linking 2015-12-01 with `--features tcc` fails: `clang: error: no
   such file or directory: '/0sra2y18…/lib'`. That path is a 52-character
   Nix output placeholder, not a store path.
3. Linking succeeds, the test binary aborts at load: `dyld: Library not
   loaded: @rpath/libtcc.dylib`.
4. Loading succeeds, the tests crash with SIGSEGV once and SIGTRAP the next
   time, only on the 3-core macos-latest runner and only when cargo runs the
   freshly built binary itself. Roughly 200 loops of an already-built binary
   on the runner and locally never crashed.

## What Didn't Work

- **Reproducing in a fresh tart macOS VM.** The Sequoia base image is
  25 GB compressed and the link was doing 1.2 MB/s, a six-hour pull. Dropped
  once a local `nix-instantiate shell.nix --show-trace` on this Mac produced
  the identical refusal from a different nixpkgs revision, which pinned the
  bug to the nixpkgs expression rather than the installer.
- **Assuming the unmerged shell-slimming branch fixed it.** That branch moves
  the C libraries behind `--arg full true`, so the default shell stops
  evaluating chipmunk and the reporter's command works. `--arg full true`
  still failed identically, and that is what CI runs. Mitigation, not fix.
- **Blaming parallel tests for the crash.** Looping the test binary 60 times
  in parallel, single-threaded and two-threaded modes on the runner gave
  zero crashes. The crash only ever happened in the position where cargo
  executes the just-built binary, which pointed at a first-use race rather
  than steady-state contention.
- **Selecting the test executable by `.target.name` in
  `cargo --message-format=json`.** Cargo names a lib target with underscores
  (`aoc_2015_12_01`) and the bin with dashes (`aoc-2015-12-01`), so the
  selector only ever found the bin's empty test binary. Match on
  `.target.src_path` ending in `src/lib.rs` instead.
- **Stale build-script output masquerading as a failed fix.** After the
  `.pc` fix, a warm target dir still fed clang the old placeholder path
  because `build.rs` declared only `rerun-if-changed=build.rs`. Verify a
  pkg-config-driven fix after `cargo clean -p <crate>`, or fix the trigger
  (see Prevention).

## Solution

All of it lives on branch `ci/ffi-nix-macos` as separate commits. The
diagnostic experiments are on `ci/tcc-stress`.

**1. chipmunk without its demo.** nixpkgs' `chipmunk` declares
`platforms = unix` but lists `glfw2` (`platforms = linux`), `libglut` and the
X11 stack in buildInputs, all for the `chipmunk_demos` binary and none for
libchipmunk. Chipmunk's CMakeLists has `BUILD_DEMOS`:

```nix
chipmunk = prev: prev.chipmunk.overrideAttrs (old: {
  buildInputs = [ ];
  cmakeFlags = (old.cmakeFlags or [ ]) ++ [ "-DBUILD_DEMOS=OFF" ];
  postInstall = "";
});
```

**2. tinycc's libtcc.pc with real paths.** `makePkgconfigItem` rewrites only
`placeholder "out"` into a substitutable `@out@`; tinycc's item also uses
`placeholder "lib"` and `placeholder "dev"`, which survive into the installed
file as raw hashes. `copyPkgconfigItems` runs `substituteAllInPlace`, so the
`@lib@`/`@dev@` forms do resolve against the output variables:

```nix
pkgconfigItems = [
  (prev.makePkgconfigItem {
    name = "libtcc";
    inherit (old) version;
    description = "Tiny C compiler backend";
    cflags = [ "-I\${includedir}" ];
    libs = [ "-L\${libdir}" "-Wl,-rpath,\${libdir}" "-ltcc" ];
    variables = { prefix = "@out@"; includedir = "@dev@/include"; libdir = "@lib@/lib"; };
  })
];
```

The rpath is one token on purpose: upstream's `-Wl,--rpath <path>` is two,
and `build.rs` splits on whitespace, which is how clang was handed a bare
path as an input file.

**3. libtcc.dylib's install name.** tinycc does not run
`fixDarwinDylibNames`, so the dylib id is `@rpath/libtcc.dylib` instead of
the store path every other nix dylib carries. Add the hook on darwin:

```nix
nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ prev.fixDarwinDylibNames ];
```

**4. A mutex around the JIT in the Rust wrapper.** libtcc's compile lock is
created lazily behind an unsynchronised flag on every platform (`tcc.h`,
`wait_sem`: `if (!p->init) <create>, p->init = 1;`). Two threads compiling
for the first time in a process both see the flag unset and both enter the
preprocessor; lldb caught two test threads stopped in `tccpp_new` at the same
instant. The wrapper owns the exclusion:

```rust
static JIT: Mutex<()> = Mutex::new(());

pub fn call_i32_fn(source: &str, symbol: &str) -> miette::Result<i32> {
    let c_source = CString::new(source).map_err(/* … */)?;
    let c_symbol = CString::new(symbol).map_err(/* … */)?;
    let _jit = JIT.lock().unwrap_or_else(PoisonError::into_inner);
    // tcc_new … tcc_delete, with both strings already validated so no
    // early return leaks the TCCState
}
```

**5. Overlay, darwin-only tinycc, tripwires.** The overrides are applied via
`nixpkgs.extend` so `pkgs.chipmunk` and `pkgs.tinycc` are the fixed
derivations everywhere in the file, rather than let-bindings that shadow
them inside `with pkgs;`. The tinycc override is darwin-only because Linux
tolerates the bad `.pc` through the cc wrapper and a source build there costs
about 65 s. Each override sits behind `lib.warnIf` on the bug itself, so the
warning fires the day nixpkgs fixes the package:

```nix
lib.warnIf (!lib.any (d: (d.pname or "") == "glfw") prev.chipmunk.buildInputs)
  "shell.nix: nixpkgs' chipmunk no longer depends on glfw2; drop the override"
  (darwinFixes.chipmunk prev)
```

Reading `pname` off glfw2 is safe on darwin; the platform check guards
`outPath`, not the attrset.

**6. CI.** The `ffi` job that runs `nix-shell` gained `macos-latest` in its
matrix with `fail-fast: false`. The matrix commit was pushed alone first and
went red with the reporter's exact error, so the history shows CI catching
the bug before it shows the fix.

## Why This Works

Each symptom had a distinct owner. Nix refuses a whole derivation when any
input is unsupported on the host, so one linux-only demo dependency took the
shell down. A pkg-config file with a bogus `-L` is harmless under nixpkgs'
wrapped GNU ld, which already has the real path and consumes the stray token
as `--rpath`'s argument, and fatal under clang, which validates input files
in the driver. A dylib with an `@rpath` id links but cannot be found at load
without an rpath in the executable. A first-use race in a lazily initialised
lock reproduces on a fresh process under a cold cache and vanishes in a warm
loop. The CI gap was the meta-cause: the only job that evaluated `shell.nix`
was Linux-only by explicit choice, and the README's first option is
macOS + nix.

## Prevention

- **Every job that proves an environment runs where the users are.** If the
  README's primary path is macOS, a Linux-only check of that path is untested.
- **Fail-first when widening CI.** Push the matrix change alone, watch it go
  red for the right reason, then push the fix. It costs one run and buys a
  reproduction that isn't your laptop.
- **pkg-config-driven `build.rs` must list its environment.** Declaring
  `rerun-if-changed=build.rs` alone disables cargo's default and replays
  cached link flags across nix shells. Emit `rerun-if-env-changed` for
  `PATH`, `PKG_CONFIG`, `PKG_CONFIG_PATH`, `PKG_CONFIG_LIBDIR` and
  `PKG_CONFIG_SYSROOT_DIR`, as 2021-12-02 and 2023-12-01 already did.
- **Overrides of upstream bugs carry a tripwire.** On an unpinned channel an
  `overrideAttrs` changes the hash forever; `lib.warnIf` on the defect itself
  says when to delete it. Prefer a condition over a version check for
  packages that rarely bump.
- **Wrap non-thread-safe C in the Rust layer, not in the test runner.**
  `--test-threads=1` would have hidden the race from every attendee who ran
  `cargo test`; the mutex is the FFI lesson.
- **Diagnose an intermittent crash where it happens.** Set
  `CARGO_TARGET_<TRIPLE>_RUNNER` to a script that wraps the binary in
  `lldb --batch -o run -k 'thread backtrace all' -k 'quit 99'`, and let the
  real `cargo test` command produce the backtrace.
- **Verify pkg-config fixes on a clean target.** `cargo clean -p <crate>`
  first, or the stale-flag replay looks like the fix did nothing.
- **Upstream candidates:** nixpkgs chipmunk (demo deps unconditional, or add
  a `withDemos` toggle), nixpkgs tinycc (`@lib@`/`@dev@` in the item,
  `fixDarwinDylibNames`), tinycc itself (`wait_sem` lazy init).

## Related Issues

- https://github.com/alycda/RustConf2026/issues/1
- Branch `ci/ffi-nix-macos` (fixes), branch `ci/tcc-stress` (diagnostics)
- nixpkgs `pkgs/by-name/ch/chipmunk/package.nix`, `pkgs/by-name/ti/tinycc/package.nix`,
  `pkgs/build-support/make-pkgconfigitem/default.nix`
- tinycc `tcc.h` `wait_sem` (lazy semaphore init), `tccpp.c` `tccpp_new`
