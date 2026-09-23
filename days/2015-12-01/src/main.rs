use std::str::FromStr;

use aoc_2015_12_01::Day;
use aoc_ornaments::{Part, Solution};

/// Run Part 1 and Part 2 against your own puzzle input.
///
/// Puzzle inputs are never committed (see `.gitignore`) — drop yours at
/// `inputs/2015-12-01.txt` in the repo root. The path is anchored to this
/// crate's directory rather than the working directory, so it resolves the
/// same however cargo is invoked, and read at runtime rather than with
/// `include_str!` so the crate still builds and tests without one.
///
/// With the `caca` feature the answers come out as FIGlet banners; without
/// it, plain lines. (`cargo run --features caca,tcc` for the full show.)
fn main() -> miette::Result<()> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../inputs/2015-12-01.txt");
    let input = std::fs::read_to_string(path)
        .map_err(|e| miette::miette!("could not read {}: {}", path, e))?;

    let mut day = Day::from_str(&input)?;
    let part1 = day.solve(Part::One)?;
    let part2 = day.solve(Part::Two)?;

    #[cfg(feature = "caca")]
    {
        let font = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/standard.flf"));

        println!("🦀:");
        print!("{}", aoc_2015_12_01::caca::figlet_banner(font, &part1)?);
        print!("{}", aoc_2015_12_01::caca::figlet_banner(font, &part2)?);
    }
    #[cfg(not(feature = "caca"))]
    {
        println!("Part 1: {part1}");
        println!("Part 2: {part2}");
    }

    Ok(())
}
