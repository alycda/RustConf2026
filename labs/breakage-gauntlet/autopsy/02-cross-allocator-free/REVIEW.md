# Autopsy 02 — Review Worksheet

You are handed one AI-generated binding: `flawed_binding.py` over
`nativelib.cpp`. The native lib renders an answer string on its own heap and
returns a raw pointer, documenting `answer_free()` as the release path. The
Python binding reads the string and cleans up. It runs fine and prints `42`.

**One planted flaw. It compiles fine and (on a bad day) crashes later.**

Do not read `SOLUTION.md` until this file is filled in.

## 1. Find it

Read `flawed_binding.py` against the contract in `nativelib.cpp`'s header
comment. What does the binding do with the returned pointer?

> _line / call:_ ____________________________________________

## 2. Name the family

Which footgun family (the reference card (`docs/reference-card.typ`, "7 · Footguns"))?

> _family:_ ____________________________________________

## 3. Write the ONE hostile input that exposes it

This flaw has **no** hostile *input* — any call triggers it. So the question is:
what would you have to do to make it fail *deterministically* instead of by
luck? (Hint: the attendee toolchain can't, but CI can.)

> _how to force it:_ __________________________________

## 4. Run it

```bash
python3 flawed_binding.py     # prints 42, looks fine — the trap
./check.sh                    # CXX/CC=<clang> to pick a compiler
```

`check.sh` builds the same free with a sanitizer. Which one, and what does it
call the error?

> _sanitizer + error name:_ ___________________________
