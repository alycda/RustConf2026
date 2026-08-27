//! The puzzle, through a malware-scanning engine.
//!
//! YARA exists to sweep gigabytes of suspect binaries against thousands of
//! rules, each holding dozens of byte patterns, and report every hit with its
//! offset. This module points it at one 21-character line of `heightseven4two5`
//! and asks the same question a thousand times.
//!
//! The joke is entirely one of scale, because the *semantics* are exact. Part
//! two's rule — spelled-out digits count, and they may overlap, so `oneight`
//! is a `1` and an `8` sharing an `e` — is not a special case here. It is
//! simply what a multi-pattern scanner does: YARA reports every occurrence of
//! every string, and two strings that overlap are two occurrences. Where the
//! plain-Rust version scans forward one character at a time and tries nine
//! words at each stop, this states nineteen patterns once and lets an
//! Aho-Corasick automaton built for threat intelligence find them all.
//!
//! ```text
//! oneight   ->  $w1 @ 0   $w8 @ 2
//! twone     ->  $w2 @ 0   $w1 @ 2
//! 999       ->  $d9 @ 0   $d9 @ 1   $d9 @ 2
//! ```
//!
//! Two things the scratchpad found before any of this was written, both of
//! which shape the code below:
//!
//! **Matches arrive grouped by string, not sorted by offset.** `eightwothree`
//! reports `$w2 @ 4`, `$w3 @ 7`, `$w8 @ 0` — in that order. A first/last taken
//! from the reported sequence would be wrong on most lines and right on enough
//! of them to look correct. [`Scanner::calibration_value`] takes a minimum and
//! a maximum by offset instead.
//!
//! **The match-reading API is macros, so it cannot be bound.** See
//! `yara_shim.c` for why the iteration lives in C and only flat integers cross
//! back.

use std::ffi::{CString, c_char, c_int};
use std::sync::Once;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::WORDS;

/// Opaque handles. Every one of these is a pointer YARA hands back and takes
/// again; nothing here needs to know what is inside one, which is what keeps
/// this binding independent of how libyara was configured.
#[repr(C)]
struct YrCompiler {
    _private: [u8; 0],
}

#[repr(C)]
struct YrRules {
    _private: [u8; 0],
}

/// Mirrors `aoc_yara_match` in `yara_shim.c`. Two plain integers — the only
/// struct that crosses this boundary, and one this repo defines on both sides
/// rather than borrowing from YARA's headers.
#[repr(C)]
#[derive(Clone, Copy)]
struct Match {
    offset: u64,
    string_index: u32,
}

unsafe extern "C" {
    fn yr_initialize() -> c_int;
    fn yr_compiler_create(compiler: *mut *mut YrCompiler) -> c_int;
    fn yr_compiler_add_string(
        compiler: *mut YrCompiler,
        string: *const c_char,
        namespace_: *const c_char,
    ) -> c_int;
    fn yr_compiler_get_rules(compiler: *mut YrCompiler, rules: *mut *mut YrRules) -> c_int;
    fn yr_compiler_destroy(compiler: *mut YrCompiler);
    fn yr_rules_destroy(rules: *mut YrRules) -> c_int;

    /// Ours, from `yara_shim.c` — see that file for why it has to exist.
    fn aoc_yara_collect(
        rules: *mut YrRules,
        data: *const u8,
        length: usize,
        out: *mut Match,
        cap: usize,
        out_count: *mut usize,
    ) -> c_int;
}

/// `yr_initialize` is process-global and must happen before anything else in
/// the library; calling it twice is an error. `Once` is what makes that safe
/// when `cargo test` runs several of these in parallel threads.
///
/// There is no matching `yr_finalize`. A library that is initialised once and
/// used until the process exits has nothing to tear down that the process exit
/// will not tear down anyway, and a `Drop`-based counter would have to be
/// correct under exactly the concurrency this `Once` exists to handle. The
/// leak is bounded, deliberate, and stated rather than hidden.
static INIT: Once = Once::new();

/// What `yr_initialize` returned, for every later caller to check.
///
/// An `AtomicI32` rather than a `static mut`: `Once` already orders the write
/// before every later read, so a `static mut` would be *sound* here — but it
/// still has to be read through a reference to be formatted into the error
/// message, and edition 2024 rejects that outright (`static_mut_refs`). The
/// atomic says the same thing and doesn't need the argument.
static INIT_STATUS: AtomicI32 = AtomicI32::new(0);

fn initialize() -> miette::Result<()> {
    INIT.call_once(|| {
        // SAFETY: `yr_initialize` takes no arguments, is documented as the
        // first call into the library, and `Once` guarantees exactly one
        // thread reaches it once.
        let status = unsafe { yr_initialize() };
        INIT_STATUS.store(status, Ordering::Release);
    });

    let status = INIT_STATUS.load(Ordering::Acquire);
    if status != 0 {
        return Err(miette::miette!("yr_initialize failed ({status})"));
    }
    Ok(())
}

/// A compiled rule set plus a scratch buffer, owned so the C resources are
/// released even if a later step fails.
///
/// One per solve, like 2021-12-02's `cpSpace` and its DuckDB connection —
/// compiling the rules is ~0.4 ms and scanning a whole puzzle input is ~1.4
/// ms, so this is a day where the setup is a third of the bill. `benches/`
/// separates the two rather than reporting the sum.
pub struct Scanner {
    rules: *mut YrRules,
    /// Reused across lines. A line can produce at most one match per byte —
    /// no two of the nine words are a prefix of one another, and a digit and a
    /// word can never start at the same position — so the longest line in the
    /// input is a hard upper bound, not a guess.
    buffer: Vec<Match>,
}

impl Scanner {
    /// Compiles the rule set: the ten digits always, the nine words only for
    /// part two.
    ///
    /// Declaration order is load-bearing. YARA's `YR_STRING.idx` is the
    /// string's position in the compiled rule, and that index is all the shim
    /// sends back — so `$d0..$d9` occupy 0..=9 and `$w1..$w9` occupy 10..=18,
    /// and [`Self::value_of`] inverts exactly that. Reordering the strings
    /// below without touching `value_of` would produce wrong answers with no
    /// error anywhere, which is why the two live next to each other.
    pub fn new(words: bool, longest_line: usize) -> miette::Result<Self> {
        initialize()?;

        let mut source = String::from("rule calibration {\n  strings:\n");
        for digit in 0..=9 {
            source.push_str(&format!("    $d{digit} = \"{digit}\"\n"));
        }
        if words {
            for (word, value) in WORDS {
                source.push_str(&format!("    $w{value} = \"{word}\"\n"));
            }
        }
        source.push_str("  condition:\n    any of them\n}\n");

        let source = CString::new(source).map_err(|e| miette::miette!("rule text: {e}"))?;

        let mut compiler: *mut YrCompiler = std::ptr::null_mut();
        // SAFETY: `yr_initialize` has succeeded, and `compiler` is a valid
        // out-parameter for one pointer.
        let status = unsafe { yr_compiler_create(&mut compiler) };
        if status != 0 || compiler.is_null() {
            return Err(miette::miette!("yr_compiler_create failed ({status})"));
        }

        // From here on the compiler must be destroyed on every path, so the
        // rest is a closure whose result is inspected after the cleanup.
        let compiled = (|| {
            // SAFETY: `compiler` is live, `source` is a NUL-terminated C
            // string that outlives the call, and a null namespace is
            // documented as "the default namespace".
            let errors =
                unsafe { yr_compiler_add_string(compiler, source.as_ptr(), std::ptr::null()) };
            if errors != 0 {
                return Err(miette::miette!(
                    "the rule text did not compile ({errors} errors)"
                ));
            }

            let mut rules: *mut YrRules = std::ptr::null_mut();
            // SAFETY: as above; `rules` is a valid out-parameter.
            let status = unsafe { yr_compiler_get_rules(compiler, &mut rules) };
            if status != 0 || rules.is_null() {
                return Err(miette::miette!("yr_compiler_get_rules failed ({status})"));
            }
            Ok(rules)
        })();

        // SAFETY: `compiler` is live and has not been destroyed. The rules
        // produced by `yr_compiler_get_rules` outlive their compiler — that is
        // the documented ownership split, and the reason this is safe here
        // rather than at the end of the solve.
        unsafe { yr_compiler_destroy(compiler) };

        Ok(Self {
            rules: compiled?,
            buffer: vec![
                Match {
                    offset: 0,
                    string_index: 0
                };
                longest_line
            ],
        })
    }

    /// Inverts the declaration order set up in [`Self::new`].
    fn value_of(string_index: u32) -> Option<u32> {
        match string_index {
            0..=9 => Some(string_index),
            // $w1 is index 10, so the words run 10..=18 and carry 1..=9.
            10..=18 => Some(string_index - 9),
            _ => None,
        }
    }

    /// The first and last digit on one line, via one scan.
    ///
    /// A line with no digits is `0`, matching the plain-Rust version — this
    /// day's baseline treats a digitless line as contributing nothing rather
    /// than as an error, and a backend that disagreed would be answering a
    /// different puzzle.
    pub fn calibration_value(&mut self, line: &str) -> miette::Result<u32> {
        if line.is_empty() {
            return Ok(0);
        }
        if line.len() > self.buffer.len() {
            return Err(miette::miette!(
                "line of {} bytes exceeds the {}-byte scratch buffer this scanner was built for",
                line.len(),
                self.buffer.len()
            ));
        }

        let mut count: usize = 0;
        // SAFETY: `rules` is live for `self`'s lifetime; `line` is a live
        // byte slice of the stated length; `buffer` is writable for
        // `buffer.len()` entries and is not aliased during the call; `count`
        // is a valid out-parameter. The shim writes at most `cap` entries and
        // reports overflow rather than running past the end.
        let status = unsafe {
            aoc_yara_collect(
                self.rules,
                line.as_ptr(),
                line.len(),
                self.buffer.as_mut_ptr(),
                self.buffer.len(),
                &mut count,
            )
        };
        if status == -1 {
            return Err(miette::miette!(
                "more matches than the scratch buffer holds — the one-match-per-byte bound in \
                 Scanner::new no longer holds"
            ));
        }
        if status != 0 {
            return Err(miette::miette!("yr_rules_scan_mem failed ({status})"));
        }

        let matches = &self.buffer[..count];
        let Some(first) = matches.iter().min_by_key(|m| m.offset) else {
            return Ok(0);
        };
        // Not `matches.first()`/`matches.last()`: YARA reports grouped by
        // string, so the sequence is in rule order, not input order.
        let last = matches
            .iter()
            .max_by_key(|m| m.offset)
            .expect("a non-empty slice has a maximum");

        let (Some(first), Some(last)) = (
            Self::value_of(first.string_index),
            Self::value_of(last.string_index),
        ) else {
            return Err(miette::miette!(
                "YARA reported a string index outside the compiled rule"
            ));
        };

        Ok(first * 10 + last)
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        // SAFETY: `rules` came from `yr_compiler_get_rules`, is non-null by
        // construction, and is destroyed exactly once because `Scanner` owns
        // it and is not `Copy`/`Clone`.
        unsafe { yr_rules_destroy(self.rules) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The overlap cases, which are the reason this library is a good fit
    /// rather than a merely funny one. `oneight` and `twone` are the shapes
    /// the puzzle statement warns about, and YARA needs no help with them.
    #[test]
    fn overlapping_words_are_two_matches() -> miette::Result<()> {
        let mut scanner = Scanner::new(true, 32)?;
        assert_eq!(scanner.calibration_value("oneight")?, 18);
        assert_eq!(scanner.calibration_value("twone")?, 21);
        assert_eq!(scanner.calibration_value("eightwothree")?, 83);
        Ok(())
    }

    /// Matches arrive grouped by string rather than sorted by offset, so this
    /// is the case that would pass on a naive first/last and fail here: the
    /// scan reports `$w2 @ 4`, `$w3 @ 7`, `$w8 @ 0`, in that order.
    #[test]
    fn the_answer_comes_from_offsets_not_report_order() -> miette::Result<()> {
        let mut scanner = Scanner::new(true, 32)?;
        assert_eq!(scanner.calibration_value("eightwothree")?, 83);
        assert_eq!(scanner.calibration_value("zoneight234")?, 14);
        Ok(())
    }

    /// Without the words, the same scanner is part one — and must not see
    /// `one` at all.
    #[test]
    fn digits_only_ignores_spelled_out_words() -> miette::Result<()> {
        let mut scanner = Scanner::new(false, 32)?;
        assert_eq!(scanner.calibration_value("oneight")?, 0);
        assert_eq!(scanner.calibration_value("two1nine")?, 11);
        Ok(())
    }

    /// A repeated pattern reports once per occurrence, not once per string.
    #[test]
    fn every_occurrence_is_reported() -> miette::Result<()> {
        let mut scanner = Scanner::new(false, 32)?;
        assert_eq!(scanner.calibration_value("999")?, 99);
        Ok(())
    }

    /// The scratch buffer's bound is a real limit with a real error, not a
    /// silent truncation — the failure a fixed-size C buffer usually becomes.
    #[test]
    fn a_line_longer_than_the_buffer_is_refused() -> miette::Result<()> {
        let mut scanner = Scanner::new(true, 4)?;
        let error = scanner
            .calibration_value("a much longer line than four bytes")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("scratch buffer"),
            "unexpected error: {error}"
        );
        Ok(())
    }
}
