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
//! This day never draws a random number, so the honest answer is a backend
//! that says so. `getrandom`'s `custom` feature lets a crate register one;
//! registering a source that always errors keeps the module free of host
//! imports (the `js` backend would add wasm-bindgen imports for the host to
//! satisfy) and turns "there is no entropy here" from a build failure into
//! a runtime `Err` on a path nothing takes.

use core::num::NonZeroU32;

/// Error code for "this module has no entropy source". `getrandom` reserves
/// everything from `CUSTOM_START` upwards for backends like this one.
const NO_ENTROPY: NonZeroU32 =
    NonZeroU32::new(getrandom::Error::CUSTOM_START + 1).expect("CUSTOM_START + 1 is nonzero");

/// The registered backend: never fills the buffer, always reports why.
fn no_entropy(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::from(NO_ENTROPY))
}

getrandom::register_custom_getrandom!(no_entropy);

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
