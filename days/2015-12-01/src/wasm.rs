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
