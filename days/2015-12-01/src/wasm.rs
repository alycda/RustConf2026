//! What this crate needs to exist as a WebAssembly module — and nothing a
//! native build ever sees, hence `#[cfg(target_arch = "wasm32")]` on the
//! module rather than a cargo feature: the raw route's whole claim is that
//! `cargo build --target wasm32-unknown-unknown` is enough.
//!
//! Every other target this crate builds for has an operating system behind
//! it. `wasm32-unknown-unknown` is the one that, by name, does not: no
//! files, no clock, no entropy, nothing but linear memory and whatever the
//! host chooses to import. Most of the dependency tree never notices. The
//! one crate that does is `getrandom`, reached through
//! `aoc-ornaments → rand → rand_core`, which refuses to compile for this
//! target until told where random bytes come from — it would rather fail
//! the build than silently hand out zeros.
//!
//! Three things live here, all of them wasm's and none of them C's:
//!
//! - the getrandom backend, so the crate compiles at all;
//! - `alloc`/`free`, so a caller with no allocator of its own can hand this
//!   module a string and a place to write the answer — the raw route;
//! - behind the `wasm` feature, the wasm-bindgen exports — the generated
//!   lap, where a tool writes the glue the raw route makes you write.
//!
//! This day never draws a random number, so the honest answer is a backend
//! that says so. `getrandom`'s `custom` feature lets a crate register one;
//! registering a source that always errors keeps the module free of host
//! imports (the `js` backend would add wasm-bindgen imports for the host to
//! satisfy) and turns "there is no entropy here" from a build failure into
//! a runtime `Err` on a path nothing takes.

use core::num::NonZeroU32;
use std::alloc::{self, Layout};

/// Error code for "this module has no entropy source". `getrandom` reserves
/// everything from `CUSTOM_START` upwards for backends like this one.
const NO_ENTROPY: NonZeroU32 =
    NonZeroU32::new(getrandom::Error::CUSTOM_START + 1).expect("CUSTOM_START + 1 is nonzero");

/// The registered backend: never fills the buffer, always reports why.
fn no_entropy(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::from(NO_ENTROPY))
}

getrandom::register_custom_getrandom!(no_entropy);

/// The alignment every borrowed block carries: enough for the `int32_t`
/// out-parameter, see [`aoc_2015_12_01_alloc`].
const ALIGN: usize = 4;

/// Lends the caller `size` bytes of this module's linear memory, aligned to
/// four, or returns null when `size` is zero or the allocator has nothing
/// left.
///
/// Four, because the C API writes its answer through an `int *` and the
/// caller borrows that pointer from here too: a one-byte-aligned block would
/// be a promise the C side cannot rely on, even where the allocator happens
/// to hand out more. The strings do not care.
///
/// The raw route's precondition. The C API in `c_api.rs` reads a string
/// the caller made and writes through a pointer the caller owns; a
/// JavaScript caller can do neither, because the only memory this module
/// can see is its own and nothing outside it can allocate there. So the
/// module hands out the space: the caller asks for `len + 1`, writes the
/// UTF-8 bytes and the NUL, and passes the offset back in as the
/// `const char *`. Four more bytes the same way for the `int *`.
///
/// Every call is one bulk crossing, the same shape as the C API's own —
/// and one more pair of symbols to bind, which is the cost the generated
/// lap hides inside its glue.
#[unsafe(no_mangle)]
pub extern "C" fn aoc_2015_12_01_alloc(size: usize) -> *mut u8 {
    let Ok(layout) = Layout::from_size_align(size, ALIGN) else {
        return core::ptr::null_mut();
    };
    if layout.size() == 0 {
        return core::ptr::null_mut();
    }
    // SAFETY: the layout has nonzero size, which is `alloc`'s one precondition.
    unsafe { alloc::alloc(layout) }
}

/// Returns memory obtained from [`aoc_2015_12_01_alloc`], with the same
/// `size`. Null (or a zero size) is a no-op.
///
/// The size travels back because the allocator wants the layout it handed
/// out and a wasm module carries no `malloc` header to recover it from: the
/// contract is "free what you were given, as you were given it", stated in
/// the signature rather than trusted to memory.
///
/// # Safety
/// `ptr` must be null or a pointer this module's `alloc` returned for
/// exactly this `size`, not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aoc_2015_12_01_free(ptr: *mut u8, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }
    // SAFETY: the caller upholds the contract above, so this is the layout
    // `alloc` was called with.
    unsafe { alloc::dealloc(ptr, Layout::from_size_align_unchecked(size, ALIGN)) }
}

/// The generated lap. Everything the raw route does by hand — copying the
/// string into linear memory, reading the result back, turning a `Result`
/// into something JavaScript can catch — wasm-bindgen writes into a JS glue
/// file next to the module. What it cannot do is invent a distinction the
/// wasm boundary does not carry: an `Err` and a `panic!` both arrive in
/// JavaScript as "something was thrown", just different somethings, and
/// the three exports below exist to produce each on demand from one
/// solver so the consumer in `wasm/` can show them being told apart.
#[cfg(feature = "wasm")]
pub mod bindgen {
    use std::str::FromStr;

    use wasm_bindgen::prelude::*;

    use crate::{Day, basement_position_pure_rust, sum_pure_rust};

    /// The floor Santa ends up on. Cannot fail: every character parses
    /// (unknown ones count as zero), and an `i32` sum of ±1s cannot
    /// overflow on any input a browser could hand over.
    #[wasm_bindgen]
    pub fn part1(input: &str) -> i32 {
        // Day::from_str returns Result for the trait's sake and never Err.
        let day = Day::from_str(input).unwrap_or(Day(Vec::new()));
        sum_pure_rust(&day)
    }

    /// The 1-based position of the first instruction that sends Santa into
    /// the basement. The `Result` path: an input that never goes negative
    /// is an ordinary, expected failure, and it crosses as a thrown JS
    /// `Error` whose message is the same one `main.rs` prints natively.
    #[wasm_bindgen]
    pub fn part2(input: &str) -> Result<i32, JsError> {
        let day = Day::from_str(input).unwrap_or(Day(Vec::new()));
        basement_position_pure_rust(&day)
            .ok_or_else(|| JsError::new("🦀 Santa never enters the basement"))
    }

    /// The same question, written the way a Rust-only codebase would have:
    /// `expect` on the `Option`. On a native target that is a panic with a
    /// message; here it is a trap — JavaScript sees a
    /// `WebAssembly.RuntimeError: unreachable`, no message, and the
    /// instance's Rust state (its allocator included) is undefined from
    /// that call on. Exported so the consumer can watch that happen, not
    /// as an API anyone should call.
    #[wasm_bindgen]
    pub fn part2_unchecked(input: &str) -> i32 {
        let day = Day::from_str(input).unwrap_or(Day(Vec::new()));
        basement_position_pure_rust(&day).expect("Santa never enters the basement")
    }
}
