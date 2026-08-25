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
//! These are built on the `checked_dead_reckon_*` functions specifically, not
//! on whichever backend `Solution::part1`/`part2` currently
//! route to. Exercise 2 is about the export *direction*; entangling it with
//! "does this build have the physics engine" would muddy both, and would make
//! the header's meaning depend on a cargo feature.

use std::ffi::{CStr, c_char, c_int};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use crate::{Day, checked_dead_reckon_pure_rust, checked_dead_reckon_with_aim_pure_rust};

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
/// The overflow status code is the one place this day departs from
/// 2015-12-01's otherwise identical `c_api`. That day summed ±1 and could not
/// realistically overflow; this one multiplies horizontal by depth, and part 2
/// on a genuine puzzle input already lands within ~10% of `i32::MAX`. An input
/// a little larger — trivially constructed by a C caller, who is not
/// restricted to real puzzle inputs — leaves the `i32`, and this has to say so
/// rather than write a wrapped number into `*out_product`.
///
/// `solve` therefore returns `Option`, and the arithmetic behind it is
/// `checked_*`. The tempting alternative — let the overflow panic and stop the
/// unwind with `catch_unwind` — was what this used to do, and it was wrong in
/// three ways worth naming, because each is a way for a guard to look present
/// and not be:
///
/// - **It was off in release.** Overflow only panics where `overflow-checks`
///   is enabled, which is the dev profile's default and not the release
///   profile's, and `days/Cargo.toml` overrides neither. The header promised
///   `-3` unconditionally; the build most likely to be shipped didn't deliver
///   it, and returned `0` and a wrapped answer instead.
/// - **It assumed unwinding.** A profile with `panic = "abort"` kills the
///   process before `catch_unwind` is reached. Arithmetic that cannot overflow
///   in the first place is the only guard that doesn't care.
/// - **It stopped the unwind, not the noise.** Rust's default hook prints
///   `attempt to multiply with overflow` to stderr before unwinding, so a C
///   caller getting a tidy `-3` also got Rust diagnostics it never asked for,
///   on what is an ordinary error rather than a bug.
///
/// The `catch_unwind` stays anyway. Nothing on this path panics now — the
/// parse returns `Result` at every fallible step and the arithmetic is
/// checked — but this is an `extern "C"` frame, where being wrong about that
/// costs undefined behavior rather than a bad answer.
fn solve_into(
    input: *const c_char,
    out_product: *mut c_int,
    solve: fn(&[crate::Command]) -> Option<i32>,
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

    let Ok(Some(product)) = catch_unwind(AssertUnwindSafe(|| solve(&day))) else {
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
    solve_into(input, out_product, checked_dead_reckon_pure_rust)
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
    solve_into(input, out_product, checked_dead_reckon_with_aim_pure_rust)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// The `-3` contract, exercised through the C entry point rather than
    /// through `checked_dead_reckon_*`, because the thing that regressed was
    /// never the arithmetic — it was whether the status code survived the
    /// trip out. Runs identically in dev and release, which is the whole
    /// point: the guard this replaced only existed where `overflow-checks`
    /// was on, so an equivalent test would have passed in dev while the
    /// shipped `cdylib` wrote a wrapped number and returned `0`.
    #[test]
    fn an_overflowing_course_reports_minus_three() {
        // 4 × 2^30 forward and 4 × 2^30 down: each axis is 2^32, so both the
        // fold and the multiply leave the i32, and no single command does.
        let quarter = 1 << 30;
        let course: String = std::iter::repeat_n(format!("forward {quarter}\n"), 4)
            .chain(std::iter::repeat_n(format!("down {quarter}\n"), 4))
            .collect();
        let course = CString::new(course).expect("no NUL bytes in a generated course");

        let mut answer: c_int = 0;
        // SAFETY: `course` is a live NUL-terminated string and `answer` is
        // writable for one `c_int`; both outlive the call.
        let status = unsafe { aoc_2021_12_02_part1(course.as_ptr(), &mut answer) };

        assert_eq!(status, -3, "expected the overflow status code");
        assert_eq!(answer, 0, "out_product must be left alone when we refuse");
    }

    /// The ordinary path, as the counterweight: the same entry point still
    /// answers a course that fits.
    #[test]
    fn a_course_that_fits_is_answered() {
        let course = CString::new("forward 5\ndown 5\nforward 8\nup 3\ndown 8\nforward 2\n")
            .expect("no NUL bytes");

        let mut answer: c_int = 0;
        // SAFETY: as above.
        let status = unsafe { aoc_2021_12_02_part1(course.as_ptr(), &mut answer) };

        assert_eq!(status, 0);
        assert_eq!(answer, 150);
    }
}
