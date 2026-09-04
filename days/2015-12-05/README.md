# Day 5: Doesn't He Have Intern-Elves For This?

Every line of the input is a candidate string; both parts count how many
satisfy a small ruleset, and part 2 swaps the ruleset for a harder one.
A thousand sixteen-character lines, no arithmetic, no parsing to speak
of — which is why this day carries the *pattern-matching* variations:
"does this text contain any of these patterns" is a question two very
serious C libraries exist to answer, and the puzzle is small enough that
neither of them competes for attention with the boundary being
demonstrated.

Where 2015-12-01's libraries were the wrong tool on purpose (a JIT
compiler to add up ±1, an ASCII-art engine to print the answer), these
two are the *right* tool at three orders of magnitude the wrong scale.
That difference is what makes this day's benchmark worth reading: one of
them actually wins.

Five solves of the same puzzle live in this one branch (each was built and verified independently):

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline, the default build) | `is_nice_pure_rust`, `is_nice_v2_pure_rust` in `src/lib.rs` |
| vectorscan | Rust → C (`hyperscan` feature) | `src/hyperscan.rs` |
| ICU regex | Rust → C (`icu` feature) | `src/icu.rs`, `src/icu_shim.c` |
| cbindgen C API | Rust → C (exported) | `src/c_api.rs`, `cbindgen.toml` |
| Dart via dart:ffi | C → Dart | `dart/solve.dart`, `dart/pubspec.yaml` |

Both C libraries are default-off cargo features. A bare `cargo build`
needs neither them nor `pkg-config`, which is what keeps CI and the
manual-setup path green; the nix shell carries both. `hyperscan` is the
module and feature name, `vectorscan` the package that provides it —
the maintained fork of Intel's Hyperscan, same API.

## The variants

**Pure Rust.** `is_nice_pure_rust` and `is_nice_v2_pure_rust` are the
puzzle solved the ordinary way — iterator walks over chars and windows.
Kept as real functions, not deleted once the C versions existed, so
`benches/nice.rs` can put all three side by side and so `src/c_api.rs`
has something to export that drags no C library behind it.

**vectorscan (`src/hyperscan.rs`).** A SIMD-vectorized multi-pattern
regex engine built to scan gigabits/sec of network traffic against
thousands of signatures at once — the matching core inside
Suricata/Snort-style intrusion detection — pointed at sixteen-character
strings. Two `hs_compile_multi()` databases, compiled once into a
`OnceLock` and scanned forever after, which is the trade Hyperscan's own
docs assume. Hyperscan has no backreferences (its vectorized model
cannot represent "whatever matched earlier"), so "any repeated letter"
becomes the 26 literals `"aa"`..`"zz"`, and part 2's non-overlapping
pair becomes all 676 two-letter literals with per-id min/max end
offsets: two occurrences are non-overlapping iff `max - min >= 2`.
That is not a workaround so much as how real signature sets are built —
many literals over one clever pattern is the library's reason to exist.

**ICU regex (`src/icu.rs`, `src/icu_shim.c`).** The opposite engine: a
full Unicode-aware backtracking regex, with capture groups,
backreferences, and locale/collation machinery it never touches here.
Backreferences change the *shape* of the solution rather than just the
tool — every rule is one short pattern (`(.)\1`, `(..).*\1`, `(.).\1`),
and the non-overlap that needed manual offset bookkeeping on the
vectorscan side falls out of the engine's own backtracking. It is called
through a hand-written C shim rather than directly: ICU renames its C
symbols with a version suffix at link time (`uregex_open` is
`uregex_open_78` on this build) via macros in its own headers, so Rust
declarations would have to hardcode that suffix and break on every ICU
upgrade. Compiling a tiny real C file against the real header lets those
macros do their job; the shim exports two functions under names we chose.

**cbindgen C API (`src/c_api.rs`, Exercise 2).** The direction reverses:
instead of Rust calling into a C library, Rust exposes itself *as* one.
Two `extern "C"` functions, `aoc_2015_12_05_part1`/`part2`, built around
out-parameters and status codes rather than `Result` — a panic unwinding
across an `extern "C"` frame is undefined behavior, so nothing in this
module can panic; a null pointer or invalid UTF-8 from a C caller is
handled as data. Built on the pure-Rust functions specifically, not on
whichever variant `Solution::part1` currently runs: this exercise is
about the export direction, and the exported library needs no C
dependency of its own. `just days bindgen 2015-12-05` generates the
header (not committed — see `days/.gitignore`).

**Dart via dart:ffi (`dart/solve.dart`, Exercise 3).** Day 1 did this
step in Python; each day demonstrates a different track. The contrast is
the interesting part: `cffi` reads the generated header at runtime and
derives its declarations from it, one source of truth, while `dart:ffi`
has no equivalent for a plain script — `lookupFunction` needs the C
signature written out as Dart types, hand-transcribed from the header
and kept in sync by hand. (`package:ffigen` generates those for larger
APIs; for two functions it would be more machinery than what it
generates.) `just days dart-demo 2015-12-05` builds everything and runs
it (needs `just setup-dart` once).

## Running things

```sh
cd days/2015-12-05 && cargo run              # plain Rust — needs no C library at all
cargo run --features icu                     # same answers, through ICU's regex engine
cargo run --features hyperscan               # same answers, through vectorscan

cargo test -p aoc-2015-12-05                             # the 9 pure-Rust cases
cargo test -p aoc-2015-12-05 --features hyperscan,icu    # all 27: three backends, both parts

just days bench 2015-12-05                   # criterion: parse + both parts, see days/README.md
cargo bench -p aoc-2015-12-05 --bench nice --features hyperscan,icu   # all three, head to head

just days bindgen 2015-12-05                 # regenerate include/aoc_2015_12_05.h
just days dart-demo 2015-12-05               # build + generate header + run dart/solve.dart
```

With both features on, ICU is what `Solution::part1`/`part2` run — it is
the direct translation of the puzzle rules where vectorscan's is a
workaround for a missing engine feature, so it is the more defensible
"if you only run one" default. With neither, it is plain Rust. All three
are tested regardless of which one `cargo run` exercises.

## Benchmarks

`benches/nice.rs` races the three implementations of one predicate over
one sixteen-character line (release bench profile, aarch64), timing a
nice line and a naughty one separately — a naughty line lets plain Rust
exit early where both scanners still traverse the whole thing:

| | pure Rust | vectorscan | ICU |
|---|---|---|---|
| `is_nice` nice / naughty | 119 ns / 103 ns | **65 ns** / **91 ns** | 4.87 µs / 1.63 µs |
| `is_nice_v2` nice / naughty | **41 ns** / **66 ns** | 388 ns / 366 ns | 2.83 µs / 4.11 µs |

**Vectorscan wins part 1**, by about 1.8x on a nice line — the first
time in this repo an exotic C library has beaten the plain-Rust baseline
at the actual puzzle. It is the case shaped like what the library is
for: 31 patterns — 30 literals and one character class — matched in
one vectorized pass over 16 bytes, against Rust's three separate walks
of the same line.

It loses part 2 by 6-9x, structurally rather than for want of tuning.
No backreferences means 702 patterns, and the match callback needs a
676-entry min/max table per call — roughly 11 KB zeroed to answer a
question about a 16-byte string.

ICU trails both by 25-70x, and the shim is why: `uregex_openC` and
`uregex_close` on every call, two or three regexes per line, compiled
and thrown away each time. The engine is not slow; paying its setup per
line is. Hoisting the compiled `URegularExpression` out of the call
would be the fix, and is exactly the shape of the trade vectorscan's
`OnceLock` already makes.

`benches/day.rs` times the whole day instead — parse and both parts,
over the statement examples and a generated thousand-line input — and
follows the feature switch, so the same file times plain Rust by default
and ICU under `--features icu`. Pure Rust, same machine: parse 17.3 µs,
part 1 126 µs, part 2 87.7 µs for a thousand lines.

## Learnings

- **A missing engine feature changes the shape of the solution, not just
  its speed.** No backreferences turned "any repeated letter" into 26
  literals and "a non-overlapping repeated pair" into 676 literals plus
  per-id offset bookkeeping — while ICU expresses both in nine
  characters. The tool's capabilities decide how much of the problem you
  still have to model yourself.
- **Right shape, wrong scale is not the same as wrong tool.** Both
  libraries are aimed at this *kind* of problem at a thousand times the
  size, and the benchmark splits the difference honestly: vectorscan
  wins the part it is shaped for and loses the part it has to emulate.
  A library being oversized is a reason to measure, not a reason to
  assume it loses.
- **Where the setup cost lives is the whole story.** Vectorscan compiles
  its databases once and scans forever; the ICU shim opens and closes a
  regex per call. That single difference is most of the 25-70x between
  them, and it is a property of the binding, not of either engine.
- **Immutable and shareable are different questions.** Hyperscan's
  compiled database is safe to share across threads; the scratch space
  `hs_scan` writes into is not. Sharing one produced intermittently
  wrong answers under `cargo test`'s default parallelism — caught by the
  test suite, not by reasoning about it. One scratch per thread via
  `thread_local!` is what the library's own docs prescribe.
- **Version-renamed C symbols call for a shim, not a cleverer
  declaration.** ICU's headers rename every C entry point with a version
  suffix at link time. Hand-written Rust declarations can only hardcode
  today's suffix; a three-dozen-line C file compiled against the real
  header lets ICU's own macros resolve it and exports stable names.
- **An exotic dependency is only shippable behind a default-off
  feature.** An unconditional `pkg-config` probe in `build.rs` is green
  in the nix shell and red on every stock CI runner and every
  manual-setup laptop. Both libraries are cargo features, `build.rs`
  probes only what is enabled, and the C API this day exports is built
  on the pure-Rust functions so it inherits neither.
- **Tracks differ in whether the generated header is a source of
  truth.** `cffi` reads it and derives its declarations; `dart:ffi`
  needs the signature hand-transcribed as Dart types. Same header, same
  cdylib, and one of the two will silently disagree with the Rust side
  if a signature ever changes.
