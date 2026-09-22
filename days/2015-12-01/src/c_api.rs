//! The C-facing surface of this crate — Exercise 2's counterpart to the
//! `tcc`/`caca` modules: those call *into* a C library from Rust, this is
//! Rust exposing *itself* to C. `cbindgen` (see `cbindgen.toml`) turns the
//! `extern "C"` functions below into a header; any language with a C FFI
//! can then load the compiled `cdylib` and call straight in — Python via
//! `cffi` in Exercise 3, or plain C.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic that
//! reaches an `extern "C"` frame aborts the process (Rust 1.81 and later —
//! before that it was undefined behavior), so nothing here can panic — bad
//! input (a null pointer, invalid UTF-8) is a real possibility from a C caller
//! and is handled as data, not asserted away.

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
use std::str::FromStr;

use crate::{Day, basement_position_pure_rust, sum_pure_rust};

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

/// Parses `input` and writes the floor Santa ends up on into `*out_floor`.
///
/// Returns `0` on success, `-1` if `input`/`out_floor` is null or `input`
/// isn't valid UTF-8, `-3` if summing the instructions panicked.
///
/// # Safety
/// `input` must point to a NUL-terminated C string. `out_floor` must point
/// to writable memory for one `int32_t`. Both must stay valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_01_part1(
    input: *const c_char,
    out_floor: *mut c_int,
) -> c_int {
    if out_floor.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(day) = Day::from_str(text) else {
        return -1;
    };

    // `sum_pure_rust` is `iter().sum::<i32>()`, and `Day::from_str` maps each
    // character to ±1, so a caller who sends more than `i32::MAX` characters
    // overflows the accumulator. That panics wherever `overflow-checks` is on
    // and wraps where it is not. Neither belongs in an `extern "C"` frame, so
    // the panic becomes `-3` here and `*out_floor` is left alone.
    let Ok(floor) = catch_unwind(AssertUnwindSafe(|| sum_pure_rust(&day))) else {
        return -3;
    };

    unsafe { *out_floor = floor };
    0
}

/// Parses `input` and writes the 1-based position of the first instruction
/// that sends Santa into the basement into `*out_position`.
///
/// Returns `0` on success, `-1` for a null/invalid-UTF-8 `input` (or a null
/// `out_position`), `-2` if Santa never enters the basement, `-3` if the scan
/// panicked.
///
/// # Safety
/// Same contract as [`aoc_2015_12_01_part1`], for `out_position`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_01_part2(
    input: *const c_char,
    out_position: *mut c_int,
) -> c_int {
    if out_position.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(day) = Day::from_str(text) else {
        return -1;
    };

    // Same backstop as part 1. The `-2` below is an ordinary answer ("Santa
    // never reaches the basement"), so a panic needs its own code rather than
    // arriving as that one.
    let Ok(found) = catch_unwind(AssertUnwindSafe(|| basement_position_pure_rust(&day))) else {
        return -3;
    };

    match found {
        Some(position) => {
            unsafe { *out_position = position };
            0
        }
        None => -2,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// A null `out_floor` must be refused before anything is written, and a
    /// null `input` must be refused before it is dereferenced. These are the
    /// rule 5 parameter checks, exercised through the C entry point rather
    /// than by reading it.
    #[test]
    fn nulls_are_refused_rather_than_dereferenced() {
        let input = CString::new("(())").expect("no NUL bytes");
        let mut floor: c_int = 11;

        // SAFETY: `input` is live and NUL-terminated. The null `out_floor` is
        // the case under test and must be rejected, not written through.
        let null_out = unsafe { aoc_2015_12_01_part1(input.as_ptr(), std::ptr::null_mut()) };
        // SAFETY: `floor` is writable for one `c_int`. The null `input` is the
        // case under test.
        let null_in = unsafe { aoc_2015_12_01_part1(std::ptr::null(), &raw mut floor) };

        assert_eq!(null_out, -1, "a null out-parameter is -1");
        assert_eq!(null_in, -1, "a null input is -1");
        assert_eq!(
            floor, 11,
            "neither call may write through the out-parameter"
        );
    }

    #[test]
    fn a_real_input_is_answered() {
        let input = CString::new("(()(()(").expect("no NUL bytes");
        let mut floor: c_int = 0;

        // SAFETY: `input` is live and NUL-terminated, `floor` is writable for
        // one `c_int`, and both outlive the call.
        let status = unsafe { aoc_2015_12_01_part1(input.as_ptr(), &raw mut floor) };

        assert_eq!(status, 0);
        assert_eq!(floor, 3, "five ( and two ) leaves Santa on floor 3");
    }

    /// Part 2's `-2` is an ordinary answer, not an error: Santa never reaches
    /// the basement. It has its own code precisely so a caller can tell it
    /// apart from the `-3` a panic would produce.
    #[test]
    fn never_reaching_the_basement_is_minus_two() {
        let input = CString::new("(((").expect("no NUL bytes");
        let mut position: c_int = 4;

        // SAFETY: as above, for `position`.
        let status = unsafe { aoc_2015_12_01_part2(input.as_ptr(), &raw mut position) };

        assert_eq!(status, -2);
        assert_eq!(
            position, 4,
            "out_position is left alone when there is no answer"
        );
    }

    #[test]
    fn the_first_basement_step_is_reported_one_based() {
        let input = CString::new("()())").expect("no NUL bytes");
        let mut position: c_int = 0;

        // SAFETY: as above, for `position`.
        let status = unsafe { aoc_2015_12_01_part2(input.as_ptr(), &raw mut position) };

        assert_eq!(status, 0);
        assert_eq!(
            position, 5,
            "the fifth instruction is the first into the basement"
        );
    }
}
