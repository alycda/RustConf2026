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

pub mod c_api;
#[cfg(feature = "chipmunk")]
pub mod chipmunk;
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
/// Kept alongside [`dead_reckon_via_chipmunk`] and [`dead_reckon_via_duckdb`]
/// — its physics-engine and its in-a-database equivalents — rather than
/// replaced by either, so `benches/dive.rs` can race all three and so a
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

/// Runs the course through Chipmunk2D instead: one rigid body, one
/// `cpSpaceStep` per command, the answer read back off the body's position.
///
/// The submarine only ever translates here — part one has no aim — so every
/// command becomes a velocity for one unit of time and the body's rotation
/// stays at zero. See [`chipmunk`] for why that is exact.
#[cfg(feature = "chipmunk")]
pub fn dead_reckon_via_chipmunk(commands: &[Command]) -> miette::Result<i32> {
    let mut submarine = chipmunk::Submarine::new()?;

    for command in commands {
        let velocity = match *command {
            Command::Forward(x) => (f64::from(x), 0.0),
            Command::Down(x) => (0.0, f64::from(x)),
            Command::Up(x) => (0.0, -f64::from(x)),
        };
        submarine.step(velocity, 0.0);
    }

    submarine.answer()
}

/// Part two through Chipmunk2D — the variant that actually earns the engine.
///
/// `aim` is not tracked in Rust at all: `down`/`up` set an *angular* velocity
/// and let the solver integrate it into the body's rotation, and `forward`
/// reads that rotation back to build its velocity. The puzzle's own
/// hand-wave (depth grows by `aim * x`, not `sin(aim) * x`) is why the
/// rotation is used as a raw scalar rather than an angle — the engine is
/// storing the number, not interpreting it.
#[cfg(feature = "chipmunk")]
pub fn dead_reckon_with_aim_via_chipmunk(commands: &[Command]) -> miette::Result<i32> {
    let mut submarine = chipmunk::Submarine::new()?;

    for command in commands {
        match *command {
            Command::Forward(x) => {
                let x = f64::from(x);
                // Read before the step: `aim` is whatever the solver has
                // integrated so far, exactly as the puzzle intends.
                submarine.step((x, submarine.aim() * x), 0.0);
            }
            Command::Down(x) => submarine.step((0.0, 0.0), f64::from(x)),
            Command::Up(x) => submarine.step((0.0, 0.0), -f64::from(x)),
        }
    }

    submarine.answer()
}

/// Narrows a DuckDB `BIGINT` answer to the puzzle's `i32`.
///
/// The database computes in 64 bits and cannot overflow at this scale, which
/// means the range check lands here rather than inside the query — the one
/// place the two number systems actually meet. Note the contrast with the
/// Chipmunk side, which computes in `f64` and checks integrality as well as
/// range: three backends, three different ways for the puzzle's `i32` to be
/// the wrong type.
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

    /// Multiply the two axes, through whichever backend is compiled in.
    ///
    /// With both features on, Chipmunk wins — deliberately, and it is the only
    /// place either variant is favoured. Two reasons: the rigid body models
    /// the puzzle's own subject (a thing with a position and an orientation),
    /// and it costs ~45 µs against DuckDB's ~4.6 ms, of which 95% is standing
    /// a database up before any question is asked. DuckDB's window function is
    /// the more elegant *statement* of part two's rule; it is not the one to
    /// pay for on every `cargo run`. Both keep their own `pub fn`s and their
    /// own tests, so the backend this doesn't route to is still verified.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        #[cfg(feature = "chipmunk")]
        {
            dead_reckon_via_chipmunk(&self.0)
        }
        #[cfg(all(feature = "duckdb", not(feature = "chipmunk")))]
        {
            dead_reckon_via_duckdb(&self.0)
        }
        #[cfg(not(any(feature = "chipmunk", feature = "duckdb")))]
        {
            Ok(dead_reckon_pure_rust(&self.0))
        }
    }

    /// Same, but commands are interpreted through the `aim`-tracking variant
    /// — same backend precedence as `part1`.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        #[cfg(feature = "chipmunk")]
        {
            dead_reckon_with_aim_via_chipmunk(&self.0)
        }
        #[cfg(all(feature = "duckdb", not(feature = "chipmunk")))]
        {
            dead_reckon_with_aim_via_duckdb(&self.0)
        }
        #[cfg(not(any(feature = "chipmunk", feature = "duckdb")))]
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
    /// selected, and so cannot vouch for this one when `chipmunk` is on.
    #[test]
    fn test_pure_rust_backend() -> miette::Result<()> {
        let day = Day::from_str(EXAMPLE)?;
        assert_eq!(dead_reckon_pure_rust(&day), 150);
        assert_eq!(dead_reckon_with_aim_pure_rust(&day), 900);
        Ok(())
    }

    /// The same two answers out of the physics engine. Not "close to": the
    /// integrator is exact under this module's conditions, so an approximate
    /// comparison here would hide the day the conditions stop holding.
    #[cfg(feature = "chipmunk")]
    #[test]
    fn test_chipmunk_backend() -> miette::Result<()> {
        let day = Day::from_str(EXAMPLE)?;
        assert_eq!(dead_reckon_via_chipmunk(&day)?, 150);
        assert_eq!(dead_reckon_with_aim_via_chipmunk(&day)?, 900);
        Ok(())
    }

    /// Each `Submarine` gets its own `cpSpace`, so two courses run back to
    /// back must not see each other's state — the failure this catches is a
    /// body or space accidentally shared or reused across calls.
    #[cfg(feature = "chipmunk")]
    #[test]
    fn test_chipmunk_runs_are_independent() -> miette::Result<()> {
        let day = Day::from_str(EXAMPLE)?;
        assert_eq!(dead_reckon_with_aim_via_chipmunk(&day)?, 900);
        assert_eq!(dead_reckon_with_aim_via_chipmunk(&day)?, 900);
        assert_eq!(dead_reckon_via_chipmunk(&day)?, 150);
        Ok(())
    }

    /// An empty course is a real input shape (a `Day` parsed from `""` has no
    /// commands): the engine must hand back 0 * 0 rather than tripping the
    /// integral/range checks in `Submarine::answer`.
    #[cfg(feature = "chipmunk")]
    #[test]
    fn test_chipmunk_empty_course() -> miette::Result<()> {
        assert_eq!(dead_reckon_via_chipmunk(&[])?, 0);
        assert_eq!(dead_reckon_with_aim_via_chipmunk(&[])?, 0);
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

    /// An empty course is a real input shape — and the one that made DuckDB's
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

    /// With both features compiled in, the two C-backed variants must agree
    /// with each other and with plain Rust. Nothing else in the suite compares
    /// them directly — each backend's own tests only check it against the
    /// puzzle's known answers.
    #[cfg(all(feature = "chipmunk", feature = "duckdb"))]
    #[test]
    fn test_all_three_backends_agree() -> miette::Result<()> {
        let day = Day::from_str(EXAMPLE)?;
        assert_eq!(dead_reckon_pure_rust(&day), dead_reckon_via_chipmunk(&day)?);
        assert_eq!(dead_reckon_pure_rust(&day), dead_reckon_via_duckdb(&day)?);
        assert_eq!(
            dead_reckon_with_aim_pure_rust(&day),
            dead_reckon_with_aim_via_chipmunk(&day)?
        );
        assert_eq!(
            dead_reckon_with_aim_pure_rust(&day),
            dead_reckon_with_aim_via_duckdb(&day)?
        );
        Ok(())
    }
}
