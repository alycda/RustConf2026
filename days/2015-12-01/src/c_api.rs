//! The C-facing surface of this crate — Exercise 2's counterpart to the
//! `tcc`/`caca` modules: those call *into* a C library from Rust, this is
//! Rust exposing *itself* to C. `cbindgen` (see `cbindgen.toml`) turns the
//! `extern "C"` functions below into a header; any language with a C FFI
//! can then load the compiled `cdylib` and call straight in — Python via
//! `cffi` in Exercise 3, or plain C.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic
//! unwinding across an `extern "C"` frame is undefined behavior, so nothing
//! here can panic — bad input (a null pointer, invalid UTF-8) is a real
//! possibility from a C caller and is handled as data, not asserted away.

use std::ffi::{CStr, c_char, c_int};
use std::str::FromStr;

use crate::{Day, basement_position_pure_rust, sum_pure_rust};

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

/// Parses `input` and writes the floor Santa ends up on into `*out_floor`.
///
/// Returns `0` on success, `-1` if `input`/`out_floor` is null or `input`
/// isn't valid UTF-8.
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

    unsafe { *out_floor = sum_pure_rust(&day) };
    0
}

/// Parses `input` and writes the 1-based position of the first instruction
/// that sends Santa into the basement into `*out_position`.
///
/// Returns `0` on success, `-1` for a null/invalid-UTF-8 `input` (or a null
/// `out_position`), `-2` if Santa never enters the basement.
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

    match basement_position_pure_rust(&day) {
        Some(position) => {
            unsafe { *out_position = position };
            0
        }
        None => -2,
    }
}
