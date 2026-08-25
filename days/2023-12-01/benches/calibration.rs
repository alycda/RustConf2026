//! Plain Rust vs. a malware scanner vs. a speech synthesiser, head to head.
//!
//! Puzzle inputs aren't committed (see `days/.gitignore`), so this builds its
//! own synthetic lines rather than reading one — same reasoning as
//! `src/main.rs`. Requires both features; without either the target is
//! skipped, which is what keeps a default `--all-targets` build green on a
//! machine with no C libraries:
//!
//! ```sh
//! cargo bench -p aoc-2023-12-01 --bench calibration --features yara,espeak
//! ```
//!
//! What is timed is the *whole* of what each backend does with a solve. For
//! YARA that means `yr_compiler_create`, compiling the rule text, one
//! `yr_rules_scan_mem` per line and the teardown; for espeak it means one
//! `espeak_TextToPhonemes` per character position of every line. Both are
//! real per-solve costs — a fresh rule set per solve is an isolation property
//! `src/yara.rs` documents, not an implementation detail to optimise away —
//! and hoisting either outside the loop would flatter that side for work it
//! actually performs on every call.
//!
//! Only part two is raced. Part one does not exist on the espeak side (a
//! phoneme cannot tell `1` from `one`; see `src/espeak.rs`), so a part-one
//! group would silently be a two-way race wearing a three-way label.
//!
//! The last group exists because of what the first one hides. YARA's rule
//! compilation is a fixed cost that a thousand lines amortise and a short
//! input does not, and the per-solve number cannot show where the line is.
//! `setup_vs_scan` separates them.

use std::hint::black_box;
use std::str::FromStr;

use aoc_2023_12_01::{
    Day, espeak::Speaker, espeak::sum_calibration_with_words_via_espeak,
    sum_calibration_with_words_pure_rust, sum_calibration_with_words_via_yara, yara::Scanner,
};
use criterion::{Criterion, criterion_group, criterion_main};

/// A real 2023 day 1 input is almost exactly a thousand lines. espeak costs
/// roughly 200 µs *per line*, though, so racing it over a thousand would mean
/// a 200 ms iteration and a criterion run measured in minutes for a ratio that
/// is already unambiguous at a tenth of the size.
const LINES: usize = 100;

/// Real lines run 30-45 characters. espeak's cost is per character position,
/// so this number matters more to it than to the other two.
const LINE_LEN: usize = 40;

/// xorshift64 — deterministic, so two runs are comparable. Same generator as
/// `benches/day.rs`, and the same caveat applies: characters uniform over
/// `a-z0-9` are the right *shape* and the wrong *distribution*, because a real
/// input is built to contain spelled-out digits and this one contains them
/// only by accident. Whatever it sums to is not a puzzle answer.
fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn synthetic(lines: usize, line_len: usize) -> Day {
    let mut out = String::with_capacity(lines * (line_len + 1));
    let mut seed = 0x2023_1201;

    for _ in 0..lines {
        for _ in 0..line_len {
            let c = if xorshift(&mut seed) % 100 < 10 {
                (b'0' + (xorshift(&mut seed) % 10) as u8) as char
            } else {
                (b'a' + (xorshift(&mut seed) % 26) as u8) as char
            };
            out.push(c);
        }
        out.push('\n');
    }

    Day::from_str(&out).expect("synthetic input parses")
}

fn bench_part2(c: &mut Criterion) {
    let day = synthetic(LINES, LINE_LEN);

    let mut group = c.benchmark_group("calibration_with_words");
    group.sample_size(20);
    group.bench_function("pure_rust", |b| {
        b.iter(|| sum_calibration_with_words_pure_rust(black_box(&day)))
    });
    group.bench_function("yara", |b| {
        b.iter(|| sum_calibration_with_words_via_yara(black_box(&day)).unwrap())
    });
    // Not checked against the other two: this backend does not agree with them
    // and is not supposed to (72.8% of lines on a real input). It is here for
    // its cost, and `espeak::agreement_with_pure_rust` is where correctness is
    // reported.
    group.bench_function("espeak", |b| {
        b.iter(|| sum_calibration_with_words_via_espeak(black_box(&day)).unwrap())
    });
    group.finish();
}

/// Where each library's time actually goes — the split the per-solve numbers
/// bury.
///
/// YARA's `Scanner::new` compiles a nineteen-string rule into an automaton
/// once; everything after it is per line. espeak's `Speaker::new` initialises
/// the engine and phonemises nineteen references, which is trivial next to
/// what it then spends per character. Two libraries, opposite shapes, and
/// neither is visible in the group above.
fn bench_setup_vs_scan(c: &mut Criterion) {
    let day = synthetic(LINES, LINE_LEN);
    let longest = day.iter().map(String::len).max().unwrap_or(0);

    let mut group = c.benchmark_group("setup_vs_scan");
    group.sample_size(20);

    group.bench_function("yara/compile_rules", |b| {
        b.iter(|| Scanner::new(black_box(true), black_box(longest)).unwrap())
    });
    group.bench_function("yara/scan_only", |b| {
        let mut scanner = Scanner::new(true, longest).expect("the rules compile");
        b.iter(|| {
            for line in day.iter() {
                black_box(scanner.calibration_value(line).unwrap());
            }
        })
    });

    group.bench_function("espeak/initialise", |b| b.iter(|| Speaker::new().unwrap()));
    group.bench_function("espeak/speak_only", |b| {
        let speaker = Speaker::new().expect("espeak initialises");
        b.iter(|| {
            for line in day.iter() {
                black_box(speaker.calibration_value(line).unwrap());
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_part2, bench_setup_vs_scan);
criterion_main!(benches);
