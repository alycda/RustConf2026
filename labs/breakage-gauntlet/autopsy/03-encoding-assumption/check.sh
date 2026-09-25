#!/usr/bin/env bash
# CI proof that the encoding-assumption flaw manifests: a silent WRONG
# ANSWER, no crash, no sanitizer needed.
# Exit 0 == flaw confirmed present. Exit non-zero == flaw absent/fixed.
#
# Toolchain: cargo (for the native cdylib) + swiftc (the binding). macOS runner.
set -uo pipefail
cd "$(dirname "$0")"

echo "== build native cdylib =="
( cd nativelib && cargo build --release ) || { echo "FAIL: cargo build"; exit 1; }

case "$(uname)" in
  Darwin) LIB=nativelib/target/release/libnativelib.dylib ;;
  *)      LIB=nativelib/target/release/libnativelib.so ;;
esac
cp "$LIB" .

echo "== compile Swift binding (bridging header + link cdylib) =="
# swiftc needs top-level code in a file named main.swift when linking (same
# gotcha the Ex 3 Swift track hits too).
cp FlawedBinding.swift main.swift
swiftc -o encoding_proof main.swift \
    -import-objc-header native.h \
    -L . -lnativelib

echo "== run (expect a wrong count, no crash) =="
if DYLD_LIBRARY_PATH=. LD_LIBRARY_PATH=. ./encoding_proof; then
    echo
    echo "FLAW CONFIRMED: encoding mismatch corrupts silently."
    exit 0
fi
echo "FAIL: expected a wrong-answer manifestation"
exit 1
