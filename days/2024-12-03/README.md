# Day 3 (2024): Mull It Over

Corrupted memory full of `mul(X,Y)` instructions, some switched off by
`don't()` and back on by `do()`. Part 1 sums every well-formed product;
part 2 respects the toggles. The second golden day — solved for real in
2024, shown in the original "(ab)Using Advent of Code as an FFI
Playground" talk — and unlike day 1 it carries no C: what it demonstrates
is that "the parse" is a design decision before it is a boundary decision.

Two parsers solve the same day (each built and verified independently):

| Variant | Approach | Files |
|---|---|---|
| nom + nom_locate | combinators over a typestate (`Day3<Part1>` / `Day3<Part2>`) | `src/lib.rs`, ported from the real 2024 solution (`alycda/Advent-of-Code`, `refactor/alycda/2024`) |
| byte cursor | hand-rolled cursor, no dependencies | `src/cursor.rs`, ported from the talk's `2024-12-03` branch (`alycda/aoc-ffi`) |

## The parsers

**nom (`src/lib.rs`).** The solution as actually written in 2024:
`parse_mul` as a combinator stack, `parse_all_mul` skip-scanning the
corruption one byte at a time, and part 2 segmenting the stream around
`don't()`/`do()` with `take_until`. Each part is a marker type on
`Day3<P>`, so the toggle state lives in the parse, not the sum. The
`fix(days)` commit on this line is part of the golden history: the
original error handling panicked on corruption shapes like `mul(x,4)` —
`ErrorKind::Digit` fell through to a catch-all `panic!` — and the fix
reclassifies every recoverable error as corruption-to-skip, with those
exact inputs pinned as tests.

**byte cursor (`src/cursor.rs`).** The talk's from-scratch counterpart:
try to read `mul(X,Y)` at the cursor, step one byte on failure — the
whole parser on one screen, no dependencies. It never had the panic
problem the nom path needed fixing for (a failed parse is just a position
to move past), and those corruption inputs are pinned here too. One
honest divergence, documented at `parse_num`: the cursor enforces the
statement's 3-digit operand cap; nom's `digit1` accepts any digit run.
The agreement tests and the benchmark generator both stay inside the
shapes where the two parsers must agree.

## Running things

```sh
cd days/2024-12-03 && cargo run           # nom pipeline, both parts
cargo test -p aoc-2024-12-03              # both parsers + their agreement

just days bench 2024-12-03                        # parses + parts, see days/README.md
cargo bench -p aoc-2024-12-03 --bench parser      # nom vs cursor, head to head
```

## Benchmarks

`benches/parser.rs` races both parsers whole-job on a 20k-character
generated corruption stream, agreement asserted on those exact bytes
before any timing, both contenders timed integer-out so neither pays a
formatting cost the other doesn't (bench profile, aarch64):

| | nom | byte cursor | ratio |
|---|---|---|---|
| part 1 | ~103.4 µs | ~34.0 µs | ~3.0× |
| part 2 | ~52.3 µs | ~30.6 µs | ~1.7× |

`benches/day.rs` shows the same day from the pipeline side: the parts sum
in nanoseconds what the parse collected — this day is all parse, which is
what makes it the right day to race two parsers on. Its most interesting
row is nom's part-2 parse running *twice as fast* as its part-1 parse:
the `don't()` segmentation `take_until`-skips disabled spans wholesale,
so half the stream never reaches `parse_all_mul` at all.

## Learnings

- **A "corrupted" input is a parser's whole test plan.** The golden
  solution shipped a panic on corruption it hadn't met (`mul(x,4)`,
  truncated `mul(2,` at a fragment end) because its error handling
  enumerated the errors it had seen instead of classifying everything
  recoverable as corruption. The cursor was immune by construction —
  when "skip one byte" is the only failure mode, there is nothing to
  enumerate.
- **Two parsers are worth more than either alone.** The agreement tests
  (and the pre-timing assertion in the bench) are the strongest
  correctness statement this day has: same bytes, two independent
  readings, same numbers. That property had to be *designed for* — the
  benchmark generator keeps digits out of its junk alphabet precisely
  because operands past the statement's 3-digit cap are the one shape
  where the parsers legitimately read differently.
- **Skipping beats reading.** The cursor beats nom ~3× when everything is
  scanned, but only ~1.7× on part 2 — nom's `take_until` skips disabled
  spans wholesale while the cursor walks them byte by byte. Where a
  combinator library gets to skip, it claws back most of its overhead;
  where it must read, the hand-rolled loop wins. Neither number is a
  verdict on nom — the nom side also allocates and carries source spans
  the puzzle never uses.
- **The parts are free; the parse is the day.** Both parts sum in
  nanoseconds what their parse collected. Any future boundary experiment
  on this day (the cursor's `&[u8]`-in/`usize`-out shape is practically
  a C signature already) would be measuring parse throughput, and the
  numbers above are its baseline.
