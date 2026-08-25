//! The C-facing surface of this crate — Exercise 2's counterpart to the
//! `chipmunk` module: that one calls *into* a C library from Rust, this is
//! Rust exposing *itself* to C. `cbindgen` (see `cbindgen.toml`) turns the
//! `extern "C"` functions below into a header; any language with a C FFI can
//! then load the compiled `cdylib` and call straight in — Kotlin via JNA in
//! Exercise 3, or plain C.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic
//! unwinding across an `extern "C"` frame is undefined behavior, so nothing
//! here can panic — bad input (a null pointer, invalid UTF-8) is a real
//! possibility from a C caller and is handled as data, not asserted away.
//!
//! These are built on `dead_reckon_pure_rust`/`dead_reckon_with_aim_pure_rust`
//! specifically, not on whichever backend `Solution::part1`/`part2` currently
//! route to. Exercise 2 is about the export *direction*; entangling it with
//! "does this build have the physics engine" would muddy both, and would make
//! the header's meaning depend on a cargo feature.

use std::ffi::{CStr, c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use crate::{Day, dead_reckon_pure_rust, dead_reckon_with_aim_pure_rust};

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

/// Shared body of both entry points: parse, run `solve`, write the answer.
///
/// The `catch_unwind` is the one place this day departs from 2015-12-01's
/// otherwise identical `c_api`. That day summed ±1 and could not realistically
/// overflow; this one multiplies horizontal by depth, and part 2 on a genuine
/// puzzle input already lands within ~10% of `i32::MAX`. An input a little
/// larger — trivially constructed by a C caller, who is not restricted to
/// real puzzle inputs — overflows, and integer overflow *panics* in the dev
/// profile. Letting that unwind out of an `extern "C"` frame is exactly the
/// UB this module's contract forbids, so it is caught and reported as a
/// status code instead.
///
/// Two things worth knowing about the guard rather than trusting it blindly:
///
/// - It only works because this workspace uses the default *unwinding* panic
///   strategy. A profile with `panic = "abort"` kills the process before
///   `catch_unwind` is reached, and the only real fix then is arithmetic that
///   cannot overflow in the first place.
/// - It stops the unwind, not the noise. Rust's default panic hook still
///   prints `attempt to multiply with overflow` to stderr before the unwind
///   begins, so a C caller that gets a tidy `-3` also gets a line of Rust
///   diagnostics it never asked for. Silencing it means installing a
///   process-global `panic::set_hook`, which a library has no business doing
///   to its host — so the noise stays, documented, rather than being fixed by
///   reaching outside this crate's own scope.
fn solve_into(
    input: *const c_char,
    out_product: *mut c_int,
    solve: fn(&[crate::Command]) -> i32,
) -> c_int {
    if out_product.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(day) = Day::from_str(text) else {
        return -1;
    };

    // The parse above can't panic (every fallible step returns a Result), so
    // only the arithmetic is wrapped.
    let Ok(product) = catch_unwind(AssertUnwindSafe(|| solve(&day))) else {
        return -3;
    };

    unsafe { *out_product = product };
    0
}

/// Parses `input` and writes `horizontal * depth` — part one's answer — into
/// `*out_product`.
///
/// Returns `0` on success, `-1` if `input`/`out_product` is null or `input`
/// isn't valid UTF-8 or isn't a valid course, `-3` if the course overflows an
/// `int32_t`.
///
/// # Safety
/// `input` must point to a NUL-terminated C string. `out_product` must point
/// to writable memory for one `int32_t`. Both must stay valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2021_12_02_part1(
    input: *const c_char,
    out_product: *mut c_int,
) -> c_int {
    solve_into(input, out_product, dead_reckon_pure_rust)
}

/// Parses `input` and writes part two's answer — the same product, with
/// `down`/`up` treated as aim adjustments — into `*out_product`.
///
/// # Safety
/// Same contract as [`aoc_2021_12_02_part1`], for `out_product`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2021_12_02_part2(
    input: *const c_char,
    out_product: *mut c_int,
) -> c_int {
    solve_into(input, out_product, dead_reckon_with_aim_pure_rust)
}
