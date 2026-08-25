//! Benchmarks for the Day 2 reference solution — a bonus, not a workshop step.
//!
//! `just days bench 2021-12-02` from the repo root, or `cargo bench` from this
//! crate. Nothing else times anything: `cargo test` skips bench targets
//! entirely, so what `just days verify` and CI give this file is the clippy
//! gate's `--all-targets`, which compiles and lints it but never runs it.
//!
//! Parsing and the parts are timed separately on purpose. Later stages of the
//! pipeline replace one piece at a time — the parse stays in Rust while the
//! dead reckoning crosses into C — and a single end-to-end number cannot show
//! which side moved. This file times the *default* build, which is plain
//! Rust; `benches/dive.rs` is where the two backends meet.
//!
//! Neither part of this day short-circuits: both consume every command, so
//! unlike a day that stops early, these timings are a straight function of
//! input length. That is what makes the generated case below meaningful.

use std::fmt::Write as _;
use std::hint::black_box;
use std::str::FromStr;

use aoc_2021_12_02::Day;
use aoc_ornaments::Solution;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// The statement example, which satisfies both parts (150 and 900). Far too
/// small to say anything about performance; it is here because it is the one
/// input that is always present, and because it makes the shape of the input
/// visible in the file that benchmarks it.
const EXAMPLE: &str = "forward 5
down 5
forward 8
up 3
down 8
forward 2";

/// How many commands the generated input gets. A real 2021 day 2 input is
/// almost exactly a thousand lines, so this is the scale the numbers should be
/// read at.
const GENERATED_COMMANDS: usize = 1000;

/// xorshift64: deterministic across runs, machines, and this repo's absence of
/// a `rand` dependency, so two benchmark runs are comparable.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// A stand-in for a real puzzle input, generated rather than committed: inputs
/// cannot live in this repo (see `days/README.md`), and the example is two
/// orders of magnitude too small to time.
///
/// **Good for timing, wrong for answers.** The three commands are drawn
/// uniformly and the amounts are 1..=9, which is the right *shape* and the
/// wrong *distribution*: a real input's `down`/`up` do not cancel, so its aim
/// climbs steadily and its part-2 depth is far larger. Whatever number this
/// input produces is not a puzzle answer and is not meant to be one — it is
/// here so both parts do a realistic amount of work.
///
/// The near-cancelling aim has one deliberate benefit: it keeps `horizontal *
/// depth` comfortably inside `i32`, which a real input does not do by much
/// (part 2 on a genuine input lands within ~10% of `i32::MAX`). A generated
/// case that overflowed would panic in the dev profile and say nothing useful
/// about speed.
fn generated(commands: usize) -> String {
    let mut out = String::with_capacity(commands * 10);
    let mut seed = 0x2021_1202;

    for _ in 0..commands {
        let word = match xorshift(&mut seed) % 3 {
            0 => "forward",
            1 => "down",
            _ => "up",
        };
        let amount = xorshift(&mut seed) % 9 + 1;
        writeln!(out, "{word} {amount}").expect("writing to a String cannot fail");
    }

    out
}

/// Your own input, if you have one. Absent on a fresh clone — `days/inputs/`
/// is gitignored — and that has to stay fine, so this is an `Option` and the
/// cases built from it simply don't appear.
fn real() -> Option<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/2021-12-02.txt");
    std::fs::read_to_string(path).ok()
}

fn inputs() -> Vec<(String, String)> {
    let mut inputs = vec![
        ("example".to_string(), EXAMPLE.to_string()),
        (
            format!("generated/{GENERATED_COMMANDS}"),
            generated(GENERATED_COMMANDS),
        ),
    ];

    if let Some(input) = real() {
        inputs.push(("real".to_string(), input));
    }

    inputs
}

fn bench_day(c: &mut Criterion) {
    let mut group = c.benchmark_group("2021-12-02");

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

        group.bench_function(BenchmarkId::new("part2", &name), |b| {
            b.iter(|| black_box(day.part2().expect("part 2")))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_day);
criterion_main!(benches);
