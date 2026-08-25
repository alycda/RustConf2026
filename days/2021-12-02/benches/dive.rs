//! Pure Rust vs. Chipmunk2D's rigid-body solver, head to head.
//!
//! Puzzle inputs aren't committed (see `days/.gitignore`), so this builds its
//! own synthetic course rather than reading one — same reasoning as
//! `src/main.rs`. Requires `--features chipmunk`; without it the target is
//! skipped, which is what keeps a default `--all-targets` build green on a
//! machine with no C library:
//!
//! ```sh
//! cargo bench -p aoc-2021-12-02 --bench dive --features chipmunk
//! ```
//!
//! What is being timed is the *whole* of what `Solution::part1`/`part2` do
//! with the feature on: `cpSpaceNew`, `cpBodyNew`, one `cpSpaceStep` per
//! command, and the teardown. Standing the space up is a real per-solve cost
//! and hiding it outside the loop would flatter the C side for work it
//! actually does on every call.
//!
//! Both parts are raced, not just one. They cost different things — part 2
//! reads `cpBodyGetAngle` back across the boundary on every `forward` and
//! part 1 never does — and a single number would average that difference
//! away.

use std::hint::black_box;
use std::str::FromStr;

use aoc_2021_12_02::{
    Day, dead_reckon_pure_rust, dead_reckon_via_chipmunk, dead_reckon_with_aim_pure_rust,
    dead_reckon_with_aim_via_chipmunk,
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
    group.bench_function("chipmunk", |b| {
        b.iter(|| dead_reckon_via_chipmunk(black_box(&course)).unwrap())
    });
    group.finish();
}

fn bench_dead_reckon_with_aim(c: &mut Criterion) {
    let course = synthetic_course(COMMANDS);

    let mut group = c.benchmark_group("dead_reckon_with_aim");
    group.bench_function("pure_rust", |b| {
        b.iter(|| dead_reckon_with_aim_pure_rust(black_box(&course)))
    });
    group.bench_function("chipmunk", |b| {
        b.iter(|| dead_reckon_with_aim_via_chipmunk(black_box(&course)).unwrap())
    });
    group.finish();
}

criterion_group!(benches, bench_dead_reckon, bench_dead_reckon_with_aim);
criterion_main!(benches);
