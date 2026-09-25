//! Native side: count occurrences of 'é' (U+00E9) in a UTF-8 C string.
//!
//! This side is CORRECT. It decodes the incoming bytes as UTF-8 (lossily, so
//! malformed input never crashes — it just can't contain 'é') and counts the
//! character. The contract, stated plainly: **input must be UTF-8.** The Day 3
//! boundary makes the same promise (`days/2024-03/c-glue/src/lib.rs`).
//!
//! The planted flaw is on the binding side: it sends the wrong encoding.

use std::ffi::{c_char, CStr};

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn count_e_acute(input: *const c_char) -> i64 {
    if input.is_null() {
        return -1;
    }
    let bytes = CStr::from_ptr(input).to_bytes();
    // Lossy so invalid bytes become U+FFFD instead of an error — no crash,
    // and a mis-encoded 'é' simply won't be found.
    let text = String::from_utf8_lossy(bytes);
    text.chars().filter(|&c| c == 'é').count() as i64
}
