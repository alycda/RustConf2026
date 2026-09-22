//! The C-facing surface of this crate — Exercise 2's counterpart to the
//! `qsort`/`cpp`/`uthash` modules: those call *into* C (and C++) from Rust,
//! this is Rust exposing *itself* to C. `cbindgen` (see `cbindgen.toml`)
//! turns the `extern "C"` functions below into a header; any language with
//! a C FFI can then load the compiled `cdylib` and call straight in — the
//! Exercise 3 tracks, or plain C.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic that
//! reaches an `extern "C"` frame aborts the process (Rust 1.81 and later —
//! before that it was undefined behavior), so nothing here can panic. On this
//! day that rules out more than the usual suspects: `Day1`'s own `FromStr`
//! expects trusted puzzle input and says so (it panics on a malformed line),
//! and both parts do unchecked `i32` arithmetic that a hostile input could
//! overflow. A C caller is not trusted input, so this module parses defensively
//! and accumulates in `i64`, reporting overflow as a status instead of a wrap
//! or a panic.
//!
//! Built on the pure-Rust baseline (`sort_pure_rust`, and part 2's naive
//! scan spelled with checked arithmetic) specifically — not whichever
//! backend the `qsort`/`cpp`/`uthash` features currently give `Solution`.
//! Exercise 2 is about the *export* direction, and entangling it with
//! "which import won" would muddy both.

#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
// Power of Ten rule 10: this module is the shim, so it carries the pedantic
// setting even though the day crate around it does not. Scoped here on
// purpose — crate-wide `pedantic` reports 17-39 findings per day, nearly all
// in puzzle code, and burying two real casts in ~150 style notes is how a
// lint stops being read. `-D warnings` belongs in CI, never in source.

use std::ffi::{CStr, c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::sort_pure_rust;

/// Reads `input` as a `&str`, or `None` if it's null or not valid UTF-8.
///
/// # Safety
/// `input` must be null or point to a NUL-terminated C string valid for the
/// duration of the call.
///
/// The scan for that terminator is unbounded. `CStr::from_ptr` reads forward
/// until it meets a NUL byte. A caller that passes an unterminated buffer
/// reads past the end of its own allocation. The terminator is the only
/// limit that exists here.
///
/// Power of Ten rule 2 wants every loop bounded, and at an FFI boundary that
/// means the caller supplies a length. The C string protocol carries no
/// length, so this bound is the caller's promise rather than a parameter.
/// A length parameter would change the signature every Exercise 3 track is
/// written against, so the limit is named here rather than skipped.
///
/// The returned lifetime `'a` is not tied to `input`. The caller chooses it
/// and `'static` type-checks. Every caller in this module reads the result
/// before it returns, which is what makes the present code correct. A new
/// caller must do the same.
unsafe fn read_input<'a>(input: *const c_char) -> Option<&'a str> {
    if input.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(input) }.to_str().ok()
}

/// Parses the two columns without trusting the caller: every line must hold
/// exactly two `i32`s, or the whole input is rejected — the defensive
/// spelling of the expectation `Day1::from_str` enforces with panics.
fn parse_columns(text: &str) -> Option<(Vec<i32>, Vec<i32>)> {
    let mut left = Vec::new();
    let mut right = Vec::new();

    for line in text.lines() {
        let mut numbers = line.split_whitespace();
        left.push(numbers.next()?.parse().ok()?);
        right.push(numbers.next()?.parse().ok()?);
        if numbers.next().is_some() {
            return None;
        }
    }

    Some((left, right))
}

/// Parses `input` and writes part 1's total distance — columns rank-sorted,
/// pairwise absolute differences summed — into `*out_distance`.
///
/// Returns `0` on success, `-1` if `input`/`out_distance` is null, `input`
/// isn't valid UTF-8, or any line isn't exactly two integers, `-2` if the
/// total doesn't fit in an `int32_t`.
///
/// # Safety
/// `input` must point to a NUL-terminated C string. `out_distance` must
/// point to writable memory for one `int32_t`. Both must stay valid for the
/// call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2024_12_01_part1(
    input: *const c_char,
    out_distance: *mut i32,
) -> c_int {
    if out_distance.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Some((mut left, mut right)) = parse_columns(text) else {
        return -1;
    };

    // Parts 1 and 2 diverge past this point — one sorts and zips, the other
    // weights by occurrence count — so there is no shared `solve_into` to put
    // the guard in, as 2015-12-05 and 2024-12-03 have. It wraps each
    // computation in place instead.
    //
    // `-2` keeps its meaning: the answer left the `int32_t`, reported by the
    // checked arithmetic below rather than by a panic. A panic is a separate
    // `-3`, because folding it into `-2` would tell a C caller "your input
    // overflowed" about a bug in here.
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        sort_pure_rust(&mut left);
        sort_pure_rust(&mut right);

        // The distances are summed in i64 — a single |l - r| can exceed
        // i32::MAX on its own (i32::MIN vs i32::MAX), which is the same trap
        // the qsort comparator documents from the other side of the boundary.
        let mut total: i64 = 0;
        for (l, r) in left.iter().zip(right.iter()) {
            let distance = (i64::from(*l) - i64::from(*r)).abs();
            let next = total.checked_add(distance)?;
            total = next;
        }

        i32::try_from(total).ok()
    }));

    let Ok(computed) = outcome else {
        return -3;
    };
    let Some(distance) = computed else {
        return -2;
    };
    unsafe { *out_distance = distance };
    0
}

/// Parses `input` and writes part 2's similarity score — each left-hand ID
/// weighted by its occurrence count in the right column — into `*out_score`.
///
/// Returns `0` on success, `-1` for the same input errors as
/// [`aoc_2024_12_01_part1`], `-2` if the score doesn't fit in an `int32_t`,
/// `-3` if the computation panicked.
///
/// # Safety
/// Same contract as [`aoc_2024_12_01_part1`], for `out_score`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2024_12_01_part2(input: *const c_char, out_score: *mut i32) -> c_int {
    if out_score.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Some((left, right)) = parse_columns(text) else {
        return -1;
    };

    // Same split as part 1: `-2` is the overflow answer, `-3` is a panic.
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // similarity_pure_rust's naive scan, spelled with checked arithmetic —
        // see the module doc for why the unchecked baseline can't cross here.
        let mut total: i64 = 0;
        for l in &left {
            let count = i64::try_from(right.iter().filter(|r| *r == l).count()).ok()?;
            let weighted = i64::from(*l).checked_mul(count)?;
            let next = total.checked_add(weighted)?;
            total = next;
        }

        i32::try_from(total).ok()
    }));

    let Ok(computed) = outcome else {
        return -3;
    };
    let Some(score) = computed else {
        return -2;
    };
    unsafe { *out_score = score };
    0
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    const EXAMPLE: &str = "3   4\n4   3\n2   5\n1   3\n3   9\n3   3\n";

    #[test]
    fn nulls_are_refused_rather_than_dereferenced() {
        let input = CString::new(EXAMPLE).expect("no NUL bytes");
        let mut out: i32 = 13;

        // SAFETY: `input` is live and NUL-terminated. The null out-parameter
        // is the case under test and must be rejected, not written through.
        let null_out = unsafe { aoc_2024_12_01_part1(input.as_ptr(), std::ptr::null_mut()) };
        // SAFETY: `out` is writable for one `i32`. The null input is the case
        // under test.
        let null_in = unsafe { aoc_2024_12_01_part1(std::ptr::null(), &raw mut out) };

        assert_eq!(null_out, -1);
        assert_eq!(null_in, -1);
        assert_eq!(out, 13, "neither call may write through the out-parameter");
    }

    #[test]
    fn both_parts_answer_the_example() {
        let input = CString::new(EXAMPLE).expect("no NUL bytes");
        let mut distance: i32 = 0;
        let mut score: i32 = 0;

        // SAFETY: `input` is live and NUL-terminated, both out-parameters are
        // writable for one `i32`, and all outlive the calls.
        let s1 = unsafe { aoc_2024_12_01_part1(input.as_ptr(), &raw mut distance) };
        // SAFETY: as above, for `score`.
        let s2 = unsafe { aoc_2024_12_01_part2(input.as_ptr(), &raw mut score) };

        assert_eq!((s1, distance), (0, 11));
        assert_eq!((s2, score), (0, 31));
    }

    /// The `-2` contract. Two columns of `i32::MIN`/`i32::MAX` make each
    /// distance ~2^32, so the i64 total leaves the `int32_t` the caller asked
    /// for. This is the case the module doc says the unchecked baseline
    /// cannot cross with: it would have wrapped and returned `0`.
    #[test]
    fn a_distance_past_int32_reports_minus_two() {
        let rows: String =
            std::iter::repeat_n(format!("{}   {}\n", i32::MIN, i32::MAX), 4).collect();
        let input = CString::new(rows).expect("no NUL bytes");
        let mut distance: i32 = 7;

        // SAFETY: as above, for `distance`.
        let status = unsafe { aoc_2024_12_01_part1(input.as_ptr(), &raw mut distance) };

        assert_eq!(status, -2, "the total distance does not fit an int32_t");
        assert_eq!(distance, 7, "out_distance is left alone when we refuse");
    }
}
