#!/usr/bin/env python3
"""AI-generated Python binding over nativelib.cpp.

The native lib hands back a raw char* allocated with the native runtime's
allocator and documents `answer_free()` as the only correct release path.
This binding ignores that and reaches for libc's free() instead, because
"a pointer is a pointer." That is a cross-allocator free.

On many machines this happens to not crash (the two allocators share pages
today), which is exactly why it survives review and ships. See SOLUTION.md;
`proof.c` + `check.sh` make it deterministic under AddressSanitizer.

    c++ -shared -fPIC nativelib.cpp -o libnative.dylib   # macOS (C++ driver)
"""
import ctypes
import sys
from pathlib import Path

here = Path(__file__).parent
libname = "libnative.dylib" if sys.platform == "darwin" else "libnative.so"
native = ctypes.CDLL(str(here / libname))
native.render_answer.restype = ctypes.c_void_p
native.answer_free.argtypes = [ctypes.c_void_p]

libc = ctypes.CDLL(None)  # process libc
libc.free.argtypes = [ctypes.c_void_p]


def get_answer():
    ptr = native.render_answer()
    text = ctypes.string_at(ptr).decode()
    # THE FLAW: freeing a native-runtime allocation with libc.free instead of
    # the native.answer_free the library told us to use.
    libc.free(ptr)              # <-- cross-allocator free
    # correct would be:  native.answer_free(ptr)
    return text


if __name__ == "__main__":
    print("answer:", get_answer())
    print("(no visible error today — that is the trap; see check.sh)")
