# Exercise 1: Pure Rust (20 min)

## Goal

A working, tested Rust solution to **your chosen day** — the thing we'll
wrap in a C boundary next.

## Tasks

1. Pick a day from [`../../days/README.md`](../../days/README.md) if you
   haven't already.
2. Paste your day's **example input** (from the puzzle statement, not your
   real input) into `EXAMPLE` in `src/lib.rs`, and the expected answers into
   the tests. Remove the `#[ignore]` lines.
3. Implement `part1` (and `part2` if time allows).
4. `cargo test` until green.

Done early? Compare your solution with the reference in
`../../days/<your-day>/` — differences are discussion material for the
debrief. Or start reading Exercise 2.

## Key Concepts

- Keep the solve functions **pure** (`&str` in, value out) — no file I/O in
  the library. This is what makes the FFI wrap in Ex 2 clean: I/O stays on
  the caller's side of the boundary.
- Idiomatic here beats clever: iterators and `match` translate into a
  *smaller* C surface than exposed cleverness does.
