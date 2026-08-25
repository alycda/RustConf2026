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
#[cfg(feature = "espeak")]
pub mod espeak;
#[cfg(feature = "yara")]
pub mod yara;

/// The nine spelled-out digits, in value order. `pub(crate)` because both C
/// variants read this list rather than repeating it — the `yara` module builds
/// its rule text from it and the `espeak` module phonemises it. Three copies
/// of these nine words would be three places for a typo to answer a slightly
/// different puzzle.
pub(crate) const WORDS: [(&str, u32); 9] = [
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

/// Runs the same scan through YARA: one compiled rule set for the whole
/// solve, one `yr_rules_scan_mem` per line, the answer taken from the reported
/// match offsets.
///
/// Part two needs nothing extra here beyond nine more strings in the rule.
/// Overlapping spelled-out digits — `oneight` being a `1` and an `8` that
/// share an `e` — are not a special case for a multi-pattern scanner; they are
/// two occurrences, which is what it reports. See [`yara`] for the two things
/// about that reporting the scratchpad had to find out first.
#[cfg(feature = "yara")]
pub fn sum_calibration_via_yara(lines: &[String]) -> miette::Result<u32> {
    scan_via_yara(lines, false)
}

/// Part two through YARA. See [`sum_calibration_via_yara`].
#[cfg(feature = "yara")]
pub fn sum_calibration_with_words_via_yara(lines: &[String]) -> miette::Result<u32> {
    scan_via_yara(lines, true)
}

/// One scanner per solve, sized to the longest line it will be given.
///
/// A fresh rule set per solve is the same isolation choice 2021-12-02 made for
/// its `cpSpace` and its DuckDB connection, and here it is also where a third
/// of the time goes — compiling the rules costs ~0.4 ms against ~1.4 ms of
/// scanning for a thousand lines. `benches/calibration.rs` reports the two
/// separately rather than burying one in the other.
#[cfg(feature = "yara")]
fn scan_via_yara(lines: &[String], words: bool) -> miette::Result<u32> {
    let longest = lines.iter().map(String::len).max().unwrap_or(0);
    let mut scanner = yara::Scanner::new(words, longest)?;

    let mut total: u32 = 0;
    for line in lines {
        total = total
            .checked_add(scanner.calibration_value(line)?)
            .ok_or_else(|| miette::miette!("calibration sum overflows a u32"))?;
    }
    Ok(total)
}

impl Solution for Day {
    type Output = u32;

    /// Sum the calibration values, digits only, through whichever backend is
    /// compiled in.
    ///
    /// YARA wins when it is on, because it is the variant that answers the
    /// puzzle exactly — see `days/2023-12-01/README.md` for the one that
    /// deliberately does not and is never routed to.
    fn part1(&mut self) -> SolutionResult<Self::Output> {
        #[cfg(feature = "yara")]
        {
            sum_calibration_via_yara(&self.0)
        }
        #[cfg(not(feature = "yara"))]
        {
            Ok(sum_calibration_pure_rust(&self.0))
        }
    }

    /// Sum the calibration values, digits and spelled-out digits — same
    /// backend precedence as `part1`.
    fn part2(&mut self) -> SolutionResult<Self::Output> {
        #[cfg(feature = "yara")]
        {
            sum_calibration_with_words_via_yara(&self.0)
        }
        #[cfg(not(feature = "yara"))]
        {
            Ok(sum_calibration_with_words_pure_rust(&self.0))
        }
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

    /// The same two answers out of the scanning engine, and — the part the
    /// per-module tests can't cover — that it agrees with plain Rust rather
    /// than merely with the puzzle's two published totals. The overlap-heavy
    /// part-two example is exactly where a multi-pattern scanner and a
    /// forward scan could plausibly disagree.
    #[cfg(feature = "yara")]
    #[test]
    fn test_yara_backend_agrees_with_pure_rust() -> miette::Result<()> {
        let part1 = Day::from_str("1abc2\npqr3stu8vwx\na1b2c3d4e5f\ntreb7uchet")?;
        assert_eq!(sum_calibration_via_yara(&part1)?, 142);
        assert_eq!(
            sum_calibration_via_yara(&part1)?,
            sum_calibration_pure_rust(&part1)
        );

        let part2 = Day::from_str(
            "two1nine\neightwothree\nabcone2threexyz\nxtwone3four\n4nineeightseven2\nzoneight234\n7pqrstsixteen",
        )?;
        assert_eq!(sum_calibration_with_words_via_yara(&part2)?, 281);
        assert_eq!(
            sum_calibration_with_words_via_yara(&part2)?,
            sum_calibration_with_words_pure_rust(&part2)
        );
        Ok(())
    }

    /// An empty input is a real shape — `Day::from_str("")` has no lines — and
    /// it is the one that would trip a scanner sized from `max().unwrap_or(0)`
    /// if the zero case were not handled.
    #[cfg(feature = "yara")]
    #[test]
    fn test_yara_empty_input() -> miette::Result<()> {
        assert_eq!(sum_calibration_via_yara(&[])?, 0);
        assert_eq!(sum_calibration_with_words_via_yara(&[])?, 0);
        Ok(())
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
