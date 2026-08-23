//! Day 1: Trebuchet?!
//!
//! Each line hides a two-digit calibration value: the first and last digit
//! in the line, combined (`a1b2c3d4e5f` -> `15`).
//!
//! --- Part One ---
//!
//! Digits are literal characters only.
//!
//! --- Part Two ---
//!
//! Digits may also be spelled out (`one` through `nine`), and spelled-out
//! digits can overlap (`eightwo` is `8` then `2`).

use std::str::FromStr;

use aoc_ornaments::{Solution, SolutionResult};

const WORDS: [(&str, u32); 9] = [
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
];

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

impl Day {
    /// The digit (or, with `words`, spelled-out digit) starting at `line[i..]`.
    fn digit_at(line: &str, i: usize, words: bool) -> Option<u32> {
        let rest = &line[i..];

        if let Some(d) = rest.chars().next().and_then(|c| c.to_digit(10)) {
            return Some(d);
        }

        if words {
            WORDS
                .iter()
                .find(|(word, _)| rest.starts_with(word))
                .map(|(_, value)| *value)
        } else {
            None
        }
    }

    /// First digit * 10 + last digit found in the line.
    fn calibration_value(line: &str, words: bool) -> u32 {
        let digits: Vec<u32> = (0..line.len())
            .filter_map(|i| Self::digit_at(line, i, words))
            .collect();

        digits.first().copied().unwrap_or(0) * 10 + digits.last().copied().unwrap_or(0)
    }

    fn compute(&self, words: bool) -> u32 {
        self.iter()
            .map(|line| Self::calibration_value(line, words))
            .sum()
    }
}

impl Solution for Day {
    type Output = u32;

    /// Sum the calibration values, digits only.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        Ok(self.compute(false))
    }

    /// Sum the calibration values, digits and spelled-out digits.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        Ok(self.compute(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;

    #[test]
    fn test_part1() -> miette::Result<()> {
        let input = "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet";
        assert_eq!("142", Day::from_str(input)?.solve(Part::One)?);
        Ok(())
    }

    #[test]
    fn test_part2() -> miette::Result<()> {
        let input = "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen";
        assert_eq!("281", Day::from_str(input)?.solve(Part::Two)?);
        Ok(())
    }
}
