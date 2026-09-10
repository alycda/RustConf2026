# Buck2, alongside the Bazel overlay

Companion to [`bazel-evaluation.md`](./bazel-evaluation.md), which should be
read first — it sets up the question (275 solved days, FFI in at least one
direction), the measurement method, and the verdict this document either
moves or doesn't.

Same approach: built, not estimated. Buck2 `2026-09-10`, prelude bundled,
reindeer at `a5d959fb`. The overlay is `.buckconfig`, `toolchains/BUCK`,
`tools/aoc_buck.bzl`, generated `days/*/BUCK`, `third-party/rust/`, and
`ffi_smoke/BUCK`. `just`, `cargo` and `shell.nix` are still untouched, and
both overlays coexist: `BUILD.bazel` and `BUCK` sit in the same directories,
generated from the same `cargo metadata` call by the same script.

**Verdict up front: Buck2 is the better of the two build systems for this
repo, and it does not change the recommendation.** It is faster than Bazel
on every axis measured, faster than *cargo* on incremental work while
staying selective, and it avoids the lockfile hazard that is the Bazel
overlay's worst finding. It also gives up the hermetic toolchain pin, needs
a second copy of the dependency list, and takes four separate discoveries to
get a C program to call a Rust function. For attendees, both remain the
wrong answer for the reason that has not changed: the workshop is about
linker flags, and both systems exist to hide them.

## Measurements

Identical work to the Bazel table: 9 days plus `_template`, 12 test targets
in each system, same machine.

| | cargo | Bazel | Buck2 |
|---|---|---|---|
| cold full build + test | **26.7s** | **190.5s** | **74.8s** |
| fully warm, nothing changed | **0.6s** | **1.3s** | **0.2s** |
| edit a pure-Rust day | 1.0s — 20 test bins | 1.5s — 1/12 re-run | **0.5s** — 1 command |
| edit an FFI day's C surface | 0.9s — 20 test bins | 1.6s — 2/12 re-run | **0.7s** — 5 commands |
| edit a day nothing binds to | 0.8s — 20 test bins | 1.3s — 1/12 re-run | **0.5s** — 1 command |

Buck2 wins every incremental row outright, including against cargo, while
keeping Bazel's selectivity. That is the one result here that could actually
change a decision: the Bazel document argued selectivity was worth nothing
at nine days because Bazel's own overhead ate the savings. Buck2 does not
have that overhead. It is a single Rust binary with no JVM, and the warm
no-op is 0.2s.

The cross-language edge is expressed the same way:

```
buck2 cquery 'rdeps(set(//days/... //ffi_smoke/...), //days/2015-12-06:header)'
  root//days/2015-12-06:c_api
  root//days/2015-12-06:header
  root//ffi_smoke:day_2015_12_06
```

Note `cquery`, not `uquery`: unconfigured queries fail against the demo
toolchains with `Unknown target 'cxx_no_default_deps'`, which is a confusing
first encounter with a query language.

## Where Buck2 is clearly better

**The prelude is bundled in the binary.** `[external_cells] prelude =
bundled` — no registry, no `bazel_dep` version list, nothing fetched, and no
way for the rules to drift out of lockstep with the binary that reads them.
The Bazel overlay pins `rules_rust 0.74.0` in MODULE.bazel and fetches from
`bcr.bazel.build`; the Buck2 overlay pins nothing because there is nothing
to pin. For a workshop repo that cares about a cold clone working, this is a
real difference.

**`days/Cargo.lock` is never touched.** The Bazel document's finding #1 —
crate_universe writing 181 lines of cbindgen's dependency tree into the
committed lockfile that the `msrv` job tests, fixable only via an
experimental flag — simply has no analogue. Reindeer resolves against its
own manifest and its own lockfile. The trade is finding #2 below.

**The third-party graph is a committed, reviewable file.** `third-party/rust/BUCK`
is 5289 lines of `http_archive` + `cargo.rust_library`, each crate with a
pinned `sha256` and a `static.crates.io` URL, all of it diffable in a pull
request. Bazel's equivalent resolution happens inside the module extension
and lands in `MODULE.bazel.lock` (2802 lines of hashes). For anyone who
wants to see what their build actually downloads, reindeer's output is the
better artifact.

**It is fast.** See the table.

## Where Buck2 is worse, for this repo specifically

**1. No hermetic toolchain — the big one.** `buck2 init` generates
`system_demo_toolchains()`, and the name is honest: `rustc` comes from PATH.
rules_rust ships a toolchain that downloads a named compiler, so
`MODULE.bazel` says `versions = ["1.85.0"]` and the floor `days/Cargo.toml`
declares *is* the build.

This was not theoretical during the evaluation. Installing reindeer needed a
newer rustc, `rustup toolchain install stable` moved the default from 1.94.1
to 1.98.1 — and Buck2 silently followed it while Bazel kept compiling
against 1.85. A build system that tracks whatever compiler is installed
cannot be the thing that proves an MSRV. Pinning it means writing a
download-and-extract toolchain rule by hand, or handing Buck2 the compiler
from `nix-shell`, which is a genuinely reasonable option for this repo and
was not tested here.

**2. The dependency list is duplicated.** Reindeer needs its own pseudo-package
at `third-party/rust/Cargo.toml`, listing every dependency again with its own
version constraints, plus its own `Cargo.lock`. crate_universe's `from_cargo`
reads `days/Cargo.toml` and `days/Cargo.lock` directly.

`days/Cargo.toml` states the property this breaks: *"Nine manifests inheriting
one table means a version bump is a single edit — the alternative is nine,
with CI green-lighting whichever ones you missed."* Reindeer makes it two
edits, and nothing checks that they agree. Closing that gap is a script
nobody has written yet.

**3. Twenty-two hand-written fixup files, found by iteration.** Reindeer
refuses to guess what a third-party build script is for, so every such crate
needs `fixups/<crate>/fixups.toml`. It warned about 18. The real number was
22, because `thiserror-impl`, `miette`, `serde_derive` and `criterion` have
no build script and still need `cargo_env = true` — they read `CARGO_PKG_*`
at *compile* time. Those four are discoverable only by running the build,
reading a `CARGO_PKG_VERSION_PATCH not defined` error, adding a file, and
re-running. crate_universe made every one of these decisions automatically.

At nine days with ten direct dependencies this is an afternoon. The fixup
count scales with the dependency surface, not the day count, so it does not
get much worse at 275 days — but it does get worse every time a day reaches
for a new crate, and the failure mode is a build error in someone else's
code.

**4. Getting cbindgen's binary needs nightly cargo.** `bindeps = true` in
`reindeer.toml` exposes a crate's executable, and reindeer's manual is
explicit that this requires a nightly cargo at buckify time (artifact
dependencies are still unstable), though not at build time. The Bazel
equivalent was `gen_binaries = ["cbindgen"]` on stable. Both end up building
cbindgen from source, which is the win the Bazel document called
unambiguous; Buck2 charges more for it.

**5. Rust-to-C took four discoveries, each with a misleading error.** In
Bazel this was one line: `cc_library(deps = [":cdylib"])`, because
rules_rust's shared library provides `CcInfo`. In Buck2:

- `deps = [":cdylib"]` on a `cxx_library` links cleanly and fails with
  `undefined symbol: aoc_2015_12_06_part1`. The default output of a
  `rust_library` is a `.rmeta`; the shared object lives behind the
  **`[cdylib]` subtarget**.
- Wrapping that subtarget needs `prebuilt_cxx_library`.
- The generated header then isn't found, because Buck2 namespaces
  `exported_headers` by package path — the C source would have to say
  `#include "days/2015-12-06/aoc_2015_12_06.h"`. Fixed with
  `header_namespace = ""`.
- With symbols and header resolved, the test binary dies at startup:
  `libdays_2015-12-06_cdylib.so: cannot open shared object file`. Buck2
  gives a `cxx_test` no RUNPATH into `buck-out` and no runfiles tree, so the
  harness has to be handed a directory to search — a genrule that copies the
  `.so` somewhere and an explicit `LD_LIBRARY_PATH`.

The `staticlib` route is not an escape: it isn't offered under
`preferred_linkage = "shared"`, and under `"any"` it produces a non-PIC
archive that fails to link into the PIC test binary with a wall of
`relocation R_X86_64_32 cannot be used against local symbol`.

There is something funny about the last one. The `-L`-does-not-imply-`-rpath`
lesson that `days/justfile` spends twenty lines documenting — the thing this
workshop exists to teach — reappeared inside the build system whose job was
to abstract it away, and the fix was the same fix: tell the loader where to
look. Buck2 does not remove that lesson. It relocates it into a file
attendees would never open.

**6. No `CARGO_MANIFEST_DIR`.** rules_rust sets it for every Rust compile;
Buck2 sets nothing, so every day's `main.rs` — which builds its input path
with `concat!(env!("CARGO_MANIFEST_DIR"), "/../../inputs/…")` — fails to
*compile*, not merely to find its file. Supplying any value fixes the build.
Neither overlay makes that path resolve correctly at runtime; doing so needs
`main.rs` to stop deriving the path at compile time, which is a first-party
source change and out of scope here.

**7. `rstest` breaks in both, differently.** Bazel sets `CARGO_MANIFEST_DIR`
but sandboxes `Cargo.toml` away (`compile_data` fixes it); Buck2 doesn't set
the variable at all, and wants `Cargo.toml` in `srcs` plus
`env = {"CARGO_MANIFEST_DIR": "."}` — where `"."` is right because Buck2
resolves relative env values against the action's source directory, which
took one wrong guess to establish.

## What is unchanged from the Bazel evaluation

- **The pkg-config wall.** Eight C libraries behind feature gates — caca,
  tcc, hyperscan, ICU, chipmunk, duckdb, yara, espeak-ng — resolved through
  `pkg-config` against nix. Untested under Buck2 for the same reason it was
  untested under Bazel, and Buck2's answer would be the same `cxx_library`
  per library. Buck2 additionally has no `cargo_build_script` equivalent for
  *first-party* build scripts, so the five days with a `build.rs` compile
  here only because those scripts no-op with no features enabled.
- **No Dart rules**, in either system.
- **macOS, remote caching, and the Kotlin/Dart/Swift per-invocation cost**:
  all still unmeasured, and the last of these is still the number the
  275-day question turns on.
- **The attendee path.** Neither overlay touches `just`, and neither should.

## Recommendation, revised

The two-interfaces-over-one-graph shape from the Bazel document stands, and
**if you build that second interface, build it with Buck2, not Bazel.** It is
2.5× faster cold, 6× faster warm, faster than cargo incrementally, it cannot
corrupt `days/Cargo.lock`, and its third-party graph is a file you can
review. The price is a compiler pin you have to build yourself, a dependency
list maintained twice, and a Rust-to-C story that took four undocumented
discoveries to get working — all of which are one-time costs paid by the
maintainer, not recurring costs paid by attendees.

That said, the thing I would still try first has not changed: a flake with
per-day derivations. It keeps one hermeticity story, it already owns the
eight C libraries that are the actual wall in both overlays, and it does not
ask you to maintain a second dependency manifest. Buck2 is the better answer
to "which of these two build systems"; it is not obviously the better answer
to "what should this repo do next".
