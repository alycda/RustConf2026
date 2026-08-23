//! Day 1: Historian Hysteria
//!
//! Two columns of location IDs, one pair per line.
//!
//! --- Part One ---
//!
//! Sort each column, pair the two columns off by rank, and sum the absolute
//! difference of every pair.
//!
//! --- Part Two ---
//!
//! Weight each left-hand ID by how many times it occurs in the right column,
//! and sum `id * occurrences`.

use std::collections::HashMap;
use std::str::FromStr;

use aoc_ornaments::{Solution, SolutionResult};
use itertools::Itertools;
use nom::{
    character::complete::{self, digit1, line_ending, space1},
    multi::separated_list1,
    sequence::separated_pair,
};

pub use crate::Day1 as Day;

/// Solution for comparing and matching numbers between two lists
///
/// This implementation solves a puzzle where:
/// 1. Numbers from two lists need to be paired by their sorted positions
/// 2. The absolute difference between each pair is calculated
/// 3. All differences are summed to produce a final result
///
/// The secondary part handles counting matching numbers between lists
#[derive(Debug, Clone)]
pub struct Day1(Vec<i32>, Vec<i32>);

impl FromStr for Day1 {
    type Err = miette::Error;

    /// Parses input string into two sorted vectors of integers
    ///
    /// # Arguments
    /// * `input` - String containing pairs of numbers separated by whitespace
    ///
    /// # Returns
    /// * `Self` - Day1 struct containing two sorted vectors
    ///
    /// # Panics
    /// * If any line doesn't contain exactly two numbers
    /// * If any number cannot be parsed as i32
    fn from_str(input: &str) -> miette::Result<Self> {
        let (mut left, mut right): (Vec<i32>, Vec<i32>) = input
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .map(|x| x.parse::<i32>().expect("a valid number"))
                    .collect_tuple()
                    .expect("Each line must have exactly two numbers")
            })
            .unzip();

        left.sort();
        right.sort();

        Ok(Self(left, right))
    }
}

impl Day1 {
    /// Nom parser implementation for handling input parsing with error handling
    ///
    /// Parses lines of space-separated integer pairs using nom combinators
    pub fn nom_parser(input: &str) -> nom::IResult<&str, Vec<(i32, i32)>, nom::error::Error<&str>> {
        separated_list1(
            line_ending::<&str, nom::error::Error<&str>>,
            separated_pair(complete::i32, space1, complete::i32),
        )(input)
    }
}

impl Solution for Day1 {
    type Output = i32;

    /// Calculates sum of absolute differences between paired numbers
    ///
    /// Pairs are formed by matching indices in the sorted vectors
    ///
    /// # Returns
    /// * Sum of absolute differences or error
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        let Day1(left, right) = self;

        let output = left
            .iter()
            .zip(right.iter())
            .map(|(l, r)| (l - r).abs())
            .sum::<Self::Output>();

        Ok(output)
    }

    /// Calculates sum of products between numbers and their frequency matches
    ///
    /// For each number in left vector, multiply it by how many times it appears
    /// in the right vector
    ///
    /// # Returns
    /// * Sum of products or error
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        let Day1(left, right) = self;

        let output = left
            .iter()
            .map(|n| n * right.iter().filter(|&x| x == n).count() as Self::Output)
            .sum::<Self::Output>();

        Ok(output)
    }
}

#[derive(Debug, Clone)]
pub struct Day1Hashmap(Vec<usize>, HashMap<usize, usize>);

impl FromStr for Day1Hashmap {
    type Err = miette::Error;

    fn from_str(input: &str) -> miette::Result<Self> {
        let mut left = vec![];
        let mut right: HashMap<usize, usize> = HashMap::new();

        for line in input.lines() {
            let mut items = line.split_whitespace();
            left.push(items.next().unwrap().parse::<usize>().unwrap());
            right
                .entry(items.next().unwrap().parse::<usize>().unwrap())
                .and_modify(|v| {
                    *v += 1;
                })
                .or_insert(1);
        }

        Ok(Self(left, right))
    }
}

impl Day1Hashmap {
    /// NOTE: unfinished upstream — the frequency map is built but never
    /// returned. Left as-is; finish it and return `Ok((input, map))`.
    pub fn nom_parser(
        input: &str,
    ) -> nom::IResult<&str, HashMap<usize, usize>, nom::error::Error<&str>> {
        let mut map = HashMap::new();

        let (_input, pairs) = separated_list1(
            line_ending::<&str, nom::error::Error<&str>>,
            separated_pair(digit1, space1, digit1),
        )(input)?;

        for (left, _right) in pairs {
            map.entry(left)
                .and_modify(|v| {
                    *v += 1;
                })
                .or_insert(1);
        }

        todo!();

        // Ok((input, map))
    }
}

impl Solution for Day1Hashmap {
    type Output = usize;

    fn part1(&mut self) -> SolutionResult<Self::Output> {
        unimplemented!("Part 1 not implemented for Day1Hashmap")
    }

    // O(n) with constant time lookups using HashMap
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        let Day1Hashmap(left, right) = self;

        let result: usize = left
            .iter()
            .map(|number| number * right.get(number).unwrap_or(&0))
            .sum();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use aoc_ornaments::Part;

    #[test]
    fn test_day1_part1() -> miette::Result<()> {
        let input = "3   4
    4   3
    2   5
    1   3
    3   9
    3   3";
        assert_eq!("11", Day1::from_str(input)?.solve(Part::One)?);
        Ok(())
    }

    #[test]
    fn day1_nom_parser() {
        let input = "3   4";
        let result = Day1::nom_parser(input);
        assert_eq!(Ok(("", vec![(3, 4)])), result);
    }

    #[test]
    fn test_day1_part2() -> miette::Result<()> {
        let input = "3   4
    4   3
    2   5
    1   3
    3   9
    3   3";
        assert_eq!("31", Day1::from_str(input)?.solve(Part::Two)?);
        Ok(())
    }

    #[test]
    fn test_day1_part2_hashmap() -> miette::Result<()> {
        let input = "3   4
    4   3
    2   5
    1   3
    3   9
    3   3";
        assert_eq!("31", Day1Hashmap::from_str(input)?.solve(Part::Two)?);
        Ok(())
    }
}
