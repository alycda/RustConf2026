//! Exercise 4: the same boundary, a runtime with no C in it.
//!
//! This crate is Ex 2's shape — a cdylib over your Ex 1 solver, one
//! `extern "C"` function per part — built for `wasm32-unknown-unknown`
//! instead of your machine. The .wasm it produces exports those functions
//! as-is, and Node's built-in `WebAssembly` API can call them with nothing
//! generated. TODO 0 is a paste; TODO 1 is the one thing the new caller
//! needs that no C caller ever did.
//!
//! Worked reference for this exact shape: ../../../days/2015-12-01/src/wasm.rs
//! (alloc/free) with ../../../days/2015-12-01/src/c_api.rs (the wrapper).

// Same unused-import allowance as Ex 2, for the same reason.
#[allow(unused_imports)]
use std::alloc::{self, Layout};
#[allow(unused_imports)]
use std::ffi::{CStr, c_char};

/// Same convention as Ex 2: -1 for a null or non-UTF-8 input.
pub const INVALID_INPUT: i64 = -1;

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part1(input: *const c_char) -> i64 {
    // TODO 0: paste the body of your Ex 2 `ex_part1` here — null check,
    // CStr, UTF-8 check, ex1_pure_rust::part1 — exactly as you wrote it.
    //
    // Nothing about it changes for this target. `*const c_char` is an i32
    // offset into linear memory now, and the i64 you return arrives in
    // JavaScript as a BigInt, but the Rust is the Rust. That is the lesson:
    // the treaty you wrote was about a C ABI, and this runtime honours the
    // same treaty with no C anywhere in it.
    //
    // Before you paste, run ./build.sh once and call the consumer: this
    // todo!() is Ex 2's step 0 again, with a different ending. There, a
    // panic across `extern "C"` aborted the process. Here it traps the
    // module — `RuntimeError: unreachable` — and the instance is still
    // there afterwards, answering. Decide for yourself which is worse.
    let _ = input;
    todo!("paste your Ex 2 ex_part1 here")
}

/// # Safety
/// `input` must be a valid NUL-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part2(input: *const c_char) -> i64 {
    let _ = input;
    todo!("same paste, part 2")
}

/// Lends the caller `size` bytes of this module's linear memory. Returns
/// null for a zero size or when the allocator has nothing left.
///
/// TODO 1: this is the export C never needed. Every caller of your Ex 2
/// library allocated the string on its own side of the boundary and
/// handed over a pointer. A JavaScript caller cannot: the only memory
/// this module can read is its own linear memory, and nothing outside the
/// module can allocate inside it. So before `ex_part1` can be called at
/// all, the module has to lend out the space for its argument.
///
/// The shape: `Layout::from_size_align(size, 1)`, refuse zero and refuse a
/// layout error (return null for both), then `alloc::alloc(layout)`. The
/// reference card calls this callee-allocates; note that here it is a
/// precondition of the call, not a string handed back after it.
#[unsafe(no_mangle)]
pub extern "C" fn ex_alloc(size: usize) -> *mut u8 {
    let _ = size;
    todo!("lend the caller `size` bytes of linear memory")
}

/// Returns memory obtained from [`ex_alloc`], with the same `size`. Null or
/// a zero size is a no-op.
///
/// TODO 1, second half. Why does `free` take the size back? Because Rust's
/// allocator wants the layout it handed out, and a wasm module carries no
/// `malloc` header to recover it from. The contract is "free what you were
/// given, as you were given it" — in the signature, not in the docs.
///
/// # Safety
/// `ptr` must be null or a pointer `ex_alloc` returned for exactly this
/// `size`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_free(ptr: *mut u8, size: usize) {
    let _ = (ptr, size);
    todo!("give the bytes back: dealloc with the layout alloc used")
}
