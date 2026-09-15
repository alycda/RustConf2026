//! Hand-written FFI binding to LAPACK's `DLASRT` — the workshop's only
//! crossing into a *Fortran* library, and the mirror image of the Fortran
//! track on 2015-12-01.
//!
//! There it is Fortran calling Rust, and the ABI's hidden arguments are
//! something to stay away from: `const char *` binds to `dimension(*)`
//! precisely so gfortran does not append a length nobody declared. Here it is
//! Rust calling Fortran, the callee *is* gfortran-compiled, and the same rule
//! runs the other way — the hidden argument is one we have to supply, and
//! nothing in the C-facing world will tell us it is missing.
//!
//! `DLASRT` is LAPACK's auxiliary sort:
//!
//! ```fortran
//! SUBROUTINE DLASRT( ID, N, D, INFO )
//!    CHARACTER          ID
//!    INTEGER            INFO, N
//!    DOUBLE PRECISION   D( * )
//! ```
//!
//! Sorting a column of `i32` location IDs with a `double precision` quicksort
//! out of a linear-algebra package is, like every variant on this day, not
//! something anyone should do. It is here because it is the shortest honest
//! route to three ABI facts a C library cannot teach.

use std::ffi::{c_char, c_int};

// The Fortran side of the treaty, transcribed by hand. Three things in this
// declaration are not in the Fortran signature above, and each is a lesson:
//
// 1. THE TRAILING UNDERSCORE. Fortran has no standard external name
//    mangling; gfortran (and flang, and every f77 descendant on Unix)
//    lowercases the subroutine name and appends one underscore, so `DLASRT`
//    exports as `dlasrt_`. That is a compiler convention, not a language
//    rule — the 2015-12-01 Fortran track spells `bind(C, name=...)` out for
//    the same reason from the other end. Ask for `dlasrt` and the link fails
//    loudly, which is the friendly half of this file.
//
// 2. EVERY ARGUMENT BY POINTER, INCLUDING `n`. Fortran passes by reference by
//    default; there is no `intent(in), value` here, so `N` is an `int *` even
//    though it is read and never written. Pass an `i32` by value and the
//    callee dereferences whatever the register holds. This is the same fact
//    the Fortran track celebrates — the out-parameter costs that language
//    nothing — seen from the side where it costs us a `&`.
//
// 3. THE HIDDEN CHARACTER LENGTH. `id_len` appears in no Fortran signature
//    anywhere. gfortran's ABI appends the length of every `character`
//    argument, as a `size_t`, after all the visible arguments; with one
//    `character` first in the list, that is one extra trailing `usize`. It is
//    *convention*, not the Fortran standard (the standard says nothing about
//    how arguments are passed; F2003's ISO_C_BINDING exists precisely so you
//    never have to know) — gfortran and classic f2c put lengths at the end,
//    and other compilers have historically put them immediately after the
//    character they describe. Which means: this binding is correct for a
//    gfortran-built LAPACK, which is what nixpkgs, every distro and
//    Accelerate ship, and is not portable in the way an `extern "C"`
//    declaration against a C header is.
//
//    And here is the part worth stopping on, measured rather than assumed
//    (aarch64-linux, nixpkgs' lapack): delete `id_len` from this declaration
//    and from the call below, and everything still links and still sorts
//    correctly. Passing 0, 9999 or usize::MAX for it also sorts correctly.
//    DLASRT declares `CHARACTER ID` — fixed length 1 — so it never reads the
//    length it was handed; it forwards a literal 1 to LSAME. The wrong
//    binding is not punished, which is worse than being punished: it is a
//    wrong treaty that passes every test you would think to write. A routine
//    with a `CHARACTER*(*)` dummy — LAPACK's own XERBLA, one frame further
//    in — does read it, and that is where the same omission becomes a
//    garbage length applied to a real string.
//
// Reference: LAPACK dlasrt.f (netlib), and gfortran's "Argument passing
// conventions" in the GNU Fortran manual.
unsafe extern "C" {
    fn dlasrt_(id: *const c_char, n: *const c_int, d: *mut f64, info: *mut c_int, id_len: usize);
}

/// Sorts an `i32` slice in place by widening it to `f64`, handing it to
/// LAPACK's `DLASRT` in increasing order, and narrowing back.
///
/// The round trip is lossless in both directions: `f64` has a 53-bit
/// significand, so every `i32` — `i32::MIN` and `i32::MAX` included — is
/// exactly representable, and a sort only permutes the values it was given.
/// (This would not survive a widening from `i64`, which is the interesting
/// half of the guarantee.)
pub fn sort(column: &mut [i32]) {
    // DLASRT's own N = 0 path returns immediately, but an empty slice's
    // `as_mut_ptr` is a dangling-though-aligned pointer, and handing a
    // dangling pointer to an assumed-size `D( * )` is a promise we would
    // rather not make. Nothing to sort, nothing to cross for.
    if column.is_empty() {
        return;
    }

    let mut d: Vec<f64> = column.iter().copied().map(f64::from).collect();

    // 'I' for increasing, 'D' for decreasing. LAPACK reads exactly one
    // character here; the hidden length below says how long we claim it is.
    // `as c_char` rather than a cast to a fixed width: c_char is i8 on
    // x86-64 and u8 on aarch64, and this file is built on both.
    let id = b'I' as c_char;
    let n = c_int::try_from(d.len()).expect("a column longer than i32::MAX to sort");
    // DLASRT writes -k here if it rejected argument k, and nothing else.
    let mut info: c_int = 0;

    // SAFETY: `id` and `n` are live for the call and are only read; `d` is a
    // contiguous, aligned buffer of exactly `n` f64s, which is what the
    // assumed-size `D( * )` needs; `info` is a live, writable c_int. The
    // final `1` is the hidden length of `id` — see the declaration above for
    // why it is here and why leaving it off is the interesting mistake.
    unsafe {
        dlasrt_(&id, &n, d.as_mut_ptr(), &mut info, 1);
    }

    // The status channel Fortran actually has: an out-parameter, because a
    // SUBROUTINE has no return value. It can only be 0 or -k for a rejected
    // argument k, so a nonzero value means this binding is wrong, not that
    // the data was — which makes it an assert rather than a Result.
    assert_eq!(0, info, "DLASRT rejected argument {}", -info);

    for (slot, sorted) in column.iter_mut().zip(d) {
        *slot = sorted as i32;
    }
}
