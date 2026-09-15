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
| R via `.C()` | C → R | `r/solve.R` |
| R via extendr (`.Call()`) | Rust → R (generated) | `src/extendr.rs`, `r/extendr.R` (feature `extendr`) |

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

**R via `.C()` (`r/solve.R`, Exercise 3).** The same library, the same two
functions, and the one consumer in this repo that reads nothing at all —
no header, no transcribed signature, no declared types. `.C()` is R's
oldest FFI: it matches on the symbol name, converts each argument by its
R *vector mode* (Writing R Extensions §5.2 — `integer` → `int *`, `raw` →
`unsigned char *`, `character` → `char **`), and then throws the C
function's return value away, because the interface was designed for C
functions of type `void`. Our C API is not `void`, so its `0`/`-1`/`-2`
never reaches R and the out-parameter is the only channel there is. The
consumer initialises it to `NA_integer_` — `INT_MIN` once it crosses, a
value no floor and no 1-based position can be — and treats "unchanged"
as failure. `just days r-demo 2015-12-01` builds the cdylib and runs it
(needs `just setup-r` once, or `nix-shell -p R --run` around it).

**R via extendr (`src/extendr.rs`, `r/extendr.R`, feature `extendr`).** The
generated lap, and the direct answer to what `.C()` cost. `#[extendr]` on
an ordinary Rust function emits R's *other* interface — `.Call()`, which
passes `SEXP`s both ways — so the string crosses as the string it already
is (no `raw` vector, no hand-appended NUL) and a `Result<i32, String>`
arrives as an R error condition that `tryCatch` can name. The status code
the raw route lost comes back. Unlike every other feature in this crate it
links no C library and probes no `pkg-config`: it needs **R itself** at
build time, which is why `just days r-extendr-demo 2015-12-01` is a
separate recipe — `r-demo` needs R to run, this one needs R to compile.

## Running things

```sh
cd days/2015-12-01 && cargo run           # pure Rust parse, libtcc-JIT solve, libcaca banner
cargo test -p aoc-2015-12-01              # all five variants share these test cases

just days bench 2015-12-01                # criterion: parse + both parts, see days/README.md
cargo bench -p aoc-2015-12-01 --bench sum # pure Rust vs libtcc JIT, head to head

just days bindgen 2015-12-01              # regenerate include/aoc_2015_12_01.h
just days python-demo 2015-12-01          # build + generate header + run python/solve.py
just days r-demo 2015-12-01               # build + run r/solve.R (no header — R reads none)
just days r-extendr-demo 2015-12-01       # the generated lap; needs R installed to *build*
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
- **A runtime can discard the contract the API was designed around.**
  `.C()` "should not return anything except through its arguments", so
  the status code `c_api.rs` routes every failure through is unobservable
  from R — not because the library chose badly, but because the *caller's*
  runtime decided. "Errors: integers cross" has a footnote now: only if
  the other side looks. What is left is in-band signalling — a sentinel in
  the out-parameter (`NA_integer_`, which is `INT_MIN` at the boundary) and
  "it did not change" as the only detectable failure, with no way to tell
  `-1` (bad input) from `-2` (Santa never reached the basement).
- **The string cannot be passed as a string.** An R `character` vector
  arrives in C as `char **`, a pointer to pointers, and this API takes
  `const char *` — hand it one and Rust reads a pointer as text. The
  honest crossing is a `raw` vector, which arrives as `unsigned char *`,
  byte for byte: `c(charToRaw(text), as.raw(0))`. The NUL is appended by
  hand, because R's strings do not carry one — the same terminator Python,
  Dart and Swift each had to think about, in a language that does not look
  like it is doing FFI at all.
- **Arguments are copied, so the variable you passed is not the one that
  was written.** `.C()` duplicates every argument before the call; the
  pointer the Rust side writes through points at R's copy, and the answer
  comes back in the returned list (`r$out`), not in `out`. `r/solve.R`
  asserts that with a `stopifnot` rather than only saying it. Who
  allocates is, once again, not the library.
- **Three boundaries, three answers to "what does a panic do?"** Exercise
  2's C API cannot let one out at all — unwinding across `extern "C"` is
  UB, so `c_api.rs` is written so nothing in it can panic. wasm traps and
  takes the instance with it. extendr catches the unwind in its generated
  wrapper and raises an ordinary R error: `tryCatch` sees a `simpleError`,
  and the session carries on. `boundary_panic` in `src/extendr.rs` exists
  to be called, and `r/extendr.R` calls it and then keeps going.
- **The generated boundary has a seam, and the raw route is how you know
  where to look.** extendr 0.9.0 implements the `Err` arm by *panicking*
  with the message and converting that into the R condition — so an
  ordinary domain error (part 2 on an input where Santa never reaches the
  basement) prints a Rust panic trace from extendr's own `into_robj.rs` to
  stderr on its way to a perfectly clean R error. The condition R receives
  is right; the route is not what the signature suggests.
- **A label made of `\U` escapes is not a stable label.** R renders
  `"\U0001F4CA"` according to the locale — the emoji under a UTF-8 one,
  the literal text `<U+0001F4CA>` under `C` — so the same script prints
  different bytes on different machines. Written as literal UTF-8 bytes in
  the source it passes through either locale unchanged, which is what the
  byte-exact label assertions in CI need.
- **Independent experiments stayed independent until they didn't.** Each
  variant was built and verified (`cargo test`, `fmt`, `clippy`, and a
  real run) on its own jj commit, as a sibling of the others rather than
  stacked on top — so a broken libcaca banner could never have blocked
  the libtcc JIT work, or vice versa. They only became one tree via an
  explicit merge commit, once each side already stood on its own.
