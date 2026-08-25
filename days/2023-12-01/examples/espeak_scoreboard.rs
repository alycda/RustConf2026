//! The espeak variant's agreement rate against plain Rust, on your own input.
//!
//! ```sh
//! cargo run -p aoc-2023-12-01 --features espeak --example espeak_scoreboard
//! ```
//!
//! An example rather than a test, for two reasons: it needs
//! `days/inputs/2023-12-01.txt`, which is gitignored and absent on a fresh
//! clone, and the number it prints is the one quoted in `src/espeak.rs` and in
//! the day README. Anyone attacking the open challenge documented there runs
//! this to find out whether they moved it.
use std::str::FromStr;

use aoc_2023_12_01::{Day, espeak, sum_calibration_with_words_pure_rust};

fn main() -> miette::Result<()> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/2023-12-01.txt");
    let input = std::fs::read_to_string(path)
        .map_err(|e| miette::miette!("could not read {}: {}", path, e))?;
    let day = Day::from_str(&input)?;

    let (agreed, total) = espeak::agreement_with_pure_rust(&day)?;
    let heard = espeak::sum_calibration_with_words_via_espeak(&day)?;
    let read = sum_calibration_with_words_pure_rust(&day);

    println!(
        "lines agreeing with plain Rust: {agreed}/{total} ({:.1}%)",
        100.0 * agreed as f64 / total as f64
    );
    println!("part 2 heard: {heard}");
    println!("part 2 read:  {read}");
    Ok(())
}
