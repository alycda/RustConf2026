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

use std::ffi::{CStr, c_char, c_int, c_uint};
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
/// isn't valid UTF-8.
///
/// # Safety
/// `input` must point to a NUL-terminated C string. `out_count` must point
/// to writable memory for one `uint32_t`. Both must stay valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_05_part1(
    input: *const c_char,
    out_count: *mut c_uint,
) -> c_int {
    if out_count.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(day) = Day::from_str(text) else {
        return -1;
    };

    let count = day.iter().filter(|line| is_nice_pure_rust(line)).count();
    unsafe { *out_count = count as c_uint };
    0
}

/// Parses `input` and writes the count of nice lines under the part-2
/// rules into `*out_count`.
///
/// Returns `0` on success, `-1` if `input`/`out_count` is null or `input`
/// isn't valid UTF-8.
///
/// # Safety
/// Same contract as [`aoc_2015_12_05_part1`], for `out_count`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_05_part2(
    input: *const c_char,
    out_count: *mut c_uint,
) -> c_int {
    if out_count.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(day) = Day::from_str(text) else {
        return -1;
    };

    let count = day.iter().filter(|line| is_nice_v2_pure_rust(line)).count();
    unsafe { *out_count = count as c_uint };
    0
}
