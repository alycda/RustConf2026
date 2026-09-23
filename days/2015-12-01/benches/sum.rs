//! Pure Rust vs. libtcc-JIT-compiled C, head to head.
//!
//! Puzzle inputs aren't committed (see `.gitignore`), so this builds
//! its own synthetic instruction stream rather than reading one — same
//! reasoning as `src/main.rs`. `sum_via_c`/`basement_position_via_c` pay
//! for a full compile-relocate-lookup-teardown cycle on every call (that's
//! what `Solution::part1`/`part2` actually do), so this is the honest cost
//! of "JIT a C function per call", not just the cost of running one.

use aoc_2015_12_01::{
    Day, basement_position_pure_rust, basement_position_via_c, sum_pure_rust, sum_via_c,
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::str::FromStr;

/// `"()())"` sums to -1 every 5 instructions, so repeating it drifts the
/// running total negative early — exercising the same early-return path
/// `basement_position_via_c` takes on real puzzle input.
fn synthetic_floors(repeats: usize) -> Vec<i32> {
    let input = "()())".repeat(repeats);
    Day::from_str(&input)
        .expect("synthetic input parses")
        .to_vec()
}

fn bench_sum(c: &mut Criterion) {
    let floors = synthetic_floors(2_000);

    let mut group = c.benchmark_group("sum");
    group.bench_function("pure_rust", |b| {
        b.iter(|| sum_pure_rust(black_box(&floors)))
    });
    group.bench_function("c_via_ffi_jit", |b| {
        b.iter(|| sum_via_c(black_box(&floors)).unwrap())
    });
    group.finish();
}

fn bench_basement_position(c: &mut Criterion) {
    let floors = synthetic_floors(2_000);

    let mut group = c.benchmark_group("basement_position");
    group.bench_function("pure_rust", |b| {
        b.iter(|| basement_position_pure_rust(black_box(&floors)))
    });
    group.bench_function("c_via_ffi_jit", |b| {
        b.iter(|| basement_position_via_c(black_box(&floors)).unwrap())
    });
    group.finish();
}

criterion_group!(benches, bench_sum, bench_basement_position);
criterion_main!(benches);
