/**
 * The wasm counterpart of `src/main.rs`: read the puzzle input from
 * `<repo>/inputs/2015-12-01.txt`, print both parts. Same input rules as
 * every other track — real inputs are never committed (see .gitignore),
 * and the path is anchored to this file so the script runs from anywhere.
 *
 * Run via: just days wasm-demo 2015-12-01 (builds the module and the
 * glue first); or directly once `pkg/` exists: npm run main
 */
import { readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

import { Cause, Console, Data, Effect, Exit } from "effect"

import { callRust, loadGenerated } from "./boundary.js"
import type { ModuleNotBuilt, RustError } from "./boundary.js"

class NoInput extends Data.TaggedError("NoInput")<{ readonly path: string }> {}

// wasm/src → wasm → 2015-12-01 → days → the repo root
const INPUT = resolve(dirname(fileURLToPath(import.meta.url)), "../../../../inputs/2015-12-01.txt")

const readInput = Effect.try({
  try: () => readFileSync(INPUT, "utf8"),
  catch: () => new NoInput({ path: INPUT }),
})

const program = Effect.gen(function* () {
  const wasm = yield* loadGenerated
  const text = yield* readInput

  // part1 cannot fail; part2's Err — never underground — is a real outcome
  // for a real input, so it is reported the way python/solve.py reports a
  // nonzero status: by name, and with a nonzero exit.
  const part1 = yield* callRust("part1", () => wasm.part1(text))
  yield* Console.log(`Part 1 🧩(🦀): ${part1}`)

  const part2 = yield* callRust("part2", () => wasm.part2(text))
  yield* Console.log(`Part 2 🧩(🦀): ${part2}`)
})

// Every typed failure has a one-line explanation. `match` on the enum, in
// Rust terms; the compiler holds the list, so a new variant is a type
// error here rather than a silent fall-through.
const describe = (e: ModuleNotBuilt | NoInput | RustError): string => {
  switch (e._tag) {
    case "ModuleNotBuilt":
      return `no generated module — ${e.hint}`
    case "NoInput":
      return `no puzzle input at ${e.path} (see .gitignore)`
    case "RustError":
      return `${e.fn} failed: ${e.detail}`
  }
}

const exit = await Effect.runPromiseExit(program.pipe(Effect.tapError((e) => Console.error(describe(e)))))

// A typed failure printed its reason above; a defect (a trap) has no
// handler, so print the cause. Either way the shell gets a nonzero exit,
// like the C harness and the other tracks.
if (Exit.isFailure(exit)) {
  if (Cause.isDie(exit.cause)) {
    console.error(Cause.pretty(exit.cause))
  }
  process.exitCode = 1
}
