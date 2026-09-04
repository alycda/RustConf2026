//! Day 3 solved again with nothing but a byte cursor — the talk's
//! `2024-12-03` branch (alycda/aoc-ffi), ported.
//!
//! The library solution parses with nom + nom_locate; this one walks the
//! bytes by hand: try to read `mul(X,Y)` at the current position, move one
//! byte forward when it isn't there. Same answers, zero dependencies — the
//! version you write when you want the whole parser on one screen. And
//! where the nom path needed a fix to stop panicking on corruption
//! mid-`mul(`, the cursor never had the problem: a failed parse is just a
//! position to move past.

/// Parses a number (1–3 digits) starting at position `pos` in the byte
/// slice. Returns `(parsed_number, new_position)`, or `None` if no valid
/// number starts there.
///
/// The 3-digit cap is the puzzle statement's own bound on `mul` operands —
/// the nom variant is looser and accepts any digit run.
pub fn parse_num(input: &[u8], pos: usize) -> Option<(usize, usize)> {
    let start = pos;
    let mut end = pos;
    while end < input.len() && end - start < 3 && input[end].is_ascii_digit() {
        end += 1;
    }
    if end == start {
        return None;
    }
    let s = std::str::from_utf8(&input[start..end]).ok()?;
    Some((s.parse().ok()?, end))
}

/// Tries to parse `mul(X,Y)` at position `pos`.
/// Returns `(product, new_position)`, or `None`.
pub fn parse_mul(input: &[u8], pos: usize) -> Option<(usize, usize)> {
    let rest = &input[pos..];
    if !rest.starts_with(b"mul(") {
        return None;
    }
    let pos = pos + 4;
    let (x, pos) = parse_num(input, pos)?;
    if input.get(pos) != Some(&b',') {
        return None;
    }
    let pos = pos + 1;
    let (y, pos) = parse_num(input, pos)?;
    if input.get(pos) != Some(&b')') {
        return None;
    }
    Some((x * y, pos + 1))
}

/// Sums every well-formed `mul(X,Y)` in the input.
pub fn part1(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut sum = 0;
    let mut i = 0;
    while i < bytes.len() {
        if let Some((product, _next)) = parse_mul(bytes, i) {
            sum += product;
        }
        i += 1;
    }
    sum
}

/// Sums the enabled `mul(X,Y)`s: `don't()` switches them off, `do()` back
/// on, most recent toggle wins, and multiplication starts enabled.
pub fn part2(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut sum = 0;
    let mut enabled = true;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"do()") {
            enabled = true;
        } else if bytes[i..].starts_with(b"don't()") {
            enabled = false;
        } else if enabled {
            if let Some((product, _next)) = parse_mul(bytes, i) {
                sum += product;
            }
        }
        i += 1;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use aoc_ornaments::{Part, Solution};

    use crate::{Day3, Part1, Part2};

    const SAMPLE_1: &str =
        "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
    const SAMPLE_2: &str =
        "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";

    #[test]
    fn test_part1() {
        assert_eq!(161, part1(SAMPLE_1));
    }

    #[test]
    fn test_part2() {
        assert_eq!(48, part2(SAMPLE_2));
    }

    /// Same bytes through both parsers, same numbers — the point of keeping
    /// a second lens on one day.
    #[test]
    fn test_agrees_with_nom() -> miette::Result<()> {
        assert_eq!(
            part1(SAMPLE_1).to_string(),
            Day3::<Part1>::from_str(SAMPLE_1)?.solve(Part::One)?
        );
        assert_eq!(
            part2(SAMPLE_2).to_string(),
            Day3::<Part2>::from_str(SAMPLE_2)?.solve(Part::Two)?
        );
        Ok(())
    }

    /// The corruption shapes the nom path needed a fix for — handled here
    /// by construction, so pin them for this parser too.
    #[test]
    fn test_corruption_is_skipped() {
        assert_eq!(8, part1("mul(x,4)mul(2,4)"));
        assert_eq!(8, part1("mul(mul(2,4)"));
        assert_eq!(8, part1("mul(2,4)mul(2,"));
    }
}
