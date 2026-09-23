//! Exercise 2, solved — the CI overlay for the attendee scaffold (see
//! .github/ci/README.md). Same file, TODOs filled: get a `*const c_char`
//! across the boundary into a `&str`, through the Ex 1 solver, and back out
//! as an `i64`, with no undefined behavior on hostile input.

use std::ffi::{CStr, c_char};

/// Error convention: return -1 when the input is null or not valid UTF-8.
pub const INVALID_INPUT: i64 = -1;

/// The four steps, once, shared by both exports: null check, wrap, validate
/// the encoding, hand over. `None` is the C side's problem, reported in band.
///
/// # Safety
/// `input` must be null or a valid NUL-terminated C string.
unsafe fn input_str<'a>(input: *const c_char) -> Option<&'a str> {
    if input.is_null() {
        return None;
    }
    // SAFETY: non-null by the check above; NUL-terminated by the caller's
    // contract, which is the whole promise this `unsafe fn` asks for.
    unsafe { CStr::from_ptr(input) }.to_str().ok()
}

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part1(input: *const c_char) -> i64 {
    // SAFETY: the caller's contract is exactly `input_str`'s.
    match unsafe { input_str(input) } {
        Some(s) => ex1_pure_rust::part1(s),
        None => INVALID_INPUT,
    }
}

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part2(input: *const c_char) -> i64 {
    // SAFETY: the caller's contract is exactly `input_str`'s.
    match unsafe { input_str(input) } {
        Some(s) => ex1_pure_rust::part2(s),
        None => INVALID_INPUT,
    }
}
