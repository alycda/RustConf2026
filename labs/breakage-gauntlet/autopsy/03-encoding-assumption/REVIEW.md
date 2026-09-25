# Autopsy 03 — Review Worksheet

You are handed one AI-generated binding: `FlawedBinding.swift` over a Rust
`nativelib` that counts occurrences of 'é' in a UTF-8 string. It passes on the
ASCII test fixtures and ships.

**One planted flaw. It compiles fine and lies later — no crash.**

Do not read `SOLUTION.md` until this file is filled in.

## 1. Find it

Read `FlawedBinding.swift` against the native contract stated at the top of
`nativelib/src/lib.rs`. How does the binding turn the Swift `String` into bytes
for the native side?

> _line / call:_ ____________________________________________

## 2. Name the family

Which footgun family (the reference card (`docs/reference-card.typ`, "7 · Footguns"))?

> _family:_ ____________________________________________

## 3. Write the ONE hostile input that exposes it

Give an input where the count comes back wrong, and one where it would come
back *right* (hiding the flaw). What is special about the difference?

> _breaks it:_ ______________  _hides it:_ ______________

## 4. Run it

```bash
./check.sh     # cargo + swiftc; macOS
```

Does anything crash? If not, what did you observe, and why is a silent wrong
answer arguably *worse* than a crash?

> _observation:_ ______________________________________
