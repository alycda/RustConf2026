//! Benchmarks for the Day 1 reference solution — a bonus, not a workshop step.
//!
//! `just days bench 2024-12-01` from the repo root, or `cargo bench` from this
//! crate. Nothing else times anything: `cargo test` skips bench targets
//! entirely, so what `just days verify` and CI give this file is the clippy
//! gate's `--all-targets`, which compiles and lints it but never runs it.
//!
//! Parsing and the parts are timed separately on purpose. This day's parse is
//! where the sort lives — `FromStr` rank-sorts both columns — so under
//! `--features qsort` or `--features cpp` the *parse* number is the one that
//! crosses the boundary, while part 1 stays a pure zip-and-sum either way.
//! `benches/sort.rs` races the sort backends against each other directly;
//! `benches/lookup.rs` does the same for part 2's counting structures.

use std::fmt::Write as _;
use std::hint::black_box;
use std::str::FromStr;

use aoc_2024_12_01::Day;
use aoc_ornaments::Solution;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// The statement example. Far too small to say anything about performance; it
/// is here because it is the one input that is always present, and because it
/// makes the shape of the input visible in the file that benchmarks it.
const EXAMPLE: &str = "3   4
4   3
2   5
1   3
3   9
3   3";

/// How many pairs the generated input gets. A real Day 1 input is a thousand
/// lines of two five-digit IDs, so this is the scale the numbers should be
/// read at.
const GENERATED_LINES: usize = 1000;

/// xorshift64: deterministic across runs, machines, and this repo's absence of
/// a `rand` dependency, so two benchmark runs are comparable.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// A stand-in for a real puzzle input, generated rather than committed: inputs
/// cannot live in this repo (see `days/README.md`), and the example is three
/// orders of magnitude too small to time.
///
/// Uniform five-digit pairs are right for timing and wrong for counting: with
/// 90 000 possible IDs and 1 000 draws per column, most left-hand IDs never
/// appear on the right, so the part 2 score is near zero. That distorts no
/// timing here — the naive scan reads the whole right column whether or not
/// it matches — but do not debug the score of an input that was never meant
/// to have one.
fn generated(lines: usize) -> String {
    let mut out = String::with_capacity(lines * 12);
    let mut seed = 0x2024_1201;

    for _ in 0..lines {
        let left = 10_000 + xorshift(&mut seed) % 90_000;
        let right = 10_000 + xorshift(&mut seed) % 90_000;
        writeln!(out, "{left}   {right}").expect("writing to a String cannot fail");
    }

    out
}

/// Your own input, if you have one. Absent on a fresh clone — inputs are
/// gitignored — and that has to stay fine, so this is an `Option` and the
/// cases built from it simply don't appear.
fn real() -> Option<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/2024-12-01.txt");
    std::fs::read_to_string(path).ok()
}

fn inputs() -> Vec<(String, String)> {
    let mut inputs = vec![
        ("example".to_string(), EXAMPLE.to_string()),
        (
            format!("generated/{GENERATED_LINES}"),
            generated(GENERATED_LINES),
        ),
    ];

    if let Some(input) = real() {
        inputs.push(("real".to_string(), input));
    }

    inputs
}

fn bench_day(c: &mut Criterion) {
    let mut group = c.benchmark_group("2024-12-01");

    for (name, input) in inputs() {
        // This is the number that moves under --features qsort/cpp: the
        // rank-sort happens here, through whichever backend FromStr chose.
        group.bench_with_input(
            BenchmarkId::new("parse", &name),
            input.as_str(),
            |b, input| b.iter(|| Day::from_str(black_box(input)).expect("the input parses")),
        );

        // Parsed once, outside the timing loop: neither part mutates the day,
        // so one value serves every iteration and what is timed is the solving
        // rather than the parsing a second time. The day goes through
        // `black_box` *inside* the loop — boxing only the result would let
        // the optimizer notice the loop-invariant input and hoist the whole
        // computation out of the iterations.
        let mut day = Day::from_str(&input).expect("the input parses");

        group.bench_function(BenchmarkId::new("part1", &name), |b| {
            b.iter(|| black_box(black_box(&mut day).part1().expect("part 1")))
        });

        // The naive scan is O(left × right) and reads every element either
        // way — no early exit to skew this one — while `--features uthash`
        // swaps in the C hash table. See benches/lookup.rs for the counting
        // structures head to head.
        group.bench_function(BenchmarkId::new("part2", &name), |b| {
            b.iter(|| black_box(black_box(&mut day).part2().expect("part 2")))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_day);
criterion_main!(benches);
