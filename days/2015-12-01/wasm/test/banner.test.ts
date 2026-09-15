/**
 * The golden test this variant unlocks — and its verdict.
 *
 * `test/fixtures/caca-banner.txt` is what `cargo run --features caca`
 * printed for the statement example "()())": the "🦀:" header, then the
 * banners for -1 (part 1) and 5 (part 2), six rows each, trailing spaces
 * intact. Captured once in `nix-shell -p libcaca pkg-config` on
 * 2026-09-15; regenerate it the same way if `caca.rs` or the font changes.
 *
 * The verdict, measured rather than hoped for: the glyph rows are
 * identical, and figlet.js puts one extra space in front of every row.
 * The reference `figlet` CLI (2.2.5, same .flf) agrees with libcaca byte
 * for byte, so two of the three engines agree and the JavaScript one is
 * off by a column — on every horizontalLayout, and with its own bundled
 * "Standard" font too, so it is figlet.js's rendering, not this font or
 * this call. A near-miss with a known width is a better exhibit than a
 * clean match, and both tests below encode it: the first holds the
 * agreement, the second holds the divergence and fails the day it closes.
 *
 * Run: npm test
 */
import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { test } from "node:test"
import { fileURLToPath } from "node:url"

import { Effect } from "effect"

import { banner } from "../src/banner.js"

const FIXTURE = resolve(dirname(fileURLToPath(import.meta.url)), "fixtures/caca-banner.txt")

/** The fixture, split back into the two banners libcaca drew. */
const cacaBanners = (): { minusOne: string; five: string } => {
  const lines = readFileSync(FIXTURE, "utf8").split("\n")
  assert.equal(lines[0], "🦀:", "fixture starts with the header main.rs prints")
  // Six rows per banner (the font's height), a trailing "" from the final newline.
  return {
    minusOne: lines.slice(1, 7).join("\n"),
    five: lines.slice(7, 13).join("\n"),
  }
}

const render = (text: string) => Effect.runPromise(banner(text))

/** figlet.js's output with its extra leading column removed. */
const shiftedLeft = (s: string) =>
  s
    .split("\n")
    .map((row) => {
      assert.ok(row.startsWith(" "), `row ${JSON.stringify(row)} has no leading column to drop`)
      return row.slice(1)
    })
    .join("\n")

test("same font, same glyphs: figlet.js and libcaca agree once the leading column is dropped", async () => {
  const caca = cacaBanners()
  assert.equal(shiftedLeft(await render("-1")), caca.minusOne)
  assert.equal(shiftedLeft(await render("5")), caca.five)
})

test("the divergence is exactly one leading column (if this fails, figlet.js converged — drop shiftedLeft)", async () => {
  const caca = cacaBanners()
  for (const [text, expected] of [
    ["-1", caca.minusOne],
    ["5", caca.five],
  ] as const) {
    const js = await render(text)
    assert.notEqual(js, expected, `figlet.js now matches libcaca byte for byte on ${JSON.stringify(text)}`)
    const jsRows = js.split("\n")
    const cacaRows = expected.split("\n")
    assert.equal(jsRows.length, cacaRows.length, "same number of rows")
    for (let i = 0; i < jsRows.length; i++) {
      assert.equal(jsRows[i], ` ${cacaRows[i]}`, `row ${i} of ${JSON.stringify(text)} differs by more than one leading space`)
    }
  }
})
