#!/usr/bin/env python3
"""Exercise 3 (Python/cffi track): call the Exercise 2 C API from Python.

No C compiler step here — cffi's ABI mode just needs declarations plus a
`dlopen` of the compiled `cdylib`. The declarations come from the real
header `just days bindgen 2015-12-01` generates (regenerated below if
missing), not a hand-duplicated copy of the same two signatures: cffi's
`cdef()` takes a restricted C subset with no preprocessor, so the include
guard / `#include` / comments cbindgen wrote are stripped before parsing.

Run via: just days python-demo 2015-12-01 (builds the header + cdylib
first); or directly once both exist: python3 python/solve.py
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

try:
    from cffi import FFI
except ImportError:
    sys.exit("cffi not found — run: just setup-python, then: source .venv/bin/activate")

DAY_DIR = Path(__file__).resolve().parent.parent
DAYS_DIR = DAY_DIR.parent
REPO_ROOT = DAYS_DIR.parent
HEADER = DAY_DIR / "include" / "aoc_2015_12_01.h"


def header_declarations() -> str:
    if not HEADER.exists():
        try:
            subprocess.run(["just", "days", "bindgen", "2015-12-01"], cwd=REPO_ROOT, check=True)
        except FileNotFoundError:
            sys.exit(
                "no header and `just` not found — install just (>= 1.31), or generate it "
                "directly: cd days/2015-12-01 && mkdir -p include && "
                "cbindgen --config cbindgen.toml --output include/aoc_2015_12_01.h src/c_api.rs"
            )
        except subprocess.CalledProcessError as e:
            sys.exit(f"header regeneration failed (exit {e.returncode}) — see the just output above")

    text = HEADER.read_text()
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)  # cbindgen's doc comments
    text = re.sub(r"^\s*#.*$", "", text, flags=re.MULTILINE)  # include guard, #include
    return text


def load_library():
    ffi = FFI()
    ffi.cdef(header_declarations())

    # A cdylib takes the host's name, not Rust's choice: libaoc_2015_12_01.so on
    # Linux, libaoc_2015_12_01.dylib on macOS, aoc_2015_12_01.dll (no lib prefix) on
    # Windows. Searching the three filenames needs no platform check —
    # whichever one cargo produced is the one that exists.
    for profile in ("debug", "release"):
        for name in ("libaoc_2015_12_01.so", "libaoc_2015_12_01.dylib", "aoc_2015_12_01.dll"):
            candidate = DAYS_DIR / "target" / profile / name
            if candidate.exists():
                return ffi, ffi.dlopen(str(candidate))

    sys.exit("no libaoc_2015_12_01.{so,dylib} / aoc_2015_12_01.dll found — run: cd days && cargo build -p aoc-2015-12-01 --lib")


def call(ffi, fn, name: str, text: str) -> int:
    """Runs one of the C API's out-param/status-code functions. A nonzero
    status is an error the C side already classified — report it and exit
    nonzero rather than printing a phantom answer."""
    out = ffi.new("int *")
    status = fn(text.encode(), out)
    if status != 0:
        sys.exit(f"{name} failed with status {status} (-1 bad input, -2 domain error)")
    return out[0]


def main() -> None:
    # A Windows python inherits the console's legacy code page for stdout
    # (cp1252 on a stock runner), and the labels below have no encoding
    # there. Say UTF-8 before the first print; a no-op where it already is.
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    ffi, lib = load_library()

    input_path = REPO_ROOT / "inputs" / "2015-12-01.txt"
    if not input_path.exists():
        sys.exit(f"no puzzle input at {input_path} (see .gitignore)")
    text = input_path.read_text()

    print(f"Part 1 🐍(🦀): {call(ffi, lib.aoc_2015_12_01_part1, 'part1', text)}")
    print(f"Part 2 🐍(🦀): {call(ffi, lib.aoc_2015_12_01_part2, 'part2', text)}")


if __name__ == "__main__":
    main()
