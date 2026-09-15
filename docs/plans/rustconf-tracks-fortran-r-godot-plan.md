# Three more tracks: Fortran, R, Godot

Written 2026-09-15 against the `wip/wasm` tip (the wasm track as built, review follow-ups
included), Discussion #2 ("Language suggestions": Fortran, R, Godot — three bullets, no
comments), and `inkandswitch/backstitch` at its 2026-09-09 push. Status: **plans, nothing
built.** Facts below were checked where the machine allowed (nixpkgs channel versions and
download sizes, crates.io versions and MSRVs, the R manual's interface table, backstitch's
manifests and contributor guide); anything I could not run is marked `[verify]`.

The frame every track lands in is the one the wasm track set: a consumer of the C API a
day already exports (the **raw route**), plus the generated lap where a generator exists;
one new row in the "who read the header" list; one new answer to at least one of the four
disagreements (memory, types, errors, encoding); a `just days <track>-demo` recipe; a
Verify cell; a paragraph and a learnings entry in the day's README. **Day: 2015-12-01 for
all three**, because it is the day carrying every variation and because its C API —
`(const char *input, int *out) -> int` — is exactly the shape two of these runtimes are
built around.

What each track is *for*, in one line each:

| Track | The disagreement it adds evidence for | Header row it joins |
|---|---|---|
| Fortran | **by reference is the native convention** — the `int *` out-parameter costs nothing, and the ABI carries arguments the signature never shows | *transcribed, then enforced*: the compiler checks every call against the interface block you wrote, not against the header |
| R | **a runtime that discards the return value** — the status code the C API was designed around is unobservable through `.C()` | *unread and unchecked*: below Kotlin — no header, no types, and the return ignored |
| Godot | **a string that is UTF-32**, and **a third panic semantics** (not abort, not trap: logged and carried on) | *negotiated*: neither side reads a header; the extension asks the host for every function pointer by name at load |

---

## 1. Fortran — the by-reference language

### Why

Every consumer so far paid something to hand the C API a pointer to write through: Python
allocates `ffi.new("int *")`, Dart `calloc`s, wasm borrows from the module. Fortran passes
everything by reference by default. `integer(c_int), intent(out) :: answer` *is* the
`int *`, and the call site is the puzzle's own variable. The runtime that hides nothing
(Dart) and the runtime that needs nothing (Fortran) sit at the same end of the ceremony
spectrum for opposite reasons, which is worth a slide by itself.

Two more things only Fortran can show. Its strings are fixed-length and blank-padded with
no terminator, so `trim(text) // c_null_char` is the NUL written by hand for the third time
in the workshop, and the `trim` is load-bearing — the padding would be part of the string.
And gfortran's ABI passes **hidden trailing arguments** (the length of every `character`
argument, as a `size_t`, after the visible ones) — a treaty with clauses the signature does
not show, which is the reverse-direction lesson below.

C interop is in the Fortran *standard* (ISO_C_BINDING, F2003): `bind(C, name=…)`, the
`c_int`/`c_char`/`c_ptr` kinds, `c_null_char`, `c_f_pointer`. The treaty is in the language
spec rather than in a library, which no other track can say.

### Route

Raw only. There is no generator that emits Fortran interfaces from a C header, so the
interface block is written by hand from `include/aoc_2015_12_01.h` — copied, like Dart's
typedefs — and then, unlike Dart, **enforced**: gfortran type-checks every call against
the block. A wrong transcription is a compile error; a transcription that is wrong *and
self-consistent* compiles and corrupts, the same as Dart. That is the row.

```fortran
program solve
  use iso_c_binding, only: c_char, c_int, c_null_char
  implicit none
  interface
    function aoc_2015_12_01_part1(input, out) bind(C, name="aoc_2015_12_01_part1") result(status)
      import :: c_char, c_int
      character(kind=c_char), dimension(*), intent(in) :: input
      integer(c_int), intent(out) :: out
      integer(c_int) :: status
    end function
  end interface
  ! read inputs/2015-12-01.txt into text; then:
  status = aoc_2015_12_01_part1(trim(text) // c_null_char, answer)
  if (status /= 0) then ... end if
  print '(A,I0)', 'Part 1 🧮(🦀): ', answer
end program
```

`dimension(*)` (assumed-size array of `c_char`), not `character(len=*)`: the former is
what `const char *` binds to; the latter would add a hidden length argument the C side
never asked for — the first place an attendee meets the hidden-argument rule, from the
safe side.

### The absurd variant (Rust → Fortran)

2024-12-01 sorts two columns. LAPACK has `DLASRT`, a Fortran subroutine that sorts a
`double precision` array. Calling it from Rust —

```rust
unsafe extern "C" {
    fn dlasrt_(id: *const c_char, n: *const c_int, d: *mut f64, info: *mut c_int, id_len: usize);
}
```

— is the hidden-argument lesson from the other side: `id_len` is not in any Fortran
signature, gfortran's ABI puts it there, and leaving it off links clean and reads garbage.
Plus the trailing underscore, plus every argument by pointer including `n`. The day gains a
`lapack` feature and a fourth sort backend in the race (`benches/sort.rs` already races
three). nixpkgs has `lapack`/`openblas`; it joins the `full` shell like the other C
libraries. Optional — a day's worth of work on its own, and the only piece of this plan
that touches 2024-12-01.

### Files and plumbing

- `days/2015-12-01/fortran/solve.f90`; `just days fortran-demo <day>` mirroring
  `swift-demo` (compile, `-L target/debug -laoc_2015_12_01 -Wl,-rpath,…`, run).
- Label emoji: 🧮 `[confirm — every track has one; pick yours]`.
- self-check `probe_fortran`: `gfortran --version` (or `flang`); `just setup-fortran`.
- Verify cell `fortran`: gfortran ships on the ubuntu and macos runner images
  `[verify: macos-latest — it comes with Homebrew's gcc, which the image preinstalls]`;
  `fortran-lang/setup-fortran` is the action if not. Windows: off, like the others.
- nix: `gfortran` 15.2.0, **102 MiB** download — a track's own install, not the default
  shell, same rule as Node.
- README: row eight in the variant table; learnings for by-reference, `trim`+NUL, hidden
  lengths, and "the treaty is in the standard".

### Cost, risk, open questions

~1 day for the raw route, ~1 more for the LAPACK variant. Low risk: gfortran is boring in
the good way. Open: (1) do you want the LAPACK variant, and if so as a sort backend in
2024-12-01's race or a standalone absurdity on 2015-12-01; (2) gfortran vs LLVM `flang` —
gfortran is what nixpkgs and the runners have; (3) whether the day's Fortran also gets a
`.Fortran()` call from R (see below), which would be the workshop's first three-language
chain.

---

## 2. R — the vector interface

### Why

R's oldest FFI, `.C()`, has a treaty unlike any other in the repo, straight from the
manual's table (Writing R Extensions §5.2, checked):

| R vector | arrives in C as |
|---|---|
| `integer` | `int *` |
| `raw` | `unsigned char *` |
| `character` | `char **` |

and: *"the compiled code should not return anything except through its arguments: C
functions should be of type void."* The return value is **discarded**. Three consequences,
each a lesson:

1. **The string cannot be passed as a string.** A `character` vector arrives as `char **`
   — a pointer to pointers — and the C API takes `const char *`; hand it a character and
   Rust reads a pointer as text. The honest crossing is a `raw` vector with the NUL
   appended: `c(charToRaw(text), as.raw(0))` arrives as `unsigned char *`, byte for byte
   what the C side reads. The NUL by hand, a fourth time.
2. **The status code is unobservable.** `.C()` calls the function for its side effects on
   the arguments and hands back the modified argument list. The `-1` / `-2` the C API was
   designed around never reaches R. The out-parameter is the only channel — so the
   consumer initialises `out` to a value no answer can be and checks whether it changed.
   Back to in-band signalling, imposed by the *caller's* runtime rather than chosen by the
   library: "Errors: integers cross" has a footnote now — *if the other side looks*.
3. **Arguments are copied.** `.C()` duplicates its arguments before the call by default;
   the pointer the C side writes through is to R's copy, and the answer comes back in the
   returned list, not in the variable you passed. Memory: who allocates is the runtime,
   twice.

```r
dyn.load("days/target/debug/libaoc_2015_12_01.so")   # .dylib on macOS; same three-name search as Python's
text <- readChar("inputs/2015-12-01.txt", file.size("inputs/2015-12-01.txt"))
r <- .C("aoc_2015_12_01_part1", input = c(charToRaw(text), as.raw(0)), out = -999L)
if (r$out == -999L) stop("part1: no answer arrived — the status went where .C sends it")
cat(sprintf("Part 1 📊(🦀): %d\n", r$out))
```

Header row: **unread and unchecked**. No header, no declared types, no return value — R
matches on the symbol name and trusts the vector modes. Kotlin at least keeps an interface.

### Generated lap: extendr

`extendr-api` 0.9.0 (MSRV 1.65, fits the 1.85 floor) is the UniFFI of R: `#[extendr] fn
part1(input: &str) -> i32`, `extendr_module!`, and a `Result<i32, String>` becomes an R
error condition — so `tryCatch` is the typed channel, and the status code the raw route
lost comes back as an R condition. What extendr does with a Rust panic is the same
question the wasm demo asked — `[verify: extendr catches panics at the boundary and
raises an R error; confirm the version's behaviour before it goes in a README]`. Cost of
the lap: the crate links against R's headers and library at build time, which means the
`R` package in the shell that builds it.

### The cross-link nobody else can offer

`.Fortran()` exists next to `.C()`, with its own column in the same table (`character` →
`CHARACTER(255)`, `raw` → none). If the Fortran track lands first, R can call the
Fortran consumer's subroutine which calls Rust — three languages, two treaties, one
puzzle, and every argument by reference the whole way. A stretch, not a deliverable.

### Files and plumbing

- `days/2015-12-01/r/solve.R`; `just days r-demo <day>` (build, `Rscript`).
- Label emoji: 📊 `[confirm]`.
- self-check `probe_r`: `Rscript --version`; `just setup-r` (brew `r` on macOS; CRAN
  pointer on Linux; the devcontainer variant carries it).
- Verify cell `r`: `r-lib/actions/setup-r` on all three stock images.
- nix: `R` 4.5.3, **646 MiB** download — the heaviest toolchain in this plan by a wide
  margin, and the strongest argument for the devcontainer-variant pattern over anything in
  `shell.nix`. extendr adds nothing beyond it.
- README: row nine, learnings for the three consequences above.

### Cost, risk, open questions

~1 day raw, ~1–2 days extendr (the build-time link against `libR` under nix on darwin is
the unknown — R.framework paths). Open: (1) raw only, or both laps; (2) whether the
"status is discarded" lesson should be *the* R exhibit — it is the best single boundary
story in this plan — or whether it makes `.C()` look like a strawman when `.Call()` exists
(it does, but `.Call()` needs R's C API on the Rust side, which is extendr's job); (3)
whether to name the third route, R's `.Call()` via a hand-written SEXP shim, as "C again"
and leave it there.

---

## 3. Godot — the engine with its own string

### Why

Godot is the first consumer whose boundary is not a calling convention but an **API of
function pointers**. A GDExtension is a shared library with one entry symbol; at load,
the engine hands it `get_proc_address`, and the library asks for every engine function
it will ever call, by name, at runtime. The `.gdextension` descriptor (an INI: entry
symbol, library path per platform, minimum engine version) is the whole static contract.
Neither side reads a header. The row is **negotiated**.

Two disagreements get genuinely new evidence:

- **Encoding.** Godot's `String` is UTF-32 — one code point per 32-bit unit, no surrogates,
  no variable width — the only such runtime in the workshop. Every `GString` → `String`
  crossing transcodes UTF-32 → UTF-8 and back; `StringName` is interned on top. "It's just
  a string" gains a seventh line, and it is the one where the *engine* made the memory
  trade the JVM refused.
- **Errors.** GDScript has no exceptions. Godot's own convention is an integer enum
  (`@GlobalScope.Error`: `OK`, `FAILED`, `ERR_INVALID_PARAMETER`, …) — the C API's status
  codes land *natively*, for the first time. And a Rust panic inside a `#[func]` is neither
  Ex 2's abort nor wasm's trap: gdext catches it, prints an engine error, and returns the
  type's default `[verify against gdext 0.5 docs: "panics in #[func] are caught and
  reported, the call returns a default"]`. Three runtimes, three panic semantics — the
  process is gone / the instance keeps answering / the engine logs it and carries on — is
  a debrief slide.

### Route: the generated lap is the practical one, and that inversion is the lesson

The raw route exists — `gdextension_interface.h` is a C header, the init function is
`extern "C"`, godot-cpp is what hand-implementing it looks like — but it is several
hundred function pointers before a class registers. So Godot inverts the workshop's
order: the generated lap (**gdext**, the `godot` crate) is what anyone ships, and the raw
route is a *reading* exercise — open the header and the `.gdextension` file and find
what the crate generated for you. Where every other track asks "what did the generator
hide?", this one asks "what would it cost to not have one?"

```rust
#[derive(GodotClass)]
#[class(base = RefCounted)]
struct Day1;

#[godot_api]
impl Day1 {
    #[func] fn part1(&self, input: GString) -> i64 { … }          // UTF-32 → String, then the solver
    #[func] fn part2(&self, input: GString) -> Variant { … }      // nil on Err — GDScript's spelling of Option
    #[func] fn part2_status(&self, input: GString, out: Array) -> Error { … }  // the C API's convention, natively
}
```

Consumer: a GDScript test run headless — `godot --headless -s test.gd` — asserting the
statement examples, the nil, and the `Error` codes; that is the CI cell. The day gains a
`godot` feature (`dep:godot`, `api-4-6` to match nixpkgs' 4.6.3-stable) and a
`days/2015-12-01/godot/` directory: `project.godot`, `aoc.gdextension`, `test.gd`.

### Backstitch, and what it says about the boundary

`inkandswitch/backstitch` — "Real-Time Version Control for Godot", MIT, alpha, 249
stars, pushed 2026-09-09 — is the production Rust ↔ Godot boundary this track should point
at, and it is doing exactly what the track proposes:

- Its Rust core (`backstitch_rust_core`, cdylib + staticlib + rlib, `rust-version =
  1.91.1`) is a **GDExtension through gdext 0.5.4** — the same crate, one minor version
  back. The macOS build ships as a `.framework`, which is the per-platform-path clause of
  the `.gdextension` descriptor made visible.
- What crosses that boundary is a **CRDT**: `automerge` 0.9 plus `samod` for sync,
  `tree-sitter` with a Godot-resource grammar for parsing `.tscn` files, `fjall` for
  storage, `tokio` for the runtime. Nobody rewrote Automerge in GDScript or C++; the
  library exists in Rust, the engine is C++, the boundary is how a version-control engine
  reaches a game editor. That is the "when is FFI the right call" slide's answer for game
  tooling, with a repo behind it — and it lands on the two repos already in your GitHub,
  `automerge` and `patchwork-experiments`.
- **Where the extension API stops, they went into the engine.** Backstitch is a
  GDExtension *plus* a C++ editor module compiled into a custom-built Godot (minimum
  4.7-stable, pinned to a commit) for editor-UI hooks the extension interface does not
  expose. That is the boundary's ceiling, measured by a team that hit it: the generated
  lap gets you the runtime, and the last mile of editor integration cost them a fork.
- It also depends on `safer-ffi` — the crate your old `aoc-ffi` explored — presumably for
  the C-shaped edges gdext does not cover `[verify by reading their `rust/src`; I read
  the manifest, not the code]`.

For the track: a README paragraph and a learnings entry, not a dependency. If the Godot
track ever wants a second exhibit, "a `.tscn` file as an Automerge document, read through
the same boundary" is the shape — but that is a talk, not a day.

### Files and plumbing

- `days/2015-12-01/src/godot.rs` behind feature `godot`; `days/2015-12-01/godot/` as
  above; `just days godot-demo <day>` (build with the feature, copy or symlink the cdylib
  where the descriptor says, run headless).
- Label emoji: 🎮 `[confirm]`.
- **MSRV.** `godot` 0.5.5 declares `rust-version = 1.94`; the repo's floor is 1.85 and the
  `msrv` job checks the workspace with default features on 1.85. Target-scoped like
  wasm-bindgen is not enough here (it is a host build); it must be **feature-gated and
  optional**, and the floor job must not enable it — which the current job already does
  not. Say so in the day's Cargo.toml the way the wasm pin is explained.
- nix: `godot` 4.6.3-stable, **332 MiB**; the runners have no Godot, so the CI cell either
  downloads the official headless build or runs inside the nix job. `api-4-6`, then; 4.7
  is current upstream and backstitch's minimum, so expect to bump both together.
- gdext generates its bindings from the engine's API JSON at build — no libclang, but a
  cold build is minutes; rust-cache matters here more than anywhere else in the repo.
- self-check `probe_godot`: `godot --version` `[verify: the binary's name on each
  platform — `godot4` on some distros]`; `just setup-godot` points at the download page
  and the devcontainer variant.

### Cost, risk, open questions

~2–3 days: the crate is easy, the harness and the per-platform descriptor are the work,
and the CI cell needs a Godot binary from somewhere. Risks: the MSRV split (real, handled
by gating); gdext's version tracking Godot's (a 4.7 bump changes the feature flag); headless
tests for a `RefCounted` class need no scene, so the harness should stay a script. Open:
(1) `api-4-6` on nixpkgs or `api-4-7` on a downloaded binary; (2) whether the track's
error exhibit is `Variant` nil, the `Error` enum, or both (I'd ship both — they are the
Option and the status code side by side); (3) how much of backstitch goes on a slide
versus a README line.

---

## 4. Order, and what each one costs

| | Build | Toolchain download | New lesson (weight) |
|---|---|---|---|
| Fortran | ~1 day (+1 LAPACK) | 102 MiB | by reference native; hidden arguments (high) |
| Godot | ~2–3 days | 332 MiB + minutes of cold build | UTF-32; third panic semantics; backstitch (high) |
| R | ~1 day raw, +1–2 extendr | 646 MiB | the discarded return value (high, and the cheapest to demonstrate) |

Recommendation: **Fortran, then Godot, then R.** Fortran first because it is a day, has
no generator to decide about, and its two lessons are unlike anything in the deck.
Godot second because it is the one with a story attached (backstitch) and the one whose
inversion — the generated lap as the only practical route — the workshop's structure
currently has no example of. R last only because of the 646 MiB; its `.C()` lesson is the
best single boundary fact in this plan and could be shown in ten lines from any machine
with R on it, generator or not. If the order is by lesson rather than by cost, R moves
first.

Every one of these is a **days-library** variant (the reference side), not an exercise:
none has the "same wrapper, new runtime" property Ex 4 was built on — Fortran and R
consume the day's C API rather than the attendee's Ex 2 crate, and Godot needs a class.
An exercise shape exists for each (Fortran and R could bind `ex2_c_glue` directly, since
both take the header's shape as-is) but that is a second decision.

## 5. Things to settle before code

- **Which emojis.** Three labels, byte-exact in CI, the Kotlin lesson.
- **Toolchains: devcontainer variants or a `--arg tracks` shell list?** Three new
  variants is the current pattern; three more `home.nix` files. R's size makes a
  variant the only sane home for it regardless.
- **Does Discussion #2 get a reply?** The three bullets have no comments; a one-line
  pointer to this plan (or to the branches, once they exist) is the obvious answer, and
  it is an outbound message, so it goes through you.
- **LAPACK on 2024-12-01, yes or no.** The only proposal here that touches a golden day.
