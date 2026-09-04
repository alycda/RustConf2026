//! Hand-written FFI binding to libc's `qsort` — the talk's original day-1
//! boundary crossing (alycda/aoc-ffi), ported into the day tree.
//!
//! There is no practical reason to sort an `i32` column with C's `qsort` —
//! Rust's built-in sort is both safer and faster, and the talk's benchmarks
//! measured exactly that. The point is the boundary mechanics on a familiar
//! algorithm: an `unsafe extern "C"` declaration, a C-shaped comparison
//! callback handed across as a function pointer, and a slice's backing
//! storage crossing as pointer + length.

use std::ffi::{c_int, c_void};

// No pkg-config probe and no build.rs: qsort is part of the C standard
// library every target already links.
// Reference: man 3 qsort on any Unix system.
unsafe extern "C" {
    fn qsort(
        // the pointer to the first element of the array to be sorted
        base: *mut c_void,
        // number of elements in the array
        num: usize,
        // size of each element in the array
        size: usize,
        // a function pointer to a 2-element comparison function
        compar: unsafe extern "C" fn(*const c_void, *const c_void) -> c_int,
    );
}

// Comparison function for qsort (ascending order).
// Must return: negative if a < b, zero if a == b, positive if a > b.
unsafe extern "C" fn compare_i32(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: qsort hands this callback valid pointers into the array it was
    // given — i32 elements here, because `sort` below only ever passes i32s.
    let (a, b) = unsafe { (*a.cast::<i32>(), *b.cast::<i32>()) };

    // IMPORTANT: don't use `a - b` here! It can overflow: a = i32::MAX,
    // b = i32::MIN panics in debug and wraps in release. The talk shipped
    // that bug live before landing on the C idiom:
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// Sorts an `i32` slice in place by handing its backing storage to `qsort`.
pub fn sort(column: &mut [i32]) {
    // SAFETY: a slice's buffer is C-compatible (contiguous, aligned), and
    // qsort only reorders elements in place — it never resizes or frees.
    unsafe {
        qsort(
            column.as_mut_ptr().cast(),
            column.len(),
            size_of::<i32>(),
            compare_i32,
        );
    }
}
