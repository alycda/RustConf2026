//! The Godot-facing surface of this crate — a third direction, after
//! `tcc.rs`/`caca.rs` (Rust calls a C library) and `c_api.rs` (Rust exposes
//! itself as one).
//!
//! A GDExtension is neither. The engine `dlopen`s this cdylib, calls one
//! entry symbol, and hands it a `get_proc_address` function pointer; the
//! extension then asks for every engine function it will ever use, by name,
//! at load time. Nobody reads a header in either direction. The whole static
//! contract is `godot/aoc.gdextension` — entry symbol, one library path per
//! platform, minimum engine version — and everything else is negotiated at
//! runtime.
//!
//! Which is why this module registers a *class* instead of exporting
//! functions: there is no symbol for GDScript to find. The `#[derive]` and
//! the `#[godot_api]` below are gdext generating the registration calls that
//! `godot-cpp` makes you write by hand.
//!
//! Note also what does *not* need building: this is the same `cdylib`
//! `c_api.rs` already produces. One shared object, two treaties.
//!
//! Enabled by the `godot` feature; see `Cargo.toml` for why it is optional.
//! Run it: `just days godot-demo 2015-12-01`.

// `::godot`, not `godot`, throughout. This module is `crate::godot` and the
// dependency is the crate `godot`, and a bare `use godot::…` inside a module
// with that name is ambiguous in edition 2018+. The leading `::` says "the
// extern crate" and settles it. Renaming the module would also settle it,
// but `src/godot.rs` is the name a reader looks for.
use ::godot::global::Error;
use ::godot::prelude::*;

use std::str::FromStr;

use crate::{Day, basement_position_pure_rust, sum_pure_rust};

/// The library's one entry point. `#[gdextension]` expands to the
/// `extern "C"` init function the engine calls — `gdext_rust_init` by
/// default, which is the `entry_symbol` in `aoc.gdextension`. Change one and
/// the engine fails the load with "entry symbol not found"; nothing at build
/// time checks that the two agree.
struct AocExtension;

#[gdextension]
unsafe impl ExtensionLibrary for AocExtension {}

/// Day 1 as an engine class. `RefCounted` rather than `Object` so GDScript
/// frees it: the engine owns the instance and drops it when the last
/// reference goes, which is one more answer to "who allocates" — here,
/// neither side of the FFI boundary does.
///
/// The name is the date, because Godot class names are global to the engine:
/// two days registering `AocDay` would collide the moment both extensions
/// loaded.
#[derive(GodotClass)]
#[class(base = RefCounted, init)]
pub struct Aoc20151201;

#[godot_api]
impl Aoc20151201 {
    /// Part 1: the floor Santa ends up on.
    ///
    /// `GString` in, `i64` out. The `GString` → `String` conversion is the
    /// encoding exhibit: Godot's string is UTF-32 — one code point per
    /// 32-bit unit, no surrogate pairs, no variable width — and `to_string()`
    /// transcodes it to UTF-8 to hand Rust something it recognises. Every
    /// crossing pays for that, in both directions.
    ///
    /// `i64`, not `i32`: GDScript's `int` is 64-bit and gdext will not narrow
    /// it for you. The C API's `int32_t` out-parameter has no counterpart
    /// here — the value is the return.
    #[func]
    fn part1(&self, input: GString) -> i64 {
        i64::from(sum_pure_rust(&parse(&input)))
    }

    /// Part 2: where Santa first enters the basement, or `nil`.
    ///
    /// `Variant` is how `Option` spells itself in GDScript — the return is
    /// either an `int` or `null`, and the caller checks with `== null`. This
    /// is the C API's `-2` (Santa never goes under) as an absent value rather
    /// than as a status code.
    #[func]
    fn part2(&self, input: GString) -> Variant {
        match self.basement_position(&input) {
            Some(position) => position.to_variant(),
            None => Variant::nil(),
        }
    }

    /// Part 2 again, the C API's way: the answer goes out through a
    /// parameter and the return is a status code.
    ///
    /// GDScript has no `int *`, so the out-parameter is an `Array` — passed
    /// by reference, appended to on success, left alone on failure. The
    /// return is `@GlobalScope.Error`, the engine's own integer error enum,
    /// which is the first time in this repo the C API's status-code
    /// convention lands *natively* in the consumer rather than being
    /// translated into an exception, a `Result`, or a thrown JS value.
    ///
    /// `ERR_DOES_NOT_EXIST` for the `-2` case: the position genuinely is not
    /// there. There is no code for the `-1` case because, as above, it cannot
    /// happen here.
    #[func]
    fn part2_status(&self, input: GString, mut out: VarArray) -> Error {
        match self.basement_position(&input) {
            Some(position) => {
                out.push(&position.to_variant());
                Error::OK
            }
            None => Error::ERR_DOES_NOT_EXIST,
        }
    }

    /// Part 2 with the error thrown away — the panic exhibit.
    ///
    /// Deliberately the shape nobody should ship: `expect` on the input the
    /// other two methods answer honestly. What happens next is the point.
    /// Exercise 2's C API cannot panic at all (unwinding across an
    /// `extern "C"` frame is UB, so every failure is data); wasm turns a
    /// panic into a trap that kills the instance. gdext does a third thing —
    /// it catches the unwind, prints an engine error with the Rust panic
    /// message, and returns the type's default. The caller gets `0` and the
    /// process carries on.
    ///
    /// `godot/test.gd` calls this and then keeps running, so the claim is
    /// tested rather than quoted.
    #[func]
    fn part2_unwrapped(&self, input: GString) -> i64 {
        self.basement_position(&input)
            .expect("🦀 Santa never enters the basement")
    }

    /// The length of `input` measured the way Rust measures it: UTF-8 bytes.
    ///
    /// Pair it with GDScript's own `String.length()`, which counts UTF-32
    /// units, i.e. code points. For ASCII the two agree and the whole
    /// disagreement is invisible; hand it one accented character and they
    /// part company. That is the encoding lesson in a number small enough to
    /// assert on.
    #[func]
    fn utf8_len(&self, input: GString) -> i64 {
        input.to_string().len() as i64
    }

    /// The shared half of both part-2 methods — a plain Rust helper, not a
    /// `#[func]`, so it never crosses the boundary.
    fn basement_position(&self, input: &GString) -> Option<i64> {
        basement_position_pure_rust(&parse(input)).map(i64::from)
    }
}

/// `GString` → `Day`, and the place where the C API's `-1` went.
///
/// `c_api.rs` returns `-1` for a null pointer or invalid UTF-8. Neither can
/// reach this side: the engine does not hand out null `GString`s, and a
/// `GString` is already valid text by construction — transcoding UTF-32 to
/// UTF-8 cannot fail. `Day::from_str` is infallible for this day too (every
/// character that is not a paren parses as a zero step), so the `Err` arm
/// below is unreachable; it falls back to an empty program rather than
/// panicking, because an unreachable arm that panics is still a panic in
/// somebody's engine log. The only failure this track can have is the domain
/// one — Santa staying above ground.
fn parse(input: &GString) -> Day {
    Day::from_str(&input.to_string()).unwrap_or_else(|_| Day(Vec::new()))
}
