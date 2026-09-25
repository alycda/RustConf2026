#!/usr/bin/env bash
# CI proof that the comparator-overflow flaw manifests deterministically.
# Exit 0 == flaw confirmed present. Exit non-zero == flaw absent/fixed.
#
# Toolchain: a stock C compiler with -fsanitize=undefined (AppleClang / gcc /
# clang all ship it). No nightly, no attendee toolchain change.
set -uo pipefail
cd "$(dirname "$0")"
CC="${CC:-cc}"

echo "== [1/2] wrong-answer proof (plain build) =="
"$CC" -O0 sortlib.c proof.c -o proof_plain
./proof_plain
plain=$?
if [ "$plain" -ne 0 ]; then echo "FAIL: wrong-answer proof did not manifest"; exit 1; fi

echo
echo "== [2/2] UBSan proof (-fsanitize=undefined) =="
"$CC" -O0 -fsanitize=undefined -fno-sanitize-recover=undefined sortlib.c proof.c -o proof_ubsan
# We EXPECT this to abort with a signed-integer-overflow diagnostic.
if ./proof_ubsan > ubsan.out 2>&1; then
    echo "FAIL: expected a UBSan trap, but the program exited cleanly"
    cat ubsan.out
    exit 1
fi
if grep -q "signed integer overflow" ubsan.out; then
    echo "OK: UBSan trapped the signed-integer overflow:"
    grep "runtime error" ubsan.out
else
    echo "FAIL: program aborted but not with the expected overflow diagnostic"
    cat ubsan.out
    exit 1
fi

echo
echo "FLAW CONFIRMED: comparator a-b overflow."
