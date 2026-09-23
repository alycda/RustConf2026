#!/usr/bin/env python3
"""Exercise 3, Python track (cffi) — solved. The CI overlay for the attendee
scaffold (see .github/ci/README.md): the same file with its TODOs filled,
run the way the scaffold says, inside the repo's venv:

    python3 bindings.py
"""

import sys
from pathlib import Path

try:
    from cffi import FFI
except ImportError:
    sys.exit("cffi not found — run: just setup-python, then: source .venv/bin/activate")

ffi = FFI()

# TODO 1, done: the two declarations from ../../ex2-c-glue/include/ex2_c_glue.h.
ffi.cdef("""
    int64_t ex_part1(const char *input);
    int64_t ex_part2(const char *input);
""")

# TODO 2, as shipped: one cargo workspace, three filenames, no platform check.
TARGET = Path(__file__).resolve().parent / ".." / ".." / "target"


def load_library():
    for profile in ("debug", "release"):
        for name in ("libex2_c_glue.so", "libex2_c_glue.dylib", "ex2_c_glue.dll"):
            candidate = TARGET / profile / name
            if candidate.exists():
                return ffi.dlopen(str(candidate.resolve()))
    sys.exit("no Ex 2 library found — run ../../ex2-c-glue/build-and-test.sh first")


lib = load_library()

# TODO 3, done: Python strs are not C strings — encode first. cffi appends
# the NUL itself; an embedded NUL would end the C string early and silently.
EXAMPLE = "\n".join([
    "987654321111111",
    "811111111111119",
    "234234234234278",
    "818181911112111",
])
EXPECTED_PART1 = 357
EXPECTED_PART2 = 3121910778619  # above 32 bits, on purpose

result = lib.ex_part1(EXAMPLE.encode("utf-8"))
if result != EXPECTED_PART1:
    print(f"part1 = {result}, expected {EXPECTED_PART1}")
    sys.exit(1)

result2 = lib.ex_part2(EXAMPLE.encode("utf-8"))
if result2 != EXPECTED_PART2:
    print(f"part2 = {result2}, expected {EXPECTED_PART2}")
    sys.exit(1)

# TODO 4, done: the hostile-input contract, from Python. ffi.NULL is the null
# pointer; a bytes object that is not UTF-8 is the other half of the contract.
assert lib.ex_part1(ffi.NULL) == -1
assert lib.ex_part1(b"\xff\xfe not utf-8") == -1

print("Ex 3 (Python) passed.")
