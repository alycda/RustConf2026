#!/usr/bin/env bash
# Exercise 2: build the Rust cdylib, generate the C header, compile and run
# the C harness. Same four beats as the Module 2 demo.
set -euo pipefail
cd "$(dirname "$0")"

# 1. Rust → shared library
cargo build --release

# 2. Rust → C header (this is the artifact worth reading — open it!)
mkdir -p include
cbindgen --output include/ex2_c_glue.h

# 3. Compile the C test against your library
# ../target: the exercises are one cargo workspace, so build output is shared
# at exercises/target/ rather than sitting inside each crate. The library
# takes the host's name, not Rust's choice: libex2_c_glue.dylib on macOS,
# libex2_c_glue.so on Linux, ex2_c_glue.dll on native Windows — where this
# script stops, because the workshop's Windows answer is WSL2 (README,
# Option 2) and Git Bash would otherwise fail on a missing .so.
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*)
    echo "native Windows: run the exercises under WSL2 — see ../../README.md, Option 2" >&2
    exit 1 ;;
esac
target="$(cd ../target/release && pwd)"
# -L/-l plus -rpath against the absolute directory, not a relative path on
# the command line: linked by a relative path the binary records exactly
# that (rustc writes no SONAME), and test_glue then loads only from a cwd
# whose parent has target/release/ — and silently from any unrelated one
# that does. This way it runs from anywhere.
cc tests/c/test_glue.c -L"$target" -lex2_c_glue -Wl,-rpath,"$target" -o test_glue

# 4. Run it
./test_glue
