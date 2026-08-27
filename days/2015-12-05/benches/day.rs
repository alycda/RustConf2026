//! Benchmarks for the Day 5 reference solution — a bonus, not a workshop step.
//!
//! `just days bench 2015-12-05` from the repo root, or `cargo bench` from this
//! crate. Nothing else times anything: `cargo test` skips bench targets
//! entirely, so what `just days verify` and CI give this file is the clippy
//! gate's `--all-targets`, which compiles and lints it but never runs it.
//!
//! Parsing and the parts are timed separately on purpose. Later stages of the
//! pipeline replace one piece at a time — the parse stays in Rust while the
//! predicate crosses into C, say — and a single end-to-end number cannot show
//! which side moved. What `part1`/`part2` measure here is whichever rule the
//! enabled features select (see `part1_rule` in `src/lib.rs`), so this same
//! file times plain Rust by default and ICU under `--features icu`. The
//! three-way comparison at one fixed input lives in `benches/nice.rs`.

use std::fmt::Write as _;
use std::hint::black_box;
use std::str::FromStr;

use aoc_2015_12_05::Day;
use aoc_ornaments::Solution;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// The statement's own examples, both parts' worth, as one input. Far too
/// small to say anything about performance; it is here because it is the one
/// input that is always present, and because it makes the shape of the input
/// visible in the file that benchmarks it.
const EXAMPLE: &str = "ugknbfddgicrmopn\naaa\njchzalrnumimnmhp\nhaegwjzuvuyypxyu\ndvszwmarrgswjxmb\nqjhvhtzxzqqjkmpb\nxxyxx\nuurcxstgmygtbstg\nieodomkazucvgmuy";

/// How many candidate strings the generated input gets, and how long each one
/// is. A real Day 5 input is a thousand lines of sixteen lowercase letters, so
/// this is the scale the numbers should be read at.
const GENERATED_LINES: usize = 1000;
const LINE_LEN: usize = 16;

/// xorshift64: deterministic across runs, machines, and this repo's absence of
/// a `rand` dependency, so two benchmark runs are comparable.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// A stand-in for a real puzzle input, generated rather than committed: inputs
/// cannot live in this repo (see `days/README.md`), and the examples are two
/// orders of magnitude too small to time.
///
/// Uniform lowercase letters from a fixed seed. That is close enough to the
/// real thing for timing and deliberately *not* close enough for counting:
/// uniform letters make a nice string rare, so the answer this input produces
/// is meaningless — only how long it takes to produce is not.
fn generated(lines: usize) -> String {
    let mut out = String::with_capacity(lines * (LINE_LEN + 1));
    let mut seed = 0x2015_1205;

    for _ in 0..lines {
        for _ in 0..LINE_LEN {
            let c = (b'a' + (xorshift(&mut seed) % 26) as u8) as char;
            write!(out, "{c}").expect("writing to a String cannot fail");
        }
        out.push('\n');
    }

    out
}

/// Your own input, if you have one. Absent on a fresh clone — `days/inputs/`
/// is gitignored — and that has to stay fine, so this is an `Option` and the
/// cases built from it simply don't appear.
fn real() -> Option<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/2015-12-05.txt");
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
    let mut group = c.benchmark_group("2015-12-05");

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

        // Both parts scan every line to the end — unlike day 1's part 2, there
        // is no early exit here, so these numbers track input length and not
        // input shape.
        group.bench_function(BenchmarkId::new("part1", &name), |b| {
            b.iter(|| black_box(day.part1().expect("part 1")))
        });

        group.bench_function(BenchmarkId::new("part2", &name), |b| {
            b.iter(|| black_box(day.part2().expect("part 2")))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_day);
criterion_main!(benches);
