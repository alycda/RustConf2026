//! Day 2: Dive!
//!
//! Each line is a command: `forward X`, `down X`, or `up X`.
//!
//! --- Part One ---
//!
//! `forward` increases horizontal position; `down`/`up` increase/decrease
//! depth. Multiply final horizontal position by final depth.
//!
//! --- Part Two ---
//!
//! `down`/`up` instead adjust an aim. `forward X` increases horizontal
//! position by `X` and depth by `aim * X`.

use std::str::FromStr;

use aoc_ornaments::{Solution, SolutionResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Forward(i32),
    Down(i32),
    Up(i32),
}

impl FromStr for Command {
    type Err = miette::Error;

    fn from_str(line: &str) -> miette::Result<Self> {
        let (word, amount) = line
            .split_once(' ')
            .ok_or_else(|| miette::miette!("malformed command: {line}"))?;
        let amount: i32 = amount
            .parse()
            .map_err(|e| miette::miette!("bad amount in {line}: {e}"))?;

        match word {
            "forward" => Ok(Self::Forward(amount)),
            "down" => Ok(Self::Down(amount)),
            "up" => Ok(Self::Up(amount)),
            _ => Err(miette::miette!("unknown command: {word}")),
        }
    }
}

/// Submarine state: horizontal distance, depth, and (part two only) aim.
#[derive(Debug, Default)]
struct Position {
    horizontal: i32,
    depth: i32,
    aim: i32,
}

impl Position {
    fn apply(&mut self, command: &Command) {
        match *command {
            Command::Forward(x) => self.horizontal += x,
            Command::Down(x) => self.depth += x,
            Command::Up(x) => self.depth -= x,
        }
    }

    fn apply_with_aim(&mut self, command: &Command) {
        match *command {
            Command::Forward(x) => {
                self.horizontal += x;
                self.depth += self.aim * x;
            }
            Command::Down(x) => self.aim += x,
            Command::Up(x) => self.aim -= x,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Day(Vec<Command>);

/// Gives the parts `self.iter()` and the rest of `Vec`'s read API directly.
impl std::ops::Deref for Day {
    type Target = Vec<Command>;

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
                .map(Command::from_str)
                .collect::<miette::Result<_>>()?,
        ))
    }
}

impl Solution for Day {
    type Output = i32;

    /// Run every command through `Position::apply`, then multiply the two axes.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        let position = self
            .iter()
            .fold(Position::default(), |mut position, command| {
                position.apply(command);
                position
            });

        Ok(position.horizontal * position.depth)
    }

    /// Same, but commands are interpreted through the `aim`-tracking variant.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        let position = self
            .iter()
            .fold(Position::default(), |mut position, command| {
                position.apply_with_aim(command);
                position
            });

        Ok(position.horizontal * position.depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;
    use rstest::rstest;

    const EXAMPLE: &str = "forward 5
down 5
forward 8
up 3
down 8
forward 2";

    #[rstest]
    #[case("forward 5", Command::Forward(5))]
    #[case("down 5", Command::Down(5))]
    #[case("up 3", Command::Up(3))]
    fn test_command_parse(#[case] input: &str, #[case] expected: Command) -> miette::Result<()> {
        assert_eq!(Command::from_str(input)?, expected);
        Ok(())
    }

    #[test]
    fn test_part1() -> miette::Result<()> {
        assert_eq!("150", Day::from_str(EXAMPLE)?.solve(Part::One)?);
        Ok(())
    }

    #[test]
    fn test_part2() -> miette::Result<()> {
        assert_eq!("900", Day::from_str(EXAMPLE)?.solve(Part::Two)?);
        Ok(())
    }
}
