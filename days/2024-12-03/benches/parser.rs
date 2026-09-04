//! The two parsers head to head: the nom + nom_locate library solution
//! against the talk's dependency-free byte cursor (`cursor.rs`), each doing
//! the whole job — read the corrupted stream, sum the products — per
//! iteration.
//!
//! ```sh
//! cd days/2024-12-03 && cargo bench --bench parser
//! ```
//!
//! Both sides are asserted to agree on the timed input before any timing —
//! see `generated`'s doc for the digit-free junk alphabet that keeps the
//! one known disagreement (operands past the statement's 3-digit cap,
//! which nom's `digit1` accepts) out of the race.

use std::fmt::Write as _;
use std::hint::black_box;
use std::str::FromStr;

use aoc_2024_12_03::{Day3, Part1, Part2, cursor};
use aoc_ornaments::Solution;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// Same scale as benches/day.rs: a real input is about twenty thousand
/// characters of corrupted memory.
const GENERATED_CHARS: usize = 20_000;

/// xorshift64: deterministic across runs, machines, and this repo's absence
/// of a `rand` dependency, so two benchmark runs are comparable.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// The same generator as benches/day.rs, same seed, same digit-free junk
/// alphabet — duplicated rather than shared because each bench target is
/// its own crate root, and twenty lines of generator read better than a
/// `#[path]` include.
fn generated(target_chars: usize) -> String {
    let mut out = String::with_capacity(target_chars + 16);
    let mut seed = 0x2024_1203;
    const JUNK: &[u8] = b"abcxyz*+,()[]!^_&%#@~?<> ";

    while out.len() < target_chars {
        match xorshift(&mut seed) % 10 {
            0..=4 => {
                for _ in 0..=(xorshift(&mut seed) % 4) {
                    let j = JUNK[(xorshift(&mut seed) as usize) % JUNK.len()];
                    out.push(j as char);
                }
            }
            5 | 6 => {
                let a = xorshift(&mut seed) % 1000;
                let b = xorshift(&mut seed) % 1000;
                write!(out, "mul({a},{b})").expect("writing to a String cannot fail");
            }
            7 => out.push_str("mul("),
            8 => out.push_str("do()"),
            _ => out.push_str("don't()"),
        }
    }

    out
}

// Both contenders are timed integer-out: going through `Solution::solve`
// here would put a `to_string` allocation on the nom side of the scale
// that the cursor side never pays — small against a 20k parse, but the
// race only means anything if the two sides do identical jobs.
fn nom_part1(input: &str) -> usize {
    let mut day = Day3::<Part1>::from_str(input).expect("the input parses");
    day.part1().expect("part 1")
}

fn nom_part2(input: &str) -> usize {
    let mut day = Day3::<Part2>::from_str(input).expect("the input parses");
    day.part2().expect("part 2")
}

fn bench_parsers(c: &mut Criterion) {
    let input = generated(GENERATED_CHARS);
    let mut group = c.benchmark_group("2024-12-03/parser");

    // Same bytes, same numbers, before any timing: a fast wrong answer is
    // not a contender.
    assert_eq!(nom_part1(&input), cursor::part1(&input));
    assert_eq!(nom_part2(&input), cursor::part2(&input));

    group.bench_with_input(
        BenchmarkId::new("nom/part1", GENERATED_CHARS),
        input.as_str(),
        |b, input| b.iter(|| black_box(nom_part1(black_box(input)))),
    );
    group.bench_with_input(
        BenchmarkId::new("cursor/part1", GENERATED_CHARS),
        input.as_str(),
        |b, input| b.iter(|| black_box(cursor::part1(black_box(input)))),
    );

    group.bench_with_input(
        BenchmarkId::new("nom/part2", GENERATED_CHARS),
        input.as_str(),
        |b, input| b.iter(|| black_box(nom_part2(black_box(input)))),
    );
    group.bench_with_input(
        BenchmarkId::new("cursor/part2", GENERATED_CHARS),
        input.as_str(),
        |b, input| b.iter(|| black_box(cursor::part2(black_box(input)))),
    );

    group.finish();
}

criterion_group!(benches, bench_parsers);
criterion_main!(benches);
