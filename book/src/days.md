# The Day Library

Your toolchain works and your repo is clean. Now you need something to put
across an FFI boundary.

That something is a *day*: one Advent of Code puzzle, solved in pure Rust, one
crate per day under `days/`. The workshop's subject is the pipeline — Rust → C
glue → bindings — not any particular puzzle, so the days are plug-in content.
You pick one and carry it through the stages.

## Pick from the menu, not from the directory listing

The menu lives in
[`days/README.md`](https://github.com/alycda/RustConf2026/blob/main/days/README.md),
next to the crates it indexes. Read it there rather than browsing the tree: the
column that matters is not the puzzle, it's the **boundary shape** — what has
to cross the FFI boundary once the Rust part is done.

A scalar in and a scalar out is one exercise. A list of strings is a different
one. A fixed-size top-three array is different again, and the interesting
question arrives before you write any C: does it come back as a returned struct,
or do you hand the callee a pointer to fill in?

Two days on the menu are marked **golden**. A golden day has the whole pipeline
worked through, in every language track, so when you get stuck there is always a
finished pattern to mirror rather than a blank file. There are two so that no
track depends on a single example.

## Running one

The `days` recipes are a `just` module, so they run from the repo root:

```sh
just days test 2023-12-01    # one day, against the puzzle's published examples
just days run  2023-12-01    # one day, against your own input
just days verify             # every day: tests, formatting, lints
```

`just days verify` is the one to run before you push. The per-day recipes take
one day; the day crates share one lockfile, so a change to a shared dependency
can break a day you never touched.

## Your own puzzle input

Puzzle inputs are yours, not the repo's: they are tied to your AoC account, and
Advent of Code
[asks that they not be redistributed](https://adventofcode.com/about#faq_copying).
So no real input and no puzzle text is ever committed here — only the small
examples printed in the problem statements, which is what the tests assert
against.

Download yours and drop it at `days/inputs/<YYYY-MM-DD>.txt`. `.gitignore`
already covers that directory, and each day reads its file at run time from a
path anchored to the crate, so `just days run` works and a day with no input
still builds and tests.

## What a day looks like inside

Three files, and only three things to write:

1. `impl FromStr for Day` — parse the input into whatever shape the puzzle
   actually wants. The scaffold starts line-based, which is a fine default and
   rarely the final answer.
2. `part1` and `part2` on the `Solution` trait.
3. The tests — paste the example from the problem statement and delete the
   `#[ignore]`.

`just days new 2019-12-04` scaffolds those from `days/_template`. Nothing needs
registering afterwards: membership is a glob, so cargo, rust-analyzer and CI all
pick the new day up by its existing.

And feel free to ignore my opinionated setup and use your own! The scaffold,
the recipes, the editor config, even the crate layout are how *I* like to solve
these — none of it is what the workshop is teaching. If you already have a way
you enjoy writing Rust, bring it. All the FFI steps need from you is a crate
that builds and a function worth calling from another language.

## Read them critically

These are solved days, written by hand over several years, and they do not all
look alike. Some parse with `split_once` and some with `nom`; most return errors
and a couple reach for `.expect()`; two of them are older solutions carried in
from another repo and still carry the seams — a `todo!()` here, an
`unimplemented!()` there, a half-finished second implementation kept because it
is interesting.

That unevenness is on purpose. The interesting question at an FFI boundary is
rarely "is this idiomatic Rust" — it is "what happens to *this particular
shape* when a caller in another language holds it, and what does a panic look
like from over there?" A tidy day and a scruffy day answer that differently.
Notice which is which as you read.
