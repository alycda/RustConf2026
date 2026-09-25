# Autopsy 01 — Review Worksheet

You are handed one AI-generated binding: a Python wrapper (`flawed_binding.py`)
over a small C helper (`sortlib.c`) that sorts sensor readings and returns the
smallest. It compiles clean and passes on the puzzle's tiny ASCII example.

**One planted flaw. It compiles fine and lies (or crashes) later.**

Do not read `SOLUTION.md` until this file is filled in.

## 1. Find it

Read `sortlib.c` and `flawed_binding.py`. Where is the boundary bug?

> _line / function:_ ____________________________________________

## 2. Name the family

Which footgun family from the reference card
(the reference card (`docs/reference-card.typ`, "7 · Footguns")) is this?

> _family:_ ____________________________________________

## 3. Write the ONE hostile input that exposes it

Give a concrete input list where the answer comes back **wrong** (not a crash —
a wrong number). Why does the tiny example hide it?

> _input:_ ____________________________________________
>
> _expected vs actual:_ ________________________________

## 4. Run it

```bash
./check.sh          # CC=<your clang> ./check.sh to pick a compiler
```

You should see (a) a wrong answer from a plain build and (b) a sanitizer that
turns the flaw from "sometimes wrong" into "always caught." Which sanitizer,
and which flag?

> _sanitizer + flag:_ __________________________________
