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

impl Day {
    /// Renders the instructions as a C array literal, e.g. `1, -1, 1`.
    fn floors_as_c_array(&self) -> String {
        self.iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Solution for Day {
    type Output = i32;

    /// Find the floor Santa ends up on, by JIT-compiling a C function that
    /// sums the instructions and calling it over FFI. See [`tcc`].
    fn part1(&mut self) -> miette::Result<Self::Output> {
        let source = format!(
            "int solve(void) {{
                static const int floors[] = {{ {floors} }};
                int total = 0;
                for (unsigned i = 0; i < sizeof(floors) / sizeof(floors[0]); i++) {{
                    total += floors[i];
                }}
                return total;
            }}",
            floors = self.floors_as_c_array()
        );

        tcc::call_i32_fn(&source, "solve")
    }

    /// Find the position of the first instruction that causes Santa to enter
    /// the basement, again via a JIT-compiled C function. See [`tcc`].
    fn part2(&mut self) -> miette::Result<Self::Output> {
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
            floors = self.floors_as_c_array()
        );

        match tcc::call_i32_fn(&source, "solve")? {
            -1 => Err(miette::miette!("Santa never enters the basement")),
            position => Ok(position),
        }
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
