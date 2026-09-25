# Autopsy 04 — Review Worksheet

You are handed one AI-generated binding: `flawed_binding.py` over a Rust
`nativelib` that sums the ASCII digits in a string. It passes on clean ASCII
input and ships.

**One planted flaw. It compiles fine and lies later — a wrong, truncated
answer, no crash.**

Do not read `SOLUTION.md` until this file is filled in.

## 1. Find it

Read `flawed_binding.py`. What happens when the incoming bytes contain an
interior `\x00`?

> _line / call:_ ____________________________________________

## 2. Name the family

Which footgun family (the reference card (`docs/reference-card.typ`, "7 · Footguns"))?

> _family:_ ____________________________________________

## 3. Write the ONE hostile input that exposes it

Give a `bytes` value that returns the wrong sum, and say what the right sum
would be. Note: this stays **in contract** — the bytes handed over are a
well-formed NUL-terminated C string.

> _input:_ ______________  _expected vs actual:_ ______________

## 4. Run it

```bash
./check.sh     # cargo + python3, then a C compiler with -fsanitize=address
```

`check.sh` also demonstrates the *out-of-contract* sibling (`readpast.c`): a
buffer with **no** terminator, caught by ASan as a read past the end. Why is
that second one banned from the duel but allowed here?

> _because:_ __________________________________________
