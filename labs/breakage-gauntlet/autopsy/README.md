# Station B — AI Binding Autopsy

Four pre-generated "AI bindings," each with **exactly one planted flaw** that
compiles clean and crashes or lies later. One per boundary family from the
reference card (the reference card (`docs/reference-card.typ`, "7 · Footguns")). Your job is code review with
a falsifiable output: find the flaw, name its family, and write the ONE hostile
input that exposes it.

| Dir | Family (reference-card.typ) | Binding shown | Manifests as | Proven by |
|-----|-----------------------------|---------------|--------------|-----------|
| [`01-comparator-overflow/`](01-comparator-overflow/) | `a - b` comparator overflows | Python + C helper | wrong minimum | wrong-answer + **UBSan** |
| [`02-cross-allocator-free/`](02-cross-allocator-free/) | free across runtimes | Python over C++ | maybe-crash | **ASan** alloc-dealloc-mismatch |
| [`03-encoding-assumption/`](03-encoding-assumption/) | encoding mismatch, silent | Swift over Rust | wrong count, no crash | wrong-answer (no sanitizer) |
| [`04-embedded-nul/`](04-embedded-nul/) | missing/embedded NUL | Python over Rust | truncated answer / read past end | wrong-answer + **ASan** |

## How to run a station

1. Open the dir. Read the binding and the native side. **Do not open
   `SOLUTION.md`** until `REVIEW.md` is filled in.
2. Fill in `REVIEW.md`: find it, name the family, write the hostile input.
3. Run `./check.sh` to watch the flaw manifest deterministically.
4. Reveal `SOLUTION.md` and compare.

## Flow (20 min)

| Phase | Time |
|-------|:----:|
| Read the binding, told only "one flaw, compiles clean, breaks later" | 2 min |
| Locate the flaw + name its family | 10 min |
| Write the one hostile input, run it | 6 min |
| Reveal: CI's planted-flaw assertion, green→red | 2 min |

## The CI gate

`.github/workflows/gauntlet.yml` runs every `check.sh`. Each asserts that its
flaw **manifests** — a wrong answer, or a sanitizer abort. On this repo (flaw
present) the assertions pass: the flaws are provably real and deterministic. In
the room the lesson runs the other way — fix the binding and the manifestation
disappears (green→red as the flaw is reintroduced). This is
"CI must build everything" made executable.

## Sanitizers are CI-only

Two flaws two are memory bugs that a vanilla toolchain catches
only by luck. CI compiles the **C-side** proof harness with stock Clang/GCC
`-fsanitize=address` / `-fsanitize=undefined` — no nightly Rust, no
`-Zsanitizer`. The attendee toolchain stays vanilla; attendees see the honest
"sometimes crashes" behaviour, and CI is the deterministic judge. Full rationale
in [`../README.md`](../README.md).
