# Autopsy 04 — SOLUTION

## The flaw

`flawed_binding.py`, in `sum_digits`:

```python
return lib.sum_digits(ctypes.c_char_p(raw))   # truncates at the first NUL
```

`ctypes.c_char_p` (like cffi's `ffi.new("char[]", ...)` used as a string, and
like C's own string functions) treats the bytes as a NUL-terminated C string.
The moment it hits the interior `\x00`, everything after it is dropped. The
native side never sees the second field.

## Family

**Forget the NUL → C reads past (or stops short of) your string.**
(the reference card, "7 · Footguns".)

## The one hostile input

```python
sum_digits(b"12\x0034")   # expected 10, actual 3  (only "12" survives)
```

This is **in contract**: the bytes actually handed to the native side
(`"12"` + terminator) are a well-formed, NUL-terminated C string. The bug is
that it is not the string the caller *meant* to send — the embedded NUL
silently cut it in half. The clean fixture `b"1234"` has no interior NUL, so it
returns 10 and hides the flaw.

## The out-of-contract sibling (`readpast.c`)

R6 names two faces of this family. The one above is the safe, in-contract
truncation. The other is forgetting the terminator entirely, so a C consumer
reads off the end of the allocation — `readpast.c` does exactly that and ASan
catches it as a `heap-buffer-overflow`. **That forged, non-terminated buffer is
banned from the duel** (safety rule: UB roulette, not a boundary lesson). It
lives here, in CI, only because `-fsanitize=address` turns the coin-flip crash
into a deterministic diagnostic.

## Why it compiles fine

`c_char_p` is the correct, idiomatic way to pass a C string; passing `bytes` to
it is well-typed. Nothing signals that the payload might legitimately contain a
`0x00`. The gap is between "a Python `bytes` object (length-prefixed, NULs
allowed)" and "a C string (NUL-terminated, NULs forbidden mid-string)" — two
different notions of "string" meeting at the boundary.

## How CI proves it (`check.sh`, real output)

```
clean : 10 (expected 10)
record: 3 (intended 10 — truncated to just '12')
OK: embedded NUL truncated the input (3 instead of 10).
...
==NNNNN==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x...
FLAW CONFIRMED: missing/embedded NUL — silent truncation + read past end.
```

## The fix

Pass an explicit length instead of relying on a terminator — e.g. expose a
`sum_digits_n(ptr, len)` taking a pointer + byte count, and hand it
`(raw, len(raw))`. When your data can contain NUL bytes, a NUL-terminated C
string is the wrong wire format.
