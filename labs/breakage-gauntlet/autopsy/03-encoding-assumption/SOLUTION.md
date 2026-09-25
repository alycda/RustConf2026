# Autopsy 03 — SOLUTION

## The flaw

`FlawedBinding.swift`, in `countEAcute`:

```swift
var bytes = Array(s.data(using: .isoLatin1) ?? Data())   // wrong encoding
```

The native side documents "input is UTF-8," but the binding serialises the
Swift `String` as **ISO Latin-1**. In Latin-1, 'é' is the single byte `0xE9`;
in UTF-8 it is the two bytes `0xC3 0xA9`. The native `from_utf8_lossy` sees a
lone `0xE9`, which is not valid UTF-8, replaces it with U+FFFD, and so counts
zero 'é' characters.

## Family

**Encoding mismatch corrupts silently — no crash, wrong answer.**
(the reference card, "7 · Footguns".)

## The one hostile input

```
"café résumé"   -> expected 3, actual 0
```

- **Breaks it:** any string containing a non-ASCII character.
- **Hides it:** any pure-ASCII string — Latin-1 and UTF-8 are byte-identical
  for code points 0–127, so every ASCII test fixture passes and the encoding
  bug is invisible until real accented data arrives.

## Why it compiles fine

`data(using: .isoLatin1)` is a completely valid Swift API returning valid
bytes; the pointer dance type-checks; the native call type-checks. Nothing is
wrong syntactically. The mismatch is purely semantic — two layers each holding
a different, unstated assumption about which encoding the bytes are in.

## How CI proves it (`check.sh`, real output)

```
count_e_acute("café résumé") = 0 (expected 3)
FLAW MANIFESTED: encoding mismatch silently produced the wrong count.
FLAW CONFIRMED: encoding mismatch corrupts silently.
```

No sanitizer, no crash — the proof is a pure wrong-answer assertion (`got !=
expected`). This is the cheapest flaw to prove and the scariest in production:
nothing ever tells you it is wrong.

## The fix

Hand the native side UTF-8, which is what it asked for:

```swift
return s.withCString { count_e_acute($0) }   // Swift's withCString is UTF-8
```
