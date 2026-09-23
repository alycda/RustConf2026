# Part 5: when an integer is not enough

Written 2026-09-23 against the `wasm` tip (PR #4, `079eeb2`), with PR #7
(`fix/ffi-hardening`, the repo-wide status-code table) assumed merged underneath it.
Status: **plan, nothing built.** Facts about this repo were checked against the tree:
2022-12-01's parser behavior was run, not read. Anything about a runtime or library I
could not run here is marked `[verify]`.

## Why this exists

Every boundary so far returns one integer. That covered every day on the menu, and in
the room it stopped being enough in two places:

- **Errors.** Ex 2's in-band `-1` collides with a real answer. 2015-12-01's statement
  examples answer `-1`, so an integer is not enough even to say "failed". The days
  moved to status + out-parameter for this reason, and #7 made the status codes one
  table.
- **Results.** 2022-12-01 is on the menu with the note "part 2 sums a top-3 internally,
  so the array is the thing worth exposing at the boundary". The workshop never
  exposes it. Some AoC answers are strings (2022-12-05's example answer is `CMZ`), and
  those cannot fit an integer at all.

The alternatives are flat arrays, structs, opaque handles, or bytes with a schema.
Each one moves a piece of the contract out of the type system and into prose, a
convention, or a schema file. **The pain points are the curriculum.** Part 5 is a
ladder: each rung fixes the previous rung's problem and brings its own.

The reference card already names the ownership contracts (`docs/reference-card.typ`,
"every pointer crossing the boundary has exactly one documented owner": by-value,
caller allocates, callee allocates with a paired `free_*`). The morning only exercises
the first one. Part 5 exercises the rest.

## Where it sits

After Ex 4 (wasm), as afternoon material. Like Ex 4's README, it carries no timing
until it has been in front of a room.

It depends on both branches below it:

- **#4 (wasm)**: Ex 4's `ex_alloc`/`ex_free` is the callee-lends-memory contract, met
  because JavaScript cannot allocate in linear memory. Part 5's rung B is the same
  contract in the other direction: the callee allocates the *result*, and the caller
  has to give it back. Ex 4 already teaches why `free` takes the size back.
- **#7 (status table)**: rung A extends `-3` ("the answer does not fit the
  out-parameter") to mean "and here is the size it needs". Without one table, that
  extension would be a per-day convention again.

Merge order: #7, then #3, then #4 (the owner's note on #3: "merge AFTER #7"). Part 5
branches from #4's head. Two follow-ups #4 needs, whatever happens to this plan:

- **`days/2015-12-01/wasm/src/raw.ts:72`** has its own `MEANING` table listing only `-1`
  and `-2`. After #7, 2015-12-01 can also return `-3` and `-4`, and they would print as
  "unknown status". Control flow is already correct, because any nonzero status
  becomes a `StatusError`. Align the table with `days/README.md` when `develop` is
  merged into #4.
- `usize` is 32 bits on `wasm32`. A day that exports `usize` across the boundary has a
  different header on each target. Part 5's exports use fixed-width types for that
  reason (see "Element type" below).

## The day: 2022-12-01, and what an array exposes

**Recommendation: 2022-12-01 for rungs A to D.** It is on the menu, the menu already
points at its array, and its answers are small. The one new cost is that it has no
`c_api.rs`, so writing one is part of Part 5's work.

What the day computes today (`src/lib.rs`): `Day(Vec<usize>)` is one total per elf,
parsed with `split("\n\n")` and then `lines()`. Part 1 is the max; part 2 is the sum of
the top three. **Observed** (a scratch crate against the `wasm` tree):

| Input | Elves | Part 1 | Part 2 |
|---|---|---|---|
| `""` | `[0]` | `0` | `0` |
| `"1\n2\n\n3\n\n"` (trailing blank line) | `[3, 3, 0]` | `3` | `6` |
| `"1\r\n2\r\n\r\n3\r\n"` (CRLF) | parse error | – | – |
| `"5"` | `[5]` | `5` | `5` |

Three things follow, and all three are the lesson rather than bugs to fix first:

1. **An empty input yields a phantom elf with 0 calories.** A blank line at the end
   yields another one. As long as the answer is `max` or a top-3 sum, a 0-calorie elf
   is invisible: it never wins, and it adds 0. **The moment the boundary returns the
   per-elf array, the phantom elf is observable.** Widening the return type turns
   behavior nobody could see into the interface. That is Hyrum's Law ("Defaults and
   absence") happening live, and it is the first demo of Part 5.
2. **`"no elves"` is unreachable.** `part1`'s `ok_or_else(|| "no elves")` can never
   fire, because the parser always produces at least one elf. With an array return,
   "empty" becomes a real case, and the C side has to decide what an empty result
   looks like. That decision matters most in rung B (see "Zero-length allocations").
3. **CRLF is a hard parse error**, not a different answer. `split("\n\n")` never
   matches `"\r\n\r\n"`, so the blank line becomes a line containing `"\r"` inside
   one block. Windows attendees who paste from a file hit this before any FFI.
   Decide in the plan's first commit whether to fix the parser (and say so as a
   behavior change) or to keep it as a teaching case. Recommendation: fix it, and
   make the phantom elf the teaching case. The phantom elf teaches about the
   boundary; CRLF only teaches about editors.

Fewer than three elves also matters: part 2 takes `take(3)`, so a top-3 array has
fewer than three entries. The array's length is data-dependent even when it looks
fixed, so it forces a length in the contract.

**Rung E wants a string answer**, because bytes are only obviously necessary when
nothing numeric can carry the result. Options:

- **Add 2022-12-05** (answer `CMZ` for the example). That is a new day, so it needs
  its solver, tests, menu row and Ex 1 row, and it is a spoiler risk if an attendee
  has picked it. It must not go on the Ex 1 menu until Part 5 ships.
- **Stay on 2022-12-01 and return a record**: `{ totals: [u64], best_index: u32,
  top3: [u64; ≤3] }`. This keeps one thread through the session but makes the result
  feel constructed.

Recommendation: stay on 2022-12-01 for the exercise, and use 2022-12-05 as a demo
only in rung E. Settle this before any code (see the last section).

### Element type

`u64`, not `usize`. The header must be the same on every target, and `usize` is 32
bits on `wasm32`. `u64` also means JavaScript receives a `BigUint64Array` or `BigInt`,
which Ex 4 already taught ("`i64` is a `BigInt`"). Totals fit in `u32` in practice, and
choosing `u32` would dodge that lesson. The workshop's rule has been to choose the
type that shows the boundary, not the one that hides it.

## The ladder

Each rung has: the export (as cbindgen would render it), what it teaches, the pain
point, and which rubric row it exercises. The rubrics are Hyrum's Law and behavioral
subtyping (Liskov/Wing), from alycda/dotfiles#178. Signatures are proposals.

### Rung A: the caller supplies the buffer

```c
/* Writes one total per elf into out[0..*out_len]. Returns 0, or -3 with
 * *out_len set to the length needed when cap is too small. */
int aoc_2022_12_01_totals(const char *input, uint64_t *out, size_t cap, size_t *out_len);
```

- **Ownership is trivial.** The caller allocates and the caller frees, and nothing
  crosses that needs to go back.
- **The pain is agreeing on size.** Either the caller calls twice (first with
  `cap = 0` to learn the length, the `snprintf` idiom), or it guesses and retries on
  `-3`.
- **It changes #7's contract, and the change has to be explicit.** #7's table says
  "on any nonzero code `*out` is untouched". Here, `-3` has to *write* `*out_len`, or
  the caller cannot size the retry. The table needs one added sentence: "a function
  that fills a caller's buffer also writes the required length on `-3`; the buffer
  itself stays untouched". This is Hyrum's remedy 2 (promise it explicitly), applied
  before anyone depends on either reading.
- **A precondition to name:** `out` may be null only when `cap == 0`. That is the one
  place a null out-parameter is not `-1`, and it needs a test for each direction.
- **Rubric:** Hyrum, "Failure modes" (what a too-small buffer leaves behind), and
  Liskov, "Preconditions" (the null-with-zero-cap exception is part of the contract;
  a track that passes null with a nonzero `cap` gets `-1`).

### Rung B: the library allocates, and exports a free function

```c
int  aoc_2022_12_01_totals_alloc(const char *input, uint64_t **out_ptr, size_t *out_len);
void aoc_2022_12_01_totals_free(uint64_t *ptr, size_t len);
```

Rust side: `Vec<u64>` → `into_boxed_slice()` (so capacity equals length) →
`Box::into_raw`. `free` rebuilds the box with `slice_from_raw_parts_mut(ptr, len)`.

- **This is Ex 4's allocator pair run the other way.** There the module lent memory
  for the input. Here it owns the output until the caller gives it back.
- **Freeing with the wrong allocator is undefined behavior.** Calling libc `free()` on
  this pointer is UB. The reference card's rule ("free with the allocator that
  allocated") becomes something attendees do, not just read.
- **`free` takes the length back**, for the reason Ex 4 already gave: the layout has
  to be reconstructed, and no header carries it. This is the rung's most important
  detail, because of the next point.
- **Finalizer APIs assume `void free(void *)`.** If it holds, this is the finding
  that motivates rung D. `[verify]` each of these against the pinned versions:
  - Dart's `NativeFinalizer` takes a native function of type
    `Void Function(Pointer<Void>)`, one argument, so `free(ptr, len)` does not fit.
    Dart needs `try`/`finally`, or a one-argument free.
  - cffi's `ffi.gc(cdata, destructor)` calls the destructor with one argument. A
    Python closure can capture `len`, so Python gets away with it.
  - JavaScript's `FinalizationRegistry` is non-deterministic, so the wasm track keeps
    `raw.ts`'s `acquireUseRelease` pattern (Effect's release runs on success, failure
    and interruption).
  - Swift uses `defer`, which is manual but scoped.
  - JNA returns a raw `Pointer`, so freeing is explicit, or done through a `Cleaner`.
  - Fortran maps the pointer with `c_f_pointer(ptr, array, [len])`, and then an
    explicit `call`.

  Every runtime's automatic cleanup wants one pointer. A free that needs the length is
  a manual obligation in exactly the runtimes where people expect not to have one.
- **Zero-length allocations.** `Box<[u64]>` of length 0 is a dangling, non-null,
  aligned pointer. The contract has to say one of two things: `free(ptr, 0)` is a
  no-op and required anyway, or a zero-length result returns null. Recommendation:
  return null for an empty result and make `free(NULL, 0)` a no-op, as `ex_free`
  already does. Test both.
- **The wasm-specific trap.** Allocating the result can grow linear memory, and
  growth detaches every existing `ArrayBuffer` view. `raw.ts` already documents this
  for inputs ("allocation detaches the old one"). For outputs it is the whole lesson:
  the view must be created *after* the call that allocated. A track that caches
  `new BigUint64Array(memory.buffer, …)` from before the call reads a detached buffer.
- **Rubric:** Liskov, "Postconditions" (the pointer is valid until `free`, and not
  after), and Hyrum, "Layout and ABI" ("ownership of returned memory" is on the
  rubric's list).

### Rung C: `#[repr(C)]` structs

```c
typedef struct { uint32_t index; uint64_t total; } AocElf;   /* 4 bytes of padding */
int aoc_2022_12_01_top3(const char *input, AocElf out[3], size_t *out_len);
```

- **The layout is transcribed by hand in every track except Swift**, which imports
  the header. Dart `Struct` subclasses, JNA `Structure` with `getFieldOrder`, cffi
  `cdef` (cffi parses C, so it is closest to Swift). cbindgen emits the struct, and
  nothing checks that the transcriptions match it.
- **The field order is chosen to create padding.** `u32` then `u64` puts 4 bytes of
  padding after `index` on every target the workshop runs on. A track that declares
  it as packed, or swaps the fields, reads `total` from the wrong offset.
- **The demo is the review's finding D3, run on purpose.** A binding with the wrong
  width or offset reads back correctly *when the memory started zeroed* and the
  values are small. Fill the output with `0xAA` before the call, and the wrong
  binding shows garbage. Every Part 5 track fills with that pattern from the start.
  That is the fix D3 asked for in the morning tracks, built in here rather than
  retrofitted.
- **Data-carrying enums have no C form.** A demo slide, not an exercise:
  `Option<u64>`, `Result`, and an enum with fields all need a tagged-union
  transcription, and that is where rung E's schema starts to look attractive.
- **Rubric:** Hyrum, "Layout and ABI" ("at an FFI boundary the implicit interface
  *is* the interface; there is no type checker on the far side").

### Rung D: opaque handle and accessor functions

```c
typedef struct AocElves AocElves;
int      aoc_2022_12_01_parse(const char *input, AocElves **out);
size_t   aoc_2022_12_01_len(const AocElves *elves);
int      aoc_2022_12_01_get(const AocElves *elves, size_t i, uint64_t *out);
void     aoc_2022_12_01_free(AocElves *elves);
```

- **There is no layout contract.** The handle owns its length, so `free` takes one
  pointer. That fixes rung B's finalizer problem: every runtime's automatic cleanup
  now fits (`NativeFinalizer`, `ffi.gc` without a closure, JNA `Cleaner`).
- **The cost is chattiness**: one call per element. It is worth one benchmark in the
  day's `benches/` against rung B's single copy. The number will be large on wasm,
  where every call crosses the JS/wasm edge.
- **The repo already teaches this from the other side.** 2015-12-01's libcaca banner
  is the opaque-canvas pattern in the *import* direction (Rust holding a C library's
  handle). Rung D is the same pattern exported. That is a one-slide cross-reference,
  not new material.
- `aoc_2022_12_01_get` with `i >= len` returns `-1`. That is a precondition failure,
  and it must not panic (it would be an index out of bounds). It gets a test.
- **Rubric:** Liskov, "History constraint" (no access after `free`; a handle has a
  lifecycle a flat array does not have).
- `[verify]`: that cbindgen emits `typedef struct AocElves AocElves;` for a
  non-`repr(C)` type referenced only by pointer. That is its documented behavior for
  opaque types, but it has not been run on this tree.

### Rung E: bytes with a schema

One function shape for everything, and the contract moves out of the header:

```c
int  aoc_2022_12_01_encode(const char *input, uint8_t **out_ptr, size_t *out_len);
void aoc_2022_12_01_bytes_free(uint8_t *ptr, size_t len);
```

Two steps, in this order:

1. **A wire format the attendees write themselves.** A `u32` count, then `count ×
   u64`, all little-endian. Every track now writes a parser, chooses an endianness
   (and finds out that `DataView` defaults to big-endian), and has no way to
   add a field without breaking every reader. This step exists to make the next one
   feel necessary rather than imposed.
2. **One schema library.** Recommendation: **protobuf**, because the schema file is an
   explicit artifact. That artifact is what the rubric discussion needs:

   ```proto
   message Totals { repeated uint64 totals = 1; }
   // v2: optional uint32 best_index = 2;
   ```

   - **Hyrum:** field numbers, unknown-field handling and defaults become the
     implicit interface. proto3 cannot tell `0` from unset on a plain scalar unless
     the field is declared `optional`. This is the phantom elf again: "no elves" and
     "one elf with 0 calories" encode differently only if the schema is written to
     tell them apart.
   - **Liskov:** schema evolution is behavioral subtyping between versions. A v2
     reader must accept v1 messages (the precondition is not strengthened). A v1
     reader must survive v2 messages (it ignores unknown fields). Renumbering a field
     is the demo of what breaks.
   - **The type checker is gone again.** A mismatched header at least had declared
     types. A mismatched decoder fails at runtime, or decodes the wrong data without
     any error.

Libraries per track, all `[verify]` for versions, nixpkgs availability and how much
code generation they need: Rust `prost`, Python `protobuf`, Dart `protobuf` plus
`protoc_plugin`, Kotlin `protobuf-kotlin`/`protobuf-java`, Swift `swift-protobuf`,
TypeScript `@bufbuild/protobuf`. All of them need `protoc` or `buf`, the kind of
installer the README exists to keep off venue Wi-Fi. For an afternoon session this
is acceptable only behind `nix-shell --arg full true` (or the flake's `.#full`
after #3) and a devcontainer variant, like wasm's.

**Contrasts, lecture only:**
- **CBOR**: self-describing, no codegen, so the contract lives in prose. It is the
  strongest Hyrum example, but it is a weaker exercise because there is nothing to
  evolve.
- **postcard**: effectively Rust-only. It is the example for "the format you choose
  decides who can consume it".
- **FlatBuffers / Cap'n Proto**: zero-copy brings lifetimes back across the boundary.
  The decoded view borrows the buffer, so rung B's "valid until free" returns. This
  makes a good last slide for the rung.

### Closing slide: the industry climbed the same ladder

- **wasm-bindgen** (Ex 4's generated lap): returning `Vec<u64>` or `String` makes the
  glue copy out and free for you. Rung B, generated. `[verify]` that `Vec<u64>` maps
  to `BigUint64Array` in the pinned version.
- **The wasm Component Model's canonical ABI**: lists and strings are lowered to
  `(ptr, len)`, allocated through an exported `cabi_realloc`. Rungs A and B,
  standardized. This connects Part 5 back to Part 4.
- **UniFFI**: compound types cross as a `RustBuffer` with UniFFI's own serialization.
  Rung E, industrialized.
- **Diplomat**: `[verify]` its current design before it goes on a slide.

## Tracks and scope

| Track | A (caller buffer) | B (callee allocates) | C, D, E |
|---|---|---|---|
| Python (cffi) | exercise | exercise: `ffi.gc` with a closure over `len` | demo reference in the day |
| Dart | exercise | exercise: `try`/`finally`; `NativeFinalizer` does not fit a two-argument free | demo reference in the day |
| Kotlin/JNA | exercise | exercise: explicit free, or `Cleaner` | – |
| Swift | exercise | exercise: `defer` | – |
| TypeScript/wasm (#4) | exercise | exercise: views created after the allocating call | rung D benchmark |
| Fortran | – | day demo: `c_f_pointer`, then an explicit free | – |
| R | day demo: see below | not through `.C()` | – |
| Godot | – | – | – |

**The exercise is rungs A and B in one track of the attendee's choice** (the four Ex 3
tracks plus Ex 4's TypeScript). That is where the per-runtime pain differs most.
Rungs C, D and E are live demos on the day (Rust, plus Python and TypeScript as the
worked references), with the other tracks as take-home material in the repo.

**R, `[verify]`:**
- `.C()` copies its arguments and cannot hand back a pointer the callee allocated, so
  rung B is impossible through `.C()`. That is worth one line in the R track's README,
  and it points at extendr/`.Call()` exactly as the 2015-12-01 R track already does.
- Rung A works: pass a `double()` vector of length `cap`, and it comes back copied in
  the result list.
- R has no 64-bit integer type (integers are 32-bit, doubles carry a 53-bit
  mantissa), so the `u64` totals need a stated conversion. The totals are small, so
  doubles are exact here, but that has to be written down. Say so in the R track's
  README rather than choosing `u32` for R's sake.

**Godot:** out of scope. gdext hands back `PackedInt64Array` natively, which is the
generated route and not a C API lesson.

## Files and plumbing (mirroring #4)

- `days/2022-12-01/src/c_api.rs`: rungs A to E, with #7's module lints, the status
  constants and the `read_input` safety paragraph; `cbindgen.toml`; `build.rs` if the
  day needs one; `Cargo.toml` `crate-type = ["rlib", "cdylib"]`.
- `days/2022-12-01/{python,dart,wasm}/`: worked references for A and B, plus the C, D
  and E demos in Python and TypeScript.
- `exercises/ex5-*`: named after the Part 5 title once there is one. The scaffold
  takes the attendee's Ex 1 solver (which returns `i64`) and adds a second solver
  function that returns `Vec<u64>`, so every day on the menu can do Part 5 and not
  only 2022-12-01. `[decision]`: some days have no natural array. For those the
  exercise's array is "every line's contribution", which works for every day on the
  menu.
- `.github/ci/exercises/ex5-*`: the solved overlay (2025 day 3, as for Ex 1 to 4;
  per-line joltages are the natural array).
- `just exercises ex5-<track>`, a Verify cell per track (anchored greps: `grep -E
  '…: N$'`, not `-F`), a `just days <track>-demo 2022-12-01` recipe per track, and a
  book chapter after #4's.
- `days/README.md`: the added `-3` sentence from rung A, and the menu row for
  2022-12-01 updated to say what it exports.

## Tests the rungs need from the start

These are the review's findings, applied up front rather than retrofitted:

- **Poisoned output slots** (`0xAA…`) in every Rust test and every track, so a
  wrong-width or wrong-offset binding fails (D3).
- **Hostile inputs in every track's Verify cell**: null, non-UTF-8, `-3` (buffer too
  small), and `i >= len` for rung D. A deleted status check must turn a cell red (D2).
- **The phantom elf pinned as current behavior** (`""` → `[0]`, trailing blank line →
  a trailing 0). If the parser is fixed later, the test fails, and the fix is then a
  visible behavior change.
- **Round-trip property tests in Rust** for rungs A, B and E: for every input,
  encode then decode equals `Day`'s totals. That is Liskov for the codec, checked by
  Rust before any track gets to disagree.
- **Differential across rungs**: A, B, D and E return the same totals for the same
  input. Every rung is a stand-in for the others, and this pins that.
- **Leak checking for rung B** in at least one track. `[verify]`: the cheapest option
  on the CI runners (ASan on the Rust side? a counting allocator behind a test
  feature?).

## Cost, risk, open questions

- **Size.** Rungs A and B across five tracks, plus three demo rungs, is several
  sessions of work, the same order as #4.
- **Toolchain weight.** Rung E's `protoc` or `buf` plus per-language plugins is the
  heaviest thing the repo would ship. Keep it behind `full` and a devcontainer
  variant, never in the five-tool contract.
- **Spoilers.** Adding 2022-12-05 for rung E puts a day in the repo that attendees
  might pick for Ex 1. Keep it off the menu, or accept the risk explicitly.
- **#7's contract changes.** The `-3`-writes-`*out_len` sentence changes #7's table.
  It should go into `days/README.md` in the same commit as the first function that
  depends on it, not before.
- **wasm and `extern "C"` with multi-word returns.** All the signatures above return
  a single `int` and use out-parameters, so wasm32's C ABI for struct returns never
  comes up. Keep it that way deliberately, and write down why. `[verify]` the current
  `wasm32-unknown-unknown` `extern "C"` ABI before anyone proposes returning a struct
  by value.

## Order

1. After #7, #3 and #4 land: branch from #4's head. The `raw.ts` `MEANING` fix lands
   on #4, not here.
2. `days/2022-12-01/src/c_api.rs` with rungs A and B, their Rust tests (poisoned
   slots, phantom elf, round trip), and the `-3` table sentence.
3. Python and TypeScript worked references for A and B. They are the two runtimes
   whose cleanup mechanisms differ most (a GC finalizer, and Effect's
   acquire/release).
4. `exercises/ex5-*` scaffold, overlay, `just` recipes, Verify cells.
5. Rungs C and D as demos, and the rung D benchmark.
6. Rung E last. It is the only rung with new toolchain weight, so it stays droppable
   without affecting anything before it.
7. The book chapter, written last, from what the rungs actually showed.

## Things to settle before code

1. **Rung E's day**: 2022-12-01 returning a record, or 2022-12-05 as a demo-only day.
   Recommendation: 2022-12-01 for the exercise, 2022-12-05 as the rung E demo.
2. **The CRLF parse error**: fix it before Part 5 (recommended, as a stated behavior
   change), or keep it as a teaching case.
3. **Exercise array for days with no natural one**: "every line's contribution", or
   restrict Ex 5 to days that have an array.
4. **Codec**: protobuf (recommended) or CBOR for rung E's exercise.
5. **The name.** "Part 5" is a slot. The title should say what hurts: something like
   *More Than an Integer*.
