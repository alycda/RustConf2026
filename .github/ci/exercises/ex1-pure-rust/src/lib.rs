//! Exercise 1, solved: Advent of Code 2025, day 3 ("Joltage Selection").
//!
//! This is the CI overlay for the attendee scaffold, not part of the
//! exercise — .github/ci/README.md says why it exists and how it is used.
//! Solver ported from alycda/learning-in-public, branch AdventOfCode/2025,
//! advent-of-code/2025/rust/day-03/src/lib.rs (the iterator-based default;
//! that file's regex and pcre2 variants are not needed here). 2025 is not on
//! the workshop day menu, so nothing here spoils a day an attendee picks.
//!
//! Each line is a string of digits. Part 1 keeps the largest 2 digits in
//! order and sums the two-digit numbers; part 2 keeps the largest 12. The
//! part-2 answer for the statement example is 3121910778619 — above 32 bits
//! on purpose, so a binding that truncates to `int` somewhere fails here
//! rather than passing by luck on a small answer.

/// Greedy selection of the `n` largest digits of `line`, in order,
/// keeping the leftmost on a tie (which leaves the most room to the right).
///
/// Only ASCII digits count; anything else on the line is skipped, and a line
/// with fewer digits than `n` yields an empty pick. The statement never
/// produces either, but a harness or a binding can, and a solver that panics
/// on them panics inside Exercise 2's `extern "C"` frame — an abort.
fn select_n_largest(line: &str, n: usize) -> Vec<u32> {
    let digits: Vec<u32> = line
        .bytes()
        .filter(u8::is_ascii_digit)
        .map(|b| u32::from(b - b'0'))
        .collect();
    if digits.len() < n {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(n);
    let mut pos = 0;
    for i in 0..n {
        let remaining = n - i - 1;
        let end = digits.len() - remaining;
        let (best_pos, best_val) = digits[pos..end]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.cmp(b.1).then(std::cmp::Ordering::Greater))
            .map(|(idx, &val)| (pos + idx, val))
            .expect("the window is never empty: pos < end");
        result.push(best_val);
        pos = best_pos + 1;
    }
    result
}

/// Sum over lines of the two-digit number made of the largest 2 digits in order.
pub fn part1(input: &str) -> i64 {
    input
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| match select_n_largest(line, 2)[..] {
            [tens, ones] => i64::from(tens * 10 + ones),
            _ => 0,
        })
        .sum()
}

/// Sum over lines of the twelve-digit number made of the largest 12 digits in order.
pub fn part2(input: &str) -> i64 {
    input
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            select_n_largest(line, 12)
                .iter()
                .fold(0i64, |acc, &d| acc * 10 + i64::from(d))
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The puzzle-statement example — the same four lines every overlay
    /// file uses, so one number proves the whole path.
    const EXAMPLE: &str = "987654321111111
811111111111119
234234234234278
818181911112111";

    #[test]
    fn part1_example() {
        assert_eq!(part1(EXAMPLE), 357);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(EXAMPLE), 3121910778619);
    }

    /// Trailing whitespace, a lone digit, a blank line: none of them may
    /// reach ex_part1 as a panic. Digits only count; too few is zero.
    #[test]
    fn hostile_lines_contribute_zero_not_a_panic() {
        let input = "5\n \n987654321111111 \nabc";
        assert_eq!(part1(input), 98);
        assert_eq!(part2(input), 987654321111);
    }

    #[test]
    fn per_line() {
        for (line, two, twelve) in [
            ("987654321111111", 98, 987654321111),
            ("811111111111119", 89, 811111111119),
            ("234234234234278", 78, 434234234278),
            ("818181911112111", 92, 888911112111),
        ] {
            assert_eq!(part1(line), two, "{line}");
            assert_eq!(part2(line), twelve, "{line}");
        }
    }
}
