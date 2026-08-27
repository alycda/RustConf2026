//! The C-facing surface of this crate (Exercise 2). Every other FFI module in
//! this repo calls *into* a C library from Rust; this is Rust exposing
//! *itself* to C. `cbindgen` (see `cbindgen.toml`) turns the `extern "C"`
//! functions below into a header; any language with a C FFI can then load the
//! compiled `cdylib` and call straight in — Swift in Exercise 3, or plain C.
//!
//! Built on the `checked_sum_calibration_*` functions specifically, not on
//! whatever `Solution::part1`/`part2` route to. This day has no exotic-library
//! variant yet and will grow one; Exercise 2 is about the export *direction*,
//! and tying the header's meaning to a cargo feature would muddy both.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic
//! unwinding across an `extern "C"` frame is undefined behavior, so nothing
//! here can panic — bad input (a null pointer, invalid UTF-8) is a real
//! possibility from a C caller and is handled as data, not asserted away.
//!
//! The panic that mattered on this day was not a hypothetical one. Until the
//! commit before this module existed, `calibration_value` walked the line by
//! byte offset and sliced at each one, so any line with a multi-byte
//! character panicked — and `é1` is valid UTF-8, which is exactly the input
//! [`read_input`] promises to *accept*. The Rust binary could never reach it
//! (real puzzle inputs are ASCII); a C caller reaches it by typing.

use std::ffi::{CStr, c_char, c_int, c_uint};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use crate::{Day, checked_sum_calibration_pure_rust, checked_sum_calibration_with_words_pure_rust};

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

/// Shared body of both entry points: parse, sum, write the answer.
///
/// `solve` returns `Option` because the header promises `-2` for a total too
/// large for a `uint32_t`, and a promise kept by "the arithmetic would have
/// panicked" is only kept where `overflow-checks` is on — the dev profile,
/// not the release one, and `days/Cargo.toml` overrides neither. That was
/// 2021-12-02's lesson, arrived at by shipping the broken version first; this
/// day starts from the answer.
///
/// It is worth being honest about the difference between the two days,
/// though. There, a genuine puzzle input landed within ~10% of `i32::MAX` and
/// `forward 100000\ndown 100000` overflowed it — the `-3` was a status code
/// callers would actually see, and there is a test that sees it. Here a line
/// is worth at most 99, so `-2` needs ~43 million lines, at least 86 MB of
/// input. The arithmetic behind it is pinned (`crate::tests::
/// the_total_refuses_to_wrap`); this path through the FFI boundary is not,
/// because constructing the input costs more than the guard is worth. An
/// FFI contract you cannot afford to exercise is a weaker promise than one
/// you can, and saying so here is cheaper than discovering it later.
///
/// The `catch_unwind` is a backstop, not the guard, and it folds into the
/// same `-2` — same shape as 2021-12-02's `-3`. Nothing on this path panics:
/// the parse is infallible, the scan is by character, the arithmetic is
/// checked. But this is an `extern "C"` frame, where being wrong about that
/// costs undefined behavior rather than a bad answer, and a caller that
/// somehow got here has learned the only thing the code can honestly tell
/// it — no answer, don't read `*out_value`.
fn solve_into(
    input: *const c_char,
    out_value: *mut c_uint,
    solve: fn(&[String]) -> Option<u32>,
) -> c_int {
    if out_value.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(day) = Day::from_str(text) else {
        return -1;
    };

    let Ok(Some(total)) = catch_unwind(AssertUnwindSafe(|| solve(&day))) else {
        return -2;
    };

    unsafe { *out_value = total };
    0
}

/// Parses `input` (one calibration line per line) and writes the sum of the
/// calibration values — part one's answer, literal digits only — into
/// `*out_value`.
///
/// Returns `0` on success, `-1` if `input`/`out_value` is null or `input`
/// isn't valid UTF-8, `-2` if the total doesn't fit in a `uint32_t`.
///
/// # Safety
/// `input` must point to a NUL-terminated C string. `out_value` must point to
/// writable memory for one `uint32_t`. Both must stay valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2023_12_01_part1(
    input: *const c_char,
    out_value: *mut c_uint,
) -> c_int {
    solve_into(input, out_value, checked_sum_calibration_pure_rust)
}

/// Parses `input` and writes part two's answer — the same sum, with
/// spelled-out digits (`one` through `nine`, overlaps included) counting too
/// — into `*out_value`.
///
/// # Safety
/// Same contract as [`aoc_2023_12_01_part1`], for `out_value`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2023_12_01_part2(
    input: *const c_char,
    out_value: *mut c_uint,
) -> c_int {
    solve_into(
        input,
        out_value,
        checked_sum_calibration_with_words_pure_rust,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Both entry points against the puzzle's own examples, through the C
    /// boundary rather than through the Rust functions they wrap — the trip
    /// out is the thing this module adds, so it is the thing to test.
    #[test]
    fn the_examples_survive_the_round_trip() {
        let part1 = CString::new("1abc2\npqr3stu8vwx\na1b2c3d4e5f\ntreb7uchet").expect("no NULs");
        let mut answer: c_uint = 0;
        // SAFETY: `part1` is a live NUL-terminated string and `answer` is
        // writable for one `c_uint`; both outlive the call.
        assert_eq!(
            unsafe { aoc_2023_12_01_part1(part1.as_ptr(), &mut answer) },
            0
        );
        assert_eq!(answer, 142);

        let part2 = CString::new(
            "two1nine\neightwothree\nabcone2threexyz\nxtwone3four\n4nineeightseven2\nzoneight234\n7pqrstsixteen",
        )
        .expect("no NULs");
        let mut answer: c_uint = 0;
        // SAFETY: as above.
        assert_eq!(
            unsafe { aoc_2023_12_01_part2(part2.as_ptr(), &mut answer) },
            0
        );
        assert_eq!(answer, 281);
    }

    /// A null `input` and a null `out_value` are both `-1`, and neither
    /// writes anything. A C caller producing one of these is ordinary, not
    /// exceptional — it is what an unchecked `malloc` or a missing file looks
    /// like from the other side.
    #[test]
    fn nulls_are_refused_rather_than_dereferenced() {
        let mut answer: c_uint = 7;
        // SAFETY: a null `input` is explicitly part of this function's
        // contract; `answer` is writable for one `c_uint`.
        assert_eq!(
            unsafe { aoc_2023_12_01_part1(std::ptr::null(), &mut answer) },
            -1
        );
        assert_eq!(answer, 7, "out_value must be left alone when we refuse");

        let text = CString::new("1abc2").expect("no NULs");
        // SAFETY: `text` is a live NUL-terminated string; a null `out_value`
        // is explicitly part of the contract.
        assert_eq!(
            unsafe { aoc_2023_12_01_part2(text.as_ptr(), std::ptr::null_mut()) },
            -1
        );
    }

    /// Valid UTF-8 that is not ASCII. This is the case the commit before this
    /// module fixed, and it is here rather than only in `lib.rs` because the
    /// difference between the two is the whole reason it was worth fixing:
    /// in `lib.rs` a panic is a backtrace, and across this frame it is UB.
    #[test]
    fn a_multibyte_input_is_answered_not_a_panic() {
        let text = CString::new("1é9\nfourété2").expect("no NULs");
        let mut answer: c_uint = 0;
        // SAFETY: `text` is a live NUL-terminated string and `answer` is
        // writable for one `c_uint`.
        assert_eq!(
            unsafe { aoc_2023_12_01_part2(text.as_ptr(), &mut answer) },
            0
        );
        assert_eq!(answer, 19 + 42);
    }
}
