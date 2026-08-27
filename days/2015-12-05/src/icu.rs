//! Day 5 solved with ICU's regex engine (`uregex.h`, part of libicui18n) —
//! a full Unicode-aware backtracking regex engine, with capture groups,
//! backreferences, and locale/collation/normalization machinery it never
//! touches here, pointed at a few hundred lowercase-ASCII 16-character
//! lines. See days/2015-12-05/README.md.
//!
//! Real backreference support changes the *shape* of the solution, not
//! just the tool: every rule is one short pattern, no enumeration. The
//! `vectorscan` sibling of this commit needed ~700 literal patterns (26 for
//! double letters, 676 for repeated pairs) specifically because Hyperscan's
//! vectorized model can't express "whatever matched earlier" — ICU can, so
//! `has_double_letter` is the four characters `(.)\1`, and the
//! non-overlapping-pair rule that needed manual offset bookkeeping over
//! there is `(..).*\1` here, handled by the engine's own backtracking.
//!
//! Calls into `src/icu_shim.c`, not straight into libicui18n — see that
//! file's header comment for why (ICU's C symbols are version-renamed at
//! link time; the shim exports names that don't change across ICU
//! versions).

use std::ffi::{CString, c_char, c_int};

unsafe extern "C" {
    fn aoc_icu_regex_count(pattern: *const c_char, text: *const c_char) -> c_int;
    fn aoc_icu_regex_find(pattern: *const c_char, text: *const c_char) -> c_int;
}

fn regex_count(pattern: &str, text: &str) -> i32 {
    let pattern = CString::new(pattern).expect("pattern has no NUL byte");
    let text = CString::new(text).expect("line has no NUL byte");
    let result = unsafe { aoc_icu_regex_count(pattern.as_ptr(), text.as_ptr()) };
    assert!(
        result >= 0,
        "ICU regex error counting matches of {pattern:?}"
    );
    result
}

fn regex_find(pattern: &str, text: &str) -> bool {
    let pattern = CString::new(pattern).expect("pattern has no NUL byte");
    let text = CString::new(text).expect("line has no NUL byte");
    let result = unsafe { aoc_icu_regex_find(pattern.as_ptr(), text.as_ptr()) };
    assert!(result >= 0, "ICU regex error matching {pattern:?}");
    result == 1
}

/// Check if a line is nice — the original rules — via ICU regex. See the
/// module docs and [`crate::is_nice_pure_rust`].
pub fn is_nice_via_icu(line: &str) -> bool {
    !regex_find("ab|cd|pq|xy", line)
        && regex_find(r"(.)\1", line)
        && regex_count("[aeiou]", line) >= 3
}

/// Check if a line is nice under the part-2 rules via ICU regex. See the
/// module docs and [`crate::is_nice_v2_pure_rust`].
pub fn is_nice_v2_via_icu(line: &str) -> bool {
    regex_find(r"(..).*\1", line) && regex_find(r"(.).\1", line)
}
