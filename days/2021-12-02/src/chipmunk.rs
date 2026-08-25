//! Hand-written FFI bindings to Chipmunk2D, a 2D rigid-body physics engine.
//!
//! There is no practical reason to answer "where did the submarine end up"
//! by standing up a physics space, adding a rigid body to it, and stepping a
//! constraint solver once per puzzle line — that's the point. But the fit is
//! closer than it looks: the puzzle's entire state is a position and an
//! orientation, which is exactly what a rigid body *is*. So instead of
//! folding three integers, this module lets the engine hold them:
//!
//! | puzzle    | Chipmunk                      |
//! |-----------|-------------------------------|
//! | horizontal| `cpBodyGetPosition(body).x`   |
//! | depth     | `cpBodyGetPosition(body).y`   |
//! | aim       | `cpBodyGetAngle(body)`        |
//!
//! Nothing is read back between commands except `aim`, and nothing is
//! accumulated in Rust: the answer is whatever the solver integrated.
//!
//! Two things had to be true for that to give the *exact* integers AoC wants,
//! and both were checked against the real library before any of this was
//! written (see this commit's message):
//!
//! 1. With zero gravity and the default damping of `1.0`, `cpSpaceStep`'s
//!    integrator reduces to `position += velocity * dt` and
//!    `angle += angular_velocity * dt`. At `dt = 1.0` and integer velocities
//!    that is exact in `f64` — no drift to round away at the end.
//! 2. Velocity **persists** across steps. That is the whole purpose of a
//!    physics engine and the whole hazard here: a body told to move `forward
//!    5` keeps moving 5 per step forever. Every command therefore zeroes both
//!    velocities before setting its own, which is the line that turns a
//!    simulation back into a fold.

/// `cpFloat` is `double` unless Chipmunk is compiled with `CP_USE_DOUBLES=0`;
/// nixpkgs builds it with the default. Asserted in the tests rather than
/// trusted, because getting this wrong is silent: the ABI would still link
/// and the numbers would just be wrong.
type CpFloat = f64;

/// `cpVect` is a plain two-double struct passed and returned **by value**.
/// `#[repr(C)]` is what makes that match the C ABI; the pair comes back in
/// two SSE registers on x86-64 SysV rather than through a hidden pointer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CpVect {
    x: CpFloat,
    y: CpFloat,
}

/// Opaque to us: Chipmunk's real `cpSpace`/`cpBody` layouts live in
/// `chipmunk_structs.h`, which is a private header. We only ever hold
/// pointers, so a zero-sized `#[repr(C)]` placeholder is the whole
/// declaration we need — and it keeps us from accidentally dereferencing one.
#[repr(C)]
struct CpSpace {
    _private: [u8; 0],
}

#[repr(C)]
struct CpBody {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn cpSpaceNew() -> *mut CpSpace;
    fn cpSpaceFree(space: *mut CpSpace);
    fn cpSpaceSetGravity(space: *mut CpSpace, gravity: CpVect);
    fn cpSpaceAddBody(space: *mut CpSpace, body: *mut CpBody) -> *mut CpBody;
    fn cpSpaceRemoveBody(space: *mut CpSpace, body: *mut CpBody);
    fn cpSpaceStep(space: *mut CpSpace, dt: CpFloat);

    fn cpBodyNew(mass: CpFloat, moment: CpFloat) -> *mut CpBody;
    fn cpBodyFree(body: *mut CpBody);
    fn cpBodySetVelocity(body: *mut CpBody, velocity: CpVect);
    fn cpBodySetAngularVelocity(body: *mut CpBody, angular_velocity: CpFloat);
    fn cpBodyGetPosition(body: *mut CpBody) -> CpVect;
    fn cpBodyGetAngle(body: *mut CpBody) -> CpFloat;
}

/// A `cpSpace` holding exactly one dynamic body — the submarine.
///
/// Owns both allocations and tears them down in the order Chipmunk expects
/// (remove the body from the space, then free the body, then the space), so
/// callers never see a raw pointer and cannot leak one by returning early.
pub struct Submarine {
    space: *mut CpSpace,
    body: *mut CpBody,
}

impl Submarine {
    /// Stands up a gravity-free space with one dynamic body at the origin.
    ///
    /// Zero gravity is not decoration: `cpSpaceNew`'s default is already
    /// `(0, 0)`, but the whole exactness argument in this module's docs rests
    /// on it, so it is set explicitly rather than inherited.
    ///
    /// Mass and moment are `1.0` — any finite nonzero pair works, because
    /// nothing here applies a force or an impulse. `INFINITY` would make the
    /// body static and stop it integrating at all.
    pub fn new() -> miette::Result<Self> {
        // SAFETY: cpSpaceNew/cpBodyNew allocate or return null; every use
        // below is null-checked before it is passed back into the library.
        unsafe {
            let space = cpSpaceNew();
            if space.is_null() {
                return Err(miette::miette!("cpSpaceNew failed to allocate a cpSpace"));
            }
            cpSpaceSetGravity(space, CpVect { x: 0.0, y: 0.0 });

            let body = cpBodyNew(1.0, 1.0);
            if body.is_null() {
                cpSpaceFree(space);
                return Err(miette::miette!("cpBodyNew failed to allocate a cpBody"));
            }
            cpSpaceAddBody(space, body);

            Ok(Self { space, body })
        }
    }

    /// The submarine's `aim`, which the engine stores as the body's rotation.
    pub fn aim(&self) -> f64 {
        // SAFETY: `self.body` is non-null for the lifetime of `self` and is
        // still owned by `self.space`.
        unsafe { cpBodyGetAngle(self.body) }
    }

    /// Runs one command: set the velocities this command implies, then let
    /// the solver integrate them for exactly one unit of time.
    ///
    /// Both velocities are zeroed first — see this module's docs. Chipmunk
    /// has no "move by" call; the only way to get a displacement out of it is
    /// to travel at a velocity for a duration.
    pub fn step(&mut self, velocity: (f64, f64), angular_velocity: f64) {
        // SAFETY: `self.body` and `self.space` are non-null and paired for
        // the lifetime of `self`; cpVect is passed by value per the C ABI.
        unsafe {
            cpBodySetVelocity(
                self.body,
                CpVect {
                    x: velocity.0,
                    y: velocity.1,
                },
            );
            cpBodySetAngularVelocity(self.body, angular_velocity);
            cpSpaceStep(self.space, 1.0);
        }
    }

    /// `horizontal * depth` — the puzzle's answer — read back off the body.
    ///
    /// The engine works in `f64` and the puzzle wants an `i32`, so the
    /// narrowing is checked rather than cast away with `as`. Every value that
    /// reaches here should be integral (see the exactness argument above); if
    /// one isn't, that's a broken assumption worth an error rather than a
    /// silently truncated answer.
    pub fn answer(&self) -> miette::Result<i32> {
        // SAFETY: as in `aim`.
        let position = unsafe { cpBodyGetPosition(self.body) };
        let product = position.x * position.y;

        if product.fract() != 0.0 {
            return Err(miette::miette!(
                "the solver drifted off the integers: {} * {} = {product}",
                position.x,
                position.y
            ));
        }
        if product < f64::from(i32::MIN) || product > f64::from(i32::MAX) {
            return Err(miette::miette!(
                "answer {product} does not fit in an i32 (horizontal {}, depth {})",
                position.x,
                position.y
            ));
        }

        Ok(product as i32)
    }
}

impl Drop for Submarine {
    fn drop(&mut self) {
        // SAFETY: both pointers were allocated by Chipmunk in `new` and have
        // not been freed. cpSpaceFree does not free the bodies a space holds,
        // so the body is removed and freed first — freeing it while the space
        // still lists it would leave cpSpaceFree walking freed memory.
        unsafe {
            cpSpaceRemoveBody(self.space, self.body);
            cpBodyFree(self.body);
            cpSpaceFree(self.space);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chipmunk compiled with `CP_USE_DOUBLES=0` would make `cpFloat` an
    /// `f32`, which links just as happily and quietly loses the exactness
    /// every answer in this module depends on.
    #[test]
    fn cpfloat_is_double() {
        assert_eq!(size_of::<CpFloat>(), size_of::<f64>());
        assert_eq!(size_of::<CpVect>(), 2 * size_of::<CpFloat>());
        assert_eq!(align_of::<CpVect>(), align_of::<CpFloat>());
    }

    /// The persistence hazard, pinned as a test: one `step` at velocity
    /// `(3, 0)` moves 3, and the *next* step at zero velocity must not move
    /// again. Without the zeroing in `step` this fails at 6.
    #[test]
    fn velocity_does_not_carry_into_the_next_step() -> miette::Result<()> {
        let mut sub = Submarine::new()?;
        sub.step((3.0, 0.0), 0.0);
        sub.step((0.0, 1.0), 0.0);

        // SAFETY: as in `aim`.
        let position = unsafe { cpBodyGetPosition(sub.body) };
        assert_eq!(position.x, 3.0, "horizontal kept thrusting");
        assert_eq!(position.y, 1.0, "depth kept thrusting");
        Ok(())
    }

    /// Angular velocity integrates into the angle the same way, which is what
    /// makes `aim` the body's rotation rather than a counter Rust keeps.
    #[test]
    fn angular_velocity_integrates_into_the_angle() -> miette::Result<()> {
        let mut sub = Submarine::new()?;
        sub.step((0.0, 0.0), 5.0);
        assert_eq!(sub.aim(), 5.0);
        sub.step((0.0, 0.0), -3.0);
        assert_eq!(sub.aim(), 2.0);
        Ok(())
    }

    /// A space with no body added to it still steps; a body never added to a
    /// space never moves. Guards against `cpSpaceAddBody` quietly dropping
    /// out of `new` — the failure mode is an answer of 0, not a crash.
    #[test]
    fn the_body_is_actually_in_the_space() -> miette::Result<()> {
        let mut sub = Submarine::new()?;
        sub.step((7.0, 0.0), 0.0);
        // SAFETY: as in `aim`.
        let position = unsafe { cpBodyGetPosition(sub.body) };
        assert_eq!(position.x, 7.0, "body is not being integrated by the space");
        Ok(())
    }
}
