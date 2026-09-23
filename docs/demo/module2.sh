#!/usr/bin/env bash
# The Module 2 live demo, scripted — the four commands from the slide, run
# against a scratch copy of exercises/ with 2024-12-03 (Mull It Over) as the
# Ex 1 solver. Three acts: the header, the abort with the todo!() still
# inside, then the four-step fix and the harness passing (NULL and invalid
# UTF-8 asserted first).
#
#   docs/demo/module2.sh <scratch-dir>        # copies exercises/ there, runs the demo
#   asciinema rec -c "docs/demo/module2.sh /tmp/demo" docs/demo/module2.cast
#
# Needs the workshop shell (cargo, cbindgen, cc): run it from `nix-shell`.
# The scratch copy is what gets solved; the repo's exercises/ is untouched.
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
scratch="${1:?usage: module2.sh <scratch-dir>}"
mkdir -p "$scratch"
scratch="$(cd "$scratch" && pwd)"
# The scratch dir gets its exercises/ wiped and rebuilt. Never let that be
# the repo (or anything inside it): `module2.sh .` must not delete the tree.
case "$scratch" in
    "$repo"|"$repo"/*)
        echo "module2.sh: scratch dir must be outside the repo, got $scratch" >&2
        exit 2 ;;
esac
rm -rf "$scratch/exercises"
tar --exclude=target -C "$repo" -cf - exercises | tar -xf - -C "$scratch"
cd "$scratch/exercises/ex2-c-glue"

# --- setup, off camera: Ex 1 solved for 2024-12-03 part 1, harness filled in
cat > ../ex1-pure-rust/src/lib.rs <<'RS'
//! Exercise 1: 2024-12-03, Mull It Over — pure Rust, `&str` in, `i64` out.

/// Sum of every well-formed `mul(X,Y)` (1–3 digit operands) in the input.
pub fn part1(input: &str) -> i64 {
    let b = input.as_bytes();
    let mut i = 0;
    let mut sum = 0i64;
    while let Some(off) = input[i..].find("mul(") {
        i += off + 4;
        let Some((x, n)) = digits(&b[i..]) else { continue };
        if b.get(i + n) != Some(&b',') { continue }
        let Some((y, m)) = digits(&b[i + n + 1..]) else { continue };
        if b.get(i + n + 1 + m) != Some(&b')') { continue }
        sum += x * y;
        i += n + 1 + m + 1;
    }
    sum
}

/// Parses 1–3 leading ASCII digits: (value, bytes consumed).
fn digits(b: &[u8]) -> Option<(i64, usize)> {
    let n = b.iter().take(3).take_while(|c| c.is_ascii_digit()).count();
    (n > 0).then(|| (std::str::from_utf8(&b[..n]).unwrap().parse().unwrap(), n))
}

/// Part 2 — not needed for the demo.
pub fn part2(input: &str) -> i64 {
    let _ = input;
    todo!("solve part 2")
}
RS
python3 - <<'PY'
p='tests/c/test_glue.c'; s=open(p).read()
s=s.replace('const char *example = "PASTE EXAMPLE INPUT HERE";',
            'const char *example = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";')
s=s.replace('long long expected_part1 = 0;','long long expected_part1 = 161;')
open(p,'w').write(s)
PY

# --- on camera
say() { printf '\n\033[2m# %s\033[0m\n' "$*"; sleep 1.2; }
run() { printf '\033[1;32m$\033[0m %s\n' "$*"; sleep 0.9; eval "$@" || true; sleep 1.4; }
say "Rust → shared library"
run cargo build
say "Rust → C header (this is the artifact worth reading)"
run cbindgen --output include/ex2_c_glue.h
run cat include/ex2_c_glue.h
say "compile the C caller against it"
run 'cc tests/c/test_glue.c -L../target/debug -lex2_c_glue -Wl,-rpath,"$PWD/../target/debug" -o test_glue'
say "run it — with the todo!() still inside"
run ./test_glue
say "a panic across extern \"C\" aborts the process. No return value, no stack trace for the caller."
say "now the four steps: null check → CStr → UTF-8 → the solver"
python3 - <<'PY'
p='src/lib.rs'; s=open(p).read()
a=s.index('#[unsafe(no_mangle)]\npub unsafe extern "C" fn ex_part1'); b=s.index('/// # Safety', a)
body='''#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_part1(input: *const c_char) -> i64 {
    if input.is_null() {
        return INVALID_INPUT; // 1. never deref a null
    }
    let cstr = unsafe { CStr::from_ptr(input) }; // 2. the pointer contract, asserted
    let Ok(s) = cstr.to_str() else {
        return INVALID_INPUT; // 3. C strings promise nothing about UTF-8
    };
    ex1_pure_rust::part1(s) // 4. the pure solver, untouched
}

'''
open(p,'w').write(s[:a]+body+s[b:])
PY
run 'sed -n "/^pub unsafe extern \"C\" fn ex_part1/,/^}/p" src/lib.rs'
run cargo build
say "NULL and invalid UTF-8 go in first — the contract holds, or nothing else runs"
run ./test_glue
