//! Day 2: Password Philosophy
//!
//! Each line is a policy and a password: `1-3 a: abcde`.
//!
//! --- Part One ---
//!
//! The policy is a min/max count: the password is valid if `a` appears
//! between 1 and 3 times.
//!
//! --- Part Two ---
//!
//! The policy is instead a pair of 1-indexed positions: the password is
//! valid if `a` occupies exactly one of them.

use std::str::FromStr;

use aoc_ornaments::{Solution, SolutionResult};

/// One `min-max letter: password` line.
#[derive(Debug, Clone)]
pub struct Policy {
    min: usize,
    max: usize,
    letter: char,
    password: String,
}

impl FromStr for Policy {
    type Err = miette::Error;

    fn from_str(line: &str) -> miette::Result<Self> {
        let (range, rest) = line
            .split_once(' ')
            .ok_or_else(|| miette::miette!("malformed line: {line}"))?;
        let (letter, password) = rest
            .split_once(": ")
            .ok_or_else(|| miette::miette!("malformed line: {line}"))?;
        let (min, max) = range
            .split_once('-')
            .ok_or_else(|| miette::miette!("malformed range: {range}"))?;

        Ok(Self {
            min: min
                .parse()
                .map_err(|e| miette::miette!("bad min in {line}: {e}"))?,
            max: max
                .parse()
                .map_err(|e| miette::miette!("bad max in {line}: {e}"))?,
            letter: letter
                .chars()
                .next()
                .ok_or_else(|| miette::miette!("missing letter in {line}"))?,
            password: password.to_string(),
        })
    }
}

impl Policy {
    /// The letter must appear between `min` and `max` times (inclusive).
    fn is_valid_by_count(&self) -> bool {
        let count = self.password.chars().filter(|&c| c == self.letter).count();

        (self.min..=self.max).contains(&count)
    }

    /// Exactly one of the two (1-indexed) positions holds the letter.
    fn is_valid_by_position(&self) -> bool {
        let chars: Vec<char> = self.password.chars().collect();
        let first = chars.get(self.min - 1) == Some(&self.letter);
        let second = chars.get(self.max - 1) == Some(&self.letter);

        first ^ second
    }
}

#[derive(Debug, Clone)]
pub struct Day(Vec<Policy>);

/// Gives the parts `self.iter()` and the rest of `Vec`'s read API directly.
impl std::ops::Deref for Day {
    type Target = Vec<Policy>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Day {
    type Err = miette::Error;

    fn from_str(input: &str) -> miette::Result<Self> {
        Ok(Self(
            input
                .lines()
                .map(Policy::from_str)
                .collect::<miette::Result<_>>()?,
        ))
    }
}

impl Day {
    fn compute(&self, f: fn(&Policy) -> bool) -> usize {
        self.iter().filter(|policy| f(policy)).count()
    }
}

impl Solution for Day {
    type Output = usize;

    /// Count passwords valid under the count policy.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        Ok(self.compute(Policy::is_valid_by_count))
    }

    /// Count passwords valid under the position policy.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        Ok(self.compute(Policy::is_valid_by_position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;
    use rstest::rstest;

    const EXAMPLE: &str = "1-3 a: abcde
1-3 b: cdefg
2-9 c: ccccccccc";

    #[rstest]
    #[case("1-3 a: abcde", true)]
    #[case("1-3 b: cdefg", false)]
    #[case("2-9 c: ccccccccc", true)]
    fn test_is_valid_by_count(#[case] input: &str, #[case] expected: bool) -> miette::Result<()> {
        assert_eq!(Policy::from_str(input)?.is_valid_by_count(), expected);
        Ok(())
    }

    #[rstest]
    #[case("1-3 a: abcde", true)]
    #[case("1-3 b: cdefg", false)]
    #[case("2-9 c: ccccccccc", false)]
    fn test_is_valid_by_position(
        #[case] input: &str,
        #[case] expected: bool,
    ) -> miette::Result<()> {
        assert_eq!(Policy::from_str(input)?.is_valid_by_position(), expected);
        Ok(())
    }

    #[test]
    fn test_part1() -> miette::Result<()> {
        assert_eq!("2", Day::from_str(EXAMPLE)?.solve(Part::One)?);
        Ok(())
    }

    #[test]
    fn test_part2() -> miette::Result<()> {
        assert_eq!("1", Day::from_str(EXAMPLE)?.solve(Part::Two)?);
        Ok(())
    }
}
