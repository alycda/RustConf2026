//! Part 2's counting structures head to head: the naive scan, Rust's
//! `HashMap`, `ahash`'s map, and the C hash table behind the `uthash`
//! feature — the talk's part-2 arc, re-measured in this tree.
//!
//! Needs the boundary feature (`required-features` keeps a bare
//! `--all-targets` skipping this file rather than failing it):
//!
//! ```sh
//! cd days/2024-12-01 && cargo bench --bench lookup --features uthash
//! ```
//!
//! Every contender does the whole job per iteration — build the frequency
//! map from the right column, query it once per left-hand ID, tear it down.
//! uthash cannot amortize its malloc-per-entry build across iterations, so
//! nothing else gets to amortize either; splitting build from query would
//! be a different (also interesting) benchmark, not this one.
//!
//! The columns draw from 0..500 over 1000 entries so collisions and real
//! matches actually occur — a map benched on all-missing keys only ever
//! times its miss path. The resulting score still means nothing as a puzzle
//! answer; see benches/day.rs on generated inputs and counting.
//!
//! Both numbers are knobs, because the ordering is a property of the
//! workload and not of the structures. `LOOKUP_LEN` sets the column length
//! and `LOOKUP_RANGE` the key range (so `LOOKUP_RANGE` is roughly the number
//! of distinct keys the map holds):
//!
//! ```sh
//! LOOKUP_LEN=10000 LOOKUP_RANGE=5000 cargo bench --bench lookup --features uthash
//! ```
//!
//! Predict the ordering before you run it. At ten times the size uthash
//! trails ahash, and at twenty times it trails std's `HashMap` too: a malloc
//! per entry and pointer-chasing lookups meet a table that no longer fits in
//! cache. The C table's lead at a thousand lines is not a law — see the
//! README.

use std::collections::HashMap;
use std::hint::black_box;

use ahash::AHashMap;
use aoc_2024_12_01::{similarity_pure_rust, similarity_via_uthash};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// Both columns at real scale — a Day 1 input is a thousand lines. Override
/// with `LOOKUP_LEN`.
const COLUMN_LEN: usize = 1000;

/// Keys draw from `0..KEY_RANGE`, so this is about how many distinct keys
/// the map ends up holding. Override with `LOOKUP_RANGE`.
const KEY_RANGE: u64 = 500;

/// A knob from the environment, or its default: benches take no arguments
/// of their own, and criterion owns the command line.
fn knob(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// xorshift64: deterministic across runs, machines, and this repo's absence
/// of a `rand` dependency, so two benchmark runs are comparable.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn column(len: usize, range: u64, seed: u64) -> Vec<i32> {
    let mut seed = seed;
    (0..len)
        .map(|_| (xorshift(&mut seed) % range) as i32)
        .collect()
}

/// The talk's `process_part_2_hashmap`, reshaped onto columns: build the
/// frequency map with a fold, then weight each left-hand ID by its count.
fn similarity_std_hashmap(left: &[i32], right: &[i32]) -> i32 {
    let counts: HashMap<i32, usize> = right.iter().fold(HashMap::new(), |mut acc, &n| {
        *acc.entry(n).or_insert(0) += 1;
        acc
    });

    left.iter()
        .map(|&n| n * *counts.get(&n).unwrap_or(&0) as i32)
        .sum()
}

/// The talk's `process_part_2_ahash`: same fold, faster hasher. SipHash
/// (std's default) buys DoS resistance; ahash trades that for speed, which
/// is a fine trade when the keys aren't attacker-controlled.
fn similarity_ahash(left: &[i32], right: &[i32]) -> i32 {
    let counts: AHashMap<i32, usize> = right.iter().fold(AHashMap::new(), |mut acc, &n| {
        *acc.entry(n).or_insert(0) += 1;
        acc
    });

    left.iter()
        .map(|&n| n * *counts.get(&n).unwrap_or(&0) as i32)
        .sum()
}

type Counter = fn(&[i32], &[i32]) -> i32;

fn bench_lookups(c: &mut Criterion) {
    let len = knob("LOOKUP_LEN", COLUMN_LEN);
    let range = knob("LOOKUP_RANGE", KEY_RANGE as usize) as u64;
    eprintln!("lookup: {len} entries per column, keys in 0..{range}");
    let left = column(len, range, 0x2024_1201);
    let right = column(len, range, 0x2024_1203);
    let mut group = c.benchmark_group("2024-12-01/lookup");

    let contenders: [(&str, Counter); 4] = [
        ("naive_scan", similarity_pure_rust),
        ("std_hashmap", similarity_std_hashmap),
        ("ahash", similarity_ahash),
        ("uthash", similarity_via_uthash),
    ];

    // Same columns, same score, before any timing: a fast wrong answer is
    // not a contender.
    let expected = similarity_pure_rust(&left, &right);
    for (name, counter) in contenders {
        assert_eq!(expected, counter(&left, &right), "{name} disagrees");
    }

    for (name, counter) in contenders {
        group.bench_function(BenchmarkId::new(name, len), |b| {
            b.iter(|| black_box(counter(black_box(&left), black_box(&right))))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_lookups);
criterion_main!(benches);
