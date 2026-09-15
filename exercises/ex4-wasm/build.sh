#!/usr/bin/env bash
# Exercise 4: build the crate for wasm32-unknown-unknown, generate the C
# header nobody on this track will read, and print the header that matters —
# the module's own export section. Ex 2's build-and-test.sh had four beats;
# the fourth (the caller) lives in wasm/ and `just exercises wasm` runs it
# after this script.
set -euo pipefail
cd "$(dirname "$0")"

# 1. Rust → wasm module. The target's std comes with rustup (`rustup target
#    add wasm32-unknown-unknown`, which `just setup-wasm` runs) or with the
#    nix shell's rustc, which has it built in. One output file, one name, on
#    every host: ../target/wasm32-unknown-unknown/debug/ex4_wasm.wasm — no
#    lib prefix, no .so/.dylib/.dll, because no host loader is involved.
cargo build --target wasm32-unknown-unknown

# 2. Rust → C header, for comparison only. Put it beside the export list
#    below: the names survived, the C types (`const char *`, `int64_t`)
#    did not — the module declares i32 and i64, and that is the treaty now.
mkdir -p include
cbindgen --output include/ex4_wasm.h

# 3. The module's own declaration of itself. This is what wasm-bindgen,
#    cffi, or you read to know what can be called: no header file, no
#    symbol table, an export section with wasm types. `memory` is in it
#    too — the caller's only way to hand this module a string.
wasm="$(cd ../target/wasm32-unknown-unknown/debug && pwd)/ex4_wasm.wasm"
node -e '
const fs = require("node:fs")
const mod = new WebAssembly.Module(fs.readFileSync(process.argv[1]))
console.log("exports:")
for (const e of WebAssembly.Module.exports(mod)) console.log(`  ${e.kind.padEnd(8)} ${e.name}`)
const imports = WebAssembly.Module.imports(mod)
console.log(imports.length === 0 ? "imports: none — this module asks nothing of the host" : "imports:")
for (const i of imports) console.log(`  ${i.kind.padEnd(8)} ${i.module}.${i.name}`)
' "$wasm"
echo "module: $wasm"
