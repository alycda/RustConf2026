# Day 1: Not Quite Lisp

Each character of the input moves Santa up or down one floor. Part 1 asks
where he ends up; part 2 asks when he first goes underground. As small as
an AoC puzzle gets — which is exactly why this is the day carrying every
FFI variation: the puzzle logic is trivial enough that nothing about it
competes for attention with the boundary being demonstrated.

Six solves of the same puzzle live in this one branch (each was built and verified independently):

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline) | `sum_pure_rust`, `basement_position_pure_rust` in `src/lib.rs` |
| libtcc JIT | Rust → C | `src/tcc.rs`, `sum_via_c`/`basement_position_via_c` in `src/lib.rs` |
| libcaca banner | Rust → C | `src/caca.rs`, `fonts/standard.flf` |
| cbindgen C API | Rust → C (exported) | `src/c_api.rs`, `cbindgen.toml` |
| Python via cffi | C → Python | `python/solve.py` |
| GDExtension via gdext | Rust ↔ engine (no header) | `src/godot.rs`, `godot/` |

## The variants

**Pure Rust.** `sum_pure_rust` and `basement_position_pure_rust` are the
puzzle solved the ordinary way — an iterator sum and a `scan`/`position`.
Kept as real functions, not deleted once the C versions existed, so
`benches/sum.rs` can put both variants of the same puzzle side by side.

**libtcc JIT (`src/tcc.rs`).** `Solution::part1`/`part2` render the parsed
input as a C array literal, hand it to libtcc as a string, JIT-compile it
in memory, and call the result through a raw function pointer — solving
"sum a list of +1/-1" by generating and compiling C at runtime. No
practical reason to; see [Learnings](#learnings) for what it's actually
good for. Run it: `cd 2015-12-01 && cargo run` prints both parts.

**libcaca banner (`src/caca.rs`).** A different direction of overkill:
`main.rs` renders each answer as a block-letter banner through libcaca's
built-in FIGlet engine (`caca_canvas_set_figfont`/`caca_put_figchar`),
loading the vendored `fonts/standard.flf`. Doesn't touch the puzzle logic
at all — this is decoration, not computation, which is the right fit for
an ASCII-art library.

**cbindgen C API (`src/c_api.rs`, Exercise 2).** The direction reverses:
instead of Rust calling into a C library, Rust exposes itself *as* one.
Two `extern "C"` functions, `aoc_2015_12_01_part1`/`part2`, built around
out-parameters and status codes rather than `Result` — a panic unwinding
across an `extern "C"` frame is undefined behavior, so nothing in this
module can panic; a null pointer or invalid UTF-8 from a C caller is
handled as data, not asserted away. `just days bindgen 2015-12-01`
generates the header (not committed — see `.gitignore`; cbindgen is
required workshop tooling, so regenerating it is always a `just` call
away).

**Python via cffi (`python/solve.py`, Exercise 3).** Consumes the header
Exercise 2 generates: `cffi`'s ABI mode `dlopen`s the built `cdylib`
directly (no C compiler step) and feeds it `cdef()` declarations read
straight from `include/aoc_2015_12_01.h`, stripped of the preprocessor
lines cffi's restricted parser can't handle. `just days python-demo
2015-12-01` builds everything and runs it (needs `just setup-python`
once).

**Godot via GDExtension (`src/godot.rs`, `godot/`).** The only variant here
that touches no C at all. A GDExtension is a shared library the engine
`dlopen`s and hands a `get_proc_address` to; the library then asks for every
engine function it will use, by name, at load, and hands back its classes the
same way. Neither side reads a header, so there is nothing for cbindgen to
generate and nothing for the consumer to transcribe — and there is no
function to export either, which is why this variant registers a *class*
(`Aoc20151201`, a `RefCounted`) rather than two `extern "C"` symbols. It is
the same `cdylib` `c_api.rs` already produces: one shared object, two
treaties.

The class exposes the day three ways on purpose, because the three are this
repo's error conventions side by side. `part1` returns an `int`. `part2`
returns a `Variant`, so "Santa never goes under" is `nil` — `Option`, in
GDScript's spelling. `part2_status` writes the answer into an `Array` and
returns `@GlobalScope.Error`, which is `c_api.rs`'s own out-parameter-plus-
status-code design landing in a runtime that already speaks it: the first
consumer in the workshop that did not have to translate it into something
else. `just days godot-demo 2015-12-01` builds it, copies the cdylib where
`godot/aoc.gdextension` says, and runs `godot/test.gd` headless (needs `just
setup-godot` once).

## Running things

```sh
cd days/2015-12-01 && cargo run           # pure Rust parse, libtcc-JIT solve, libcaca banner
cargo test -p aoc-2015-12-01              # all five variants share these test cases

just days bench 2015-12-01                # criterion: parse + both parts, see days/README.md
cargo bench -p aoc-2015-12-01 --bench sum # pure Rust vs libtcc JIT, head to head

just days bindgen 2015-12-01              # regenerate include/aoc_2015_12_01.h
just days python-demo 2015-12-01          # build + generate header + run python/solve.py
just days godot-demo 2015-12-01           # build --features godot + run godot/test.gd headless
```

## Benchmarks

`benches/sum.rs` races `sum_pure_rust`/`basement_position_pure_rust`
against their `_via_c` counterparts on a synthetic 2000-repeat instruction
stream (release profile, aarch64):

| | pure Rust | C via FFI/JIT | ratio |
|---|---|---|---|
| sum | ~557ns | ~1.27ms | ~2,300x |
| basement_position | ~2ns | ~1.26ms | ~600,000x |

The gap is almost entirely compile/relocate/teardown overhead — `part1`/
`part2` JIT from scratch on every call, nothing is cached. The
`basement_position` ratio is the more honest number: the benchmark input
(`"()())"` repeated) dips negative almost immediately, so Rust's `scan`
stops at position 5 of 10,000 — while the C variant still pays for a full
JIT cycle around a loop that exits just as early. The boundary cost
dwarfs the work on both sides of it, in both directions.

## Learnings

- **Direction changes the discipline, not just the code.** Calling C from
  Rust (`tcc.rs`, `caca.rs`) is a library-discovery problem: find the
  headers, link the symbols, keep the FFI surface `unsafe extern "C" {
  }`. Exposing Rust to C (`c_api.rs`) is an ABI-safety problem: a panic
  crossing that boundary is UB, so the design has to route every failure
  through data (status codes) instead of Rust's usual `Result`/`panic!`.
- **Not every C "library" is a library.** `figlet` (the classic CLI tool)
  ships no `.so` and no header in nixpkgs — nothing to link against.
  `libcaca`, which looks unrelated, happens to embed a real FIGlet-font
  renderer behind a proper C API. The lesson generalizes: check for an
  actual header + shared object before designing around a tool's name.
- **`pkg-config` + nix `buildInputs` is a clean, repeatable discovery
  pattern.** Both `tcc.rs` and `caca.rs` link via the identical
  `build.rs` idiom — shell out to `pkg-config --libs <name>`, forward the
  flags as `cargo:rustc-link-arg`. Nix's `pkg-config` setup hook wires up
  `PKG_CONFIG_PATH` for every `buildInput` automatically; nothing here
  hardcodes a `/nix/store/...` path.
- **cbindgen's scope has to be aimed, not just configured.** Pointed at
  the crate root, cbindgen also walks `tcc.rs`/`caca.rs`'s own `unsafe
  extern "C" { }` *import* blocks and redeclares libtcc's and libcaca's
  APIs in the generated header — noise with nothing to do with this
  day's C API. Pointing it at `src/c_api.rs` directly scopes it to just
  the two functions meant to be exported.
- **Edition 2024 makes both FFI directions say `unsafe` explicitly.**
  Foreign function *imports* need `unsafe extern "C" { }` (not just
  `extern "C" { }`); exported functions need `#[unsafe(no_mangle)]` (not
  bare `#[no_mangle]`). Both used to be implicit.
- **One generated header, three languages, one source of truth.**
  `python/solve.py` doesn't hand-duplicate `aoc_2015_12_01_part1`'s
  signature — it reads the same header C would `#include`, strips what
  `cffi`'s restricted parser can't handle (the include guard, `#include`,
  block comments), and hands the rest to `cdef()`. If the Rust signature
  changes, regenerating the header is the only step; nothing in Python
  needs editing to match.
- **The `.gdextension` descriptor is the entire static contract, and
  nothing validates it.** Entry symbol, minimum engine version, one library
  path per platform — an INI file no tool generates and no build step
  checks. Every other track has something that would catch a mistake: a
  header the compiler reads, a `pubspec`, a `module.modulemap` clang has to
  open. Here a wrong row is not an error. The engine finds no match, loads
  nothing, and the failure arrives as `Identifier "Aoc20151201" not
  declared` — a message about the script, several layers from the file that
  was actually wrong. Worse, the engine only *looks* for the descriptor
  during a filesystem scan, so a fresh checkout fails that way until
  `godot --headless --import` has run once.
- **Godot's string is UTF-32, which is a memory trade nobody else here
  made.** One code point per 32-bit unit: no surrogate pairs, no variable
  width, indexing is O(1) and four times the bytes. Every `GString` ↔
  `String` crossing transcodes. `godot/test.gd` asserts the consequence
  rather than asserting the claim — `(🦀)` is 3 code points to GDScript and
  6 bytes to Rust, `(é)` is 3 and 4 — so this is a measurement, and ASCII
  hides it completely.
- **A third panic semantics, and the docs were optimistic about it.**
  Exercise 2's C API cannot panic at all (unwinding across `extern "C"` is
  UB, so every failure is a status code); wasm turns a panic into a trap
  that kills the instance. gdext catches the unwind, prints the Rust message
  as an engine error with a GDScript backtrace, and carries on — which reads
  like the friendliest of the three until you ask what the caller got back.
  "The call returns the type's default" is the received wisdom and it is not
  what happens: gdext 0.5.5 does not write the return slot at all.
  `part2_unwrapped("(((")` reads `0` from a fresh slot and `5` from one the
  caller just used for a successful call. Nothing on the GDScript side can
  tell that from an answer. The trap that takes your instance and says so is
  the *more* forgiving design.
- **Where this boundary stops, in production.**
  [backstitch](https://github.com/inkandswitch/backstitch) — real-time
  version control for Godot — is the grown-up version of this variant: a
  Rust core built on gdext 0.5, carrying an Automerge CRDT and a
  tree-sitter grammar for `.tscn` files into the editor, with the macOS
  build shipped as a `.framework` (the per-platform clause of the descriptor
  above, made visible). Nobody rewrote Automerge in GDScript, which is the
  "when is FFI the right call" answer for game tooling. And it is a
  GDExtension *plus* a C++ editor module compiled into a custom Godot build,
  because the extension API does not expose the editor-UI hooks they needed.
  The generated lap gets you the runtime; the last mile cost them a fork.
- **Independent experiments stayed independent until they didn't.** Each
  variant was built and verified (`cargo test`, `fmt`, `clippy`, and a
  real run) on its own jj commit, as a sibling of the others rather than
  stacked on top — so a broken libcaca banner could never have blocked
  the libtcc JIT work, or vice versa. They only became one tree via an
  explicit merge commit, once each side already stood on its own.
