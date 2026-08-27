//! The C-facing surface of this crate — Exercise 2: Rust exposing *itself*
//! to C. `cbindgen` (see `cbindgen.toml`) turns the `extern "C"` functions
//! below into a header; any language with a C FFI can then load the
//! compiled `cdylib` and call straight in — the Exercise 3 tracks, or
//! plain C.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic
//! unwinding across an `extern "C"` frame is undefined behavior, so
//! nothing here can panic. That is why this surface is built on the byte
//! cursor (`crate::cursor`) rather than the nom solution: the cursor is
//! panic-free for arbitrary bytes by construction (a failed parse is a
//! position to move past, and its operands are capped at the statement's
//! three digits), while the nom path panics on an operand too long for
//! `usize` — `digit1` accepts any digit run and `Product::new` then
//! `expect`s the parse. Trusted puzzle input never does that; a C caller
//! is not trusted input. The README's "practically a C signature already"
//! line about the cursor, cashed in.
//!
//! Sums are `u64` out-parameters. Overflow is unreachable through this
//! surface rather than checked: each product is at most 999 × 999 (the
//! cursor's 3-digit cap) and each costs at least eight input bytes, so
//! exceeding a `u64` would take an input north of a hundred terabytes —
//! and a NUL-terminated C string that large cannot be handed over intact.

use std::ffi::{CStr, c_char, c_int};

use crate::cursor;

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

/// Scans `input` and writes part 1's sum of every well-formed `mul(X,Y)`
/// into `*out_sum`.
///
/// Returns `0` on success, `-1` if `input`/`out_sum` is null or `input`
/// isn't valid UTF-8. Corruption is not an error — skipping it is the
/// puzzle.
///
/// # Safety
/// `input` must point to a NUL-terminated C string. `out_sum` must point
/// to writable memory for one `uint64_t`. Both must stay valid for the
/// call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2024_12_03_part1(input: *const c_char, out_sum: *mut u64) -> c_int {
    if out_sum.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };

    unsafe { *out_sum = cursor::part1(text) as u64 };
    0
}

/// Scans `input` and writes part 2's sum — only the `mul(X,Y)`s enabled by
/// the most recent `do()`/`don't()` toggle count — into `*out_sum`.
///
/// Returns `0` on success, `-1` for the same input errors as
/// [`aoc_2024_12_03_part1`].
///
/// # Safety
/// Same contract as [`aoc_2024_12_03_part1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2024_12_03_part2(input: *const c_char, out_sum: *mut u64) -> c_int {
    if out_sum.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };

    unsafe { *out_sum = cursor::part2(text) as u64 };
    0
}
