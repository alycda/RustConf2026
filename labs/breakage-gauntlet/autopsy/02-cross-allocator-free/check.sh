#!/usr/bin/env bash
# CI proof that the cross-allocator-free flaw manifests deterministically.
# Exit 0 == flaw confirmed present. Exit non-zero == flaw absent/fixed.
#
# Toolchain: stock clang/gcc with -fsanitize=address. The mismatch (native
# operator new[] vs libc free) is caught deterministically; a plain build might
# survive by luck, which is the whole point of "never rely on a chance segfault."
set -uo pipefail
cd "$(dirname "$0")"
CXX="${CXX:-c++}"
CC="${CC:-cc}"
# Enable the alloc/dealloc-mismatch check (default-on with libc++/Linux; make it
# explicit so the proof is deterministic across platforms).
export ASAN_OPTIONS="${ASAN_OPTIONS:-}:alloc_dealloc_mismatch=1"

echo "== ASan proof (native new[] freed with libc free) =="
# Compile the native TU as C++ and the harness as C, then link with the C++
# driver (so the C++ runtime is pulled in). ASan on every step.
"$CXX" -O0 -g -fsanitize=address -c nativelib.cpp -o nativelib.o
"$CC"  -O0 -g -fsanitize=address -c proof.c      -o proof.o
"$CXX" -O0 -g -fsanitize=address nativelib.o proof.o -o proof_asan

if ./proof_asan > asan.out 2>&1; then
    echo "FAIL: expected an ASan abort, but the program exited cleanly"
    cat asan.out
    exit 1
fi
if grep -q "alloc-dealloc-mismatch" asan.out; then
    echo "OK: AddressSanitizer flagged the cross-allocator free:"
    grep -m1 "alloc-dealloc-mismatch" asan.out
    echo
    echo "FLAW CONFIRMED: free with the wrong allocator across runtimes."
    exit 0
fi
echo "FAIL: program aborted but not with the expected alloc-dealloc-mismatch"
cat asan.out
exit 1
