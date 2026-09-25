//! The C-facing surface of this crate — Exercise 2: Rust exposing *itself*
//! to C. `cbindgen` (see `cbindgen.toml`) turns the `extern "C"` functions
//! below into a header; any language with a C FFI can then load the compiled
//! `cdylib` and call straight in.
//!
//! Plain status codes and out-parameters, not `Result`: a Rust panic that
//! reaches an `extern "C"` frame aborts the process (Rust 1.81 and later —
//! before that it was undefined behavior), so nothing here may panic — a null
//! pointer, invalid UTF-8, a line that is not an instruction the grid can
//! hold, and a total too large for the out-parameter are all real
//! possibilities from a C caller, and all of them are handled as data. The
//! codes are the repo-wide table in `days/README.md` ("C API status codes").
//!
//! Built on [`Day::run`] with the same per-light rules `Solution::part1` and
//! `part2` pass it ([`switch`], [`dim`]), not on `Solution::solve` itself:
//! `solve` folds every failure into one `miette::Error` and renders the
//! answer as a `String`, and at this boundary a line that is not an
//! instruction and a total that overflows need different codes.

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

use crate::{Action, Day, dim, switch};

// The repo-wide status codes (days/README.md, "C API status codes").
const INVALID_INPUT: c_int = -1;
const OVERFLOW: c_int = -3;
const INTERNAL: c_int = -4;

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

/// Shared body of both entry points: parse `input` (one instruction per
/// line), run the grid under `rule`, and write the total into `*out_value`.
///
/// The total arrives as a `u64` and is narrowed with `try_from`, so a total
/// past `u32::MAX` is `-3`. That is reachable: 2,148 lines of `toggle 0,0
/// through 999,999` (about 58 KB) take part 2's brightness past it, and until
/// this split that input came back as `-1`, a claim that a line was not an
/// instruction.
///
/// The `catch_unwind` is a backstop, not the guard. Nothing on this path
/// panics for an input a C caller can realistically send: the parse returns
/// `Err` at every fallible step, and a single light's brightness would need
/// over two billion instructions (tens of gigabytes) to leave the `u32` that
/// [`dim`] adds to unchecked. That last one is named rather than fixed, and
/// if it is ever reached with `overflow-checks` on, the caller gets `-4`
/// rather than an abort.
///
/// # Safety
/// The contract of [`aoc_2015_12_06_part1`]. It dereferences both pointers,
/// so it is an `unsafe fn` even though it is not exported.
unsafe fn solve(
    input: *const c_char,
    out_value: *mut c_uint,
    rule: fn(Action, u32) -> u32,
) -> c_int {
    if out_value.is_null() {
        return INVALID_INPUT;
    }
    let Some(text) = (unsafe { read_input(input) }) else {
        return INVALID_INPUT;
    };
    let Ok(day) = Day::from_str(text) else {
        return INVALID_INPUT;
    };

    let Ok(total) = catch_unwind(AssertUnwindSafe(|| day.run(rule))) else {
        return INTERNAL;
    };
    let Ok(value) = c_uint::try_from(total) else {
        return OVERFLOW;
    };
    unsafe { *out_value = value };
    0
}

/// Parses `input` and writes the number of lights left on into `*out_value`.
///
/// Returns `0` on success, `-1` if `input`/`out_value` is null, `input` isn't
/// valid UTF-8, or a line isn't an instruction the grid can hold, `-4` if the
/// computation panicked. On any nonzero return `*out_value` is left
/// untouched. (Part 1 counts lights, at most 1,000,000, so it cannot return
/// `-3`.)
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
    // SAFETY: the caller's contract is exactly `solve`'s.
    unsafe { solve(input, out_value, switch) }
}

/// Parses `input` and writes the total brightness into `*out_value`.
///
/// Returns `0` on success, `-1` on the same conditions as part 1, `-3` if
/// the total brightness doesn't fit in a `uint32_t`, `-4` if the computation
/// panicked. On any nonzero return `*out_value` is left untouched.
///
/// # Safety
/// Same contract as [`aoc_2015_12_06_part1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_06_part2(
    input: *const c_char,
    out_value: *mut c_uint,
) -> c_int {
    // SAFETY: the caller's contract is exactly `solve`'s.
    unsafe { solve(input, out_value, dim) }
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

    /// A rule that sets every light it touches to `u32::MAX`, so one
    /// full-grid instruction totals about 4.3 × 10^15. It reaches the
    /// overflow a C caller reaches with 58 KB of `toggle`s, at a millionth
    /// of the cost.
    fn saturating_rule(_action: Action, _light: u32) -> u32 {
        u32::MAX
    }

    fn panicking_rule(_action: Action, _light: u32) -> u32 {
        panic!("a rule an attendee edited into something fallible");
    }

    /// A total past `u32::MAX` is `-3`, not the `-1` it used to share with
    /// "not an instruction", and the out-parameter is left alone.
    #[test]
    fn a_total_past_uint32_reports_minus_three() {
        let text = CString::new("turn on 0,0 through 999,999").expect("no NULs");
        let mut answer: c_uint = 7;
        // SAFETY: `text` is a live NUL-terminated string and `answer` is
        // writable for one `c_uint`; both outlive the call.
        let status = unsafe { solve(text.as_ptr(), &raw mut answer, saturating_rule) };

        assert_eq!(status, -3, "the total does not fit a uint32_t");
        assert_eq!(answer, 7, "out_value must be left alone on overflow");
    }

    /// The backstop, exercised by injecting the panic, because neither
    /// shipped rule panics on any input a caller can afford to send.
    #[test]
    fn a_panicking_rule_reports_minus_four() {
        let text = CString::new("turn on 0,0 through 0,0").expect("no NULs");
        let mut answer: c_uint = 7;
        // SAFETY: as above.
        let status = without_panic_noise(|| unsafe {
            solve(text.as_ptr(), &raw mut answer, panicking_rule)
        });

        assert_eq!(status, -4, "a caught panic must arrive as the status code");
        assert_eq!(answer, 7, "out_value must be left alone when we refuse");
    }

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
            unsafe { aoc_2015_12_06_part1(part1.as_ptr(), &raw mut answer) },
            0
        );
        assert_eq!(answer, 998_996);

        let part2 =
            CString::new("turn on 0,0 through 0,0\ntoggle 0,0 through 999,999").expect("no NULs");
        let mut answer: c_uint = 0;
        // SAFETY: as above.
        assert_eq!(
            unsafe { aoc_2015_12_06_part2(part2.as_ptr(), &raw mut answer) },
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
            unsafe { aoc_2015_12_06_part1(std::ptr::null(), &raw mut answer) },
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
    /// would abort the process rather than print a backtrace.
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
                unsafe { aoc_2015_12_06_part1(text.as_ptr(), &raw mut answer) },
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
            unsafe { aoc_2015_12_06_part1(text.as_ptr(), &raw mut answer) },
            -1
        );
    }
}
