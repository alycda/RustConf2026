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

#[cfg(feature = "duckdb")]
pub mod duckdb;

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

/// Runs the course in plain Rust and returns `horizontal * depth`.
///
/// Kept alongside [`dead_reckon_via_duckdb`] — its in-a-database equivalent —
/// rather than replaced by it, so `benches/dive.rs` can race the two and so a
/// regression in whichever one `cargo run` doesn't exercise still fails the
/// suite.
pub fn dead_reckon_pure_rust(commands: &[Command]) -> i32 {
    let position = commands
        .iter()
        .fold(Position::default(), |mut position, command| {
            position.apply(command);
            position
        });

    position.horizontal * position.depth
}

/// Part two's aim rules, in plain Rust. See [`dead_reckon_pure_rust`].
pub fn dead_reckon_with_aim_pure_rust(commands: &[Command]) -> i32 {
    let position = commands
        .iter()
        .fold(Position::default(), |mut position, command| {
            position.apply_with_aim(command);
            position
        });

    position.horizontal * position.depth
}

/// Narrows a DuckDB `BIGINT` answer to the puzzle's `i32`.
///
/// The database computes in 64 bits and cannot overflow at this scale, which
/// means the range check lands here rather than inside the query — the one
/// place the two number systems actually meet.
#[cfg(feature = "duckdb")]
fn narrow(product: i64) -> miette::Result<i32> {
    i32::try_from(product).map_err(|_| {
        miette::miette!("answer {product} does not fit in an i32 (DuckDB computed it in BIGINT)")
    })
}

/// Loads the course into an in-memory DuckDB and folds it with SQL.
///
/// Part one is two `SUM`s and a multiply. See [`duckdb`] for what the
/// scratchpad found about isolation, NULLs and the deprecated result API.
#[cfg(feature = "duckdb")]
pub fn dead_reckon_via_duckdb(commands: &[Command]) -> miette::Result<i32> {
    let course = duckdb::Course::load(commands)?;
    narrow(course.scalar(duckdb::PART1_SQL)?)
}

/// Part two through DuckDB — the half that earns the database.
///
/// `aim` is a running total, and a running total is a window function:
/// `SUM(...) OVER (ORDER BY idx)`. That is arguably a more direct statement of
/// the puzzle's rule than the fold in [`dead_reckon_with_aim_pure_rust`] is.
#[cfg(feature = "duckdb")]
pub fn dead_reckon_with_aim_via_duckdb(commands: &[Command]) -> miette::Result<i32> {
    let course = duckdb::Course::load(commands)?;
    narrow(course.scalar(duckdb::PART2_SQL)?)
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

    /// Multiply the two axes — in SQL when the `duckdb` feature is on (see
    /// [`dead_reckon_via_duckdb`]), in plain Rust otherwise.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        #[cfg(feature = "duckdb")]
        {
            dead_reckon_via_duckdb(&self.0)
        }
        #[cfg(not(feature = "duckdb"))]
        {
            Ok(dead_reckon_pure_rust(&self.0))
        }
    }

    /// Same, but commands are interpreted through the `aim`-tracking variant
    /// — same feature switch as `part1`.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        #[cfg(feature = "duckdb")]
        {
            dead_reckon_with_aim_via_duckdb(&self.0)
        }
        #[cfg(not(feature = "duckdb"))]
        {
            Ok(dead_reckon_with_aim_pure_rust(&self.0))
        }
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

    /// The pure-Rust functions, called directly rather than through
    /// `Solution` — which routes to whichever backend the feature set
    /// selected, and so cannot vouch for this one when `duckdb` is on.
    #[test]
    fn test_pure_rust_backend() -> miette::Result<()> {
        let day = Day::from_str(EXAMPLE)?;
        assert_eq!(dead_reckon_pure_rust(&day), 150);
        assert_eq!(dead_reckon_with_aim_pure_rust(&day), 900);
        Ok(())
    }

    /// The same two answers out of the database.
    #[cfg(feature = "duckdb")]
    #[test]
    fn test_duckdb_backend() -> miette::Result<()> {
        let day = Day::from_str(EXAMPLE)?;
        assert_eq!(dead_reckon_via_duckdb(&day)?, 150);
        assert_eq!(dead_reckon_with_aim_via_duckdb(&day)?, 900);
        Ok(())
    }

    /// An empty course is a real input shape — and the one that made the
    /// COALESCE necessary, since `SUM` over no rows is NULL.
    #[cfg(feature = "duckdb")]
    #[test]
    fn test_duckdb_empty_course() -> miette::Result<()> {
        assert_eq!(dead_reckon_via_duckdb(&[])?, 0);
        assert_eq!(dead_reckon_with_aim_via_duckdb(&[])?, 0);
        Ok(())
    }

    /// DuckDB answers in BIGINT, so a course that overflows the puzzle's i32
    /// is reported as an error by `narrow` rather than wrapping.
    #[cfg(feature = "duckdb")]
    #[test]
    fn test_duckdb_reports_overflow_rather_than_wrapping() {
        let course = [Command::Forward(100_000), Command::Down(100_000)];
        let error = dead_reckon_via_duckdb(&course).unwrap_err().to_string();
        assert!(error.contains("10000000000"), "unexpected error: {error}");
    }
}
