//! The C-facing surface of this crate — Exercise 2: Rust exposing *itself*
//! to C. `cbindgen` (see `cbindgen.toml`) turns the `extern "C"` functions
//! below into a header; any language with a C FFI can then load the
//! compiled `cdylib` and call straight in — the Exercise 3 tracks, or
//! plain C.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic that
//! reaches an `extern "C"` frame aborts the process (Rust 1.81 and later —
//! before that it was undefined behavior), so nothing here can panic. That is
//! why this surface is built on the byte cursor (`crate::cursor`) rather than
//! the nom solution: the cursor is panic-free for arbitrary bytes by
//! construction (a failed parse is a position to move past, and its operands
//! are capped at the statement's three digits), while the nom path panics on
//! an operand too long for `usize` — `digit1` accepts any digit run and
//! `Product::new` then `expect`s the parse. Trusted puzzle input never does
//! that; a C caller is not trusted input. The README's "practically a C
//! signature already" line about the cursor, cashed in.
//!
//! Sums are `u64` out-parameters. Overflow is unreachable through this
//! surface rather than checked: each product is at most 999 × 999 (the
//! cursor's 3-digit cap) and each costs at least eight input bytes, so
//! exceeding a `u64` would take an input north of a hundred terabytes —
//! and a NUL-terminated C string that large cannot be handed over intact.

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

use crate::cursor;

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

/// Shared body of both entry points: read the input, scan it, write the sum.
///
/// Both parts differ only in which cursor scan runs, so the guard lives here
/// once. The module doc explains why this surface is built on the cursor
/// rather than the nom solution: the cursor is panic-free for arbitrary bytes
/// by construction. `catch_unwind` is the backstop for that claim being
/// wrong, not the thing that makes it true, and it costs an abort rather than
/// a bad answer to find out the hard way.
fn solve_into(input: *const c_char, out_sum: *mut u64, solve: fn(&str) -> usize) -> c_int {
    if out_sum.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };

    let Ok(sum) = catch_unwind(AssertUnwindSafe(|| solve(text))) else {
        return -2;
    };

    unsafe { *out_sum = sum as u64 };
    0
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
    solve_into(input, out_sum, cursor::part1)
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
    solve_into(input, out_sum, cursor::part2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// See 2015-12-05's copy: the panic hook is process-global, so a test
    /// that provokes a panic on purpose silences and restores it.
    fn without_panic_noise<T>(f: impl FnOnce() -> T) -> T {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let out = f();
        std::panic::set_hook(previous);
        out
    }

    fn panicking_scan(_input: &str) -> usize {
        panic!("the nom path this surface deliberately does not use");
    }

    /// The module doc claims the cursor is panic-free for arbitrary bytes by
    /// construction. This proves the backstop behind that claim: if the claim
    /// is ever wrong, a C caller gets `-2` rather than an aborted process.
    #[test]
    fn a_panicking_scan_reports_minus_two() {
        let input = CString::new("mul(2,3)").expect("no NUL bytes");
        let mut sum: u64 = 9;

        let status =
            without_panic_noise(|| solve_into(input.as_ptr(), &raw mut sum, panicking_scan));

        assert_eq!(status, -2, "a caught panic must arrive as the status code");
        assert_eq!(sum, 9, "out_sum must be left alone when we refuse");
    }

    #[test]
    fn a_real_input_still_scans() {
        let input = CString::new("xmul(2,4)%&mul[3,7]!@^mul(5,5)").expect("no NUL bytes");
        let mut sum: u64 = 0;

        // SAFETY: `input` is a live NUL-terminated string and `sum` is
        // writable for one `u64`. Both outlive the call.
        let status = unsafe { aoc_2024_12_03_part1(input.as_ptr(), &raw mut sum) };

        assert_eq!(status, 0);
        assert_eq!(sum, 33, "2*4 + 5*5, with the malformed mul[3,7] skipped");
    }
}
