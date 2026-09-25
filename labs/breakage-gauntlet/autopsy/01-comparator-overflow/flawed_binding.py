#!/usr/bin/env python3
"""AI-generated Python binding over sortlib.c (built as a shared library).

Exposes `smallest_reading` to Python via ctypes. The Python side is faithful:
the flaw is entirely in the C comparator it wraps. This file is here so the
reviewer sees the *whole* generated boundary, binding included.

Build the library first (check.sh does this for you):
    cc -shared -fPIC sortlib.c -o libsortlib.so     # Linux
    cc -shared -fPIC sortlib.c -o libsortlib.dylib   # macOS
"""
import ctypes
import sys
from pathlib import Path

here = Path(__file__).parent
libname = "libsortlib.dylib" if sys.platform == "darwin" else "libsortlib.so"
lib = ctypes.CDLL(str(here / libname))
lib.smallest_reading.restype = ctypes.c_int
lib.smallest_reading.argtypes = [ctypes.POINTER(ctypes.c_int), ctypes.c_size_t]


def smallest_reading(readings):
    n = len(readings)
    arr = (ctypes.c_int * n)(*readings)
    return lib.smallest_reading(arr, n)


if __name__ == "__main__":
    # Tiny ASCII-scale example: works fine, ships with confidence.
    print("small:", smallest_reading([5, 2, 9, 1, 7]))  # -> 1, correct

    # Real sensor data spans the full i32 range. Two readings far apart:
    readings = [2_000_000_000, -2_000_000_000, 5]
    print("wide :", smallest_reading(readings),
          "(correct answer is -2000000000)")
