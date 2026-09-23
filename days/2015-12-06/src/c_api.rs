//! The C-facing surface of this crate — Exercise 2: Rust exposing *itself*
//! to C. `cbindgen` (see `cbindgen.toml`) turns the `extern "C"` functions
//! below into a header; any language with a C FFI can then load the compiled
//! `cdylib` and call straight in.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic
//! unwinding across an `extern "C"` frame is undefined behavior, so nothing
//! here can panic — a null pointer, invalid UTF-8, or a line that is not an
//! instruction the grid can hold are all real possibilities from a C caller,
//! and all of them are handled as data.

use std::ffi::{CStr, c_char, c_int, c_uint};
use std::str::FromStr;

use aoc_ornaments::{Part, Solution};

use crate::Day;

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

/// Parses `input` (one instruction per line), runs `part`, and writes the
/// answer into `*out_value`. Returns `0` on success, `-1` if either pointer
/// is null, `input` isn't valid UTF-8, or a line isn't an instruction the
/// 1000×1000 grid can hold.
///
/// # Safety
/// The contract of [`aoc_2015_12_06_part1`].
unsafe fn solve(input: *const c_char, out_value: *mut c_uint, part: Part) -> c_int {
    if out_value.is_null() {
        return -1;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return -1;
    };
    let Ok(mut day) = Day::from_str(text) else {
        return -1;
    };
    let Ok(answer) = day.solve(part) else {
        return -1;
    };
    // `solve` renders the answer for display; the value itself is the crate's
    // `Output`, a u32, so this parse cannot fail for anything `part1`/`part2`
    // produce — and if it ever did, the caller gets a -1, not a panic.
    let Ok(value) = answer.parse::<c_uint>() else {
        return -1;
    };
    unsafe { *out_value = value };
    0
}

/// Parses `input` and writes the number of lights left on into `*out_value`.
///
/// Returns `0` on success, `-1` if `input`/`out_value` is null, `input` isn't
/// valid UTF-8, or a line isn't an instruction the grid can hold.
///
/// # Safety
/// `input` must be null or a NUL-terminated C string. `out_value` must be
/// null or point to writable memory for one `uint32_t`. Both must stay valid
/// for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_06_part1(
    input: *const c_char,
    out_value: *mut c_uint,
) -> c_int {
    unsafe { solve(input, out_value, Part::One) }
}

/// Parses `input` and writes the total brightness into `*out_value`.
///
/// Returns `0` on success, `-1` on the same conditions as part 1.
///
/// # Safety
/// Same contract as [`aoc_2015_12_06_part1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_06_part2(
    input: *const c_char,
    out_value: *mut c_uint,
) -> c_int {
    unsafe { solve(input, out_value, Part::Two) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Both entry points against the statement's examples, through the C
    /// boundary rather than through the Rust functions they wrap — the trip
    /// out is the thing this module adds, so it is the thing to test.
    #[test]
    fn the_examples_survive_the_round_trip() {
        let part1 = CString::new(
            "turn on 0,0 through 999,999\ntoggle 0,0 through 999,0\nturn off 499,499 through 500,500",
        )
        .expect("no NULs");
        let mut answer: c_uint = 0;
        // SAFETY: `part1` is a live NUL-terminated string and `answer` is
        // writable for one `c_uint`; both outlive the call.
        assert_eq!(
            unsafe { aoc_2015_12_06_part1(part1.as_ptr(), &mut answer) },
            0
        );
        assert_eq!(answer, 998_996);

        let part2 =
            CString::new("turn on 0,0 through 0,0\ntoggle 0,0 through 999,999").expect("no NULs");
        let mut answer: c_uint = 0;
        // SAFETY: as above.
        assert_eq!(
            unsafe { aoc_2015_12_06_part2(part2.as_ptr(), &mut answer) },
            0
        );
        assert_eq!(answer, 2_000_001);
    }

    /// A null `input` and a null `out_value` are both `-1`, and neither
    /// writes anything.
    #[test]
    fn nulls_are_refused_rather_than_dereferenced() {
        let mut answer: c_uint = 7;
        // SAFETY: a null `input` is explicitly part of this function's
        // contract; `answer` is writable for one `c_uint`.
        assert_eq!(
            unsafe { aoc_2015_12_06_part1(std::ptr::null(), &mut answer) },
            -1
        );
        assert_eq!(answer, 7, "out_value must be left alone when we refuse");

        let text = CString::new("turn on 0,0 through 0,0").expect("no NULs");
        // SAFETY: `text` is a live NUL-terminated string; a null `out_value`
        // is explicitly part of the contract.
        assert_eq!(
            unsafe { aoc_2015_12_06_part2(text.as_ptr(), std::ptr::null_mut()) },
            -1
        );
    }

    /// An instruction the grid cannot hold is a `-1` from the C side, where
    /// in Rust it is an `Err` — and never a panic, which across this frame
    /// would be undefined behavior rather than a backtrace.
    #[test]
    fn a_bad_instruction_is_a_status_not_a_panic() {
        let mut answer: c_uint = 7;
        for bad in [
            "flip 0,0 through 1,1",
            "turn on 1000,0 through 1,1",
            "toggle 5,5 through 4,4",
        ] {
            let text = CString::new(bad).expect("no NULs");
            // SAFETY: `text` is a live NUL-terminated string and `answer` is
            // writable for one `c_uint`.
            assert_eq!(
                unsafe { aoc_2015_12_06_part1(text.as_ptr(), &mut answer) },
                -1,
                "{bad:?}"
            );
            assert_eq!(answer, 7, "out_value must be left alone on {bad:?}");
        }
    }

    /// Valid UTF-8 that is not ASCII reaches the parser as text and is
    /// refused as an instruction, not as bytes.
    #[test]
    fn a_multibyte_input_is_answered_not_a_panic() {
        let text = CString::new("allumer 0,0 through 1,1").expect("no NULs");
        let mut answer: c_uint = 0;
        // SAFETY: `text` is a live NUL-terminated string and `answer` is
        // writable for one `c_uint`.
        assert_eq!(
            unsafe { aoc_2015_12_06_part1(text.as_ptr(), &mut answer) },
            -1
        );
    }
}
