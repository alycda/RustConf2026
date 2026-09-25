#!/usr/bin/env bash
# CI proof that the missing/embedded-NUL flaw manifests, in BOTH variants:
#   1. EMBEDDED NUL  — a wrong (truncated) answer, no crash, no sanitizer.
#   2. READ PAST END — a deterministic heap-buffer-overflow under ASan.
# Exit 0 == flaw confirmed present. Exit non-zero == flaw absent/fixed.
#
# Toolchain: cargo + python3 (variant 1) and a C compiler with
# -fsanitize=address (variant 2).
set -uo pipefail
cd "$(dirname "$0")"
CC="${CC:-cc}"

echo "== [1/2] embedded-NUL truncation (wrong answer, no crash) =="
( cd nativelib && cargo build --release ) || { echo "FAIL: cargo build"; exit 1; }
out="$(python3 flawed_binding.py)"
echo "$out"
# The record line must show the truncated 3, not the intended 10.
if echo "$out" | grep -q "record: 3 "; then
    echo "OK: embedded NUL truncated the input (3 instead of 10)."
else
    echo "FAIL: expected a truncated sum of 3"
    exit 1
fi

echo
echo "== [2/2] read-past-end (ASan heap-buffer-overflow) =="
"$CC" -O0 -g -fsanitize=address readpast.c -o readpast_asan
if ./readpast_asan > asan.out 2>&1; then
    echo "FAIL: expected an ASan abort, but the program exited cleanly"
    cat asan.out
    exit 1
fi
if grep -q "heap-buffer-overflow" asan.out; then
    echo "OK: AddressSanitizer flagged the read past the unterminated buffer:"
    grep -m1 "heap-buffer-overflow" asan.out
else
    echo "FAIL: program aborted but not with the expected heap-buffer-overflow"
    cat asan.out
    exit 1
fi

echo
echo "FLAW CONFIRMED: missing/embedded NUL — silent truncation + read past end."
