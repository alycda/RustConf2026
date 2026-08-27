//! Hand-written FFI binding to the C++ `std::sort` shim in `cpp_sort.cpp`,
//! which build.rs compiles when the `cpp` feature is on.
//!
//! The talk's `2024-12-01-CPP` branch reached `std::sort` through autocxx,
//! which generated this binding from the header — plus a bindgen/libclang
//! build-time dependency to do the generating. The port hand-writes the one
//! function instead: the boundary is C-shaped either way (autocxx emitted
//! exactly this signature under the hood), hand-written FFI is the thing
//! this workshop teaches, and a bare CI runner has no libclang.

unsafe extern "C" {
    fn cpp_sort_i32(data: *mut i32, len: usize);
}

/// Sorts an `i32` slice in place with C++'s `std::sort` through the shim.
pub fn sort(column: &mut [i32]) {
    // SAFETY: a slice's buffer is C-compatible (contiguous, aligned), and
    // std::sort over `data, data + len` only reorders elements in place.
    unsafe { cpp_sort_i32(column.as_mut_ptr(), column.len()) }
}
