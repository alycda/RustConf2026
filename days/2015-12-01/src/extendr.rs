//! The generated lap of the R track (cargo feature `extendr`), and the
//! answer to what `.C()` cost: [`crate::c_api`] exports a C API that R's
//! oldest interface can reach but whose *return value* it discards, so
//! `r/solve.R` cannot see the `-1`/`-2` status at all. extendr generates
//! the other kind of boundary — R's `.Call()` interface, which passes
//! `SEXP`s both ways — and with it the status code comes back as a thing R
//! has first-class handling for: an error condition, caught by `tryCatch`.
//!
//! What `#[extendr]` generates is one `extern "C" fn wrap__<name>(SEXP…) ->
//! SEXP` per function, plus the metadata R packages use to write their own
//! R-side wrappers. `r/extendr.R` calls the `wrap__` symbols directly
//! rather than building a package around them, which keeps this one file
//! and one script next to the raw route it is being compared against.
//!
//! Off by default, and it has to be: unlike every other feature in this
//! crate, building it needs R itself — extendr links against `libR` and
//! reads R's headers at build time, so `cargo build --features extendr`
//! wants `R` on the machine (`just setup-r`, or `nix shell nixpkgs#R
//! --command`). The MSRV floor job builds the workspace with default
//! features, which is what keeps that requirement out of everyone's way.

use std::str::FromStr;

use extendr_api::prelude::*;

use crate::{Day, basement_position_pure_rust, sum_pure_rust};

/// Part 1, the typed way: `&str` in, `i32` out, and the failure as an
/// `Err` rather than as an out-of-band integer.
///
/// `std::result::Result` spelled in full because `extendr_api::prelude`
/// brings its own `Result` alias into scope. The `String` becomes an R
/// error condition — the channel `.C()` has no equivalent for.
#[extendr]
fn part1(input: &str) -> std::result::Result<i32, String> {
    let day = Day::from_str(input).map_err(|e| format!("part1: {e}"))?;
    Ok(sum_pure_rust(&day))
}

/// Part 2, and the exhibit: this is the one that can genuinely fail. The C
/// API answers a `-2` here that `.C()` throws away; through extendr the
/// same condition arrives in R as an error with a message, so the caller
/// finds out *which* failure it was without a sentinel.
#[extendr]
fn part2(input: &str) -> std::result::Result<i32, String> {
    let day = Day::from_str(input).map_err(|e| format!("part2: {e}"))?;
    basement_position_pure_rust(&day)
        .ok_or_else(|| "part2: Santa never enters the basement".to_string())
}

/// Deliberately panics, and exists to be called: "what does a panic do at
/// this boundary?" is a question every track in this workshop has to
/// answer, and the answers differ (Exercise 2's C API cannot let one out at
/// all — a panic across `extern "C"` is UB — while wasm traps and takes the
/// instance with it).
///
/// extendr's generated wrapper catches the unwind and raises an R error, so
/// R's answer is the gentlest of the three: `tryCatch` sees a
/// `simpleError`, and the session carries on. `r/extendr.R` runs this and
/// then keeps going, which is the proof.
#[extendr]
fn boundary_panic() -> i32 {
    panic!("a Rust panic, raised inside #[extendr]");
}

extendr_module! {
    mod aoc_2015_12_01;
    fn part1;
    fn part2;
    fn boundary_panic;
}
