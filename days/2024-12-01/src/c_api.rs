//! The C-facing surface of this crate — Exercise 2's counterpart to the
//! `qsort`/`cpp`/`uthash` modules: those call *into* C (and C++) from Rust,
//! this is Rust exposing *itself* to C. `cbindgen` (see `cbindgen.toml`)
//! turns the `extern "C"` functions below into a header; any language with
//! a C FFI can then load the compiled `cdylib` and call straight in — the
//! Exercise 3 tracks, or plain C.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic
//! unwinding across an `extern "C"` frame is undefined behavior, so nothing
//! here can panic. On this day that rules out more than the usual suspects:
//! `Day1`'s own `FromStr` expects trusted puzzle input and says so (it
//! panics on a malformed line), and both parts do unchecked `i32`
//! arithmetic that a hostile input could overflow. A C caller is not
//! trusted input, so this module parses defensively and accumulates in
//! `i64`, reporting overflow as a status instead of a wrap or a panic.
//!
//! Built on the pure-Rust baseline (`sort_pure_rust`, and part 2's naive
//! scan spelled with checked arithmetic) specifically — not whichever
//! backend the `qsort`/`cpp`/`uthash` features currently give `Solution`.
//! Exercise 2 is about the *export* direction, and entangling it with
//! "which import won" would muddy both.

use std::ffi::{CStr, c_char, c_int};

use crate::sort_pure_rust;

/// Reads `input` as a `&str`, or `None` if it's null or not valid UTF-8.
///
/// # Safety
/// `input` must be null or point to a NUL-terminated C string valid for the
/// duration of the call.
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

    sort_pure_rust(&mut left);
    sort_pure_rust(&mut right);

    // The distances are summed in i64 — a single |l - r| can exceed
    // i32::MAX on its own (i32::MIN vs i32::MAX), which is the same trap
    // the qsort comparator documents from the other side of the boundary.
    let mut total: i64 = 0;
    for (l, r) in left.iter().zip(right.iter()) {
        let distance = (i64::from(*l) - i64::from(*r)).abs();
        let Some(next) = total.checked_add(distance) else {
            return -2;
        };
        total = next;
    }

    let Ok(distance) = i32::try_from(total) else {
        return -2;
    };
    unsafe { *out_distance = distance };
    0
}

/// Parses `input` and writes part 2's similarity score — each left-hand ID
/// weighted by its occurrence count in the right column — into `*out_score`.
///
/// Returns `0` on success, `-1` for the same input errors as
/// [`aoc_2024_12_01_part1`], `-2` if the score doesn't fit in an `int32_t`.
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

    // similarity_pure_rust's naive scan, spelled with checked arithmetic —
    // see the module doc for why the unchecked baseline can't cross here.
    let mut total: i64 = 0;
    for l in &left {
        let count = right.iter().filter(|r| *r == l).count() as i64;
        let Some(weighted) = i64::from(*l).checked_mul(count) else {
            return -2;
        };
        let Some(next) = total.checked_add(weighted) else {
            return -2;
        };
        total = next;
    }

    let Ok(score) = i32::try_from(total) else {
        return -2;
    };
    unsafe { *out_score = score };
    0
}
