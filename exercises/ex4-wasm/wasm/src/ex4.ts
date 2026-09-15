/**
 * Exercise 4: call your Ex 2 wrapper — built for wasm32-unknown-unknown —
 * from TypeScript, with nothing generated. Fill in the TODOs top to bottom.
 *
 * Needs Node 22+ (`just setup-wasm`) and `npm ci` once in this directory
 * (the recipe below does that for you). Run from the repo root:
 *
 *     just exercises wasm        # ../build.sh, then this file
 *
 * or, once ../build.sh has produced the module: npm run ex4
 *
 * Worked reference for every step: ../../../../days/2015-12-01/wasm/src/raw.ts
 * (the same route against that day's C API). Step 0, before touching
 * anything: run it. Your crate's todo!() traps at the first call — not an
 * abort, a `RuntimeError: unreachable` — and this file shows what is left.
 */
import { existsSync, readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

import { Cause, Console, Data, Effect, Exit } from "effect"

// --- what can go wrong, by name --------------------------------------------
// Two of these are the boundary's: a sentinel from your wrapper, and a trap
// from the module. The rest are this script's own plumbing.

class ModuleNotBuilt extends Data.TaggedError("ModuleNotBuilt")<{ readonly hint: string }> {}
class ModuleHasImports extends Data.TaggedError("ModuleHasImports")<{ readonly imports: ReadonlyArray<string> }> {}
class OutOfMemory extends Data.TaggedError("OutOfMemory")<{ readonly size: number }> {}
class TodoNotDone extends Data.TaggedError("TodoNotDone")<{ readonly todo: string }> {}

/** Your Ex 2 convention, seen from here: -1 means the boundary refused the input. */
class InvalidInput extends Data.TaggedError("InvalidInput")<{ readonly fn: string }> {}

// --- the module ------------------------------------------------------------
// TODO 2: nothing to fill in here, but read it. There is no header to parse
// and no library to dlopen: the .wasm cargo produced is one file with one
// name on every host (no lib prefix, no .so/.dylib/.dll), and it declares
// its own exports, with wasm types — build.sh printed them. `memory` is
// among them, and it is the only way this script can hand the module a
// string.

interface Ex4 {
  readonly memory: WebAssembly.Memory
  ex_alloc(size: number): number
  ex_free(ptr: number, size: number): void
  // i64 in Rust is BigInt here: a JavaScript number cannot hold one.
  ex_part1(input: number): bigint
  ex_part2(input: number): bigint
}

// wasm/src → wasm → ex4-wasm → exercises; the workspace target dir is there.
const EXERCISES = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")

const loadModule: Effect.Effect<Ex4, ModuleNotBuilt | ModuleHasImports> = Effect.gen(function* () {
  const path = ["debug", "release"]
    .map((profile) => resolve(EXERCISES, "target/wasm32-unknown-unknown", profile, "ex4_wasm.wasm"))
    .find((p) => existsSync(p))
  if (path === undefined) {
    return yield* new ModuleNotBuilt({ hint: "run ../build.sh first (or: just exercises wasm)" })
  }
  const mod = new WebAssembly.Module(readFileSync(path))
  // A module that imports nothing can be instantiated with nothing. Yours
  // imports nothing — that is what "no C in it" buys — so if this fires,
  // something in your crate started asking the host for things.
  const imports = WebAssembly.Module.imports(mod)
  if (imports.length > 0) {
    return yield* new ModuleHasImports({ imports: imports.map((i) => `${i.module}.${i.name}`) })
  }
  return new WebAssembly.Instance(mod, {}).exports as unknown as Ex4
})

// --- traps ------------------------------------------------------------------
// A Rust panic is a wasm trap: JavaScript sees `WebAssembly.RuntimeError`,
// with no message from Rust, and the instance is still there afterwards.
// It is a bug, not an outcome to recover from, so it goes to Effect's defect
// channel — `catchAll` cannot see it, the way `?` never catches a panic.
const call = <A>(fn: string, thunk: () => A): Effect.Effect<A> =>
  Effect.suspend(() => {
    try {
      return Effect.succeed(thunk())
    } catch (cause) {
      return Effect.die(new Error(`trap in ${fn}: ${cause instanceof Error ? cause.message : String(cause)}`))
    }
  })

// --- borrowing memory ---------------------------------------------------------
// ex_alloc on the way in, ex_free on the way out, with the same size, and the
// free runs however the middle ends — success, failure, trap. Effect's
// acquireUseRelease is Rust's Drop and Dart's try/finally in one call.
const borrow = <A, E>(wasm: Ex4, size: number, use: (ptr: number) => Effect.Effect<A, E>) =>
  Effect.acquireUseRelease(
    // Through `call`, because this is where step 0 traps: with TODO 1 still
    // a todo!(), the very first ex_alloc is the panic.
    call("ex_alloc", () => wasm.ex_alloc(size)).pipe(
      Effect.flatMap((ptr) => (ptr === 0 ? Effect.fail(new OutOfMemory({ size })) : Effect.succeed(ptr))),
    ),
    use,
    (ptr) => call("ex_free", () => wasm.ex_free(ptr, size)),
  )

// --- the string crosses ---------------------------------------------------------
// TODO 3: get `bytes` into the module's memory as a C string, and hand back
// the pointer. Three things to do, in this order:
//
//   1. borrow `bytes.length + 1` bytes  — the +1 is the NUL; C reads until it
//      finds one, and ex_alloc hands back uninitialised bytes, so it is yours
//      to write
//   2. make a view AFTER the borrow      — `new Uint8Array(wasm.memory.buffer,
//      ptr, bytes.length + 1)`; a view made before an alloc can be dead after
//      it, because growth detaches the old buffer
//   3. copy `bytes` in, write the 0     — `view.set(bytes)`, `view[bytes.length] = 0`
//
// Then run `use(ptr)`. The skeleton below borrows and hands `use` the
// pointer with nothing written; replace the TodoNotDone with steps 2 and 3.
const withCString = <A, E>(wasm: Ex4, bytes: Uint8Array, use: (ptr: number) => Effect.Effect<A, E>) =>
  borrow(wasm, bytes.length + 1, (ptr) =>
    Effect.gen(function* () {
      yield* new TodoNotDone({ todo: "TODO 3 — write the bytes and the NUL into linear memory at ptr" })
      return yield* use(ptr)
    }),
  )

// --- the call -----------------------------------------------------------------
// TODO 4: call your wrapper and read the answer. `wasm.ex_part1(ptr)` returns
// a BigInt — compare against `5n`, not `5` — and -1n is your Ex 2 sentinel,
// which becomes the typed failure InvalidInput here so a caller can match on
// it. Replace the TodoNotDone with the call and the check.
const part1 = (wasm: Ex4, bytes: Uint8Array): Effect.Effect<bigint, InvalidInput | OutOfMemory | TodoNotDone> =>
  withCString(wasm, bytes, (ptr) =>
    Effect.gen(function* () {
      const _ = ptr
      return yield* new TodoNotDone({ todo: "TODO 4 — call wasm.ex_part1(ptr); -1n is InvalidInput, anything else is the answer" })
    }),
  )

// --- your day ---------------------------------------------------------------------
const EXAMPLE = "PASTE YOUR DAY'S EXAMPLE INPUT HERE"
const EXPECTED_PART1 = 0n // from the puzzle statement — a BigInt literal, note the n

const utf8 = (text: string) => new TextEncoder().encode(text)

const program = Effect.gen(function* () {
  const wasm = yield* loadModule

  const got = yield* part1(wasm, utf8(EXAMPLE))
  if (got !== EXPECTED_PART1) {
    yield* Console.error(`part1 = ${got}, expected ${EXPECTED_PART1}`)
    return false
  }
  yield* Console.log(`part1(example) = ${got} ✓  (a BigInt: ${typeof got})`)

  // TODO 5: the hostile-input contract, from this side. Two calls your Ex 2
  // C harness made, spelled for this runtime:
  //
  //   NULL      — there is no null here, only the integer 0. What does
  //               `wasm.ex_part1(0)` return? (Expected: -1n. Read what your
  //               wrapper's null check is actually checking now.)
  //   garbage   — bytes no TextEncoder would produce, written straight into
  //               the buffer: `new Uint8Array([0xff, 0xfe])` through part1.
  //               Expected: InvalidInput, i.e. -1n from the UTF-8 check.
  //
  // Both lines below are written for you; they only pass once TODO 3 and 4
  // are done. Then the question for the debrief: what did this runtime need
  // that the C header could not say?
  const nullResult = yield* call("ex_part1", () => wasm.ex_part1(0))
  const garbage = yield* part1(wasm, new Uint8Array([0xff, 0xfe])).pipe(
    Effect.map(() => "an answer — WRONG"),
    Effect.catchTag("InvalidInput", () => Effect.succeed("InvalidInput (-1n)")),
  )
  yield* Console.log(`contract: ex_part1(0) = ${nullResult}; ex_part1(0xff 0xfe) -> ${garbage}`)
  return nullResult === -1n && garbage === "InvalidInput (-1n)"
})

const describe = (e: ModuleNotBuilt | ModuleHasImports | OutOfMemory | TodoNotDone | InvalidInput): string => {
  switch (e._tag) {
    case "ModuleNotBuilt":
      return `no module — ${e.hint}`
    case "ModuleHasImports":
      return `the module imports ${e.imports.join(", ")} — the raw route needs a module that asks the host for nothing`
    case "OutOfMemory":
      return `ex_alloc(${e.size}) returned null`
    case "TodoNotDone":
      return e.todo
    case "InvalidInput":
      return `${e.fn} returned the sentinel (-1): the boundary refused the input`
  }
}

const exit = await Effect.runPromiseExit(program.pipe(Effect.tapError((e) => Console.error(describe(e)))))
if (Exit.isFailure(exit)) {
  if (Cause.isDie(exit.cause)) {
    // A trap. Nothing in Rust unwound, nothing was cleaned up, and the
    // instance would still answer the next call — Ex 2's abort was the
    // kinder failure. Step 0 is seeing this once, on purpose.
    console.error(Cause.pretty(exit.cause))
  }
  process.exitCode = 1
} else if (exit.value) {
  console.log("Ex 4 (wasm) passed.")
} else {
  process.exitCode = 1
}
