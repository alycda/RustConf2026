# AI in This Workshop (yes, really — read this)

This file is for **you, the attendee**. It's also loaded by AI coding agents
(Claude Code, Codex, and friends) working in this repo, which is exactly the
point: this workshop is transparent about AI because pretending it doesn't
exist teaches you nothing.

## The honest part

AI helped build this workshop. Concretely, it was used for:

- **C test boilerplate and build scripts** — mechanical, verbose, well-trodden
- **Build archaeology** — the Swift-on-Linux toolchain question, for one; the
  findings live in `.devcontainer/swift/README.md`, because the facilitator
  *will* forget what she learned
- **Suggesting which AoC days suit FFI practice**, and which C libraries are
  interesting to (ab)use
- Porting solved puzzles between repos and untangling helper-crate
  dependencies

Every one of those was reviewed, tested, and is maintained by a human. That's
the contract.

## The part that matters: what AI can't do for you today

AI can produce a finished binding for any puzzle in this repo in about a
minute. If that's what you wanted, you could have stayed home — and that's
precisely why the *learning* here is somewhere else:

1. **Boundary design.** Deciding what crosses the FFI boundary — a struct? an
   opaque handle? a string-in/int-out facade? — is a judgment call about
   ownership, encodings, and the lowest common denominator of every runtime
   you target. AI will happily generate any of the three; it won't tell you
   which one your production system will regret in a year.
2. **Debugging at the boundary.** Segfaults, encoding mismatches, and linker
   errors don't come with stack traces that respect language boundaries. The
   intuition for *where to look* is built by hitting these errors yourself —
   that's the productive struggle this workshop is designed around.
3. **Knowing when the answer is wrong.** Generated FFI code fails in ways that
   compile fine and crash later. Exercise 2 is built around `todo!()` for this
   reason: the boundary is yours to specify, and a plausible-looking
   `extern "C"` signature can be wrong in ways the compiler will never tell
   you. Review skill is the durable skill.

## Guardrails for using AI during the exercises

AI assistance is **welcome** in every exercise. The goal is understanding the
boundary, not typing speed. Some rules of engagement that will keep the
learning intact:

- **Delegate the boring, keep the boundary.** Let AI write the C test harness
  scaffolding or recall `cbindgen` flags. Write the `extern "C"` interface and
  the ownership contract yourself first — then compare with what AI suggests,
  and argue with it.
- **Hit the error before asking for the fix.** When the linker screams or
  Swift can't find a symbol, read the error and form a hypothesis before
  pasting it into a chat. The debrief at the end of the workshop harvests the
  best broken things — your error is a contribution.
- **Ask AI "why", not just "fix".** "Why does this comparator overflow?"
  teaches; "make the red go away" doesn't.
- **Never paste your AoC puzzle input into anything public** — and don't
  commit it here either (see below).

## Rules for agents working in this repo

- **Never commit AoC puzzle inputs or full puzzle text.** Only the small
  example inputs from problem statements may appear in tests
  ([AoC's request](https://adventofcode.com/about#faq_copying)). This covers
  doc comments too: paraphrase the puzzle, don't paste it. Nothing automated
  enforces this. `days/.gitignore` ignores `**/inputs/*` and the root
  `.gitignore` ignores a stray `inputs/`, but neither stops puzzle *text* in a
  doc header.
- **`days/README.md` is the menu and the authority.** Adding a day means
  adding a row to its table; its Rules section is where a per-day decision
  gets settled once instead of re-litigated. It does not yet record a
  dependency policy — until it does, do not assume the days are std-only.
  `exercises/ex1-pure-rust/Cargo.toml` is explicit that they are not.
- **Nix is the default attendee path, not facilitator-only tooling.** Every
  fix hint in `scripts/self-check.sh` leads with it — *"nix shell provides it:
  direnv allow"* — and names rustup only as the parenthetical fallback. There
  are three supported routes, and attendee-facing steps must not assume only
  one of them:
  1. **`direnv allow`** (or `nix-shell`). The easy path, and the expected one.
  2. **A devcontainer**, for attendees who can't or won't do the admin-level
     nix install — that is what they are for. The `[linux]` setup recipes point
     at them directly.
  3. **Bring your own toolchain** — rustup, a system cbindgen, a C compiler.

  What is actually gated is **rustc, cargo, cbindgen, and a C compiler**, plus
  `just` to invoke anything. Per-track tools (Swift, Kotlin/JNA, Python, Dart)
  are optional, one per attendee, via `just setup-<track>`. **Jujutsu is
  genuinely facilitator-only** — attendees clone with git.
- **`exercises/` is the attendee's working directory and a separate cargo
  workspace from `days/`.** It has its own `Cargo.lock`, its members are
  listed by name rather than globbed, and the crates are supposed to be
  unfinished — they are built around `todo!()`. Do not "fix" them into
  compiling cleanly, and do not add them to `days/`, whose CI gate would then
  fail for everyone.
- Follow the existing exercise-folder pattern when adding stages:
  `exercises/exN-*/README.md`, one directory per workshop block.
