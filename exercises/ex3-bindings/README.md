# Exercise 3: Bindings in Your Track (35 min)

## Goal

Call the library you built in Ex 2 from **one** higher-level language.

## Route

Bind directly against your `include/ex2_c_glue.h`. You'll touch every
concept from Module 3 — loading, lookup, string conversion, ownership.

## Tasks

0. Ex 2 must be green. `../ex2-c-glue/build-and-test.sh` produces the
   header every track binds against and the library every track loads;
   both are gitignored, so they exist only after that script has run on
   this machine.
1. Pick **one** track below and open its file. The TODOs run top to
   bottom; each file's header comment has the exact command to run it
   (`just exercises <track>` from the repo root runs the same line).
2. Fill them in and make the run pass against your day's example.
3. Prove the hostile-input contract from your language — the last TODO
   in every track. That is where the debrief question below gets its answer.

## Tracks

| Track | Start here | Worked reference |
|-------|-----------|------------------|
| Python | `python/` | [`../../days/2024-12-01/python/solve.py`](../../days/2024-12-01/python/solve.py) |
| Swift | `swift/` | [`../../days/2024-12-01/swift/solve.swift`](../../days/2024-12-01/swift/solve.swift) |
| Kotlin/JNA | `kotlin/` | [`../../days/2024-12-01/kotlin/solve.kt`](../../days/2024-12-01/kotlin/solve.kt) |
| Dart | `dart/` | [`../../days/2024-12-01/dart/solve.dart`](../../days/2024-12-01/dart/solve.dart) |

The debrief question you're collecting an answer to: **what did your
language runtime need that the C header couldn't say?**
