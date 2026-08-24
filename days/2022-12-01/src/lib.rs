//! Day 1: Calorie Counting
//!
//! Each elf's inventory is a block of calorie counts, blocks separated by
//! a blank line.
//!
//! --- Part One ---
//!
//! Find the elf carrying the most total calories.
//!
//! --- Part Two ---
//!
//! Sum the total calories carried by the top three elves.

use std::str::FromStr;

use aoc_ornaments::{Solution, SolutionResult};

/// Each element is one elf's total calories.
#[derive(Debug, Clone)]
pub struct Day(Vec<usize>);

/// Gives the parts `self.iter()` and the rest of `Vec`'s read API directly.
impl std::ops::Deref for Day {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Day {
    type Err = miette::Error;

    fn from_str(input: &str) -> miette::Result<Self> {
        Ok(Self(
            input
                .split("\n\n")
                .map(|elf| {
                    elf.lines()
                        .map(|line| {
                            line.parse::<usize>()
                                .map_err(|e| miette::miette!("bad calorie count {line}: {e}"))
                        })
                        .sum()
                })
                .collect::<miette::Result<_>>()?,
        ))
    }
}

impl Solution for Day {
    type Output = usize;

    /// The single highest elf total.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        self.iter()
            .max()
            .copied()
            .ok_or_else(|| miette::miette!("no elves"))
    }

    /// Sum of the three highest elf totals.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        let mut totals = self.0.clone();
        totals.sort_unstable_by(|a, b| b.cmp(a));

        Ok(totals.iter().take(3).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;

    const EXAMPLE: &str = "1000
2000
3000

4000

5000
6000

7000
8000
9000

10000";

    #[test]
    fn test_part1() -> miette::Result<()> {
        assert_eq!("24000", Day::from_str(EXAMPLE)?.solve(Part::One)?);
        Ok(())
    }

    #[test]
    fn test_part2() -> miette::Result<()> {
        assert_eq!("45000", Day::from_str(EXAMPLE)?.solve(Part::Two)?);
        Ok(())
    }
}
