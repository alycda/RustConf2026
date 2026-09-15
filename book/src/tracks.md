# One Header, Nine Front Doors

**Not required reading**, like the chapter before it. That chapter is the
record of one C API crossed from four languages — Python, Kotlin, Swift,
Dart — and everything that went wrong on the way. This one is the other
five runtimes the same header has since been carried into, and what each of
them asked for that C never did. It is longest on wasm, because wasm is the
one with no C in it at all, and because it is Exercise 4.

Every section answers the same three questions. What crosses the boundary,
and in what shape. Which single lesson that runtime teaches that none of
the others can. And the front door: what you install, what the recipe runs,
and where the runtime's own version number gets to matter.

## wasm: the boundary with no C in it (Exercise 4)

The day's C API is two `extern "C"` functions: a `const char *` in, an
`int *` out, a status code back. Build the same crate for
`wasm32-unknown-unknown` and those two functions are still there, as
*exports* of a `.wasm` module, and Node's built-in `WebAssembly` object can
call them with nothing generated. No header is read by anyone, because the
module carries its own:

```js
WebAssembly.Module.exports(module)   // the treaty, readable before anything runs
```

That is the first thing wasm asks for that C never did: the types in that
export list are wasm types. Where the header said `const char *` and
`int *`, the module says `i32` and `i32`. The names cannot drift — a
missing export fails at lookup, not at the first call — and the C types did
not survive the trip. You get a stricter contract with less in it.

### The caller has no allocator

Every C consumer of this day allocated on its own side: `ffi.new`, `calloc`,
`&slot`. JavaScript cannot allocate inside a module's linear memory, so the
module has to lend it one. `src/wasm.rs` exports `alloc` and `free`, and the
consumer borrows `len + 1` bytes for the string — writing the NUL itself,
because `alloc` does not zero — and four more for the `int *`, calls, reads
the answer back through a `DataView`, and returns both. `free` takes the
size back, because there is no `malloc` header on the other side to
recover it from.

This is the reference card's callee-allocates contract, arriving not as a
returned string but as a precondition of calling at all. In the day's
consumer each borrow is an `acquireUseRelease`, so the free runs on
success, on failure and on a trap alike. Your Exercise 4 makes you write
that by hand first, which is the point.

### Two failure kinds, and a third the others don't have

The C API has one error channel: the status code. wasm has three.

- The status code still comes back, and the raw consumer turns `-1` and
  `-2` into two typed failures.
- A Rust `Err` on the generated lap arrives in JavaScript as a thrown
  `Error`.
- A panic traps. The module raises `RuntimeError: unreachable`, and then
  the interesting part: **the instance is still there afterwards, and the
  next call answers correctly.** Nothing unwound. Nothing was cleaned up.
  Exercise 2's "the process is gone" is the kinder failure, because a
  module that keeps answering after a trap is one whose answers you can no
  longer vouch for.

What no generator can do is keep the second and third apart on the way out:
`part2`'s `Err` and `part2_unchecked`'s `expect` both arrive as "something
was thrown". The day's `wasm/src/boundary.ts` is the fifteen lines that put
them back — a thrown `Error` becomes a typed failure the program can match
on, a `WebAssembly.RuntimeError` becomes a defect that `catchAll` cannot
see. Read it before you write your own.

### Two routes, one build

The day carries both routes, and the recipes keep them apart on purpose.

**The raw route** (`wasm/src/raw.ts`, and your Exercise 4) is the bare
`cargo build --target wasm32-unknown-unknown`: the C API as exports, your
own `alloc`/`free`, the string written into linear memory by hand. The raw
consumer refuses a module whose import section is non-empty, because that
is the generated lap's module and only its glue can answer it.

**The generated lap** (`src/wasm.rs` behind the `wasm` feature,
`wasm/src/main.ts`) is wasm-bindgen writing the glue the raw route made you
write: the string copy, the read-back, `Err` into a thrown `Error`. Three
`#[wasm_bindgen]` exports of the same solver. It is the UniFFI of this
track, and like UniFFI it costs a generator.

Both routes write the same `aoc_2015_12_01.wasm` into the same target
directory, and the last build wins. The recipes rebuild the bare module
after generating the glue; if you run the two builds yourself in the wrong
order, `raw.ts` names the situation rather than surfacing
`Import #0 "__wbindgen_placeholder__"`.

### The toolchain, again

wasm is the track where the toolchain lesson from the previous chapter
comes back with two new clauses.

**The std for a target and the linker for it are separate deliveries.**
nixpkgs' rustc ships the `wasm32-unknown-unknown` std in its sysroot and no
`rust-lld` next to it, so the first wasm build compiled and died at the
link step with `` linker `lld` not found ``. `shell.nix` carries `lld` now.
rustup's toolchains bundle it and never show you this.

**A target with no OS refuses the dependencies that assume one.** The first
build died in `getrandom`, reached through a helper crate's `rand`
dependency, on a day that never draws a random number: the crate would
rather fail to compile than hand out zeros. The day registers a backend
that says "no entropy here", target-gated so a bare build needs no flags.
The alternative, getrandom's `js` backend, would have given the raw route a
module with imports and a dependency on the very generator it exists to do
without.

**The generator is version-coupled to its CLI.** wasm-bindgen's CLI refuses
a module built with any other patch version of the crate, so
`days/Cargo.toml` pins the crate exactly, and every path that installs the
CLI reads that pin from the lockfile: CI asks `cargo pkgid`, the wasm
devcontainer reads `days/Cargo.lock` and takes nixpkgs' matching
`wasm-bindgen-cli_0_2_<n>` attribute, and `just days wasm-demo` checks
what is on PATH against the lock before it spends a build. If you have a
hand-installed CLI at the wrong version, the guard names both numbers and
three ways out. The day this drifted, and why "match the nix channel" was
the wrong rule, is written up in
`docs/solutions/build-errors/wasm-bindgen-cli-version-mismatch-devcontainer-channel.md`.

**Front door.** Node 22 and `npm ci`. The exercise commits its
`package-lock.json` on purpose — `npm ci` refuses to run without one and
refuses to change one, so the attendee never resolves anything — which is
the opposite of the Dart track's rule, and the day's `exercises/.gitignore`
says why. `just setup-wasm` covers the target and the Node pointer;
`node_modules/` is ignored at the root, so a stray `npm install` cannot
end up in your history. If you would rather install nothing, the wasm
devcontainer variant carries Node, the CLI at the lockfile's version, and
`wasm-pack`.

## Fortran: by reference is somebody's native convention

Fortran is the oldest language here and the only one whose C interop is
specified by its own standard: `ISO_C_BINDING`, `bind(C)`, `c_int`,
`c_null_char` are Fortran 2003. Nothing to pip-install, no jar to pin, no
module map. There is also no generator that emits Fortran interfaces from a
C header, so the consumer transcribes the two signatures by hand into an
`interface` block — and then, unlike Dart's transcription, gfortran
*enforces* it: every call is type-checked against the block. Wrong and
self-consistent still compiles and still corrupts, but wrong and
inconsistent is a compile error.

The one lesson this track teaches alone is that the out-parameter costs it
nothing. Every other consumer paid something to hand the C API a place to
write. Fortran passes everything by reference by default, so
`integer(c_int), intent(out) :: answer` *is* the `int *`, and the call site
is the puzzle's own variable. The design decision that looks most like a
concession to C turns out to be free in the language that predates it.

The trap is the string. A Fortran string has a length, not a terminator, so
`// c_null_char` is the NUL written by hand — a fourth time in this repo —
and `const char *` binds to `character(kind=c_char), dimension(*)`, not
`character(len=*)`. The second compiles and is wrong: gfortran's ABI appends
the length of every `character` argument as a hidden trailing `size_t`,
after all the visible ones, that C never declared a parameter to receive.
The call still works, because C ignores an argument it was not told about,
which is exactly why it is worth knowing before you meet it from the other
direction. The golden day does: `days/2024-12-01` calls LAPACK's `DLASRT`
from Rust, where that hidden length is one *you* supply — and where, it
turned out, leaving it off links clean and sorts correctly, because `DLASRT`
never reads it. The wrong treaty passes every test you would write.

**Front door.** `gfortran`, and only that. `just setup-fortran` names the
package per OS; the recipe compiles `solve.f90` against the day's cdylib
with two `-L`/`-rpath` pairs, the Swift track's spelling of the same
search.

## R: the runtime that discards the return value

R's oldest FFI is `.C()`, and it is designed for C functions of type
`void`: it matches on the symbol name, converts each argument by its R
vector mode, calls, and hands back the *modified argument list*. The return
value is thrown away. Our C API is not `void`, so its `0`/`-1`/`-2` never
reaches R at all. Three consequences, each a lesson:

1. **The string cannot be passed as a string.** An R `character` vector
   arrives in C as `char **`, a pointer to pointers, and the API takes
   `const char *`. Hand it one and Rust reads a pointer as text. The honest
   crossing is a `raw` vector with the NUL appended by hand,
   `c(charToRaw(text), as.raw(0))`, which arrives byte for byte.
2. **The status code is unobservable.** The out-parameter is the only
   channel, so the consumer initialises it to a sentinel no answer can be
   — `NA_integer_`, which is `INT_MIN` once it crosses — and treats
   "unchanged" as failure. In-band signalling, imposed by the caller's
   runtime rather than chosen by the library. "Errors: integers cross" has
   a footnote now: *if the other side looks*.
3. **Arguments are copied.** `.C()` duplicates every argument before the
   call. The pointer the Rust side writes through points at R's copy, and
   the answer comes back in the returned list, not in the variable you
   passed. Who allocates is, once again, not the library.

The generated lap here is extendr, and it is the direct answer to what
`.C()` cost: `.Call()` passes R objects both ways, the string crosses as
the string it already is, and a `Result<i32, String>` arrives as an R error
condition that `tryCatch` can name. It also gives this repo its third answer
to "what does a panic do?": the C API cannot let one out at all, wasm traps
and keeps answering, extendr catches the unwind and raises an ordinary R
error, and the session carries on.

**Front door.** `Rscript`, and nothing else — `.C()` is in base R. R is
also the largest download of any track, which is why `just setup-r` points
at CRAN and brew rather than at `shell.nix`, and why the extendr lap is a
separate recipe: `r-demo` needs R to *run*, `r-extendr-demo` needs R to
*compile*, because extendr links `libR` at build time.

## Godot: the boundary is a class, not a function

This is the only track that touches no C at all. A GDExtension is a shared
library the engine `dlopen`s and hands a `get_proc_address` to; the library
asks for every engine function it will use, by name, at load, and hands back
its classes the same way. Neither side reads a header, so there is nothing
for cbindgen to generate and nothing for the consumer to transcribe — and
there is no function to export either. The day registers a *class*, a
`RefCounted` with the two solvers as methods, in the same cdylib that
already carries the C API. One shared object, two treaties.

The class exposes the day three ways on purpose, because they are this
repo's error conventions side by side: `part1` returns an `int`; `part2`
returns a `Variant`, so "no answer" is `nil` — `Option`, in GDScript's
spelling; and `part2_status` writes the answer into an `Array` and returns
`@GlobalScope.Error`, which is the C API's own out-parameter-plus-status
design landing in a runtime that already speaks it.

Two lessons this track teaches alone:

- **The `.gdextension` descriptor is the entire static contract, and
  nothing validates it.** Entry symbol, minimum engine version, one library
  path per platform, in an INI file no tool generates and no build step
  checks. A wrong row is not an error. The engine finds no match, loads
  nothing, and the failure arrives as `Identifier "Aoc20151201" not
  declared` — a message about the script, several layers from the file
  that was wrong. And the engine only looks for the descriptor during a
  filesystem scan, so a fresh checkout fails that way until
  `godot --headless --import` has run once.
- **A panic leaves the return slot unwritten.** The received wisdom is that
  gdext returns the type's default after a caught panic. It does not: it
  does not write the slot at all, so a call after a panic reads `0` from a
  fresh slot and the *previous answer* from one the caller just used.
  Nothing on the GDScript side can tell that from a result. The wasm trap
  that takes your instance's credibility and says so is the more forgiving
  design.

Godot's string is UTF-32, one code point per 32-bit unit, so every crossing
transcodes; the day's headless test measures it — `(🦀)` is three code
points to GDScript and six bytes to Rust — rather than asserting it.

**Front door.** The engine binary, which is also the headless one; no
editor is needed and nothing here opens one. Distros disagree on its name
and its major version, so `scripts/godot-bin.sh` owns that rule and
`just setup-godot` points at the download. The gdext crate declares a
Rust floor above the workspace's, so the feature is optional and never
enabled by default: the floor job proves the floor, the nix job proves the
feature. First build is minutes, because the crate generates bindings for
the whole engine API from a JSON file it ships.

## What the nine have in common

Nine runtimes, one header, and the header is the least of it. What each
one actually asked for was a decision the C API had already made without
knowing it: who allocates, where the length lives, which side owns the
error, what a panic becomes. C let every caller answer those for itself.
The runtimes that could not — because they have no allocator, or no return
value, or no function to export — are the ones that taught the most.
