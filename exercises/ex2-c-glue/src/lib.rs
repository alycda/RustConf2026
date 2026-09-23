//! Exercise 2: wrap your Ex 1 solution in a C ABI.
//!
//! This file is where Rust stops being polite. Your job: get a `*const c_char`
//! safely across the boundary, into a `&str`, through your solver, and back
//! out as an `i64` — without undefined behavior on hostile input.
//!
//! Worked reference for this exact shape: ../../../days/2024-12-01/src/c_api.rs
//! (and days/2024-12-03/src/c_api.rs, which chose the panic-free parser on
//! purpose — nothing behind an `extern "C"` frame may panic).

// CStr is unused until you implement step 2 below — it is imported here as
// part of the guidance, so the scaffold ships without a warning either way.
#[allow(unused_imports)]
use std::ffi::{CStr, c_char};

/// Error convention: return -1 when the input is null or not valid UTF-8.
/// In-band errors are the simplest possible convention — Module 4 discusses
/// what production code does instead (out-params, error codes, last-error).
pub const INVALID_INPUT: i64 = -1;

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
// Edition 2024: exporting a symbol is an unsafe promise (a name collision is
// UB the linker arranges), so the attribute itself must say so.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part1(input: *const c_char) -> i64 {
    // TODO, step by step:
    //   1. If `input` is null, return INVALID_INPUT — never deref a null.
    //   2. Wrap the pointer: `CStr::from_ptr(input)` (this is why the fn is
    //      `unsafe` — you are asserting the pointer contract holds).
    //   3. Validate encoding: `.to_str()` gives Ok(&str) only for UTF-8.
    //      Return INVALID_INPUT on Err — C strings promise nothing.
    //   4. Call your solver: `ex1_pure_rust::part1(s)`.
    //
    // Note: a panic that crosses an `extern "C"` boundary aborts the whole
    // process (Rust ≥1.81). The todo!() below does exactly that if you run
    // the C harness before implementing — which is itself a lesson.
    let _ = input;
    todo!("cross the boundary: null check → CStr → UTF-8 → ex1_pure_rust::part1")
}

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
// Edition 2024: exporting a symbol is an unsafe promise (a name collision is
// UB the linker arranges), so the attribute itself must say so.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part2(input: *const c_char) -> i64 {
    let _ = input;
    todo!("same dance, part 2")
}
