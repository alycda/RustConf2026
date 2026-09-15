# Day 1: Not Quite Lisp

Each character of the input moves Santa up or down one floor. Part 1 asks
where he ends up; part 2 asks when he first goes underground. As small as
an AoC puzzle gets — which is exactly why this is the day carrying every
FFI variation: the puzzle logic is trivial enough that nothing about it
competes for attention with the boundary being demonstrated.

Eleven solves of the same puzzle live in this one branch (each was built and verified independently):

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline) | `sum_pure_rust`, `basement_position_pure_rust` in `src/lib.rs` |
| libtcc JIT | Rust → C | `src/tcc.rs`, `sum_via_c`/`basement_position_via_c` in `src/lib.rs` |
| libcaca banner | Rust → C | `src/caca.rs`, `fonts/standard.flf` |
| cbindgen C API | Rust → C (exported) | `src/c_api.rs`, `cbindgen.toml` |
| Python via cffi | C → Python | `python/solve.py` |
| Fortran via ISO_C_BINDING | C → Fortran | `fortran/solve.f90` |
| R via `.C()` | C → R | `r/solve.R` |
| R via extendr (`.Call()`) | Rust → R (generated) | `src/extendr.rs`, `r/extendr.R` (feature `extendr`) |
| GDExtension via gdext | Rust ↔ engine (no header) | `src/godot.rs`, `godot/` |
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

**Fortran via ISO_C_BINDING (`fortran/solve.f90`, Exercise 3).** The same
header, read by a person rather than a parser: there is no generator that
emits Fortran interfaces from C, so the two signatures are transcribed by
hand into an `interface` block — and then enforced, because gfortran
type-checks every call against that block. Transcribed like Dart's
typedefs, checked like Swift's module import; checked against what we
wrote down, though, not against what Rust exports, so a transcription
that is wrong *and self-consistent* still compiles and still corrupts.
What this track alone shows is that the C API's out-parameter costs this
language nothing: `integer(c_int), intent(out) :: answer` *is* the
`int *`, and the call site is the puzzle's own variable. `just days
fortran-demo 2015-12-01` builds everything and runs it (needs `just
setup-fortran` once).

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
just days fortran-demo 2015-12-01         # build + generate header + compile and run fortran/solve.f90
just days r-demo 2015-12-01               # build + run r/solve.R (no header — R reads none)
just days r-extendr-demo 2015-12-01       # the generated lap; needs R installed to *build*
just days godot-demo 2015-12-01           # build --features godot + run godot/test.gd headless

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
- **By reference is somebody's native convention.** Every consumer of
  this C API so far paid something to hand it a place to write: Python
  allocates `ffi.new("int *")`, Dart `calloc`s and frees, Swift takes
  `&slot`. Fortran passes everything by reference by default, so the
  out-parameter is a declaration and nothing else — `integer(c_int),
  intent(out) :: answer`, and the call site is the ordinary variable.
  The design decision that looks most like a concession to C turns out
  to be free in the oldest language here.
- **A Fortran string has a length, not a terminator** — so `//
  c_null_char` is the NUL written by hand, and `trim` is what keeps the
  padding out. Fortran characters are fixed-length and blank-padded: a
  `character(len=4096)` buffer holding five characters holds 4091 spaces
  after them, and they are part of the string. `fortran/solve.f90` reads
  with `access='stream'` into an allocatable sized from `INQUIRE`, so
  there is nothing to trim and the `trim` is defensive rather than
  load-bearing — which is the honest version, and why the trap is in a
  comment instead of in the code. Note what `trim` does *not* remove: a
  newline is not a blank.
- **The ABI carries arguments the signature never shows.** `const char
  *` binds to `character(kind=c_char), dimension(*)`, not to
  `character(len=*)`. The second compiles and is wrong: gfortran's ABI
  appends the length of every `character` argument as a hidden trailing
  `size_t`, after all the visible ones, and C never declared a parameter
  to receive it. The call still works — C ignores an argument it was not
  told about — which is exactly what makes it worth knowing before
  meeting it from the other direction, where the hidden length is one
  *you* have to supply and leaving it off links clean and reads garbage.
- **This treaty is in the language standard, not in a library.**
  `ISO_C_BINDING`, `bind(C)`, `c_int`, `c_null_char` are Fortran 2003 —
  no cffi to pip-install, no JNA jar to pin, no `dart:ffi` package, no
  modulemap. It is the only track here whose C interop was specified by
  the language committee, and the only one that fetches nothing at all.
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
