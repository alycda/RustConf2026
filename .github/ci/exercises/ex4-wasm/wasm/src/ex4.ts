/**
 * Exercise 4, solved — the CI overlay for the attendee scaffold (see
 * .github/ci/README.md). Same file, TODOs 3 and 4 filled, on the overlay's
 * day (2025 day 3): part 1 is 357, and the answer comes back as a BigInt.
 *
 * Run from the repo root: just exercises wasm
 */
import { existsSync, readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

import { Cause, Console, Data, Effect, Exit } from "effect"

class ModuleNotBuilt extends Data.TaggedError("ModuleNotBuilt")<{ readonly hint: string }> {}
class ModuleHasImports extends Data.TaggedError("ModuleHasImports")<{ readonly imports: ReadonlyArray<string> }> {}
class OutOfMemory extends Data.TaggedError("OutOfMemory")<{ readonly size: number }> {}

/** Your Ex 2 convention, seen from here: -1 means the boundary refused the input. */
class InvalidInput extends Data.TaggedError("InvalidInput")<{ readonly fn: string }> {}

// TODO 2, read: one file, one name, its own export section. `memory` is the
// only way to hand the module a string.
interface Ex4 {
  readonly memory: WebAssembly.Memory
  ex_alloc(size: number): number
  ex_free(ptr: number, size: number): void
  ex_part1(input: number): bigint
  ex_part2(input: number): bigint
}

const EXERCISES = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")

const loadModule: Effect.Effect<Ex4, ModuleNotBuilt | ModuleHasImports> = Effect.gen(function* () {
  const path = ["debug", "release"]
    .map((profile) => resolve(EXERCISES, "target/wasm32-unknown-unknown", profile, "ex4_wasm.wasm"))
    .find((p) => existsSync(p))
  if (path === undefined) {
    return yield* new ModuleNotBuilt({ hint: "run ../build.sh first (or: just exercises wasm)" })
  }
  const mod = new WebAssembly.Module(readFileSync(path))
  const imports = WebAssembly.Module.imports(mod)
  if (imports.length > 0) {
    return yield* new ModuleHasImports({ imports: imports.map((i) => `${i.module}.${i.name}`) })
  }
  return new WebAssembly.Instance(mod, {}).exports as unknown as Ex4
})

// A trap is a bug, not an outcome: defect channel, where catchAll cannot see it.
const call = <A>(fn: string, thunk: () => A): Effect.Effect<A> =>
  Effect.suspend(() => {
    try {
      return Effect.succeed(thunk())
    } catch (cause) {
      return Effect.die(new Error(`trap in ${fn}: ${cause instanceof Error ? cause.message : String(cause)}`))
    }
  })

// ex_alloc in, ex_free out with the same size, however the middle ends.
const borrow = <A, E>(wasm: Ex4, size: number, use: (ptr: number) => Effect.Effect<A, E>) =>
  Effect.acquireUseRelease(
    call("ex_alloc", () => wasm.ex_alloc(size)).pipe(
      Effect.flatMap((ptr) => (ptr === 0 ? Effect.fail(new OutOfMemory({ size })) : Effect.succeed(ptr))),
    ),
    use,
    (ptr) => call("ex_free", () => wasm.ex_free(ptr, size)),
  )

// TODO 3, done: borrow len + 1, make the view AFTER the borrow (growth
// detaches the old buffer), copy the bytes in, write the NUL yourself —
// ex_alloc hands back uninitialised bytes and C reads until it finds a 0.
const withCString = <A, E>(wasm: Ex4, bytes: Uint8Array, use: (ptr: number) => Effect.Effect<A, E>) =>
  borrow(wasm, bytes.length + 1, (ptr) =>
    Effect.gen(function* () {
      const view = new Uint8Array(wasm.memory.buffer, ptr, bytes.length + 1)
      view.set(bytes)
      view[bytes.length] = 0
      return yield* use(ptr)
    }),
  )

// TODO 4, done: the call returns a BigInt; -1n is the Ex 2 sentinel, which
// becomes a typed failure here so a caller can match on it.
const part1 = (wasm: Ex4, bytes: Uint8Array): Effect.Effect<bigint, InvalidInput | OutOfMemory> =>
  withCString(wasm, bytes, (ptr) =>
    Effect.gen(function* () {
      const got = yield* call("ex_part1", () => wasm.ex_part1(ptr))
      if (got === -1n) return yield* new InvalidInput({ fn: "ex_part1" })
      return got
    }),
  )

// The overlay's day: 2025 day 3 (not on the workshop menu, so nothing here
// spoils a day an attendee picks). 357 is the statement's part-1 answer.
const EXAMPLE = ["987654321111111", "811111111111119", "234234234234278", "818181911112111"].join("\n")
const EXPECTED_PART1 = 357n

const utf8 = (text: string) => new TextEncoder().encode(text)

const program = Effect.gen(function* () {
  const wasm = yield* loadModule

  const got = yield* part1(wasm, utf8(EXAMPLE))
  if (got !== EXPECTED_PART1) {
    yield* Console.error(`part1 = ${got}, expected ${EXPECTED_PART1}`)
    return false
  }
  yield* Console.log(`part1(example) = ${got} ✓  (a BigInt: ${typeof got})`)

  // TODO 5: the hostile-input contract from this side — NULL is 0, garbage
  // is two raw bytes no encoder would produce. Both must be the sentinel.
  const nullResult = yield* call("ex_part1", () => wasm.ex_part1(0))
  const garbage = yield* part1(wasm, new Uint8Array([0xff, 0xfe])).pipe(
    Effect.map(() => "an answer — WRONG"),
    Effect.catchTag("InvalidInput", () => Effect.succeed("InvalidInput (-1n)")),
  )
  yield* Console.log(`contract: ex_part1(0) = ${nullResult}; ex_part1(0xff 0xfe) -> ${garbage}`)
  return nullResult === -1n && garbage === "InvalidInput (-1n)"
})

const describe = (e: ModuleNotBuilt | ModuleHasImports | OutOfMemory | InvalidInput): string => {
  switch (e._tag) {
    case "ModuleNotBuilt":
      return `no module — ${e.hint}`
    case "ModuleHasImports":
      return `the module imports ${e.imports.join(", ")} — the raw route needs a module that asks the host for nothing`
    case "OutOfMemory":
      return `ex_alloc(${e.size}) returned null`
    case "InvalidInput":
      return `${e.fn} returned the sentinel (-1): the boundary refused the input`
  }
}

const exit = await Effect.runPromiseExit(program.pipe(Effect.tapError((e) => Console.error(describe(e)))))
if (Exit.isFailure(exit)) {
  if (Cause.isDie(exit.cause)) console.error(Cause.pretty(exit.cause))
  process.exitCode = 1
} else if (exit.value) {
  console.log("Ex 4 (wasm) passed.")
} else {
  process.exitCode = 1
}
