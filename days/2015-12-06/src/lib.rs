//! Advent of Code 2015, day 6: Probably a Fire Hazard.
//!
//! A 1000×1000 grid of lights, all off, and a list of instructions over
//! inclusive rectangles: `turn on 0,0 through 999,999`, `toggle 0,0 through
//! 999,0`, `turn off 499,499 through 500,500`. Part 1 reads the words as
//! switches and counts the lights left on. Part 2 reads the same words as
//! brightness: `turn on` adds one, `turn off` subtracts one but never below
//! zero, `toggle` adds two — and sums the brightness of every light.
//!
//! Same input, same rectangles, two meanings. The instruction list is what
//! crosses the boundary in Exercise 2; the grid never leaves this crate.

use std::str::FromStr;

use aoc_ornaments::{Solution, SolutionResult};

/// Lights per side. Coordinates run 0..=999 in both axes.
pub const GRID: usize = 1000;

/// The verb of one instruction. What it does to a light depends on the part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    TurnOn,
    TurnOff,
    Toggle,
}

/// One line of input: a verb and an inclusive rectangle, `from` at or before
/// `to` on both axes, every coordinate on the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub action: Action,
    pub from: (usize, usize),
    pub to: (usize, usize),
}

impl FromStr for Instruction {
    type Err = miette::Error;

    fn from_str(line: &str) -> miette::Result<Self> {
        let (action, rest) = if let Some(rest) = line.strip_prefix("turn on ") {
            (Action::TurnOn, rest)
        } else if let Some(rest) = line.strip_prefix("turn off ") {
            (Action::TurnOff, rest)
        } else if let Some(rest) = line.strip_prefix("toggle ") {
            (Action::Toggle, rest)
        } else {
            return Err(miette::miette!("unknown instruction: {line:?}"));
        };
        let (from, to) = rest
            .split_once(" through ")
            .ok_or_else(|| miette::miette!("no `through` in {line:?}"))?;
        let from = corner(from, line)?;
        let to = corner(to, line)?;
        if from.0 > to.0 || from.1 > to.1 {
            return Err(miette::miette!("corners out of order in {line:?}"));
        }
        Ok(Self { action, from, to })
    }
}

/// `x,y`, both on the grid.
fn corner(text: &str, line: &str) -> miette::Result<(usize, usize)> {
    let (x, y) = text
        .split_once(',')
        .ok_or_else(|| miette::miette!("bad corner {text:?} in {line:?}"))?;
    let coordinate = |value: &str| -> miette::Result<usize> {
        let n: usize = value
            .parse()
            .map_err(|e| miette::miette!("bad coordinate {value:?} in {line:?}: {e}"))?;
        if n >= GRID {
            return Err(miette::miette!(
                "coordinate {n} is off the {GRID}×{GRID} grid in {line:?}"
            ));
        }
        Ok(n)
    };
    Ok((coordinate(x)?, coordinate(y)?))
}

/// The parsed instruction list, in input order.
#[derive(Debug, Clone)]
pub struct Day(Vec<Instruction>);

/// Gives the parts `self.iter()` and the rest of `Vec`'s read API directly.
impl std::ops::Deref for Day {
    type Target = Vec<Instruction>;

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
                .filter(|line| !line.is_empty())
                .map(Instruction::from_str)
                .collect::<miette::Result<_>>()?,
        ))
    }
}

/// Part 1's reading of a verb: a light is `0` or `1`, and `toggle` flips it.
pub fn switch(action: Action, light: u32) -> u32 {
    match action {
        Action::TurnOn => 1,
        Action::TurnOff => 0,
        Action::Toggle => light ^ 1,
    }
}

/// Part 2's reading of the same verb: brightness, floored at zero.
pub fn dim(action: Action, light: u32) -> u32 {
    match action {
        Action::TurnOn => light + 1,
        Action::TurnOff => light.saturating_sub(1),
        Action::Toggle => light + 2,
    }
}

impl Day {
    /// Applies every instruction to a grid that starts dark, `rule` deciding
    /// what each verb does to one light, and returns the grid's total.
    ///
    /// Row-major, and each rectangle is walked row by row so the inner loop
    /// is one contiguous slice — 1000×1000 cells times a few hundred
    /// instructions is small enough that this is all the cleverness it needs.
    pub fn run(&self, rule: fn(Action, u32) -> u32) -> u64 {
        let mut grid = vec![0u32; GRID * GRID];
        for instruction in self.iter() {
            for y in instruction.from.1..=instruction.to.1 {
                let row = &mut grid[y * GRID..(y + 1) * GRID];
                for light in &mut row[instruction.from.0..=instruction.to.0] {
                    *light = rule(instruction.action, *light);
                }
            }
        }
        grid.iter().map(|&light| u64::from(light)).sum()
    }
}

impl Solution for Day {
    /// Part 1 is at most 1,000,000; part 2's real answers are in the tens of
    /// millions. Both fit, and `u32` is what the C API hands across.
    type Output = u32;

    fn part1(&mut self) -> SolutionResult<Self::Output> {
        u32::try_from(self.run(switch))
            .map_err(|_| miette::miette!("lit count does not fit in a u32"))
    }

    fn part2(&mut self) -> SolutionResult<Self::Output> {
        u32::try_from(self.run(dim))
            .map_err(|_| miette::miette!("total brightness does not fit in a u32"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;
    use rstest::rstest;

    #[rstest]
    #[case("turn on 0,0 through 999,999", Action::TurnOn, (0, 0), (999, 999))]
    #[case("toggle 0,0 through 999,0", Action::Toggle, (0, 0), (999, 0))]
    #[case("turn off 499,499 through 500,500", Action::TurnOff, (499, 499), (500, 500))]
    fn parses_the_statement_lines(
        #[case] line: &str,
        #[case] action: Action,
        #[case] from: (usize, usize),
        #[case] to: (usize, usize),
    ) -> miette::Result<()> {
        assert_eq!(
            Instruction::from_str(line)?,
            Instruction { action, from, to }
        );
        Ok(())
    }

    #[rstest]
    #[case("flip 0,0 through 1,1")]
    #[case("turn on 0,0 to 1,1")]
    #[case("turn on 1000,0 through 1,1")]
    #[case("toggle 5,5 through 4,4")]
    #[case("turn off 0,x through 1,1")]
    fn refuses_what_the_grid_cannot_hold(#[case] line: &str) {
        assert!(Instruction::from_str(line).is_err(), "{line:?} parsed");
    }

    /// Each statement example as the standalone effect the text describes,
    /// on a dark grid.
    #[rstest]
    #[case("turn on 0,0 through 999,999", "1000000")]
    #[case("toggle 0,0 through 999,0", "1000")]
    #[case("turn off 499,499 through 500,500", "0")]
    fn test_part1(#[case] input: &str, #[case] expected: &str) -> miette::Result<()> {
        assert_eq!(expected, Day::from_str(input)?.solve(Part::One)?);
        Ok(())
    }

    /// The three in order: everything on, the first row toggled off, the
    /// middle four off.
    #[test]
    fn test_part1_sequence() -> miette::Result<()> {
        let input = "turn on 0,0 through 999,999
toggle 0,0 through 999,0
turn off 499,499 through 500,500";
        assert_eq!("998996", Day::from_str(input)?.solve(Part::One)?);
        Ok(())
    }

    #[rstest]
    #[case("turn on 0,0 through 0,0", "1")]
    #[case("toggle 0,0 through 999,999", "2000000")]
    fn test_part2(#[case] input: &str, #[case] expected: &str) -> miette::Result<()> {
        assert_eq!(expected, Day::from_str(input)?.solve(Part::Two)?);
        Ok(())
    }

    /// `turn off` never goes below zero — the floor is the rule, not an
    /// accident of unsigned arithmetic.
    #[test]
    fn brightness_floors_at_zero() -> miette::Result<()> {
        let input = "turn off 0,0 through 9,9
turn on 0,0 through 0,0";
        assert_eq!("1", Day::from_str(input)?.solve(Part::Two)?);
        Ok(())
    }
}
