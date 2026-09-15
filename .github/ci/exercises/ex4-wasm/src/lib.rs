//! Exercise 4, solved — the CI overlay for the attendee scaffold (see
//! .github/ci/README.md). Same file, TODOs filled: the Ex 2 wrapper pasted
//! in unchanged (TODO 0), and the allocator pair the wasm caller needs
//! (TODO 1).

use std::alloc::{self, Layout};
use std::ffi::{CStr, c_char};

/// Same convention as Ex 2: -1 for a null or non-UTF-8 input.
pub const INVALID_INPUT: i64 = -1;

/// The four steps, once, shared by both exports — exactly Ex 2's helper.
///
/// # Safety
/// `input` must be null or a valid NUL-terminated C string.
unsafe fn input_str<'a>(input: *const c_char) -> Option<&'a str> {
    if input.is_null() {
        return None;
    }
    // SAFETY: non-null by the check above; NUL-terminated by the caller's
    // contract, which is the whole promise this `unsafe fn` asks for.
    unsafe { CStr::from_ptr(input) }.to_str().ok()
}

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part1(input: *const c_char) -> i64 {
    // TODO 0, done: the Ex 2 body, byte for byte. `*const c_char` is an
    // offset into linear memory on this target and the i64 arrives in
    // JavaScript as a BigInt, and none of that is visible from here.
    // SAFETY: the caller's contract is exactly `input_str`'s.
    match unsafe { input_str(input) } {
        Some(s) => ex1_pure_rust::part1(s),
        None => INVALID_INPUT,
    }
}

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part2(input: *const c_char) -> i64 {
    // SAFETY: the caller's contract is exactly `input_str`'s.
    match unsafe { input_str(input) } {
        Some(s) => ex1_pure_rust::part2(s),
        None => INVALID_INPUT,
    }
}

/// Lends the caller `size` bytes of this module's linear memory. Returns
/// null for a zero size or when the allocator has nothing left.
///
/// TODO 1, done: the export C never needed. A JavaScript caller cannot
/// allocate inside linear memory, so the module lends the space out.
#[unsafe(no_mangle)]
pub extern "C" fn ex_alloc(size: usize) -> *mut u8 {
    let Ok(layout) = Layout::from_size_align(size, 1) else {
        return core::ptr::null_mut();
    };
    if layout.size() == 0 {
        return core::ptr::null_mut();
    }
    // SAFETY: the layout has nonzero size, which is `alloc`'s one precondition.
    unsafe { alloc::alloc(layout) }
}

/// Returns memory obtained from [`ex_alloc`], with the same `size`. Null or
/// a zero size is a no-op.
///
/// # Safety
/// `ptr` must be null or a pointer `ex_alloc` returned for exactly this
/// `size`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_free(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    // SAFETY: the caller upholds the contract above, so this is the layout
    // `alloc` was called with.
    unsafe { alloc::dealloc(ptr, Layout::from_size_align_unchecked(size, 1)) }
}
