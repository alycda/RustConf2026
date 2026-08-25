//! Plain Rust vs. vectorscan vs. ICU regex, one predicate, one input.
//!
//! `cargo bench -p aoc-2015-12-05 --bench nice --features hyperscan,icu` —
//! not `just days bench`, which runs only the `day` target: this one needs
//! both C libraries, so it declares `required-features` and cargo skips it
//! (rather than failing it) on a default build.
//!
//! The three implementations answer the same question three ways, and each
//! pays for it somewhere different:
//!
//! - `is_nice_pure_rust` walks the line with iterators. No boundary at all.
//! - `..._via_hyperscan` scans it against a database compiled once into a
//!   `OnceLock` — 31 patterns for part 1 (26 double letters, 4 forbidden
//!   pairs, one `[aeiou]` class), 702 for part 2 (676 pair literals plus 26
//!   sandwich patterns), because Hyperscan's vectorized model has no
//!   backreferences. The compile is amortized away by the first iteration;
//!   what is timed is one `hs_scan` plus the match-callback bookkeeping.
//! - `..._via_icu` calls into `src/icu_shim.c`, which `uregex_openC`s the
//!   pattern, runs it, and closes it — *per call*. Two or three of those per
//!   line. That is what `Solution::part1` actually does, so this is the honest
//!   cost of the ICU variant rather than the cost of a pre-compiled regex.
//!
//! Timed per line, not per file, so the number is comparable to the one a
//! single `is_nice_*` call costs anywhere else.

use std::hint::black_box;

use aoc_2015_12_05::{
    hyperscan::{is_nice_v2_via_hyperscan, is_nice_via_hyperscan},
    icu::{is_nice_v2_via_icu, is_nice_via_icu},
    is_nice_pure_rust, is_nice_v2_pure_rust,
};
use criterion::{Criterion, criterion_group, criterion_main};

/// One nice line and one naughty one per part, from the statement. Both are
/// timed: a naughty line can exit early in Rust (a forbidden pair is found and
/// the rest never runs) where the scanners still traverse the whole line, and
/// benchmarking only the nice ones would hide exactly that.
const PART1_NICE: &str = "ugknbfddgicrmopn";
const PART1_NAUGHTY: &str = "haegwjzuvuyypxyu";
const PART2_NICE: &str = "qjhvhtzxzqqjkmpb";
const PART2_NAUGHTY: &str = "uurcxstgmygtbstg";

fn bench_part1(c: &mut Criterion) {
    for (label, line) in [("nice", PART1_NICE), ("naughty", PART1_NAUGHTY)] {
        let mut group = c.benchmark_group(format!("is_nice/{label}"));
        group.bench_function("pure_rust", |b| {
            b.iter(|| is_nice_pure_rust(black_box(line)))
        });
        group.bench_function("vectorscan", |b| {
            b.iter(|| is_nice_via_hyperscan(black_box(line)))
        });
        group.bench_function("icu", |b| b.iter(|| is_nice_via_icu(black_box(line))));
        group.finish();
    }
}

fn bench_part2(c: &mut Criterion) {
    for (label, line) in [("nice", PART2_NICE), ("naughty", PART2_NAUGHTY)] {
        let mut group = c.benchmark_group(format!("is_nice_v2/{label}"));
        group.bench_function("pure_rust", |b| {
            b.iter(|| is_nice_v2_pure_rust(black_box(line)))
        });
        group.bench_function("vectorscan", |b| {
            b.iter(|| is_nice_v2_via_hyperscan(black_box(line)))
        });
        group.bench_function("icu", |b| b.iter(|| is_nice_v2_via_icu(black_box(line))));
        group.finish();
    }
}

criterion_group!(benches, bench_part1, bench_part2);
criterion_main!(benches);
