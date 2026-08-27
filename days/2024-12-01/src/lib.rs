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

pub mod c_api;
#[cfg(feature = "cpp")]
mod cpp;
#[cfg(feature = "qsort")]
mod qsort;
#[cfg(feature = "uthash")]
mod uthash;

pub use crate::Day1 as Day;

/// Sorts one column in plain Rust. Kept alongside [`sort_via_qsort`] and
/// [`sort_via_cpp`] — its libc and C++ equivalents — so a benchmark can
/// compare the backends.
pub fn sort_pure_rust(column: &mut [i32]) {
    column.sort();
}

/// Sorts one column by handing its backing storage to libc's `qsort` over
/// FFI. See [`qsort`] and [`sort_pure_rust`].
#[cfg(feature = "qsort")]
pub fn sort_via_qsort(column: &mut [i32]) {
    qsort::sort(column);
}

/// Sorts one column with C++'s `std::sort` through a C-shaped shim.
/// See [`cpp`] and [`sort_pure_rust`].
#[cfg(feature = "cpp")]
pub fn sort_via_cpp(column: &mut [i32]) {
    cpp::sort(column);
}

/// Part 2's similarity score as the baseline computes it — the naive scan,
/// O(left × right). Kept alongside [`similarity_via_uthash`] — its C hash
/// table equivalent — so a benchmark can compare the two.
pub fn similarity_pure_rust(left: &[i32], right: &[i32]) -> i32 {
    left.iter()
        .map(|n| n * right.iter().filter(|&x| x == n).count() as i32)
        .sum()
}

/// Part 2's similarity score with the frequency map built and queried in C.
/// See [`uthash`] and [`similarity_pure_rust`].
#[cfg(feature = "uthash")]
pub fn similarity_via_uthash(left: &[i32], right: &[i32]) -> i32 {
    uthash::similarity(left, right)
}

/// A sorting backend, chosen at compile time — the talk's
/// `2024-12-01-zero-cost-abstraction` branch, ported.
///
/// This is a zero-cost abstraction because:
/// - the marker types are zero-sized — no runtime memory,
/// - monomorphization creates a specialized copy of every generic function
///   per backend at compile time,
/// - there are no vtables and no dynamic dispatch — everything resolves
///   statically, as if the per-backend functions were written by hand.
pub trait Sorter {
    fn sort(column: &mut [i32]);
}

/// Marker type for Rust's built-in sort.
pub struct NativeSort;

impl Sorter for NativeSort {
    fn sort(column: &mut [i32]) {
        sort_pure_rust(column);
    }
}

/// Marker type for libc's `qsort`.
#[cfg(feature = "qsort")]
pub struct CSort;

#[cfg(feature = "qsort")]
impl Sorter for CSort {
    fn sort(column: &mut [i32]) {
        sort_via_qsort(column);
    }
}

/// Marker type for C++'s `std::sort`.
#[cfg(feature = "cpp")]
pub struct CppSort;

#[cfg(feature = "cpp")]
impl Sorter for CppSort {
    fn sort(column: &mut [i32]) {
        sort_via_cpp(column);
    }
}

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
        // Rank-pairing needs both columns sorted. Backend precedence when
        // several features are on: qsort (the talk's headline crossing),
        // then the C++ shim, then plain Rust — every backend stays
        // reachable by name via parse_with either way.
        #[cfg(feature = "qsort")]
        {
            Self::parse_with::<CSort>(input)
        }
        #[cfg(all(feature = "cpp", not(feature = "qsort")))]
        {
            Self::parse_with::<CppSort>(input)
        }
        #[cfg(not(any(feature = "qsort", feature = "cpp")))]
        {
            Self::parse_with::<NativeSort>(input)
        }
    }
}

impl Day1 {
    /// Parses the two columns and rank-sorts them through the chosen
    /// backend. Monomorphized per `S`: `parse_with::<NativeSort>` and
    /// `parse_with::<CSort>` compile to separate specialized functions,
    /// so picking a backend costs nothing at runtime.
    pub fn parse_with<S: Sorter>(input: &str) -> miette::Result<Self> {
        let (mut left, mut right): (Vec<i32>, Vec<i32>) = input
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .map(|x| x.parse::<i32>().expect("a valid number"))
                    .collect_tuple()
                    .expect("Each line must have exactly two numbers")
            })
            .unzip();

        S::sort(&mut left);
        S::sort(&mut right);

        Ok(Self(left, right))
    }

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

        // The occurrence counting goes through the C hash table when the
        // `uthash` feature is on (see [`similarity_via_uthash`]), and stays
        // the naive scan otherwise.
        #[cfg(feature = "uthash")]
        {
            Ok(similarity_via_uthash(left, right))
        }
        #[cfg(not(feature = "uthash"))]
        {
            Ok(similarity_pure_rust(left, right))
        }
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

    /// The explicit monomorphization the default build exercises implicitly
    /// through `FromStr` — spelling the backend out at the call site.
    #[test]
    fn test_day1_part1_native_sort() -> miette::Result<()> {
        let input = "3   4
    4   3
    2   5
    1   3
    3   9
    3   3";
        assert_eq!(
            "11",
            Day1::parse_with::<NativeSort>(input)?.solve(Part::One)?
        );
        Ok(())
    }

    /// Both monomorphized parses must build the same ranked columns.
    #[cfg(feature = "qsort")]
    #[test]
    fn test_day1_parse_backends_agree() -> miette::Result<()> {
        let input = "3   4
    4   3
    2   5
    1   3
    3   9
    3   3";
        let mut native = Day1::parse_with::<NativeSort>(input)?;
        let mut c = Day1::parse_with::<CSort>(input)?;

        assert_eq!(native.solve(Part::One)?, c.solve(Part::One)?);
        assert_eq!(native.solve(Part::Two)?, c.solve(Part::Two)?);
        Ok(())
    }

    /// Same agreement over the C++ marker the merge added.
    #[cfg(feature = "cpp")]
    #[test]
    fn test_day1_cpp_parse_agrees() -> miette::Result<()> {
        let input = "3   4
    4   3
    2   5
    1   3
    3   9
    3   3";
        let mut native = Day1::parse_with::<NativeSort>(input)?;
        let mut cpp = Day1::parse_with::<CppSort>(input)?;

        assert_eq!(native.solve(Part::One)?, cpp.solve(Part::One)?);
        assert_eq!(native.solve(Part::Two)?, cpp.solve(Part::Two)?);
        Ok(())
    }

    /// The talk's `test_both_agree`, sharpened: the two backends must order
    /// identically, including the extremes that broke the `a - b` comparator.
    #[cfg(feature = "qsort")]
    #[test]
    fn test_day1_sorts_agree() {
        let mut via_c = vec![3, 1, 4, 1, 5, 9, 2, 6, i32::MAX, i32::MIN];
        let mut pure = via_c.clone();

        sort_via_qsort(&mut via_c);
        sort_pure_rust(&mut pure);

        assert_eq!(via_c, pure);
    }

    /// The two backends must order identically, including the i32 extremes
    /// — std::sort compares with `<`, so unlike a subtracting C comparator
    /// there is no overflow to fall into, and this pins that.
    #[cfg(feature = "cpp")]
    #[test]
    fn test_day1_cpp_sort_agrees() {
        let mut via_cpp = vec![3, 1, 4, 1, 5, 9, 2, 6, i32::MAX, i32::MIN];
        let mut pure = via_cpp.clone();

        sort_via_cpp(&mut via_cpp);
        sort_pure_rust(&mut pure);

        assert_eq!(via_cpp, pure);
    }

    /// Both counters over the same columns, same score — including a key
    /// the right column never holds (count 0), a negative key, and an
    /// empty right column (uthash's NULL-is-an-empty-table case).
    #[cfg(feature = "uthash")]
    #[test]
    fn test_day1_similarity_agrees() {
        let left = [3, 4, 2, 1, 3, 3, -7];
        let right = [4, 3, 5, 3, 9, 3, -7];

        assert_eq!(
            similarity_pure_rust(&left, &right),
            similarity_via_uthash(&left, &right)
        );
        // The statement example's 31, minus 7: the negative key matches
        // once and a negative ID weights its count below zero.
        assert_eq!(31 - 7, similarity_via_uthash(&left, &right));
        assert_eq!(0, similarity_via_uthash(&left, &[]));
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
