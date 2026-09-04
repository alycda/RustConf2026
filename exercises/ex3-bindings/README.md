# Exercise 3: Bindings in Your Track (35 min)

## Goal

Call the library you built in Ex 2 from **one** higher-level language.

## Pick your route

Every track offers two routes — pick based on how Ex 2 went:

- **Hand-written route** (recommended first): bind directly against your
  `include/ex2_c_glue.h`. You'll touch every concept from Module 3 —
  loading, lookup, string conversion, ownership.
- **Generated route**: add UniFFI to your crate instead (pattern:
  [`../../days/2024-03/uniffi/`](../../days/2024-03/uniffi/)) and let the
  bindgen produce the bindings. Less typing, different lesson: read what it
  generated and find where it does the steps you did by hand in Ex 2.

## Tracks

| Track | Start here | Worked reference |
|-------|-----------|------------------|
| Python | `python/` | `../../days/2024-03/uniffi/tests/python/` (generated) |
| Swift | `swift/` | `../../step8-uniffi-swift/` |
| Kotlin/JNI | `kotlin/` | `../../step7-uniffi-kotlin/` |
| Dart | `dart/` | `../../days/2024-03/dart/` |

The debrief question you're collecting an answer to: **what did your
language runtime need that the C header couldn't say?**
