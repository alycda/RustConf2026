#!/usr/bin/env python3
"""Exercise 3, Python track (cffi).

Bind your Ex 2 library and call it from Python. Fill in the TODOs top to
bottom; run with:  python3 bindings.py
"""

import sys
from pathlib import Path

from cffi import FFI

ffi = FFI()

# TODO 1: declare the C interface. Open ../../ex2-c-glue/include/ex2_c_glue.h
# and copy the two function declarations here (cffi parses C, so the header's
# lines work almost verbatim):
ffi.cdef("""
    // paste the two int64_t ex_part*(const char*) declarations here
""")

# TODO 2: load the library. Adjust the extension for your platform
# (.dylib on macOS, .so on Linux).
LIB_PATH = Path(__file__).parent / ".." / ".." / "ex2-c-glue" / "target" / "release" / "libex2_c_glue.dylib"
lib = ffi.dlopen(str(LIB_PATH.resolve()))

# TODO 3: call it. Python strs are NOT C strings — encode first.
# Question worth answering while you're here: what does cffi do with the
# NUL terminator, and what would happen with an embedded NUL?
EXAMPLE = "PASTE YOUR DAY'S EXAMPLE INPUT HERE"
EXPECTED_PART1 = 0  # from the puzzle statement

result = None  # replace: lib.ex_part1(EXAMPLE.encode("utf-8"))

if result != EXPECTED_PART1:
    print(f"part1 = {result}, expected {EXPECTED_PART1}")
    sys.exit(1)

# TODO 4 (the fun one): prove the hostile-input contract holds from Python.
# What Python value even *is* a null pointer here? (Hint: ffi.NULL)
assert lib.ex_part1(ffi.NULL) == -1

print("Ex 3 (Python) passed.")
