/**
 * The three outcomes, on the puzzle statement's own examples, so that the
 * routing in boundary.ts is asserted rather than described:
 *
 *   Ok             -> success
 *   Err            -> a typed failure the program recovers from
 *   panic (a trap) -> a defect that `catchAll` is offered and cannot take
 *
 * Exits nonzero if any of the three lands in the wrong channel. CI runs
 * this; so should you, once, to watch the last line.
 *
 * Run: npm run demo   (after `just days wasm-demo 2015-12-01` has built pkg/)
 */
import { Console, Effect, Exit } from "effect"

import { callRust, loadGenerated } from "./boundary.js"

const program = Effect.gen(function* () {
  const wasm = yield* loadGenerated
  let wrong = 0

  yield* Console.log("Rust Ok(..) -> success:")
  for (const [input, expected] of [
    ["(())", 0],
    [")))", -3],
  ] as const) {
    const got = yield* callRust("part1", () => wasm.part1(input))
    yield* Console.log(`  part1(${JSON.stringify(input)}) = ${got}${got === expected ? "" : `  WRONG, expected ${expected}`}`)
    if (got !== expected) wrong++
  }
  const pos = yield* callRust("part2", () => wasm.part2("()())"))
  yield* Console.log(`  part2("()())") = ${pos}${pos === 5 ? "" : "  WRONG, expected 5"}`)
  if (pos !== 5) wrong++

  yield* Console.log("\nRust Err(..) -> typed failure (recoverable, and the compiler knows its name):")
  const recovered = yield* callRust("part2", () => wasm.part2("(((")).pipe(
    Effect.map((n) => `position ${n} — WRONG, this input never goes underground`),
    Effect.catchTag("RustError", (e) => Effect.succeed(`recovered: ${e.detail}`)),
  )
  yield* Console.log(`  part2("(((") -> ${recovered}`)
  if (!recovered.startsWith("recovered")) wrong++

  yield* Console.log("\nRust panic! -> defect (catchAll is offered it and cannot take it):")
  const exit = yield* Effect.exit(
    callRust("part2_unchecked", () => wasm.part2_unchecked("(((")).pipe(
      Effect.catchAll(() => Effect.succeed(-1)),
    ),
  )
  const survived = Exit.isSuccess(exit)
  yield* Console.log(`  part2_unchecked("(((") swallowed by catchAll? ${survived ? "yes — WRONG" : "no — correct, it is a defect"}`)
  if (survived) wrong++

  // The trap did not take the instance with it. That is not reassurance:
  // the panic never unwound, so nothing on the Rust side was cleaned up,
  // and a module that keeps answering after a trap is a module whose
  // answers you can no longer vouch for.
  const after = yield* callRust("part1", () => wasm.part1("("))
  yield* Console.log(`  after the trap, part1("(") = ${after} — the instance is still answering`)

  return wrong
})

const exit = await Effect.runPromiseExit(program)
if (Exit.isSuccess(exit)) {
  if (exit.value > 0) {
    console.error(`\n${exit.value} outcome(s) landed in the wrong channel`)
    process.exitCode = 1
  } else {
    console.log("\nall three outcomes in their right channels")
  }
} else {
  console.error(String(exit.cause))
  process.exitCode = 1
}
