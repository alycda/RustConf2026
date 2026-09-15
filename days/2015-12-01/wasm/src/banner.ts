/**
 * The banner, rendered by the other engine.
 *
 * `caca.rs` banners each answer through libcaca's FIGfont engine, fed
 * `fonts/standard.flf`. libcaca cannot come to wasm32-unknown-unknown —
 * it is a C library found through pkg-config, and there is no libcaca for
 * the target — but the font is data, and figlet.js (patorjk) implements
 * the same FIGfont spec in JavaScript and takes raw `.flf` contents
 * through `parseFont`. So the answer crosses the boundary as a number,
 * and the banner is drawn on this side by a second, unrelated engine
 * reading the same file. `test/banner.test.ts` says how closely the two
 * agree.
 *
 * figlet.js is an untyped, callback-era library; this is the twenty lines
 * that make it a typed Effect service. (Not `@effect-ts/figlet`: that
 * package is pinned to the pre-v3 `@effect-ts/core` namespace and does
 * not interoperate with `effect`.)
 */
import { readFile } from "node:fs/promises"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

import { Data, Effect } from "effect"
import figlet from "figlet"

/** The same file `main.rs` hands to libcaca. */
export const STANDARD_FLF = resolve(dirname(fileURLToPath(import.meta.url)), "../../fonts/standard.flf")

/** figlet.js registers fonts by name; this is the name the .flf gets here. */
const FONT_NAME = "standard-flf"

export class FontError extends Data.TaggedError("FontError")<{
  readonly path: string
  readonly detail: string
}> {}

/** Parse the vendored font into figlet.js's registry. Idempotent. */
export const loadStandardFont: Effect.Effect<void, FontError> = Effect.tryPromise({
  try: async () => {
    const data = await readFile(STANDARD_FLF, "utf8")
    figlet.parseFont(FONT_NAME, data)
  },
  catch: (e) =>
    new FontError({ path: STANDARD_FLF, detail: e instanceof Error ? e.message : String(e) }),
})

/**
 * Render `text` as block letters, one string of `\n`-joined rows — the
 * shape `caca::figlet_banner` returns, minus its trailing newline.
 */
export const banner = (text: string): Effect.Effect<string, FontError> =>
  loadStandardFont.pipe(
    Effect.andThen(
      Effect.try({
        try: () => figlet.textSync(text, { font: FONT_NAME }),
        catch: (e) =>
          new FontError({ path: STANDARD_FLF, detail: e instanceof Error ? e.message : String(e) }),
      }),
    ),
  )
