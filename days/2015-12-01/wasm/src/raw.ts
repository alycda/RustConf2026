/**
 * The raw route: the C API, called from JavaScript with nothing generated.
 *
 * `cargo build --target wasm32-unknown-unknown` turns `src/c_api.rs`'s two
 * `extern "C"` functions into wasm exports as-is — plus the `alloc`/`free`
 * pair from `src/wasm.rs`, which is what this file needs that no C caller
 * ever did. What wasm-bindgen's glue does for `main.ts` is done here by
 * hand, in the order it has to happen:
 *
 *   1. instantiate the module — one file, one name, no dynamic linker
 *   2. borrow `len + 1` bytes of its linear memory; write the UTF-8 and the NUL
 *   3. borrow four more for the `int *`
 *   4. call; a nonzero status is a typed failure, a trap is a defect
 *   5. read the answer back through a DataView; give both buffers back
 *
 * This is the file Exercise 4 has an attendee write. Everything hidden by
 * the generated lap is visible here, including the two things it never
 * shows: `memory.buffer` is re-read on every access because a growing
 * allocation detaches the old one, and each borrow is released through
 * `acquireUseRelease` — Effect's `Drop` — so the free runs on success,
 * failure and trap alike.
 *
 * Run via: just days wasm-demo 2015-12-01 (builds the module first); or
 * directly once the .wasm exists under days/target: npm run raw
 */
import { existsSync, readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

import { Cause, Console, Data, Effect, Exit } from "effect"

// wasm/src → wasm → 2015-12-01 → days; the repo root is one above that.
const DAYS = resolve(dirname(fileURLToPath(import.meta.url)), "../../..")
const INPUT = resolve(DAYS, "../inputs/2015-12-01.txt")

class ModuleNotBuilt extends Data.TaggedError("ModuleNotBuilt")<{ readonly hint: string }> {}
class BadModule extends Data.TaggedError("BadModule")<{ readonly detail: string }> {}
/** The module in target/ is the generated lap's, not the raw route's. */
class GeneratedModule extends Data.TaggedError("GeneratedModule")<{
  readonly path: string
  readonly imports: ReadonlyArray<string>
}> {}
class NoInput extends Data.TaggedError("NoInput")<{ readonly path: string }> {}
class OutOfMemory extends Data.TaggedError("OutOfMemory")<{ readonly size: number }> {}

/**
 * Call one raw export. The only thing this ABI can throw is a trap — a
 * Rust panic — and a trap is a bug, not an outcome to recover from, so it
 * goes to Effect's defect channel, where `catchAll` cannot see it. There is
 * no failure channel here at all: the exports return integers, and the
 * integers are handled below as data, which is the C API's whole design.
 * (boundary.ts, the generated lap's file, has a two-channel version of this
 * because wasm-bindgen's glue throws Errors for `Err`; nothing here does.)
 */
const callExport = <A>(fn: string, thunk: () => A): Effect.Effect<A> =>
  Effect.suspend(() => {
    try {
      return Effect.succeed(thunk())
    } catch (cause) {
      return Effect.die(new Error(`trap in ${fn}: ${cause instanceof Error ? cause.message : String(cause)}`))
    }
  })

/** A nonzero status from the C API. The C side already classified it; this
 *  just gives the number a name the compiler can see. */
class StatusError extends Data.TaggedError("StatusError")<{
  readonly fn: string
  readonly status: number
  readonly meaning: string
}> {}

const MEANING: Record<number, string> = {
  [-1]: "null pointer or invalid UTF-8 (the boundary refused the input)",
  [-2]: "Santa never enters the basement (the solver refused the puzzle)",
}

/** The export section, restated as types. There is no header to read: the
 *  module declares its own exports, with wasm types — `i32` where the C API
 *  says `const char *` and `int *`. The C types did not survive the trip. */
interface Raw {
  readonly memory: WebAssembly.Memory
  aoc_2015_12_01_alloc(size: number): number
  aoc_2015_12_01_free(ptr: number, size: number): void
  aoc_2015_12_01_part1(input: number, out: number): number
  aoc_2015_12_01_part2(input: number, out: number): number
}

// Debug first, then release — the same search the other tracks make. Just
// one filename to look for, though: a wasm module is `<crate>.wasm` on every
// host, because no host's loader is involved.
const loadRaw: Effect.Effect<Raw, ModuleNotBuilt | BadModule | GeneratedModule> = Effect.gen(function* () {
  const candidates = ["debug", "release"].map((profile) =>
    resolve(DAYS, "target/wasm32-unknown-unknown", profile, "aoc_2015_12_01.wasm"),
  )
  const path = candidates.find((p) => existsSync(p))
  if (path === undefined) {
    return yield* new ModuleNotBuilt({
      hint: "run: cd days && cargo build -p aoc-2015-12-01 --lib --target wasm32-unknown-unknown",
    })
  }
  const module = yield* Effect.try({
    try: () => new WebAssembly.Module(readFileSync(path)),
    catch: (e) => new BadModule({ detail: e instanceof Error ? e.message : String(e) }),
  })
  // The import section is readable before anything runs, and it is the
  // header's other half: what the module asks of the host. The raw route's
  // module asks for nothing (see `wasm.rs` for the one dependency that
  // would have made it ask). A module that does ask is the generated lap's
  // — `cargo build --features wasm` writes it to the same path — and only
  // wasm-bindgen's glue knows how to answer it. Better to say so than to
  // let instantiate fail on "Import #0" with no idea why there is one.
  const imports = WebAssembly.Module.imports(module)
  if (imports.length > 0) {
    return yield* new GeneratedModule({ path, imports: imports.map((i) => `${i.module}.${i.name}`) })
  }
  const instance = yield* Effect.try({
    try: () => new WebAssembly.Instance(module, {}),
    catch: (e) => new BadModule({ detail: e instanceof Error ? e.message : String(e) }),
  })
  return instance.exports as unknown as Raw
})

/**
 * Borrow `size` bytes of the module's memory for the duration of `use`.
 * Acquire is `alloc`, release is `free` with the same size, and release
 * runs however `use` ends — including by trap, which is the case `finally`
 * exists for in the Dart track and `Drop` exists for in Rust.
 */
const borrow = <A, E>(raw: Raw, size: number, use: (ptr: number) => Effect.Effect<A, E>) =>
  Effect.acquireUseRelease(
    Effect.suspend(() => {
      const ptr = raw.aoc_2015_12_01_alloc(size)
      return ptr === 0 ? Effect.fail(new OutOfMemory({ size })) : Effect.succeed(ptr)
    }),
    use,
    (ptr) => Effect.sync(() => raw.aoc_2015_12_01_free(ptr, size)),
  )

/**
 * One call through the C API: input bytes in, `int` out, status checked.
 * Takes bytes rather than a string so the hostile-input check below can
 * hand over bytes no string would encode to.
 */
const call = (raw: Raw, fn: "part1" | "part2", bytes: Uint8Array) =>
  borrow(raw, bytes.length + 1, (input) =>
    borrow(raw, 4, (out) =>
      Effect.gen(function* () {
        // The view is created after BOTH borrows: either alloc may have
        // grown the memory, and growth detaches every earlier ArrayBuffer.
        // alloc hands back uninitialised bytes, so the NUL is written, not
        // assumed — the C side reads until it finds one.
        const view = new Uint8Array(raw.memory.buffer, input, bytes.length + 1)
        view.set(bytes)
        view[bytes.length] = 0

        const f = fn === "part1" ? raw.aoc_2015_12_01_part1 : raw.aoc_2015_12_01_part2
        const status = yield* callExport(fn, () => f(input, out))
        if (status !== 0) {
          return yield* new StatusError({ fn, status, meaning: MEANING[status] ?? "unknown status" })
        }
        // Little-endian, because wasm is; `true` is the treaty's byte order.
        return new DataView(raw.memory.buffer).getInt32(out, true)
      }),
    ),
  )

const utf8 = (text: string) => new TextEncoder().encode(text)

const program = Effect.gen(function* () {
  const raw = yield* loadRaw

  if (!existsSync(INPUT)) return yield* new NoInput({ path: INPUT })
  const text = readFileSync(INPUT, "utf8")

  yield* Console.log(`Part 1 🧩(🦀): ${yield* call(raw, "part1", utf8(text))}`)
  yield* Console.log(`Part 2 🧩(🦀): ${yield* call(raw, "part2", utf8(text))}`)

  // The hostile-input contract, proved from this side. NULL is the integer
  // 0 here — there is no other spelling — and invalid UTF-8 is two bytes
  // written straight into the borrowed buffer, which no TextEncoder would
  // ever produce. Both must come back as status -1, never as an answer.
  const nullStatus = yield* borrow(raw, 4, (out) =>
    callExport("part1", () => raw.aoc_2015_12_01_part1(0, out)),
  )
  const garbage = yield* call(raw, "part1", new Uint8Array([0xff, 0xfe])).pipe(
    Effect.map(() => "an answer — WRONG"),
    Effect.catchTag("StatusError", (e) => Effect.succeed(`status ${e.status}`)),
  )
  yield* Console.log(`contract: part1(NULL) -> status ${nullStatus}; part1(0xff 0xfe) -> ${garbage}`)
  return nullStatus === -1 && garbage === "status -1"
})

const describe = (
  e: ModuleNotBuilt | BadModule | GeneratedModule | NoInput | OutOfMemory | StatusError,
): string => {
  switch (e._tag) {
    case "GeneratedModule":
      return (
        `${e.path} imports ${e.imports.length} thing(s) from the host (${e.imports[0] ?? ""}…) — ` +
        "that is the wasm feature's module, and only wasm-bindgen's glue can satisfy it. " +
        "The raw route wants the bare one: cd days && cargo build -p aoc-2015-12-01 --lib --target wasm32-unknown-unknown"
      )
    case "ModuleNotBuilt":
      return `no wasm module — ${e.hint}`
    case "BadModule":
      return `the module did not instantiate: ${e.detail}`
    case "NoInput":
      return `no puzzle input at ${e.path} (see .gitignore)`
    case "OutOfMemory":
      return `alloc(${e.size}) returned null`
    case "StatusError":
      return `${e.fn} failed with status ${e.status}: ${e.meaning}`
  }
}

const exit = await Effect.runPromiseExit(program.pipe(Effect.tapError((e) => Console.error(describe(e)))))
if (Exit.isFailure(exit)) {
  if (Cause.isDie(exit.cause)) console.error(Cause.pretty(exit.cause))
  process.exitCode = 1
} else if (!exit.value) {
  console.error("the hostile-input contract did not hold")
  process.exitCode = 1
}
