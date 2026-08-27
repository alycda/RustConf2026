//! Day 3: Mull It Over
//!
//! Multiplication instructions buried in corrupted text, some of them
//! switched off.
//!
//! --- Part One ---
//!
//! Find every well-formed `mul(a,b)`, ignore the corruption around it, and
//! sum the products.
//!
//! --- Part Two ---
//!
//! `do()` and `don't()` toggle whether a `mul` counts; the most recent
//! toggle wins, and multiplication starts enabled.

use std::{marker::PhantomData, num::ParseIntError, str::FromStr};

use aoc_ornaments::{Solution, SolutionResult};
use nom::{
    IResult,
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1},
    error::ErrorKind,
    sequence::{preceded, terminated, tuple},
};
use nom_locate::LocatedSpan;

pub mod cursor;

type Span<'a> = LocatedSpan<&'a str>;

pub use crate::Day3 as Day;

#[derive(Debug)]
pub struct Part1;

#[derive(Debug)]
pub struct Part2;

#[derive(Debug)]
pub struct Day3<P>(Vec<Product>, PhantomData<P>);

impl<P> std::ops::Deref for Day3<P> {
    type Target = Vec<Product>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a multiplication operation with two operands
#[derive(Debug, Clone, Copy)]
pub struct Product(usize, usize);

impl FromStr for Product {
    type Err = ParseIntError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (a, b) = input.split_once(",").expect("valid input");

        Ok(Product(a.parse()?, b.parse()?))
    }
}

impl Product {
    /// Creates a new Product from string representations of numbers
    ///
    /// # Panics
    /// Panics if either string cannot be parsed as usize
    pub fn new(a: &str, b: &str) -> Self {
        Self(a.parse().expect("a number"), b.parse().expect("a number"))
    }

    /// Computes the product of the two numbers
    pub fn value(&self) -> usize {
        self.0 * self.1
    }
}

impl<P> Day3<P> {
    /// Parses multiplication expressions in the EXACT format "mul(x,y)"
    fn parse_mul(input: &str) -> IResult<&str, (&str, &str)> {
        let (input, _trash) = take_until("mul(")(input)?;

        preceded(
            tag("mul("),
            terminated(
                tuple((
                    digit1,
                    // consume the comma
                    preceded(char(','), digit1),
                )),
                char(')'),
            ),
        )(input)
    }

    fn parse_all_mul(mut input: &str) -> IResult<&str, Vec<Product>> {
        let mut products = Vec::new();

        while !input.is_empty() {
            match Self::parse_mul(input) {
                Ok((remainder, product)) => {
                    products.push(Product::new(product.0, product.1));
                    input = remainder;
                }
                // Any recoverable parse error means the bytes at the cursor are
                // corruption, not a mul() — skip one byte and rescan. digit1
                // failing after `mul(` (ErrorKind::Digit, e.g. `mul(x,4)` or a
                // truncated `mul(2,` at a fragment end) lands here too; the
                // old catch-all panicked on exactly that input.
                Err(nom::Err::Error(err)) => match err.code {
                    ErrorKind::TakeUntil => {
                        input = "";
                    }
                    _ if input.len() > 1 => {
                        input = &input[1..];
                    }
                    _ => {
                        input = "";
                    }
                },
                Err(e) => {
                    dbg!(e);
                    break;
                }
            }
        }

        Ok((input, products))
    }
}

impl FromStr for Day3<Part1> {
    type Err = miette::Error;

    fn from_str(input: &str) -> miette::Result<Self> {
        let (_, products) = Day3::<Part1>::parse_all_mul(input).unwrap();

        Ok(Day3(products, PhantomData))
    }
}

impl Solution for Day3<Part1> {
    type Output = usize;

    /// sums all products
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        let output: Self::Output = self.iter().map(|p| p.value()).sum();

        Ok(output)
    }

    /// sums all products
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        unimplemented!("Part 2")
    }
}

impl FromStr for Day3<Part2> {
    type Err = miette::Error;

    fn from_str(input: &str) -> miette::Result<Self> {
        let input = Span::new(input);
        let mut products = Vec::new();

        // Get everything before first don't()
        let (mut current, _initial) =
            match take_until::<_, _, nom::error::Error<Span>>("don't()")(input) {
                Ok((remainder, initial)) => {
                    let (_, initial_products) =
                        Day3::<Part2>::parse_all_mul(initial.fragment()).unwrap();
                    products.extend(initial_products);
                    (remainder, initial)
                }
                Err(_) => {
                    let (_, products) = Day3::<Part2>::parse_all_mul(input.fragment()).unwrap();
                    return Ok(Day3(products, PhantomData));
                }
            };

        while !current.is_empty() {
            // Skip don't()
            let (after_dont, _) = tag::<_, _, nom::error::Error<Span>>("don't()")(current).unwrap();

            // Find next do()
            match take_until::<_, _, nom::error::Error<Span>>("do()")(after_dont) {
                Ok((after_do, _disabled_section)) => {
                    // Skip do()
                    let (remainder, _) =
                        tag::<_, _, nom::error::Error<Span>>("do()")(after_do).unwrap();

                    // Process enabled section until next don't()
                    match take_until::<_, _, nom::error::Error<Span>>("don't()")(remainder) {
                        Ok((next_dont, enabled)) => {
                            let (_, new_products) =
                                Day3::<Part2>::parse_all_mul(enabled.fragment()).unwrap();
                            products.extend(new_products);
                            current = next_dont;
                        }
                        Err(_) => {
                            // Process until end
                            let (_, new_products) =
                                Day3::<Part2>::parse_all_mul(remainder.fragment()).unwrap();
                            products.extend(new_products);
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }

        Ok(Day3(products, PhantomData))
    }
}

impl Solution for Day3<Part2> {
    type Output = usize;

    /// sums all products
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        unimplemented!("Part 1")
    }

    /// sums all products
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        let output: Self::Output = self.iter().map(|p| p.value()).sum();

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;
    use rstest::rstest;

    /// Part one's statement example. It holds no `don't()`, so part 2
    /// scores it identically — the tie is a property of the example, and
    /// the grid below says so out loud instead of leaving it to be
    /// rediscovered.
    const EXAMPLE_1: &str =
        "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
    /// Part two's statement example — the one whose two answers differ
    /// (161/48), which is what makes it the track input of choice.
    const EXAMPLE_2: &str =
        "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

    /// Both parts over both examples, both parsers — so a track's CI cell
    /// can hand either example verbatim and assert only numbers this suite
    /// pins, instead of reshaping an input until the figures come out
    /// conveniently (the 2023-12-01 lesson).
    #[rstest]
    #[case::example1(EXAMPLE_1, "161")]
    #[case::example2(EXAMPLE_2, "161")]
    fn test_part1_over_both_examples(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, Day3::<Part1>::from_str(input)?.solve(Part::One)?);
        assert_eq!(expected, cursor::part1(input).to_string());
        Ok(())
    }

    #[rstest]
    #[case::example1(EXAMPLE_1, "161")]
    #[case::example2(EXAMPLE_2, "48")]
    fn test_part2_over_both_examples(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, Day3::<Part2>::from_str(input)?.solve(Part::Two)?);
        assert_eq!(expected, cursor::part2(input).to_string());
        Ok(())
    }

    #[test]
    fn test_part1() -> miette::Result<()> {
        let input = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
        assert_eq!("161", Day3::<Part1>::from_str(input)?.solve(Part::One)?);
        Ok(())
    }

    /// The inputs that used to reach the panic arm: a non-digit right
    /// after `mul(` fails digit1 with ErrorKind::Digit, which the old
    /// catch-all turned into a crash instead of skipping as corruption.
    #[test]
    fn test_part1_corruption_is_skipped_not_panicked() -> miette::Result<()> {
        assert_eq!(
            "8",
            Day3::<Part1>::from_str("mul(x,4)mul(2,4)")?.solve(Part::One)?
        );
        assert_eq!(
            "8",
            Day3::<Part1>::from_str("mul(mul(2,4)")?.solve(Part::One)?
        );
        assert_eq!(
            "8",
            Day3::<Part1>::from_str("mul(2,4)mul(2,")?.solve(Part::One)?
        );
        Ok(())
    }

    #[test]
    fn test_part2() -> miette::Result<()> {
        let input = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";
        assert_eq!("48", Day3::<Part2>::from_str(input)?.solve(Part::Two)?);
        Ok(())
    }
}
