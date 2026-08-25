# Day Library

## Menu

| Day | Problem | Boundary shape | Status |
|-----|---------|----------------|--------|
| [2015-12-01](2015-12-01/) | Not Quite Lisp | char stream → `i32`; part 2 returns a position, not a total | Rust reference — the live-demo day |
| [2015-12-05](2015-12-05/) | Doesn't He Have Intern-Elves For This? | lines → `usize` count; one predicate per line, the ruleset swapped by function pointer | Rust reference |

## Rules

- **Never commit real puzzle inputs or full puzzle text**
  ([AoC's request](https://adventofcode.com/about#faq_copying)). Only the small example
  inputs from the problem statement live in tests; the doc header of each day paraphrases
  the puzzle rather than quoting it. `.gitignore` here ignores `**/inputs/*`, so drop your
  own input at `days/inputs/<YYYY-MM-DD>.txt` — every `main` reads it from there at
  runtime rather than with `include_str!`, so a day still builds and tests without one.
- **Days are named `YYYY-MM-DD`**, the full date, because `2015-01` reads as January to
  everyone who hasn't been told otherwise and sorts wrong the moment a second event year
  shows up.
- **Days are not std-only.** `_template` bakes in `aoc-ornaments` (which owns the
  `Solution` trait and `Part`), `miette` for errors, and `rstest` for table-driven tests;
  individual days reach for `derive_more`, `itertools`, `nom`, or `nom_locate` as the
  puzzle warrants. That is a deliberate reversal of an earlier dependency-light rule: the
  C-glue stage has to strip a real crate's trait and error type at the boundary, which is
  the thing worth watching happen. It is the boundary layer's job to be small, not the
  day's.

## Benchmarks (bonus)

`2015-12-05` carries criterion benchmarks; nothing in the workshop needs one, and not
every day has them.

```sh
just days bench 2015-12-05
just days bench 2015-12-05 --save-baseline pure-rust   # criterion flags pass through

# a day's second bench, where one exists, races the C variants against plain Rust and
# so declares required-features — `just days bench` skips it, run it directly:
cargo bench -p aoc-2015-12-05 --bench nice --features hyperscan,icu
```

The `day` bench times the parse and the parts separately, over the statement examples, a
generated input at roughly the scale of a real one, and — only if you have dropped one at
`days/inputs/<day>.txt` — your own. The generated input exists because puzzle inputs
cannot be committed here and the examples are far too small to time; it is built from a
fixed seed, so two runs are comparable.

The point of them is comparison, not absolute numbers. What a `nice` bench compares is
three implementations of one predicate — plain Rust, a vectorized C matcher, a full
Unicode regex engine — over the same lines, which is the only way "the boundary costs
something" becomes a figure instead of a claim. Between days, `--save-baseline` before a
part goes out through C and `--baseline` after it does the same job.

`cargo test` skips bench targets entirely, so `just days verify` only proves a bench
still compiles (via the clippy gate, which lints `--all-targets`). It never runs one.
