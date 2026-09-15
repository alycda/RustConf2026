# Day 1: Not Quite Lisp

Each character of the input moves Santa up or down one floor. Part 1 asks
where he ends up; part 2 asks when he first goes underground. As small as
an AoC puzzle gets — which is exactly why this is the day carrying every
FFI variation: the puzzle logic is trivial enough that nothing about it
competes for attention with the boundary being demonstrated.

Seven solves of the same puzzle live in this one branch (each was built and verified independently):

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline) | `sum_pure_rust`, `basement_position_pure_rust` in `src/lib.rs` |
| libtcc JIT | Rust → C | `src/tcc.rs`, `sum_via_c`/`basement_position_via_c` in `src/lib.rs` |
| libcaca banner | Rust → C | `src/caca.rs`, `fonts/standard.flf` |
| cbindgen C API | Rust → C (exported) | `src/c_api.rs`, `cbindgen.toml` |
| Python via cffi | C → Python | `python/solve.py` |
| wasm, raw | Rust → wasm → JS, **not C at all** | `src/wasm.rs` (`alloc`/`free`), `wasm/src/raw.ts` |
| wasm, generated | Rust → wasm-bindgen → Effect | `src/wasm.rs` (`bindgen`), `wasm/src/{boundary,main,demo,banner}.ts` |

The first five are the C ABI wearing different hats. The last two are the
first boundary in this day that is not C: a different calling convention
(wasm value types — `i32`, `i64`, nothing else), a different memory model
(one linear memory the caller cannot allocate into), and a different failure
semantics (a trap, not a signal). Same solver, same answers, and the C
header is no longer the treaty — the module's own export section is.

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

**wasm, the raw route (`wasm/src/raw.ts`, Exercise 4).** The C API again —
`cargo build --target wasm32-unknown-unknown` exports the same two
`extern "C"` functions as wasm exports, and Node's built-in `WebAssembly`
calls them with nothing generated. What the C harness and Python never
needed, this caller cannot do without: JavaScript has no allocator inside
the module's linear memory, so `src/wasm.rs` exports `alloc`/`free`, and the
script borrows `len + 1` bytes for the string (writing the NUL itself —
`alloc` does not zero), four more for the `int *`, calls, reads the answer
back through a `DataView`, and returns both. Each borrow is an
`acquireUseRelease`, Effect's `Drop`, so the free runs on success, failure
and trap alike. The two status codes become two typed failures; a trap is a
defect. The hostile-input contract is proved from this side too: NULL is
the integer `0` here, and invalid UTF-8 is two bytes written straight into
the buffer. `just days wasm-demo 2015-12-01` runs it after the generated lap.

**wasm, the generated lap (`src/wasm.rs` → `wasm/src/main.ts`).** Behind
the `wasm` feature, three `#[wasm_bindgen]` exports of the same solver, and
wasm-bindgen writes the glue the raw route makes you write — the string
copy, the read-back, `Err` into a thrown `Error`. What no generator can
do is keep Rust's two failure kinds apart on the way out: `part2`'s `Err`
and `part2_unchecked`'s `expect` both arrive as "something was thrown".
`wasm/src/boundary.ts` is the fifteen lines that put them back —
a thrown `Error` becomes a typed failure the program can match on, a
`WebAssembly.RuntimeError` becomes a defect `catchAll` cannot see — and
`wasm/src/demo.ts` asserts all three channels on the statement examples.
The banner comes along too: libcaca cannot come to this target, but
`fonts/standard.flf` is data, and `wasm/src/banner.ts` renders the same
answers with figlet.js reading the same file. `wasm/test/banner.test.ts`
holds the verdict (see [Learnings](#learnings)). The generator is
version-coupled to its CLI, and `days/Cargo.toml` says how the pin is kept.

## Running things

```sh
cd days/2015-12-01 && cargo run           # pure Rust parse, libtcc-JIT solve, libcaca banner
cargo test -p aoc-2015-12-01              # the five C-shaped variants share these test cases (src/wasm.rs is target-gated: not here)

just days bench 2015-12-01                # criterion: parse + both parts, see days/README.md
cargo bench -p aoc-2015-12-01 --bench sum # pure Rust vs libtcc JIT, head to head

just days bindgen 2015-12-01              # regenerate include/aoc_2015_12_01.h
just days python-demo 2015-12-01          # build + generate header + run python/solve.py

just days wasm-demo 2015-12-01            # wasm32 build + wasm-bindgen glue + every consumer in wasm/
just days wasm-pack-demo 2015-12-01       # the same, with wasm-pack fetching the generator itself
cd wasm && npm run main | raw | demo       # the consumers, once the module, pkg/ and node_modules exist (one wasm-demo does all three)
cd wasm && npm test                       # the banner test only — the boundary is exercised by raw and demo above
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
- **A target with no OS refuses the dependencies that assume one.** The
  first wasm32 build died in `getrandom`, reached through
  `aoc-ornaments → rand`, on a day that never draws a random number: the
  crate would rather fail to compile than hand out zeros. `src/wasm.rs`
  registers a backend that says "no entropy here" — target-gated, so a
  bare `cargo build --target wasm32-unknown-unknown` needs no flags — and
  the alternative, getrandom's `js` backend, would have given the raw
  route a module with imports and a dependency on the generator it exists
  to do without.
- **The std for a target and the linker for it are separate deliveries.**
  nixpkgs' rustc ships `wasm32-unknown-unknown`'s std in its sysroot and no
  `rust-lld` next to it; the build compiled and died at the link step with
  ``linker `lld` not found``. `shell.nix` now carries `lld` (13 MiB). rustup's
  toolchains bundle it and never show this.
- **When the caller has no allocator, the callee lends one.** Every C
  consumer of this day allocated on its own side. JavaScript cannot
  allocate inside a module's linear memory, so `alloc`/`free` are exports —
  the callee-allocates contract from the reference card, arriving as a
  precondition of calling at all rather than as a returned string. `free`
  takes the size back because there is no `malloc` header to recover it
  from.
- **The header is inside the module now.** `WebAssembly.Module.exports`
  and `.imports` are the treaty, readable before anything runs, with wasm
  types: `i32` where the C header said `const char *` and `int *`. Names
  cannot drift silently — a missing export is an error at lookup — and the
  C types did not survive the trip. Both halves matter: the raw consumer
  refuses a module whose import section is non-empty, because that is the
  generated lap's module and only its glue can answer it.
- **One target directory, two modules.** `cargo build --features wasm` and
  the bare build write the same `aoc_2015_12_01.wasm`, and the last build
  wins. Found by running the two recipes back to back; the recipes now
  rebuild the bare module after generating the glue, and `raw.ts` names the
  situation instead of surfacing `Import #0 "__wbindgen_placeholder__"`.
- **A trap does not take the instance with it.** `part2_unchecked` panics,
  the module traps, JavaScript gets `RuntimeError: unreachable` — and the
  next call to `part1` answers correctly. Nothing unwound, nothing was
  cleaned up, and a module that keeps answering after a trap is one whose
  answers you can no longer vouch for. Ex 2's "the process is gone" is the
  kinder failure.
- **Two engines, one font, one column.** libcaca and figlet.js render
  `fonts/standard.flf` to the same glyph rows, and figlet.js puts one extra
  space in front of every row — on every layout option, and with its own
  bundled "Standard" font too. The reference `figlet` CLI (2.2.5) matches
  libcaca byte for byte, so two of three implementations of the FIGfont
  spec agree and the JavaScript one is off by a column. The test encodes
  both halves and fails the day figlet.js converges.
- **Independent experiments stayed independent until they didn't.** Each
  variant was built and verified (`cargo test`, `fmt`, `clippy`, and a
  real run) on its own jj commit, as a sibling of the others rather than
  stacked on top — so a broken libcaca banner could never have blocked
  the libtcc JIT work, or vice versa. They only became one tree via an
  explicit merge commit, once each side already stood on its own.
