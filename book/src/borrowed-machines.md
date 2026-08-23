# CI: Verifying on Borrowed Machines

[Step -1](provisioning.md) made you responsible for your machine. This page
is about the machines nobody is responsible for: CI runners. The repo's
`Verify` workflow runs the same `scripts/self-check.sh` you ran at home,
on GitHub's stock images — and the interesting part is everything those
images already contain that we never asked for.

## The dirty environment

A GitHub runner is not a clean machine. It arrives with rustc and cargo
(current stable, on all three OSes), working C toolchains (gcc, Apple
clang, even MinGW on Windows), a JDK and `kotlinc`, Swift on two of the
three images, and a Python that — on macOS — can already `import cffi`.

None of that was provisioned by this repo. It is convenience, shipped by
GitHub for the median workflow, and it quietly lifts our provisioning
responsibility — right up until an image update takes something away.
That's the deal, and it's worth stating plainly: **preinstalled toolchains
are a courtesy, not a contract.** Nothing upstream promises `kotlinc`
will be there next month, and nothing on our side pins it.

This has a recognizable failure signature. When a matrix cell that has
been green for months goes red and *our diff is empty*, we didn't break
it — the borrowed machine changed underneath us. Knowing that signature
in advance is most of the diagnosis.

## Mapping before trusting

Before the workflow asserted anything, it ran the self-check *bare* on
every OS × track cell, no setup steps at all, to map what the images
actually cover. Findings, as of the mapping run:

| | linux | macos | windows |
|---|---|---|---|
| rustc/cargo (1.85 floor) | ✅ | ✅ | ✅ |
| C compiler + linker | ✅ | ✅ | ✅ MinGW |
| cbindgen | ❌ | ❌ | ❌ |
| Kotlin/JNA | ✅ | ✅ | ✅ |
| Swift | ✅ | ✅ | ○ |
| Python (cffi) | ○ | ✅ | ○ |
| Dart | ○ | ○ | ○ |

Two things the map taught us that guessing would not have: `cbindgen` is
the *single* required tool no image ships — and native Windows is far
closer to workshop-ready than our own "use WSL2" answer assumes (the
script never even reached its Windows failure hint; MinGW compiles and
links).

## The provisioning policy that falls out

- **Provision what no image ships.** `cbindgen`, installed explicitly in
  every cell, prebuilt and pinned. This is the one install that turns the
  required-toolchain rows green everywhere.
- **Provision tracks the way the workshop does.** The Python cell builds
  the same repo-local `.venv` that `just setup-python` builds — CI should
  exercise the attendee's recipe, not a shortcut that only works on
  runners. Dart gets the official `setup-dart` action, because the
  workshop's own answer is written for humans with package managers.
- **Don't provision what the image ships — but keep the fix on ice.**
  Kotlin and Swift are green for free today. The workflow carries their
  full provisioning steps *commented out*, checksummed and (for Kotlin)
  already validated, so the day an image update drops them, the fix is
  uncommenting a block — not archaeology under time pressure.
- **Never bend the self-check to flatter the runner.** The script's exit
  code is the attendee's contract: missing `cbindgen` fails, because at
  the workshop it would fail *you*. CI meets the contract by provisioning;
  a red cell on an unprovisioned runner is the map working, not a bug.

## Why the matrix is unusually small Today

Billed minutes round up per job, and macOS bills at 10×, Windows at 2× —
so the full 18-cell grid costs ~78 billed minutes per push while a
linux-only column costs ~6. The full map only needs taking when the
question is "what do the images ship?"; day-to-day pushes only ask "did
*we* break something?", and linux answers that at 1×. The grid widens
back to three OSes for pre-workshop sweeps, one `os:` line away.
