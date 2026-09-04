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
# at exercises/target/ rather than sitting inside each crate. The extension is
# the host's, not Rust's choice: .dylib on macOS, .so elsewhere.
case "$(uname)" in
  Darwin) LIB=../target/release/libex2_c_glue.dylib ;;
  *)      LIB=../target/release/libex2_c_glue.so ;;
esac
cc tests/c/test_glue.c "$LIB" -o test_glue

# 4. Run it
./test_glue
