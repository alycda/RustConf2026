/**
 * The boundary, from the JavaScript side.
 *
 * `wasm-bindgen --target nodejs` writes `../pkg/aoc_2015_12_01.js`: CommonJS
 * glue that loads `aoc_2015_12_01_bg.wasm`, copies each string argument
 * into linear memory, and turns a Rust `Err` into a thrown JS `Error`. What
 * it cannot do is keep Rust's two failure kinds apart on the way out:
 *
 *   Rust `Err(e)`   -> a thrown `Error` (with the message)   expected, describable
 *   Rust `panic!`   -> a wasm trap, `WebAssembly.RuntimeError` a bug; state now undefined
 *
 * A `try/catch` sees both as "something was thrown". Effect has exactly the
 * two channels needed to rebuild the distinction — a typed failure the
 * program can match on, and a defect that `catchAll` cannot reach — and
 * `callRust` below is the whole of the routing.
 */
import { createRequire } from "node:module"

import { Data, Effect } from "effect"

/** The generated module is not there. Not a Rust failure: a build-order one. */
export class ModuleNotBuilt extends Data.TaggedError("ModuleNotBuilt")<{
  readonly hint: string
}> {}

/**
 * A Rust `Err` that crossed. Expected, recoverable, and typed — the
 * compiler will not let a caller forget it exists.
 *
 * (`detail`, not `message`: TaggedError already extends Error, which owns
 * `message`.)
 */
export class RustError extends Data.TaggedError("RustError")<{
  readonly fn: string
  readonly detail: string
}> {}

/** What wasm-bindgen declared in `aoc_2015_12_01.d.ts`, restated here so the
 *  rest of this directory typechecks whether or not `pkg/` exists yet. */
export interface Generated {
  part1(input: string): number
  part2(input: string): number
  part2_unchecked(input: string): number
}

// The glue is CommonJS; createRequire is the plain way to load that from an
// ES module, and it resolves relative to this file rather than the cwd.
const require = createRequire(import.meta.url)

export const loadGenerated: Effect.Effect<Generated, ModuleNotBuilt> = Effect.try({
  try: () => require("../pkg/aoc_2015_12_01.js") as Generated,
  catch: () =>
    new ModuleNotBuilt({
      hint: "run `just days wasm-demo 2015-12-01` first — it builds the crate for wasm32-unknown-unknown and runs wasm-bindgen into wasm/pkg/",
    }),
})

/**
 * Call one generated function and put its outcome in the right channel.
 *
 * `suspend` keeps the call lazy: the Rust function runs when the Effect
 * runs, not when this expression is built — the same inertness as a
 * `Future` before `.await`.
 */
export const callRust = <A>(fn: string, thunk: () => A): Effect.Effect<A, RustError> =>
  Effect.suspend(() => {
    try {
      return Effect.succeed(thunk())
    } catch (cause) {
      if (cause instanceof WebAssembly.RuntimeError) {
        // A trap is a bug in the module — `expect`, an out-of-bounds slice,
        // an overflow check in a debug build. Promoting it to a typed
        // failure would invite callers to "handle" a broken invariant, so it
        // goes to the defect channel, where `catchAll` cannot see it. Note
        // that the instance is still alive after this and may well keep
        // answering; whether its answers can be trusted is another matter.
        return Effect.die(new Error(`trap in Rust fn ${fn}: ${cause.message}`))
      }
      return Effect.fail(
        new RustError({
          fn,
          detail: cause instanceof Error ? cause.message : String(cause),
        }),
      )
    }
  })
