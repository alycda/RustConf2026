//! Pure Rust vs. a fold expressed in SQL, head to head.
//!
//! Puzzle inputs aren't committed (see `days/.gitignore`), so this builds its
//! own synthetic course rather than reading one — same reasoning as
//! `src/main.rs`. Requires `--features duckdb`; without it the target is
//! skipped, which is what keeps a default `--all-targets` build green on a
//! machine with no C library:
//!
//! ```sh
//! cargo bench -p aoc-2021-12-02 --bench dive --features duckdb
//! ```
//!
//! What is being timed is the *whole* of what `Solution::part1`/`part2` do
//! with the feature on: open an in-memory database, connect, create the table,
//! bulk-load the course, run the query, tear it all down. That is a real
//! per-solve cost — a fresh database per solve is not an implementation
//! detail here but the isolation property `src/duckdb.rs` documents — and
//! hoisting it outside the loop would flatter the database for work it
//! actually performs on every call.
//!
//! The third group exists because of what the first two hide. Standing the
//! database up dominates so completely that `part1` and `part2` look nearly
//! identical, and the interesting number — what the *query* costs, which is
//! the only part DuckDB is actually built for — never appears. `query_only`
//! reuses one loaded `Course` so the two are separable.

use std::hint::black_box;
use std::str::FromStr;

use aoc_2021_12_02::duckdb::{Course, PART1_SQL, PART2_SQL};
use aoc_2021_12_02::{
    Day, dead_reckon_pure_rust, dead_reckon_via_duckdb, dead_reckon_with_aim_pure_rust,
    dead_reckon_with_aim_via_duckdb,
};
use criterion::{Criterion, criterion_group, criterion_main};

/// One fixed input for every case, at the scale of a real puzzle input
/// (a genuine 2021 day 2 input is almost exactly a thousand lines).
const COMMANDS: usize = 1000;

/// xorshift64 — deterministic, so two runs are comparable. Same generator as
/// `benches/day.rs`, and the same caveat applies: uniform commands with
/// amounts 1..=9 make the aim very nearly cancel, which is the wrong
/// *distribution* for a real input but the right amount of *work*, and keeps
/// `horizontal * depth` inside `i32` where a realistic aim would overflow it.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn synthetic_course(commands: usize) -> Day {
    let mut out = String::with_capacity(commands * 10);
    let mut seed = 0x2021_1202;

    for _ in 0..commands {
        let word = match xorshift(&mut seed) % 3 {
            0 => "forward",
            1 => "down",
            _ => "up",
        };
        let amount = xorshift(&mut seed) % 9 + 1;
        out.push_str(word);
        out.push(' ');
        out.push_str(&amount.to_string());
        out.push('\n');
    }

    Day::from_str(&out).expect("synthetic input parses")
}

fn bench_dead_reckon(c: &mut Criterion) {
    let course = synthetic_course(COMMANDS);

    let mut group = c.benchmark_group("dead_reckon");
    group.bench_function("pure_rust", |b| {
        b.iter(|| dead_reckon_pure_rust(black_box(&course)))
    });
    group.bench_function("duckdb", |b| {
        b.iter(|| dead_reckon_via_duckdb(black_box(&course)).unwrap())
    });
    group.finish();
}

fn bench_dead_reckon_with_aim(c: &mut Criterion) {
    let course = synthetic_course(COMMANDS);

    let mut group = c.benchmark_group("dead_reckon_with_aim");
    group.bench_function("pure_rust", |b| {
        b.iter(|| dead_reckon_with_aim_pure_rust(black_box(&course)))
    });
    group.bench_function("duckdb", |b| {
        b.iter(|| dead_reckon_with_aim_via_duckdb(black_box(&course)).unwrap())
    });
    group.finish();
}

/// The query alone, against an already-loaded database — the number the
/// per-solve timings above bury. Part 2's window function versus part 1's two
/// plain aggregates is a comparison worth having on its own, and it is
/// invisible while both are dominated by `duckdb_open`.
fn bench_query_only(c: &mut Criterion) {
    let commands = synthetic_course(COMMANDS);
    let loaded = Course::load(&commands).expect("the course loads");

    let mut group = c.benchmark_group("query_only");
    group.bench_function("part1_aggregates", |b| {
        b.iter(|| loaded.scalar(black_box(PART1_SQL)).unwrap())
    });
    group.bench_function("part2_window_function", |b| {
        b.iter(|| loaded.scalar(black_box(PART2_SQL)).unwrap())
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_dead_reckon,
    bench_dead_reckon_with_aim,
    bench_query_only
);
criterion_main!(benches);
