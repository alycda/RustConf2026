//! Benchmarks for the Day 1 reference solution — a bonus, not a workshop step.
//!
//! `just days bench 2023-12-01` from the repo root, or `cargo bench` from this
//! crate. Nothing else times anything: `cargo test` skips bench targets
//! entirely, so what `just days verify` and CI give this file is the clippy
//! gate's `--all-targets`, which compiles and lints it but never runs it.
//!
//! Parsing and the parts are timed separately on purpose. Later stages of the
//! pipeline replace one piece at a time — the parse stays in Rust while the
//! scan crosses into C, say — and a single end-to-end number cannot show which
//! side moved. This day has no second implementation to race yet; when it
//! grows one, that race belongs in its own bench target beside this file.
//!
//! Neither part short-circuits at the *line* level: both look at every
//! character of every line, because the last digit can be the last character.
//! So a single input length would look like the whole story, and isn't:
//! `bench_digit_density` holds the length fixed and varies what the characters
//! *are*, and the spread it finds is larger than the gap between the two
//! parts. Read its doc comment before reading any number here as a fact about
//! the algorithm — that is the kind of shape you have to know before a number
//! that crosses the C boundary can be read at all.

use std::fmt::Write as _;
use std::hint::black_box;
use std::str::FromStr;

use aoc_2023_12_01::Day;
use aoc_ornaments::Solution;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// Part two's statement example, which satisfies both parts — 209 and 281,
/// two different answers from one input. (Part one's own example answers 142
/// either way, which makes it useless for telling the parts apart.) Far too
/// small to say anything about performance; it is here because it is the one
/// input that is always present, and because it makes the shape of the input
/// visible in the file that benchmarks it.
const EXAMPLE: &str = "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen";

/// How many lines the generated input gets. A real 2023 day 1 input is almost
/// exactly a thousand lines, so this is the scale the numbers should be read
/// at.
const GENERATED_LINES: usize = 1000;

/// How long each generated line is. Real ones vary either side of this.
const GENERATED_LINE_LEN: usize = 40;

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
/// **Good for timing, wrong for answers.** Characters are drawn uniformly from
/// `a-z0-9` with the given digit share, which is the right *shape* and the
/// wrong *distribution*: a real input's letters are not uniform, because it is
/// built to contain spelled-out digits, and this one contains them only by
/// accident. The number it produces is not a puzzle answer and is not meant to
/// be one.
///
/// What that does *not* distort is the work. Whether `one` is present or not,
/// the scan still tries all nine words at every non-digit character; a miss
/// costs a full pass and a hit stops early, and at these densities hits are
/// rare either way. Digit *density* does change the work — considerably, and
/// not in the direction you would guess — which is why it is a parameter here
/// rather than a constant, and why it gets a benchmark of its own below.
fn generated(lines: usize, line_len: usize, digit_share: u64) -> String {
    let mut out = String::with_capacity(lines * (line_len + 1));
    let mut seed = 0x2023_1201;

    for _ in 0..lines {
        for _ in 0..line_len {
            let c = if xorshift(&mut seed) % 100 < digit_share {
                (b'0' + (xorshift(&mut seed) % 10) as u8) as char
            } else {
                (b'a' + (xorshift(&mut seed) % 26) as u8) as char
            };
            out.push(c);
        }
        writeln!(out).expect("writing to a String cannot fail");
    }

    out
}

/// Your own input, if you have one. Absent on a fresh clone — `inputs/`
/// is gitignored — and that has to stay fine, so this is an `Option` and the
/// cases built from it simply don't appear.
fn real() -> Option<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../inputs/2023-12-01.txt");
    std::fs::read_to_string(path).ok()
}

fn inputs() -> Vec<(String, String)> {
    let mut inputs = vec![
        ("example".to_string(), EXAMPLE.to_string()),
        (
            format!("generated/{GENERATED_LINES}"),
            generated(GENERATED_LINES, GENERATED_LINE_LEN, 10),
        ),
    ];

    if let Some(input) = real() {
        inputs.push(("real".to_string(), input));
    }

    inputs
}

fn bench_day(c: &mut Criterion) {
    let mut group = c.benchmark_group("2023-12-01");

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

/// The same amount of input, at four digit densities, part two only.
///
/// This was written to measure one thing and measured a different one, which
/// is why it is still here. The prediction was that digits are *cheap*:
/// `digit_at` returns as soon as it sees one and never reaches the nine-word
/// scan, so a line of nothing but digits should be the fastest case and a line
/// with none the slowest. Every case is the same length, so nothing else could
/// account for a spread.
///
/// Measured (aarch64, criterion medians, 1000 lines × 40 chars):
///
/// | digits | part 2  |
/// |--------|---------|
/// | 0%     | ~96 µs  |
/// | 10%    | ~121 µs |
/// | 50%    | ~208 µs |
/// | 100%   | ~191 µs |
///
/// Backwards, and then non-monotonic: the case doing the *most* word scanning
/// is the cheapest, and the worst case is neither extreme but the middle.
/// Whatever dominates here, it is not the nine-word scan.
///
/// Two throwaway probes (scratchpad, not committed) point at what does, and
/// they are worth naming as directions rather than figures — crude timing
/// loops next to criterion's warmed-up medians:
///
/// - Running the same scan *without* the intermediate `Vec` — `first`/`last`
///   tracked in two locals instead — leaves the 0% case unchanged and takes a
///   large bite out of every other one, growing with how many digits there are
///   to push. `Day::calibration_value` collects every digit it finds before
///   taking the first and the last, and at 0% that `collect()` yields nothing
///   and never touches the heap at all.
/// - At a fixed 50% density, laying the digits out strictly alternating
///   instead of at random is reliably faster than the random layout by around
///   a quarter. Same length, same density, same work — only the predictability
///   of the digit/not-digit branch differs, which is why 50% is the worst
///   column and 0% and 100% are not.
///
/// So the number to carry forward is not "part two costs 1.8x part one". It is
/// that on this day, what the characters *are* moves the time by more than
/// which part is running, for reasons that have nothing to do with the string
/// matching the puzzle is ostensibly about — which is exactly the sort of
/// thing a C variant would otherwise get quiet credit or quiet blame for
/// later.
fn bench_digit_density(c: &mut Criterion) {
    let mut group = c.benchmark_group("digit_density");

    for share in [0, 10, 50, 100] {
        let input = generated(GENERATED_LINES, GENERATED_LINE_LEN, share);
        let mut day = Day::from_str(&input).expect("the input parses");

        group.bench_function(BenchmarkId::new("part2", format!("{share}%")), |b| {
            b.iter(|| black_box(day.part2().expect("part 2")))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_day, bench_digit_density);
criterion_main!(benches);
