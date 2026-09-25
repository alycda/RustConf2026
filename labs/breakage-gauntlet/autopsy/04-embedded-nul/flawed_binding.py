#!/usr/bin/env python3
"""AI-generated Python binding over nativelib (sum_digits).

The binding receives a record whose digits arrive in two fields joined by a
separator byte that, in this data source, happens to be 0x00. It builds the
C string with ctypes.c_char_p and passes it straight through. c_char_p stops
at the first NUL, so the native side only ever sees the first field — a
silently truncated (wrong) sum. No crash.

This is an IN-CONTRACT input: the bytes handed over are a well-formed,
NUL-terminated C string. It is just not the string the caller *thought* they
sent. (Contrast readpast.c, which forgets the terminator entirely — that is the
out-of-contract, read-past-end variant, allowed only here in CI under ASan.)

    (cd nativelib && cargo build --release)
"""
import ctypes
import sys
from pathlib import Path

here = Path(__file__).parent
libname = "libnativelib.dylib" if sys.platform == "darwin" else "libnativelib.so"
lib = ctypes.CDLL(str(here / "nativelib" / "target" / "release" / libname))
lib.sum_digits.restype = ctypes.c_int64
lib.sum_digits.argtypes = [ctypes.c_char_p]


def sum_digits(raw: bytes) -> int:
    # THE FLAW: c_char_p treats the bytes as a C string and truncates at the
    # first NUL. Everything after the embedded \x00 is silently dropped.
    return lib.sum_digits(ctypes.c_char_p(raw))


if __name__ == "__main__":
    # ASCII example with no embedded NUL: works fine, ships.
    print("clean :", sum_digits(b"1234"), "(expected 10)")

    # Real record: field "12", separator 0x00, field "34". Intended sum 10.
    record = b"12\x0034"
    print("record:", sum_digits(record), "(intended 10 — truncated to just '12')")
