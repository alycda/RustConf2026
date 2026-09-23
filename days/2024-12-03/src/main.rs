use std::str::FromStr;

use aoc_2024_12_03::{Day, Part1, Part2};
use aoc_ornaments::{Part, Solution};

/// Run Part 1 and Part 2 against your own puzzle input.
///
/// Each part is a distinct phantom type — `Day<Part1>` only implements part 1
/// and `Day<Part2>` only part 2 — so the input is parsed once per part.
///
/// Puzzle inputs are never committed (see `.gitignore`) — drop yours at
/// `inputs/2024-12-03.txt` in the repo root. The path is anchored to this
/// crate's directory rather than the working directory, so it resolves the
/// same however cargo is invoked, and read at runtime rather than with
/// `include_str!` so the crate still builds and tests without one.
fn main() -> miette::Result<()> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../inputs/2024-12-03.txt");
    let input = std::fs::read_to_string(path)
        .map_err(|e| miette::miette!("could not read {}: {}", path, e))?;

    let part1 = Day::<Part1>::from_str(&input)?.solve(Part::One)?;
    let part2 = Day::<Part2>::from_str(&input)?.solve(Part::Two)?;

    println!("Part 1: {}", part1);
    println!("Part 2: {}", part2);

    Ok(())
}
