# Day 1: Trebuchet?!

Each line hides a two-digit calibration value — the first digit in the line and
the last, combined (`a1b2c3d4e5f` → `15`) — and the answer is their sum. Part 2
says the digits may also be spelled out, and that spelled-out digits may
overlap: `eightwo` is an `8` and a `2`. A scan and a sum.

This day goes in both directions. Two C libraries come *in* — a malware
scanner and a speech synthesiser, neither of which has any business here — and
Rust goes *out* as a C library of its own, to be called from Swift. The puzzle
suits the export half: two functions, one string in, one number out, an answer
that fits in a `uint32_t`, so the generated header is small enough to read in
one go.

The import half is where it gets interesting, because the two libraries fail
and succeed in opposite ways. One of them does not solve the puzzle at all,
deliberately, and that is the most useful thing on this page.

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline) | `sum_calibration_pure_rust`, `sum_calibration_with_words_pure_rust` in `src/lib.rs` |
| YARA | Rust → C | `src/yara.rs`, `yara_shim.c`, `sum_calibration_via_yara`/`..._with_words_via_yara` |
| espeak-ng | Rust → C | `src/espeak.rs` — **an open challenge, see below** |
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

**YARA (`src/yara.rs`).** The engine that sweeps gigabytes of suspect binaries
against thousands of rules, pointed at one 21-character line of
`heightseven4two5`, a thousand times. The scale is the whole joke, because the
semantics are exact: part two says spelled-out digits count and may overlap, so
`oneight` is a `1` and an `8` sharing an `e` — and for a multi-pattern scanner
that is not a special case, it is two occurrences. Nineteen strings stated
once, an Aho-Corasick automaton built for threat intelligence, every match
reported with its offset.

```text
oneight  ->  $w1 @ 0   $w8 @ 2
twone    ->  $w2 @ 0   $w1 @ 2
999      ->  $d9 @ 0   $d9 @ 1   $d9 @ 2
```

It is the only variant here with a C file of its own. Everything this calls —
`yr_initialize`, the compiler pair, `yr_rules_scan_mem`, the destructors — is
an ordinary exported symbol that `src/yara.rs` declares with opaque pointer
types. Reading the *matches* is not a symbol: `yr_rule_strings_foreach` and
`yr_string_matches_foreach` are preprocessor macros walking structs built from
`DECLARE_REFERENCE`, whose layout depends on how libyara was configured.
`yara_shim.c` does that walking in C and hands back flat integers.

**espeak-ng (`src/espeak.rs`) — the one that does not work.** See [the open
challenge](#the-open-challenge-espeak-ng) below. Nothing routes to it.

**cbindgen C API (`src/c_api.rs`, Exercise 2).** Two `extern "C"` functions,
`aoc_2023_12_01_part1`/`part2`, built around out-parameters and status codes
rather than `Result`, because a panic unwinding across an `extern "C"` frame is
undefined behavior. `0` on success, `-1` for a null pointer or invalid UTF-8,
`-2` for a total too large for a `uint32_t`. `just days bindgen 2023-12-01`
generates the header (not committed — see `.gitignore`); cbindgen is
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
cd days/2023-12-01 && cargo run                          # pure Rust
cargo run  -p aoc-2023-12-01 --features yara             # through the malware scanner
cargo test -p aoc-2023-12-01 --features yara,espeak      # 22 tests, every backend

# The scoreboard for the open challenge, against your own input:
cargo run -p aoc-2023-12-01 --features espeak --example espeak_scoreboard

just days bench 2023-12-01                               # parse, both parts, digit density
cargo bench -p aoc-2023-12-01 --bench calibration --features yara,espeak   # three-way

just days bindgen 2023-12-01                             # regenerate the header
just days swift-demo 2023-12-01                          # build + header + run solve.swift
```

There is no `cargo run --features espeak`. That is not an oversight — see
below.

## The open challenge: espeak-ng

This is the one variant in the repo that does not solve its puzzle, and it is
here because of *how* it fails.

Part two's whole difficulty is that `1` and `one` mean the same thing while
being nothing alike as text. A text-to-speech front end has to have solved that
before it can say either out loud — so the equivalence is already in the
library, not as a table someone wrote for this puzzle but as the ordinary
business of reading English, with pronunciation dictionaries for a hundred
languages behind it:

```text
espeak_TextToPhonemes("1") -> wɒn      ("one")   -> wɒn
espeak_TextToPhonemes("2") -> tuː      ("two")   -> tuː
espeak_TextToPhonemes("8") -> eɪt      ("eight") -> eɪt
```

Identical, once the stress marks are stripped. No nine-word table, no overlap
rule. That is a library whose vocabulary already contains the sentence the
puzzle needed — 2021-12-02's DuckDB window function, one better.

And it still cannot finish. On a real 1000-line input it agrees with plain Rust
on **728 lines out of 1000 (72.8%)**, for three reasons, in increasing order of
how stuck they are:

- **Multi-digit runs are read as numbers.** `23seven` at position 0 is "twenty
  three", `twɛnti…`, which does not start with `tuː`. Scanning suffixes one
  character at a time — `Speaker` does exactly what `Day::digit_at` does, for
  exactly this reason — recovers the last digit of a run and never the first.
- **Coarticulation blurs the boundaries.** `twobfr` does not begin with `tuː`;
  speaking re-syllabifies across what follows. It runs both ways:
  `djnrmpxjbsbpgzvtjkhq6pkkfshx` holds one `6` and is *heard* as an `8`, out of
  letters that happen to sound like one.
- **Part one is unavailable to the current design.** It counts literal digits
  and ignores the word `one` — and in the whole-suffix window this variant asks
  in, those are the same three sounds. There is no
  `sum_calibration_via_espeak`. (This was written as "unavailable, full stop".
  It isn't — see the two-window lead below, which is the correction.)

**If you can do better, the scoreboard is real.** Each failure mode is pinned
by a test named `unsolved_*` in `src/espeak.rs`; one of those starting to
*fail* is the good outcome. `espeak::agreement_with_pure_rust` computes the
percentage rather than quoting it, and `examples/espeak_scoreboard.rs` runs it
against your own input, so an improvement is measured instead of claimed.

### The most promising lead: two window sizes

Ask espeak **twice** per position — once with a one-character window, once with
the full suffix — and take the one-character answer if it is a digit, falling
back to the suffix otherwise.

It works because a lone character is pronounced as itself or as its *letter
name*, and no letter name collides with a digit name:

```text
"1" -> wˈɒn      "o" -> ˈəʊ      "n" -> ˈɛn
                 "e" -> ˈiː      "t" -> tˈiː      "w" -> dˈʌbəljˌuː
```

A one-character match therefore identifies a *literal* digit — which is part
one, and is why the bullet above had to be walked back. It also dissolves the
multi-digit run problem with no new vocabulary at all: `16` is read
position-by-position as `1` then `6`, while the *word* `sixteen` falls through
to the suffix window and still matches `six`.

Cost: two calls per position on a variant already spending ~600 µs per line.
It does nothing for coarticulation, which would be the last failure standing.

### The wall: re-parsing spoken numbers back into digits

The obvious repair for `23seven` is to teach the table the number words, so
`twˈɛnti` maps back to a leading `2`. It needs less machinery than it sounds
like — the scan visits every position anyway, so each position only needs the
*leading* digit of the number starting there, not a decomposition — and about
eighteen more references (`ten`..`nineteen`, `twenty`..`ninety`) would cover
it. Two objections that look fatal aren't: input leading zeros are spoken
(`07` → `zˈiəɹəʊ sˈɛvən`, `007` keeps both), and the zeros `100` → `wˈɒnhˈʌndɹɪd`
swallows are picked up by the later positions regardless.

What kills it is a collision no ordering survives:

```text
"16"      -> sˈɪkstiːn        "19"   -> nˈaɪntiːn
"sixteen" -> sˈɪkstiːn        "nine" -> nˈaɪn
"six"     -> sˈɪks
```

`16` and `sixteen` are the **same sound** and want different answers — the
digits `16` are a `1` and a `6`; the word `sixteen` is only a `6`.
Longest-match-first reads both as `1`, shortest-first reads both as `6`, and
the puzzle's own example line `7pqrstsixteen` is on the losing side of
longest-match. Phonemisation destroyed the information that separates them,
which is exactly why the two-window design — which never asks the question —
is the better lead.

### Ruled out, with measurements

- **SSML is not available on this API.** `espeak_TextToPhonemes` reads the tags
  aloud as words: `<say-as interpret-as="characters">23</say-as>` comes back as
  `sˈeɪaz ɪntˈɜːpɹɪtaz ˈiːkwəlz kˈaɹɪktəz twˈɛnti θɹˈiː slˈaʃ sˈeɪaz`. Still
  true with `espeakSSML` (0x10) OR'd into `textmode` — that flag belongs to
  `espeak_Synth`, and this function ignores it.
- **Injecting a leading `0` to force digit mode does not work.** A leading zero
  does trigger digit-by-digit reading, but only once the run reaches four
  digits: `0123` → "zero one two three", while `016` → "zero sixteen" and
  `023seven` → "zero, twenty three, seven". Puzzle runs are one to three
  digits, so it fires exactly where it is not needed.
- **Length alone never triggers digit mode.** Unlike engines that give up past
  four or five digits, espeak-ng scales all the way — `12345678901` → "twelve
  billion three hundred and forty five million…". There is no threshold to
  reach.
- **Phone-number shapes are not recognised.** `555-1234` → "five hundred and
  fifty five, dash, one thousand two hundred and thirty four".

Still untried: `espeakPHONEMES_IPA` and comparing IPA rather than espeak's own
notation; `espeak_SetPhonemeTrace` with a per-phoneme callback, to get
*positions* back instead of prefix-matching a string, which would sidestep
prefix collisions entirely; or a voice whose dictionary treats digits
differently. For text-shaping, `_` is the only separator espeak splits on
silently (`2_3` → "two three"); space and a non-grouping comma also work, while
`-`, `.`, `:` and `/` each insert a spoken word.

One warning for anyone starting: **all seven of the puzzle's statement example
lines agree.** The published example is short lines with well-separated digits
and one short run — exactly the shape espeak handles. A fix validated only
against it will look finished and move the real number not at all.

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


### The three-way race

`benches/calibration.rs` puts the backends against each other on one fixed
synthetic input, part two only — part one does not exist on the espeak side, so
a part-one group would be a two-way race wearing a three-way label. 100 lines
× 40 characters rather than 1000, because espeak costs ~600 µs *per line* and a
realistic input would mean a criterion run measured in minutes for a ratio that
is unambiguous at a tenth of the size:

| | time | vs pure Rust |
|---|---|---|
| pure Rust | ~11.1 µs | — |
| YARA | ~349.6 µs | 31x |
| espeak-ng | ~61.2 ms | 5,500x |

**The two libraries are slow in opposite shapes**, which is the reason to have
run both rather than picking one:

| | setup | work |
|---|---|---|
| YARA | ~295.2 µs (compile 19 strings into an automaton) | ~142.1 µs (scan 100 lines) |
| espeak-ng | ~130.4 µs (initialise + 19 references) | ~61.5 ms (phonemise 100 lines) |

At this size **YARA spends twice as long compiling its rules as scanning with
them** — 68% of the bill is setup. espeak's setup is 0.2% of its bill;
essentially all of its time is the work, because it phonemises a suffix at
every one of forty character positions on every line.

That makes YARA the only variant in this repo whose verdict depends on input
size. Rule compilation is fixed at ~295 µs and scanning runs at ~1.42 µs/line
here — which the throwaway C program written before any Rust independently put
at ~1.35 µs/line over a thousand lines — so the crossover is around **200
lines**: below it this variant mostly compiles, above it mostly scans. A real
puzzle input sits at 1000, comfortably past it. A bench run against the
statement example's seven lines would have measured almost nothing but setup
and reported it as the cost of scanning.

Same lesson family as 2021-12-02's chipmunk-vs-DuckDB — where the cost lives is
library-specific and the average hides it — with the shapes swapped: there the
database was 95% setup and the physics engine uniform per command; here the
scanner is the front-loaded one and the synthesiser uniform per character.

The espeak row is a cost measurement only. That backend does not agree with the
other two and is not supposed to; correctness for it is
`espeak::agreement_with_pure_rust`, not this table.
## Learnings

- **The parts of a C API that are macros are the parts FFI cannot reach.**
  Every YARA function this day calls is an ordinary exported symbol, bound in
  Rust with opaque pointer types and no knowledge of any struct. Reading the
  *matches* is not a function at all: `yr_rule_strings_foreach` and
  `yr_string_matches_foreach` are preprocessor macros that walk `YR_RULE`,
  `YR_STRING` and `YR_MATCH` by field, and those structs are assembled from
  `DECLARE_REFERENCE`, whose layout depends on how libyara was built.
  Transcribing three internal structs to chase them would compile against one
  build of YARA and silently read garbage from the next. Six lines of C in
  `yara_shim.c` do the walking where the header defines it and hand back flat
  integers. The general rule is worth carrying: enumerate a library's API by
  what is *exported*, not by what is *documented*, and budget a shim for the
  difference.
- **A library can be exactly right about the hard part and still not finish.**
  Part two's difficulty is that `1` and `one` mean the same thing and look
  nothing alike. A speech synthesiser has already solved that — it must, before
  it can say either aloud — so espeak-ng hands back identical phonemes for
  both, out of pronunciation dictionaries for a hundred languages, with no
  table and no overlap rule. It is the sharpest "almost useful" this repo has
  found. It also agrees with plain Rust on only 73% of a real input, because
  the same engine reads `23` as "twenty three" and re-syllabifies `twobfr`
  until it stops sounding like `two`. Fitting the *hardest* requirement is not
  the same as fitting the requirements.
- **"Impossible" usually means "impossible given the question I asked".** Part
  one counts literal digits and ignores the word `one`; after phonemisation
  those are the same three sounds. This README said the property that makes
  espeak brilliant at part two therefore made part one *unavailable*, full
  stop — and that was wrong, in a way worth leaving on the page rather than
  quietly editing out. Asking a *different* question separates them
  immediately: in a one-character window `o` is `ˈəʊ` and `1` is `wˈɒn`, so a
  match there is a literal digit. The phonemes never lost the distinction; the
  surrounding context did. What made the claim feel safe was that it was true
  of every experiment run so far, which is exactly the shape a wrong
  impossibility proof has.
- **A variant is allowed to be a demonstration instead of an answer.** There
  is still no `sum_calibration_via_espeak` and `Solution` still never routes
  to that backend, because a confidently wrong number is worse than no number.
  Keeping `..._pure_rust` alive as a real function is what makes that
  affordable — the day has a correct answer regardless of which experiments
  succeed, so an experiment is free to fail in public and be written up.
- **Ask whether a library's state is per-handle or global before designing
  around it, not after the tests flake.** YARA hands out a `YR_RULES` and the
  variant makes a fresh one per solve, so nothing is shared and no lock is
  needed. espeak-ng keeps its translator, its voice and its output buffer in
  process globals, and `espeak_TextToPhonemes` returns a pointer into a
  *static* buffer the next call overwrites — two threads in it at once is a
  data race and a use-after-overwrite simultaneously. One process-wide `Mutex`,
  and the phonemes copied out before it is released. Two libraries in one day,
  opposite answers, and `cargo test`'s parallelism would have found the
  difference the expensive way.
- **The statement example is not a test of a partial solution.** All seven of
  the published example lines agree with espeak — short lines, well-separated
  digits, one short run: exactly the shape it handles. The real input is
  30-to-45-character letter soup and agrees on 73%. A fix validated only
  against the example looks finished and moves the real number not at all,
  which is why the scoreboard is a computed function and an example binary
  rather than a figure in a comment.
- **Two libraries, opposite cost shapes — and one of them has no single
  verdict.** YARA at 100 lines spends twice as long compiling its nineteen
  strings as scanning with them; espeak spends 0.2% of its time on setup and
  the rest on work. Because YARA's setup is fixed and its scan is per line, the
  crossover sits near 200 lines: below that the variant mostly compiles, above
  it mostly scans. It is the only variant here whose answer to "how slow is
  it?" is "how much input?" — and benching it at the statement example's seven
  lines would have measured setup and called it scanning.
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
