//! Native side: sum the ASCII digit values in a UTF-8 C string.
//!
//! CORRECT and total: null -> -1, otherwise sum every '0'..='9' byte's value.
//! e.g. "1234" -> 1+2+3+4 = 10. The flaw is on the binding side, which builds
//! a C string that contains an interior NUL and so gets silently truncated.

use std::ffi::{c_char, CStr};

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn sum_digits(input: *const c_char) -> i64 {
    if input.is_null() {
        return -1;
    }
    // from_ptr stops at the FIRST NUL — that is exactly the truncation the
    // binding will trip over.
    let bytes = CStr::from_ptr(input).to_bytes();
    bytes
        .iter()
        .filter(|b| b.is_ascii_digit())
        .map(|b| (b - b'0') as i64)
        .sum()
}
