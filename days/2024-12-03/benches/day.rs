//! Benchmarks for the Day 3 reference solution — a bonus, not a workshop step.
//!
//! `just days bench 2024-12-03` from the repo root, or `cargo bench` from this
//! crate. Nothing else times anything: `cargo test` skips bench targets
//! entirely, so what `just days verify` and CI give this file is the clippy
//! gate's `--all-targets`, which compiles and lints it but never runs it.
//!
//! This day is all parse — the parts just sum products the parse already
//! collected — so the parse rows are the whole story and the part rows exist
//! to prove exactly that. Each part has its own `FromStr` (the `do()`/
//! `don't()` state lives in `Day3<Part2>`'s parse), so both parses are
//! timed. `benches/parser.rs` races this nom pipeline against the
//! byte-cursor parser in `cursor.rs`.

use std::fmt::Write as _;
use std::hint::black_box;
use std::str::FromStr;

use aoc_2024_12_03::{Day3, Part1, Part2};
use aoc_ornaments::Solution;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// The part-two statement example — the one whose two answers differ
/// (161/48). Far too small to say anything about performance; it is here
/// because it is the one input that is always present, and because it makes
/// the shape of the input visible in the file that benchmarks it.
const EXAMPLE: &str = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

/// How many characters of corrupted memory the generated input gets. A real
/// Day 3 input is a few lines totalling about twenty thousand characters, so
/// this is the scale the numbers should be read at.
const GENERATED_CHARS: usize = 20_000;

/// xorshift64: deterministic across runs, machines, and this repo's absence
/// of a `rand` dependency, so two benchmark runs are comparable.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// A stand-in for a real puzzle input, generated rather than committed:
/// inputs cannot live in this repo (see `days/README.md`), and the example
/// is three orders of magnitude too small to time.
///
/// The junk alphabet deliberately holds no digits: a stray digit after a
/// truncated `mul(` fragment could grow an operand past three digits, which
/// the statement caps and the byte cursor rejects but nom's `digit1`
/// accepts — the one known disagreement between the two parsers, kept out
/// of the generator so `benches/parser.rs` can assert they agree on every
/// input it times. The resulting score is deterministic but means nothing
/// as a puzzle answer.
fn generated(target_chars: usize) -> String {
    let mut out = String::with_capacity(target_chars + 16);
    let mut seed = 0x2024_1203;
    const JUNK: &[u8] = b"abcxyz*+,()[]!^_&%#@~?<> ";

    while out.len() < target_chars {
        match xorshift(&mut seed) % 10 {
            // a short run of corruption
            0..=4 => {
                for _ in 0..=(xorshift(&mut seed) % 4) {
                    let j = JUNK[(xorshift(&mut seed) as usize) % JUNK.len()];
                    out.push(j as char);
                }
            }
            // a well-formed mul with statement-sized (1-3 digit) operands
            5 | 6 => {
                let a = xorshift(&mut seed) % 1000;
                let b = xorshift(&mut seed) % 1000;
                write!(out, "mul({a},{b})").expect("writing to a String cannot fail");
            }
            // a truncated fragment — corruption to both parsers
            7 => out.push_str("mul("),
            8 => out.push_str("do()"),
            _ => out.push_str("don't()"),
        }
    }

    out
}

/// Your own input, if you have one. Absent on a fresh clone — inputs are
/// gitignored — and that has to stay fine, so this is an `Option` and the
/// cases built from it simply don't appear.
fn real() -> Option<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/2024-12-03.txt");
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
    let mut group = c.benchmark_group("2024-12-03");

    for (name, input) in inputs() {
        group.bench_with_input(
            BenchmarkId::new("parse1", &name),
            input.as_str(),
            |b, input| {
                b.iter(|| Day3::<Part1>::from_str(black_box(input)).expect("the input parses"))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse2", &name),
            input.as_str(),
            |b, input| {
                b.iter(|| Day3::<Part2>::from_str(black_box(input)).expect("the input parses"))
            },
        );

        // Parsed once, outside the timing loop: the products are already
        // collected, so these rows should be nanoseconds — their job is to
        // show that the parse rows above are the whole cost of this day.
        // The day goes through `black_box` *inside* the loop — boxing only
        // the result would let the optimizer notice the loop-invariant
        // input and hoist the whole computation out of the iterations.
        let mut day1 = Day3::<Part1>::from_str(&input).expect("the input parses");
        let mut day2 = Day3::<Part2>::from_str(&input).expect("the input parses");

        group.bench_function(BenchmarkId::new("part1", &name), |b| {
            b.iter(|| black_box(black_box(&mut day1).part1().expect("part 1")))
        });

        group.bench_function(BenchmarkId::new("part2", &name), |b| {
            b.iter(|| black_box(black_box(&mut day2).part2().expect("part 2")))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_day);
criterion_main!(benches);
