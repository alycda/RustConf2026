# labs: the Breakage Gauntlet — four seeded binding flaws, sanitizer-proven in CI, and a hostile-caller duel

PR description for `labs/breakage-gauntlet` → `develop`. Written 2026-09-24; not yet opened.

An optional block after Ex 3 where attendees author failures on purpose. Every break has to map to a footgun family on the reference card, and a crash that maps to nothing scores nothing. Two stations:

- **Station B, the AI Binding Autopsy** (`labs/breakage-gauntlet/autopsy/`): four pre-generated bindings, one planted flaw each. Review it, name the family, write the one hostile input that exposes it, run `check.sh`. Each case is a tiny native library, the flawed binding, and a proof harness, with no dependency on the exercises or the day library.
- **Station A, the Hostile-Caller Duel** (`labs/breakage-gauntlet/duel/`): pairs swap generated Ex 2 headers and attack each other's glue using only inputs the header says it accepts. A forged pointer is UB roulette and scores nothing.

| Case | Flaw | Stack | Proven by |
|---|---|---|---|
| 01 | qsort comparator `x - y` overflows | C | wrong minimum, then UBSan |
| 02 | libc `free()` on `operator new[]` memory | Python over C++ | ASan `alloc-dealloc-mismatch` |
| 03 | Swift serializes Latin-1, native reads UTF-8 | Swift over Rust | wrong count, no crash |
| 04 | `c_char_p` truncates at an interior NUL; plus a read past an unterminated buffer | Python over Rust | wrong answer, then ASan |

**Where it comes from.** This is a port of a July branch on the old repo. The rest of that branch's siblings didn't survive the rebuild; this one does because the cases never touched the old exercises or day crates. Changes are prose only: the "sealed until 9 AM" framing is gone, the incident-card pipeline is not ported, and citations into files this repo doesn't have point at the reference card's "7 · Footguns" section instead. Case 04 keeps ctypes rather than cffi: the truncation is identical in both, and the binding is the fixture, not the track.

**One correction to the duel.** The old rules card said "any real answer is `>= 0`". #7's review named the problem: 2015-12-01's statement examples answer `-1`, so the Ex 2 scaffold's sentinel collides with a correct result. That is now attack class 5, the one class that scores against a defender who did everything the scaffold asked.

**Sanitizers are CI-only, and not through Nix.** `.github/workflows/gauntlet.yml` compiles the C-side proof harnesses with the runner's stock clang and `-fsanitize=address` / `-fsanitize=undefined`. It runs only on changes under `labs/breakage-gauntlet/`. Nothing in `rust.yml`, `env-check.yml`, `just check`, or the five-tool attendee contract changes. The ASan and UBSan runtimes ship with the runner's clang; relying on them from a nixpkgs clang on Darwin is unverified and this lab doesn't need it.

**What this fills.** #7 lists "five tracks never send a failing input" as out of scope, and the Part 5 plan asks for hostile inputs in every Verify cell. Neither exercises the review skill itself. The autopsy does: the output of a review here is an input that fails, not an opinion.

**Verified**
- All four `check.sh` scripts pass locally on Apple clang 17 and Swift 6.2.4: 01 and 02 abort under UBSan and ASan with the expected diagnostic, 04 truncates to 3 then aborts under ASan, 03 counts 0 of 3.
- `actionlint` clean on the workflow.
- The same workflow ran green on ubuntu and macOS on the old repo on 2026-07-02. The first run on this repo is in progress on this branch.

**Not covered**
- No timing. Like other material that hasn't been in front of a room, the per-station clocks are design estimates.
- No wasm variant of case 04 yet. A `(ptr, len)` boundary has no NUL to truncate at, which makes it the one footgun that track cannot have; that's a natural follow-up once #4 lands.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
