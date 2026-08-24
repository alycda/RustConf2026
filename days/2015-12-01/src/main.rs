use std::path::Path;
use std::str::FromStr;

use aoc_2015_12_01::{Day, caca};
use aoc_ornaments::{Part, Solution};

/// Run Part 1 and Part 2 against your own puzzle input.
///
/// Puzzle inputs are never committed (see `days/.gitignore`) — drop yours at
/// `days/inputs/2015-12-01.txt`. The path is anchored to this crate's directory
/// rather than the working directory, so it resolves the same however cargo
/// is invoked, and read at runtime rather than with `include_str!` so the
/// crate still builds and tests without one.
fn main() -> miette::Result<()> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../inputs/2015-12-01.txt");
    let input = std::fs::read_to_string(path)
        .map_err(|e| miette::miette!("could not read {}: {}", path, e))?;

    let mut day = Day::from_str(&input)?;
    let part1 = day.solve(Part::One)?;
    let part2 = day.solve(Part::Two)?;

    let font = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/standard.flf"));
    print!("{}", caca::figlet_banner(font, &part1)?);
    print!("{}", caca::figlet_banner(font, &part2)?);

    Ok(())
}
