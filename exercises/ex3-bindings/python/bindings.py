#!/usr/bin/env python3
"""Exercise 3, Python track (cffi).

Bind your Ex 2 library and call it from Python. Fill in the TODOs top to
bottom; run inside the repo's venv (`just setup-python` once, then
`source .venv/bin/activate` — `.venv\\Scripts\\activate` on Windows):

    python3 bindings.py
"""

import sys
from pathlib import Path

try:
    from cffi import FFI
except ImportError:
    sys.exit("cffi not found — run: just setup-python, then: source .venv/bin/activate")

ffi = FFI()

# TODO 1: declare the C interface. Open ../../ex2-c-glue/include/ex2_c_glue.h
# and copy the two function declarations here (cffi parses C, so the header's
# lines work almost verbatim):
ffi.cdef("""
    // paste the two int64_t ex_part*(const char*) declarations here
""")

# TODO 2: load the library. Nothing to fill in here, but read it: the
# exercises are one cargo workspace, so the cdylib lands in ../../target/
# (not inside ex2-c-glue/), and it takes the host's name, not Rust's —
# libex2_c_glue.so on Linux, libex2_c_glue.dylib on macOS, ex2_c_glue.dll
# (no lib prefix) on Windows. Searching the three needs no platform check:
# whichever file cargo produced is the one that exists.
TARGET = Path(__file__).resolve().parent / ".." / ".." / "target"


def load_library():
    for profile in ("debug", "release"):
        for name in ("libex2_c_glue.so", "libex2_c_glue.dylib", "ex2_c_glue.dll"):
            candidate = TARGET / profile / name
            if candidate.exists():
                return ffi.dlopen(str(candidate.resolve()))
    sys.exit("no Ex 2 library found — run ../../ex2-c-glue/build-and-test.sh first")


lib = load_library()

# TODO 3: call it. Python strs are NOT C strings — encode first.
# Question worth answering while you're here: what does cffi do with the
# NUL terminator, and what would happen with an embedded NUL?
EXAMPLE = "PASTE YOUR DAY'S EXAMPLE INPUT HERE"
EXPECTED_PART1 = 0  # from the puzzle statement

result = None  # TODO 3: replace with  lib.ex_part1(EXAMPLE.encode("utf-8"))
if result is None:
    sys.exit("TODO 3 not done — call lib.ex_part1 with the encoded example")

if result != EXPECTED_PART1:
    print(f"part1 = {result}, expected {EXPECTED_PART1}")
    sys.exit(1)

# TODO 4 (the fun one): prove the hostile-input contract holds from Python.
# What Python value even *is* a null pointer here? (Hint: ffi.NULL)
assert lib.ex_part1(ffi.NULL) == -1

print("Ex 3 (Python) passed.")
