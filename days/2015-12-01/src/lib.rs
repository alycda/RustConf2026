//! Day 1: Not Quite Lisp

use std::str::FromStr;

use aoc_ornaments::Solution;

mod tcc;

/// A collection of instructions to move between floors.
#[derive(Debug, derive_more::Deref)]
pub struct Day(Vec<i32>);

impl FromStr for Day {
    type Err = miette::Error;

    /// Parse the input into a collection of instructions.
    ///
    /// ## Example
    ///
    /// - `(` moves Santa up one floor.
    /// - `)` moves Santa down one floor.
    ///
    fn from_str(input: &str) -> miette::Result<Self> {
        let parsed = input
            .chars()
            .map(|c| match c {
                '(' => 1,
                ')' => -1,
                _ => 0,
            })
            .collect();

        Ok(Self(parsed))
    }
}

/// Renders instructions as a C array literal, e.g. `1, -1, 1`.
fn floors_as_c_array(floors: &[i32]) -> String {
    floors
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Sums the instructions in plain Rust. Kept alongside [`sum_via_c`] — its
/// JIT-compiled-C equivalent — so `benches/sum.rs` can compare the two.
pub fn sum_pure_rust(floors: &[i32]) -> i32 {
    floors.iter().sum()
}

/// Sums the instructions by JIT-compiling a C function with libtcc and
/// calling it over FFI. See [`tcc`] and [`sum_pure_rust`].
pub fn sum_via_c(floors: &[i32]) -> miette::Result<i32> {
    let source = format!(
        "int solve(void) {{
            static const int floors[] = {{ {floors} }};
            int total = 0;
            for (unsigned i = 0; i < sizeof(floors) / sizeof(floors[0]); i++) {{
                total += floors[i];
            }}
            return total;
        }}",
        floors = floors_as_c_array(floors)
    );

    tcc::call_i32_fn(&source, "solve")
}

/// Finds the 1-based position of the first instruction that causes Santa to
/// enter the basement, in plain Rust. Kept alongside [`basement_position_via_c`]
/// so `benches/sum.rs` can compare the two.
pub fn basement_position_pure_rust(floors: &[i32]) -> Option<i32> {
    floors
        .iter()
        .scan(0, |floor, &x| {
            *floor += x;
            Some(*floor)
        })
        .position(|floor| floor < 0)
        .map(|pos| pos as i32 + 1)
}

/// Finds the basement-entering position via a JIT-compiled C function. See
/// [`tcc`] and [`basement_position_pure_rust`].
pub fn basement_position_via_c(floors: &[i32]) -> miette::Result<Option<i32>> {
    let source = format!(
        "int solve(void) {{
            static const int floors[] = {{ {floors} }};
            int total = 0;
            for (unsigned i = 0; i < sizeof(floors) / sizeof(floors[0]); i++) {{
                total += floors[i];
                if (total < 0) return (int) i + 1;
            }}
            return -1;
        }}",
        floors = floors_as_c_array(floors)
    );

    match tcc::call_i32_fn(&source, "solve")? {
        -1 => Ok(None),
        position => Ok(Some(position)),
    }
}

impl Solution for Day {
    type Output = i32;

    /// Find the floor Santa ends up on. See [`sum_via_c`].
    fn part1(&mut self) -> miette::Result<Self::Output> {
        sum_via_c(&self.0)
    }

    /// Find the position of the first instruction that causes Santa to enter
    /// the basement. See [`basement_position_via_c`].
    fn part2(&mut self) -> miette::Result<Self::Output> {
        basement_position_via_c(&self.0)?
            .ok_or_else(|| miette::miette!("Santa never enters the basement"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;
    use rstest::rstest;

    #[rstest]
    #[case("(())", 0)]
    #[case("()()", 0)]
    #[case("(((", 3)]
    #[case("(()(()(", 3)]
    #[case("))(((((", 3)]
    #[case("())", -1)]
    #[case("))(", -1)]
    #[case(")))", -3)]
    #[case(")())())", -3)]
    fn test_day1_part1(#[case] input: &str, #[case] expected: i32) -> miette::Result<()> {
        let mut day = Day::from_str(input)?;
        assert_eq!(day.solve(Part::One)?, expected.to_string());

        Ok(())
    }

    #[rstest]
    #[case(")", 1)]
    #[case("()())", 5)]
    fn test_day1_part2(#[case] input: &str, #[case] expected: i32) -> miette::Result<()> {
        let mut day = Day::from_str(input)?;
        assert_eq!(day.solve(Part::Two)?, expected.to_string());

        Ok(())
    }
}
