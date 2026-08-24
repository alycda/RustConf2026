//! Exercise 1: solve your chosen AoC day in pure, idiomatic Rust.
//!
//! Don't think about FFI yet — write the Rust you'd want to write.
//! We'll break it at the boundary in Exercise 2.
//!
//! Signature guidance: `&str` in, `i64` out covers every day on the menu.
//! If your day's answer doesn't fit i64, talk to the facilitator — that's
//! an interesting boundary conversation, not a problem.

/// Solve part 1 of your chosen day.
pub fn part1(input: &str) -> i64 {
    let _ = input;
    todo!("solve part 1 of your chosen day — see ../days/README.md for the menu, or pick your own")
}

/// Solve part 2 of your chosen day.
pub fn part2(input: &str) -> i64 {
    let _ = input;
    todo!("solve part 2")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Paste the EXAMPLE input from your day's puzzle statement here.
    /// (Statement examples are fine to keep in the repo — your real puzzle
    /// input is not. Don't paste that anywhere.)
    const EXAMPLE: &str = "PASTE YOUR DAY'S EXAMPLE INPUT HERE";

    #[test]
    fn environment_works() {
        // Green out of the box — proves your toolchain runs tests.
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[ignore = "remove this line once you've pasted your example input and expected answer"]
    fn part1_example() {
        // Replace 0 with the expected answer from the puzzle statement.
        assert_eq!(part1(EXAMPLE), 0);
    }

    #[test]
    #[ignore = "remove this line when you reach part 2"]
    fn part2_example() {
        assert_eq!(part2(EXAMPLE), 0);
    }
}
