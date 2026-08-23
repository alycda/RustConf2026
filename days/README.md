# Day Library

## Menu

| Day | Problem | Boundary shape | Status |
|-----|---------|----------------|--------|

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
