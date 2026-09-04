//! The three sort backends head to head — pure Rust, libc's `qsort`, and
//! C++'s `std::sort` through the C shim — on one fixed input.
//!
//! Needs both boundary features (`required-features` in Cargo.toml keeps a
//! bare `--all-targets` skipping this file rather than failing it):
//!
//! ```sh
//! cd days/2024-12-01 && cargo bench --bench sort --features qsort,cpp
//! ```
//!
//! Each backend sorts a fresh unsorted clone every iteration
//! (`iter_batched`, with the clone outside the timing) — timing repeat
//! sorts of an already-sorted column would hand whichever backend ran
//! first a pre-sorted input and call it fast.

use std::hint::black_box;

use aoc_2024_12_01::{sort_pure_rust, sort_via_cpp, sort_via_qsort};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

/// One column at real scale — a Day 1 input is a thousand lines.
const COLUMN_LEN: usize = 1000;

/// xorshift64: deterministic across runs, machines, and this repo's absence
/// of a `rand` dependency, so two benchmark runs are comparable.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn column(len: usize) -> Vec<i32> {
    let mut seed = 0x2024_1201;
    (0..len)
        .map(|_| (10_000 + xorshift(&mut seed) % 90_000) as i32)
        .collect()
}

/// The shared shape all three backends already have — what makes them
/// raceable from one loop.
type SortBackend = fn(&mut [i32]);

fn bench_sorts(c: &mut Criterion) {
    let unsorted = column(COLUMN_LEN);
    let mut group = c.benchmark_group("2024-12-01/sort");

    let backends: [(&str, SortBackend); 3] = [
        ("pure_rust", sort_pure_rust),
        ("qsort", sort_via_qsort),
        ("cpp_std_sort", sort_via_cpp),
    ];

    for (name, sort) in backends {
        group.bench_with_input(
            BenchmarkId::new(name, COLUMN_LEN),
            &unsorted,
            |b, unsorted| {
                b.iter_batched(
                    || unsorted.clone(),
                    |mut column| {
                        sort(&mut column);
                        black_box(column)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_sorts);
criterion_main!(benches);
