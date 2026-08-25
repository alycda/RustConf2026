//! Day 5: Doesn't He Have Intern-Elves For This?

use std::str::FromStr;

use aoc_ornaments::Solution;

pub mod c_api;
pub mod hyperscan;
pub mod icu;

#[derive(Debug, derive_more::Deref)]
pub struct Day(Vec<String>);

impl FromStr for Day {
    type Err = miette::Error;

    fn from_str(input: &str) -> miette::Result<Self> {
        Ok(Self(input.lines().map(str::to_string).collect()))
    }
}

/// Check if a line is nice, in plain Rust. Kept alongside
/// [`icu::is_nice_via_icu`] and [`hyperscan::is_nice_via_hyperscan`] for
/// comparison.
///
/// - it contains at least three vowels (aeiou)
/// - a double letter (like xx)
/// - does not contain the strings ab, cd, pq, or xy.
pub fn is_nice_pure_rust(line: &str) -> bool {
    !has_forbidden_pair(line) && has_double_letter(line) && (count_vowels(line) >= 3)
}

/// Check if a line is nice using the new rules, in plain Rust. Kept
/// alongside [`icu::is_nice_v2_via_icu`] and
/// [`hyperscan::is_nice_v2_via_hyperscan`] for comparison.
///
/// - it contains a pair of any two letters that appears at least twice in the string without overlapping
/// - it contains at least one letter which repeats with exactly one letter between them
pub fn is_nice_v2_pure_rust(line: &str) -> bool {
    has_non_overlapping_pair(line) && has_sandwich_letter(line)
}

fn has_non_overlapping_pair(s: &str) -> bool {
    let chars: Vec<_> = s.chars().collect();
    chars.windows(2).enumerate().any(|(i, w1)| {
        chars[i + 2..]
            .windows(2)
            .any(|w2| w1[0] == w2[0] && w1[1] == w2[1])
    })
}

fn has_sandwich_letter(s: &str) -> bool {
    s.as_bytes().windows(3).any(|w| w[0] == w[2])
}

fn has_forbidden_pair(s: &str) -> bool {
    ["ab", "cd", "pq", "xy"]
        .iter()
        .any(|&pair| s.contains(pair))
}

fn has_double_letter(s: &str) -> bool {
    s.chars().zip(s.chars().skip(1)).any(|(a, b)| a == b)
}

fn count_vowels(s: &str) -> usize {
    s.chars().filter(|&c| is_vowel(c)).count()
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

impl Day {
    fn compute(&self, f: fn(&str) -> bool) -> usize {
        self.iter().filter(|line| f(line)).count()
    }
}

impl Solution for Day {
    type Output = usize;

    /// Count the number of nice strings in the input, via ICU regex — see
    /// [`icu::is_nice_via_icu`]. [`hyperscan::is_nice_via_hyperscan`] is the
    /// other sibling's answer to the same rules; ICU wins the default here
    /// because its regex is the more direct translation of the puzzle
    /// rules, where vectorscan's is a ~700-pattern workaround for missing
    /// backreferences (see hyperscan's module docs) — both are fully
    /// tested below regardless of which one `cargo run` exercises.
    fn part1(&mut self) -> aoc_ornaments::SolutionResult<Self::Output> {
        Ok(self.compute(icu::is_nice_via_icu))
    }

    /// Count the number of nice strings in the input using the new rules,
    /// via ICU regex. See [`icu::is_nice_v2_via_icu`] and the note on
    /// [`Solution::part1`].
    fn part2(&mut self) -> aoc_ornaments::SolutionResult<Self::Output> {
        Ok(self.compute(icu::is_nice_v2_via_icu))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("ugknbfddgicrmopn", true)]
    #[case("aaa", true)]
    #[case("jchzalrnumimnmhp", false)]
    #[case("haegwjzuvuyypxyu", false)]
    #[case("dvszwmarrgswjxmb", false)]
    fn test_cases_part1_pure_rust(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_nice_pure_rust(input), expected);
    }

    #[rstest]
    #[case("ugknbfddgicrmopn", true)]
    #[case("aaa", true)]
    #[case("jchzalrnumimnmhp", false)]
    #[case("haegwjzuvuyypxyu", false)]
    #[case("dvszwmarrgswjxmb", false)]
    fn test_cases_part1_hyperscan(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(hyperscan::is_nice_via_hyperscan(input), expected);
    }

    #[rstest]
    #[case("ugknbfddgicrmopn", true)]
    #[case("aaa", true)]
    #[case("jchzalrnumimnmhp", false)]
    #[case("haegwjzuvuyypxyu", false)]
    #[case("dvszwmarrgswjxmb", false)]
    fn test_cases_part1_icu(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(icu::is_nice_via_icu(input), expected);
    }

    #[rstest]
    #[case("qjhvhtzxzqqjkmpb", true)]
    #[case("xxyxx", true)]
    #[case("uurcxstgmygtbstg", false)]
    #[case("ieodomkazucvgmuy", false)]
    fn test_cases_part2_pure_rust(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_nice_v2_pure_rust(input), expected);
    }

    #[rstest]
    #[case("qjhvhtzxzqqjkmpb", true)]
    #[case("xxyxx", true)]
    #[case("uurcxstgmygtbstg", false)]
    #[case("ieodomkazucvgmuy", false)]
    fn test_cases_part2_hyperscan(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(hyperscan::is_nice_v2_via_hyperscan(input), expected);
    }

    #[rstest]
    #[case("qjhvhtzxzqqjkmpb", true)]
    #[case("xxyxx", true)]
    #[case("uurcxstgmygtbstg", false)]
    #[case("ieodomkazucvgmuy", false)]
    fn test_cases_part2_icu(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(icu::is_nice_v2_via_icu(input), expected);
    }
}
