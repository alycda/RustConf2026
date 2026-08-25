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

pub mod c_api;

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
    ///
    /// `char_indices()` rather than `0..line.len()`: the scan slices the line
    /// at every offset it produces, and a byte offset that lands inside a
    /// multi-byte character panics — `&"é1"[1..]` is not a slice, it is
    /// "start byte index 1 is not a char boundary", in every profile. On
    /// ASCII the two iterators produce the same offsets, so no answer
    /// changes; the difference only shows on input this crate cannot
    /// currently receive — and is about to, once it has a C-callable
    /// surface, where an unwinding panic is undefined behavior rather than
    /// a backtrace.
    fn calibration_value(line: &str, words: bool) -> u32 {
        let digits: Vec<u32> = line
            .char_indices()
            .filter_map(|(i, _)| Self::digit_at(line, i, words))
            .collect();

        digits.first().copied().unwrap_or(0) * 10 + digits.last().copied().unwrap_or(0)
    }
}

/// Sums every line's calibration value, digits only — the puzzle solved the
/// ordinary way.
///
/// A real `pub fn` rather than a body inside `Solution::part1` for the same
/// reason 2015-12-01's `sum_pure_rust` and 2021-12-02's
/// `dead_reckon_pure_rust` are: [`c_api`] needs an entry point whose meaning
/// doesn't depend on which backend a later cargo feature compiles in, and a
/// benchmark needs something to race one against. The `_pure_rust` suffix is
/// the repo's marker for exactly that — the plain implementation, kept alive
/// rather than replaced when a C library turns up to do the same job.
///
/// Panics if the sum leaves a `u32`. See
/// [`checked_sum_calibration_pure_rust`] for who cares and why.
pub fn sum_calibration_pure_rust(lines: &[String]) -> u32 {
    checked_sum_calibration_pure_rust(lines).expect("calibration sum overflows a u32")
}

/// [`sum_calibration_pure_rust`] without the panic, for callers that have to
/// answer an oversized input rather than die on it — [`c_api`], which cannot
/// let an unwind cross its `extern "C"` frame.
///
/// This is precaution, not necessity, and the difference is worth stating.
/// 2021-12-02 needed its equivalent: a genuine puzzle input already landed
/// within ~10% of `i32::MAX`. Here a line is worth at most 99, so the sum
/// needs roughly 43 million lines — at least 86 MB of input — to reach
/// `u32::MAX`. No puzzle input is remotely near that; a C caller is not
/// restricted to puzzle inputs, and the cost of being right about it is one
/// `checked_add`.
pub fn checked_sum_calibration_pure_rust(lines: &[String]) -> Option<u32> {
    checked_sum(lines, false)
}

/// Part two's rules — spelled-out digits count too — in plain Rust. See
/// [`sum_calibration_pure_rust`].
pub fn sum_calibration_with_words_pure_rust(lines: &[String]) -> u32 {
    checked_sum_calibration_with_words_pure_rust(lines).expect("calibration sum overflows a u32")
}

/// [`sum_calibration_with_words_pure_rust`] without the panic. See
/// [`checked_sum_calibration_pure_rust`].
pub fn checked_sum_calibration_with_words_pure_rust(lines: &[String]) -> Option<u32> {
    checked_sum(lines, true)
}

fn checked_sum(lines: &[String], words: bool) -> Option<u32> {
    checked_total(lines.iter().map(|line| Day::calibration_value(line, words)))
}

/// Adds per-line calibration values, refusing to wrap.
///
/// `try_fold` rather than `sum()`: `sum()` over a `u32` iterator panics where
/// `overflow-checks` is on and wraps where it isn't — the dev profile and the
/// release profile respectively — so a guard whose mechanism is "it would
/// have panicked" is a guard the shipped `cdylib` does not have. 2021-12-02
/// learned that one the expensive way; this day inherits the answer.
///
/// Split out from [`checked_sum`] so the refusal is testable at all. The
/// status code it feeds needs ~43 million lines of real input to reach, which
/// is not a test anyone will run; a fold over two `u32`s is. What that pins is
/// the arithmetic, not the whole path — see [`c_api`] for the part that stays
/// unexercised.
fn checked_total(values: impl IntoIterator<Item = u32>) -> Option<u32> {
    values.into_iter().try_fold(0u32, u32::checked_add)
}

impl Solution for Day {
    type Output = u32;

    /// Sum the calibration values, digits only.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        Ok(sum_calibration_pure_rust(&self.0))
    }

    /// Sum the calibration values, digits and spelled-out digits.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        Ok(sum_calibration_with_words_pure_rust(&self.0))
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

    /// The `_pure_rust` functions called directly rather than through
    /// `Solution`. Today they are the same code path; they will not stay that
    /// way — the point of the suffix is that `Solution` routes to whichever
    /// backend a cargo feature selected, and once one exists it can no longer
    /// vouch for this one.
    #[test]
    fn test_pure_rust_backend() -> miette::Result<()> {
        let day = Day::from_str("1abc2\npqr3stu8vwx\na1b2c3d4e5f\ntreb7uchet")?;
        assert_eq!(sum_calibration_pure_rust(&day), 142);

        // 29 + 83 + 13: the first three lines of part two's example.
        let day = Day::from_str("two1nine\neightwothree\nabcone2threexyz")?;
        assert_eq!(sum_calibration_with_words_pure_rust(&day), 125);
        Ok(())
    }

    /// The refusal behind [`c_api`]'s `-2`, exercised where it is cheap to
    /// exercise. Reaching it through a real input means ~43 million lines;
    /// reaching the arithmetic means two numbers.
    ///
    /// The `u32::MAX` case is the one that matters, and it must hold in
    /// release as well as dev — a `sum()` here would have wrapped there and
    /// reported success.
    #[test]
    fn the_total_refuses_to_wrap() {
        assert_eq!(checked_total([12, 38]), Some(50));
        assert_eq!(checked_total([0u32; 0]), Some(0));
        assert_eq!(checked_total([u32::MAX, 1]), None);
        assert_eq!(checked_total([u32::MAX - 98, 99]), None);
    }

    /// A line whose characters are not all one byte wide. Asserted for its
    /// value rather than merely for not panicking: the claim worth pinning
    /// is that the scan still finds the digits on both sides of the
    /// multi-byte character, not just that it survives reaching them.
    ///
    /// `é` is two bytes, so the old `0..line.len()` scan sliced at offset 1
    /// — inside it — and panicked in every profile. No real puzzle input
    /// looks like this; a `const char *` from a C caller can, and valid
    /// UTF-8 is exactly what the C API promises to accept.
    #[test]
    fn a_multibyte_line_is_scanned_by_character() -> miette::Result<()> {
        assert_eq!("19", Day::from_str("1é9")?.solve(Part::One)?);
        assert_eq!("42", Day::from_str("fourété2")?.solve(Part::Two)?);
        Ok(())
    }
}
