//! The C-facing surface of this crate — Exercise 2's counterpart to the
//! `hyperscan`/`icu` modules: those call *into* a C library from Rust,
//! this is Rust exposing *itself* to C. `cbindgen` (see `cbindgen.toml`)
//! turns the `extern "C"` functions below into a header; any language
//! with a C FFI can then load the compiled `cdylib` and call straight in
//! — Dart via `dart:ffi` in Exercise 3, or plain C.
//!
//! Built on the plain-Rust implementations (`is_nice_pure_rust`/
//! `is_nice_v2_pure_rust`), not the hyperscan/icu ones: this exercise is
//! about the export *direction*, not about which regex engine wins — see
//! days/2015-12-05/README.md.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic that
//! reaches an `extern "C"` frame aborts the process (Rust 1.81 and later —
//! before that it was undefined behavior), so nothing here can panic — bad
//! input (a null pointer, invalid UTF-8) is a real possibility from a C
//! caller and is handled as data, not asserted away.

#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
// Power of Ten rule 10: this module is the shim, so it carries the pedantic
// setting even though the day crate around it does not. Scoped here on
// purpose — crate-wide `pedantic` reports 17-39 findings per day, nearly all
// in puzzle code, and burying two real casts in ~150 style notes is how a
// lint stops being read. `-D warnings` belongs in CI, never in source.

use std::ffi::{CStr, c_char, c_int, c_uint};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use crate::{Day, is_nice_pure_rust, is_nice_v2_pure_rust};

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

/// Parses `input` (one candidate string per line) and writes the count of
/// nice lines under the original rules into `*out_count`.
///
/// Returns `0` on success, `-1` if `input`/`out_count` is null or `input`
/// isn't valid UTF-8, `-2` if counting panicked, `-3` if the count exceeds
/// `uint32_t`.
///
/// # Safety
/// `input` must point to a NUL-terminated C string. `out_count` must point
/// to writable memory for one `uint32_t`. Both must stay valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_05_part1(
    input: *const c_char,
    out_count: *mut c_uint,
) -> c_int {
    solve_into(input, out_count, is_nice_pure_rust)
}

/// Shared body of both entry points: parse, count the lines `predicate`
/// accepts, write the count.
///
/// Both parts differ only in that predicate, so the guard lives here once
/// rather than twice. The `catch_unwind` is a backstop, not the guard —
/// nothing on this path panics today, because the parse is infallible and
/// both predicates are pure character tests. It stays because this is an
/// `extern "C"` frame, where being wrong about that costs an abort rather
/// than a bad answer, and because a workshop attendee editing a predicate is
/// exactly the reader who finds out otherwise.
fn solve_into(input: *const c_char, out_count: *mut c_uint, predicate: fn(&str) -> bool) -> c_int {
    if out_count.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(day) = Day::from_str(text) else {
        return -1;
    };

    let Ok(count) = catch_unwind(AssertUnwindSafe(|| {
        day.iter().filter(|line| predicate(line.as_str())).count()
    })) else {
        return -2;
    };

    // `count` is a `usize`. The old `as c_uint` truncated silently on a
    // 64-bit target, so a caller with more than `u32::MAX` nice lines got a
    // small number and `0` — a wrong answer reported as success, which is the
    // failure mode the status codes exist to prevent. It takes a ~8GiB input
    // to reach, and unreachable is still not checked.
    let Ok(count) = c_uint::try_from(count) else {
        return -3;
    };

    unsafe { *out_count = count };
    0
}

/// Parses `input` and writes the count of nice lines under the part-2
/// rules into `*out_count`.
///
/// Returns `0` on success, `-1` if `input`/`out_count` is null or `input`
/// isn't valid UTF-8, `-2` if counting panicked, `-3` if the count exceeds
/// `uint32_t`.
///
/// # Safety
/// Same contract as [`aoc_2015_12_05_part1`], for `out_count`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_05_part2(
    input: *const c_char,
    out_count: *mut c_uint,
) -> c_int {
    solve_into(input, out_count, is_nice_v2_pure_rust)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Runs `f` with the panic hook silenced, so an intentionally panicking
    /// predicate does not print a backtrace over the test output. The hook is
    /// process-global, which is why this restores it.
    fn without_panic_noise<T>(f: impl FnOnce() -> T) -> T {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let out = f();
        std::panic::set_hook(previous);
        out
    }

    fn panicking_predicate(_line: &str) -> bool {
        panic!("a predicate an attendee edited into something fallible");
    }

    /// The guard, exercised through `solve_into` rather than through a real
    /// predicate, because neither shipped predicate can panic — that is the
    /// point of the backstop. Injecting the panic is the only way to prove
    /// the status code survives the trip out instead of aborting the process.
    #[test]
    fn a_panicking_predicate_reports_minus_two() {
        let input = CString::new("aaa\nbbb\n").expect("no NUL bytes");
        let mut count: c_uint = 7;

        let status =
            without_panic_noise(|| solve_into(input.as_ptr(), &raw mut count, panicking_predicate));

        assert_eq!(status, -2, "a caught panic must arrive as the status code");
        assert_eq!(count, 7, "out_count must be left alone when we refuse");
    }

    #[test]
    fn a_real_input_still_counts() {
        let input = CString::new("ugknbfddgicrmopn\njchzalrnumimnmhp\n").expect("no NUL bytes");
        let mut count: c_uint = 0;

        // SAFETY: `input` is a live NUL-terminated string and `count` is
        // writable for one `c_uint`. Both outlive the call.
        let status = unsafe { aoc_2015_12_05_part1(input.as_ptr(), &raw mut count) };

        assert_eq!(status, 0);
        assert_eq!(count, 1, "the first line is nice, the second is not");
    }
}
