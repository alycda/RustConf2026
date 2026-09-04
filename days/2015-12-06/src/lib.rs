//! Day scaffold.
//!
//! Three things to fill in:
//!
//! 1. `Day`'s shape and its `FromStr` — the parse starts line-based, which
//!    is a fine default but rarely the final answer.
//! 2. `part1` / `part2`.
//! 3. The tests — paste the example input from the problem statement and
//!    drop the `#[ignore]`. Example inputs only: never commit your real
//!    puzzle input or the full puzzle text.

use std::str::FromStr;

use aoc_ornaments::{Solution, SolutionResult};

#[derive(Debug, Clone)]
pub struct Day(Vec<String>);

/// Gives the parts `self.iter()` and the rest of `Vec`'s read API directly.
impl std::ops::Deref for Day {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Day {
    type Err = miette::Error;

    fn from_str(input: &str) -> miette::Result<Self> {
        Ok(Self(input.lines().map(str::to_string).collect()))
    }
}

impl Solution for Day {
    /// Whatever both parts return. Must be `Display + Default`.
    type Output = usize;

    fn part1(&mut self) -> SolutionResult<Self::Output> {
        todo!("part 1")
    }

    fn part2(&mut self) -> SolutionResult<Self::Output> {
        todo!("part 2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;

    #[test]
    #[ignore = "scaffold — add the example input, then drop this attribute"]
    fn test_part1() -> miette::Result<()> {
        let input = "";
        assert_eq!("0", Day::from_str(input)?.solve(Part::One)?);
        Ok(())
    }

    #[test]
    #[ignore = "scaffold — add the example input, then drop this attribute"]
    fn test_part2() -> miette::Result<()> {
        let input = "";
        assert_eq!("0", Day::from_str(input)?.solve(Part::Two)?);
        Ok(())
    }
}
