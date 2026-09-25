# The Breakage Gauntlet

An optional block, after Ex 3, where attendees **author failures on purpose**.
Every break has to map to a named footgun family from the reference card
(`docs/reference-card.typ`, "7 · Footguns"); a spectacular crash that maps to
nothing scores nothing. The debrief already bets its payoff on "what broke?".
The Gauntlet makes that deliberate instead of incidental, and makes review the
skill being exercised: the output of a review here is a hostile input that
fails, not an opinion.

This carries no timing until it has been in front of a room. The
per-station clocks below are the estimates it was designed to, not measurements.

## Two stations

| # | Station | What the attendee does | Designed for |
|---|---------|------------------------|:------------:|
| A | **Hostile-Caller Duel** | Write `attack.c` against a partner's published Ex 2 header using only in-contract inputs; the partner runs it against their own glue. | 20 min |
| B | **AI Binding Autopsy** | Review one pre-generated binding with a single planted flaw. Find it, name its family, write the ONE hostile input that exposes it, run it. | 20 min |

### Station A — Hostile-Caller Duel

1. Pairs swap **published headers only** (`exercises/ex2-c-glue/include/ex2_c_glue.h`,
   as `build-and-test.sh` generates it). Not the source.
2. Attacker fills in [`duel/attack_template.c`](duel/attack_template.c) with
   in-contract inputs: NULL, invalid UTF-8, an embedded NUL, an absurd length,
   or an input whose real answer is the sentinel.
3. Defender builds the attacker's `attack.c` against their own cdylib and runs it.
4. Score. Catalog-predicted breaks only.

Rules card, printable: [`duel/RULES.md`](duel/RULES.md). It needs nothing
beyond the Ex 2 toolchain.

### Station B — AI Binding Autopsy

1. Attendee gets one binding with one planted flaw, told only "compiles clean,
   crashes or lies later."
2. Code review: locate the flaw, name its family, in `REVIEW.md`.
3. Write the one hostile input that exposes it; run `./check.sh`.
4. Reveal `SOLUTION.md` and compare.

Four seeded flaws, one per boundary family: [`autopsy/`](autopsy/). Each is
self-contained (a tiny native library, the flawed binding, a proof harness) and
none of them depend on the exercises or the day library.

## The safety rule (read this before scoring anything)

All sabotage stays **inside the declared failure modes of the published header
contract**:

```
ex_part1(input: *const c_char) -> i64
// input must be a valid NUL-terminated C string or null
```

Before a break counts, adjudicate the input:

- **In-contract** — a well-formed, NUL-terminated C string, or NULL. These are
  values the header *says it accepts*. Scoreable.
- **Void** — a hand-forged wild or unterminated pointer, or non-string bytes
  the header never promised. That is UB roulette, not a boundary lesson: zero
  points, however good the crash.

The "forget the NUL" family is exercised in the duel via an *embedded* NUL (a
legal C string that truncates silently), never via an unterminated buffer. The
read-past-end variant of that family lives only in the Autopsy's proof harness,
behind a sanitizer, where it is deterministic.

## Scoring

A point is awarded **iff both**:

1. the input is a well-formed NUL-terminated C string (or NULL), **and**
2. the observed failure maps to a named catalog family.

A NULL or invalid-UTF-8 input against a *correctly implemented* defender scores
**0**: the Ex 2 harness already asserts `ex_part1(NULL) == -1` and
`ex_part1("\xff\xfe…") == -1`. Those classes score only against a defender who
skipped the check. The sentinel class is the exception that scores against a
correct defender: the scaffold's `-1` is only a sentinel if the day's answer can
never be `-1`, and on 2015-12-01 it can.

| Break | Points |
|-------|:------:|
| In-contract input + catalog-predicted failure (crash, silent wrong answer, truncation) | **1** |
| NULL / invalid-UTF-8 against a defender who skipped the guard | **1** |
| A real answer that equals the sentinel, against any defender | **1** |
| NULL / invalid-UTF-8 against a correct defender (handled, returns `-1`) | **0** |
| Out-of-contract forged pointer / non-string bytes (UB roulette) | **0 (void)** |

Ties broken at the debrief: the best-told bug wins the room.

## Decision: sanitizers are CI-only

The attendee toolchain is the five-tool contract from the README (`rustc`,
`cargo`, `cbindgen`, a C compiler, `just`) and this lab adds nothing to it. Two
of the Autopsy flaws (the cross-allocator free and the read past an
unterminated buffer) are memory bugs a vanilla toolchain catches only by luck.
They are proven **deterministically in CI** by compiling the C-side proof
harness with the runner's stock clang and `-fsanitize=address` /
`-fsanitize=undefined`. No nightly Rust, no `-Zsanitizer`, and not through the
Nix shell: the sanitizer runtimes come with the runner's clang, and that is the
only place they are relied on.

Attendees meet those two flaws the honest way: a crash-or-wrong-answer that may
or may not fire on their machine, which is exactly the "never rely on a chance
segfault" lesson. CI is the impartial judge that makes the flaw deterministic.

Sanitizers are enabled **only** in `.github/workflows/gauntlet.yml`, never in
`rust.yml`, `env-check.yml`, or `just check`.
