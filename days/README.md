# Day Library

## Menu

| Day | Problem | Boundary shape | Status |
|-----|---------|----------------|--------|
| [2015-12-01](2015-12-01/) | Not Quite Lisp | char stream → `i32`; part 2 returns a position, not a total | Rust reference — the live-demo day |

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
