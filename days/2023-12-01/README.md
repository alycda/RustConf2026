# Day 1: Trebuchet?!

Each line hides a two-digit calibration value — the first digit in the line and
the last, combined (`a1b2c3d4e5f` → `15`) — and the answer is their sum. Part 2
says the digits may also be spelled out, and that spelled-out digits may
overlap: `eightwo` is an `8` and a `2`. A scan and a sum.

No exotic C library here yet. What this day carries instead is the other half
of the workshop: the direction reverses, and Rust becomes the library. The
puzzle is a good fit for it — two functions, one string in, one number out, and
an answer that fits in a `uint32_t` — so the header stays small enough to read
in one go, which is what makes it a decent thing to hand to a fourth language.

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline) | `sum_calibration_pure_rust`, `sum_calibration_with_words_pure_rust` in `src/lib.rs` |
| cbindgen C API | Rust → C (exported) | `src/c_api.rs`, `cbindgen.toml` |
| Swift | C → Swift | `swift/solve.swift`, `swift/module.modulemap` |

## The variants

**Pure Rust.** `sum_calibration_pure_rust` and
`sum_calibration_with_words_pure_rust` are the puzzle solved the ordinary way,
plus `checked_` variants that return `Option` instead of panicking. They are
real `pub fn`s rather than bodies inside `Solution::part1`/`part2` for the same
reason 2015-12-01's `sum_pure_rust` and 2021-12-02's `dead_reckon_pure_rust`
are: the C API needs an entry point whose meaning doesn't depend on which
backend a later cargo feature compiles in, and a benchmark needs something to
race one against.

**cbindgen C API (`src/c_api.rs`, Exercise 2).** Two `extern "C"` functions,
`aoc_2023_12_01_part1`/`part2`, built around out-parameters and status codes
rather than `Result`, because a panic unwinding across an `extern "C"` frame is
undefined behavior. `0` on success, `-1` for a null pointer or invalid UTF-8,
`-2` for a total too large for a `uint32_t`. `just days bindgen 2023-12-01`
generates the header (not committed — see `days/.gitignore`); cbindgen is
pointed at `src/c_api.rs` rather than at the crate, so a day that later grows a
module full of *imported* `extern "C"` declarations doesn't get somebody else's
API redeclared inside its own header.

**Swift (`swift/solve.swift`, Exercise 3).** The first track in this repo where
nobody retypes the header. `swift/module.modulemap` points clang at the
generated header, `import AocDay` puts both functions in scope as ordinary
typed Swift functions, and every call is checked against the real declarations
at build time. `just days swift-demo 2023-12-01` regenerates the header, builds
the `cdylib`, compiles and runs it (needs `just setup-swift` once).

## Running things

```sh
cd days/2023-12-01 && cargo run                # pure Rust
cargo test -p aoc-2023-12-01                   # 8 tests, Rust side and C side

just days bench 2023-12-01                     # criterion: parse, both parts, digit density
just days bindgen 2023-12-01                   # regenerate include/aoc_2023_12_01.h
just days swift-demo 2023-12-01                # build + header + run swift/solve.swift
```

## The four Exercise 3 tracks, side by side

This is the fourth language track in the repo, and the four of them now cover
the whole spread of how much a runtime can know about a C header:

| Track | Day | Where the signature lives | When a mismatch is caught |
|---|---|---|---|
| Python / cffi | 2015-12-01 | read from the real header at runtime | script start |
| Dart / dart:ffi | 2015-12-05 | written twice, native types and Dart types | never (runtime, if at all) |
| Kotlin / JNA | 2021-12-02 | written once, marshalled by convention | never (renames at load time) |
| Swift | 2023-12-01 | not written at all — clang reads the header | compile time |

Swift is the only one where a changed header is a build failure. That is not a
claim from the documentation: replacing the out-parameter's type with `Int64`
in a throwaway file fails with

```
error: cannot convert value of type 'UnsafeMutablePointer<Int64>' to
       expected argument type 'UnsafeMutablePointer<UInt32>'
```

— the header's own type, in the error message, before anything runs.

The cost is on the other side. The three interpreted tracks `dlopen` a path
they compute at runtime, so one script covers both cargo profiles; Swift links,
so "debug or release" becomes `-L`/`-rpath` flags in the `swift-demo` recipe.
The search still happens. It just happens once, earlier, and by the build.

## Benchmarks

`benches/day.rs`, criterion medians, aarch64, bench profile. A generated input
of 1000 lines × 40 characters at 10% digits — real puzzle-input scale, and
built from a fixed seed so two runs are comparable:

| | parse | part 1 | part 2 |
|---|---|---|---|
| generated / 1000 lines | ~15.7 µs | ~63.3 µs | ~117.0 µs |
| statement example (7 lines) | ~130 ns | ~133 ns | ~272 ns |

Read on its own that says "part 2 costs 1.85x part 1, and solving costs 4–7x
parsing". The second benchmark group is there because that reading is mostly
wrong.

**What the characters are matters more than which part is running.** Holding
length, line count and everything else fixed, and varying only the share of
characters that are digits, part 2 alone:

| digits | part 2 |
|---|---|
| 0% | ~96 µs |
| 10% | ~121 µs |
| 50% | ~208 µs |
| 100% | ~191 µs |

That was written expecting the opposite. `digit_at` returns the moment it sees
a literal digit and never reaches the nine-word scan, so an input of nothing but
digits should be the cheapest and an input with none the dearest. Instead the
case doing the *most* word scanning is the fastest, and the worst case is
neither extreme but the middle — a spread of 2.2x, wider than the gap between
the two parts.

Two throwaway probes (scratchpad, not committed — crude beside criterion's
warmed-up medians, so directions rather than figures) say where it comes from,
and neither is the word matching:

- Running the same scan **without the intermediate `Vec`** — `first` and `last`
  kept in two locals — leaves the 0% column alone and takes a large bite out of
  every other one, growing with how many digits there are to push.
  `calibration_value` collects every digit it finds before taking the first and
  the last, and at 0% that `collect()` yields nothing and never touches the
  heap at all.
- At a fixed 50% density, laying the digits out **strictly alternating** rather
  than at random is reliably about a quarter faster. Same length, same density,
  same work — the only difference is whether the digit/not-digit branch is
  predictable, which is why 50% is the worst column and both extremes are not.

The generated input is good for timing and wrong for answers: its characters
are uniform over `a-z0-9`, where a real input is *built* to contain spelled-out
digits. Whatever number it sums to is not a puzzle answer.

## Learnings

- **Exposing a day to C found a panic the day could not reach on its own.**
  `calibration_value` walked the line by byte offset and sliced at each one, so
  any line with a multi-byte character died on "start byte index 1 is not a
  char boundary" — and not only in dev, the way an overflow check would; a
  slice index panics in every profile. Nothing in the repo could get there:
  `main.rs` reads a real input, real inputs are ASCII, both statement examples
  are ASCII, the suite was green and the answer was right. Then the C API
  arrives and promises to accept *valid UTF-8*, which `é1` is. The bug was
  always there; what changed is that the caller became someone who could type
  it, and that a panic stopped being a backtrace and started being undefined
  behavior. `char_indices()` yields exactly the offsets `0..len` did for ASCII,
  so no answer moved.
- **The same guard, on a different day, can be untestable — and should say so.**
  2021-12-02 needed a status code for an answer too large for its integer,
  because a genuine puzzle input already landed within ~10% of `i32::MAX` and
  eight lines of C-caller input blew past it. This day inherits the *reasoning*
  — arithmetic that cannot overflow, rather than a `catch_unwind` around a
  panic that only exists where `overflow-checks` is on — but not the
  reachability: a line here is worth at most 99, so `-2` needs ~43 million
  lines, at least 86 MB. So `checked_total` is split out from `checked_sum`
  precisely so the *refusal* is testable at all even though the *path* isn't,
  and `c_api` states plainly which half is covered. An FFI contract you cannot
  afford to exercise is a weaker promise than one you can, and the place to
  admit that is the header's own doc comment.
- **Swift is the only track where the header is checked, and that changes what
  the exercise teaches.** cffi reads the real header at runtime; dart:ffi and
  JNA read nothing and take your word for it, in two spellings or one. Swift
  hands the header to clang and type-checks every call against it, so this is
  the one track where a signature that drifted is a build failure with the
  header's own types in the error text. The trade is real and worth stating in
  the same breath: a compiled track has to find the library at link time, so
  the "debug or release" search that lives in three `dlopen` scripts moves into
  the recipe's `-L`/`-rpath` flags instead. Nothing disappeared; it moved
  earlier.
- **`#filePath` is not `__file__`, and it fails in the direction that looks
  like it works.** Every other track self-locates at runtime — `__file__`,
  `Platform.script`, `codeSource.location` — and Swift appears to offer the
  same thing. It doesn't: `#filePath` is baked in at compile time as *verbatim
  whatever path swiftc was given*, so a relative invocation leaves a relative
  path inside the binary that then resolves against whoever's working directory
  it is run from. Compiled one way and run from `/`, the script confidently
  claimed to live in `/swift`. `Bundle.main.bundlePath` asks the runtime where
  the executable actually is. The tell was running the built binary from an
  unrelated directory — which is a thing worth doing to every track's
  self-location, because the failure is silent and the fix is one line.
- **A compiled track needs an ignore rule that an interpreted one doesn't.**
  Python and Dart compile nothing; Kotlin's `solve.jar` happened to match the
  repo-wide `*.jar`. A Swift binary called `solve` matches no pattern in either
  `.gitignore` and would have been committed on the first run of the demo. It
  goes in `.build/` — Swift's own name for build output, already ignored — and
  `solve.swift` climbs one extra directory as a result. Every new toolchain
  brings an artifact the existing rules were not written for; checking `jj
  status` after the *first* successful run, rather than after the commit, is
  what catches it.
- **Branching early means inheriting a repo that is missing pieces, and the
  fix is to port them, not to invent them.** This day's baseline predates the
  `bindgen` recipe, the `**/include/*.h` ignore rule, the `bench` recipe and
  the criterion pin — all added on other days' lines. Each was ported across
  rather than rewritten, and the shared-file changes were kept in their own
  commits rather than folded into the day's work. The check worth doing at the
  end is the lockfile: the regenerated `days/Cargo.lock` is identical
  package-for-package to the one CI has already proved elsewhere, modulo the
  two day crates that exist on this line and not on that one.
- **A benchmark that only confirms what you expected has told you nothing.**
  The digit-density group exists to prove a small, obvious claim — digits
  short-circuit the word scan, so more digits should be faster — and it
  disproved it, twice, with the worst case in the middle. The mechanism turned
  out to be an intermediate `Vec` and a mispredicted branch: two things the
  puzzle is not about, in a day whose entire subject is string matching. This
  is the number that would have been quietly attributed to a C library if one
  had been wired in first and benched at a single density. Measure the plain
  version, at more than one shape, *before* there is anything to compare it to.
