//! Benchmarks for the Day 1 reference solution — a bonus, not a workshop step.
//!
//! `just days bench 2015-12-01` from the repo root, or `cargo bench` from this
//! crate. Nothing else times anything: `cargo test` skips bench targets
//! entirely, so what `just days verify` and CI give this file is the clippy
//! gate's `--all-targets`, which compiles and lints it but never runs it.
//!
//! Parsing and the parts are timed separately on purpose. Later stages of the
//! pipeline replace one piece at a time — the parse stays in Rust while the
//! sum crosses into C, say — and a single end-to-end number cannot show which
//! side moved. This day has no second implementation to race; what it has
//! instead is a part 2 that stops at the first basement entry, which makes its
//! timing a function of *where* that happens rather than input length — the
//! kind of shape you have to know before a number that crosses the C boundary
//! can be read at all.

use std::fmt::Write as _;
use std::hint::black_box;
use std::str::FromStr;

use aoc_2015_12_01::Day;
use aoc_ornaments::Solution;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// A statement example that satisfies *both* parts: part 1 ends on floor -1,
/// and part 2 finds a basement entry (position 5) — `part2` returns an error
/// on any input that never goes below ground, so the pure part-1 examples
/// like `"(())"` cannot be benchmarked here. Far too small to say anything
/// about performance; it is here because it is the one input that is always
/// present, and because it makes the shape of the input visible in the file
/// that benchmarks it.
const EXAMPLE: &str = "()())";

/// How many instructions the generated input gets. A real Day 1 input is a
/// single line of about seven thousand parens, so this is the scale the
/// numbers should be read at.
const GENERATED_CHARS: usize = 7000;

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
/// An even coin per paren: a symmetric walk this long dips below ground early
/// with near certainty, and the seed is fixed, so part 2's `expect` below is
/// deterministic — if it survives one run it survives every run.
fn generated(chars: usize) -> String {
    let mut out = String::with_capacity(chars);
    let mut seed = 0x2015_1201;

    for _ in 0..chars {
        let c = if xorshift(&mut seed) % 2 == 0 {
            '('
        } else {
            ')'
        };
        write!(out, "{c}").expect("writing to a String cannot fail");
    }

    out
}

/// Your own input, if you have one. Absent on a fresh clone — `inputs/`
/// is gitignored — and that has to stay fine, so this is an `Option` and the
/// cases built from it simply don't appear.
fn real() -> Option<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../inputs/2015-12-01.txt");
    std::fs::read_to_string(path).ok()
}

fn inputs() -> Vec<(String, String)> {
    let mut inputs = vec![
        ("example".to_string(), EXAMPLE.to_string()),
        (
            format!("generated/{GENERATED_CHARS}"),
            generated(GENERATED_CHARS),
        ),
    ];

    if let Some(input) = real() {
        inputs.push(("real".to_string(), input));
    }

    inputs
}

fn bench_day(c: &mut Criterion) {
    let mut group = c.benchmark_group("2015-12-01");

    for (name, input) in inputs() {
        group.bench_with_input(
            BenchmarkId::new("parse", &name),
            input.as_str(),
            |b, input| b.iter(|| Day::from_str(black_box(input)).expect("the input parses")),
        );

        // Parsed once, outside the timing loop: neither part mutates the day,
        // so one value serves every iteration and what is timed is the solving
        // rather than the parsing a second time.
        let mut day = Day::from_str(&input).expect("the input parses");

        group.bench_function(BenchmarkId::new("part1", &name), |b| {
            b.iter(|| black_box(day.part1().expect("part 1")))
        });

        // Stops at the first basement entry: compare this number across input
        // *shapes*, not sizes. An input whose basement comes late times the
        // whole scan; one that opens with `)` times almost nothing.
        group.bench_function(BenchmarkId::new("part2", &name), |b| {
            b.iter(|| black_box(day.part2().expect("part 2 — input must reach the basement")))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_day);
criterion_main!(benches);
