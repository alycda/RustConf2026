# Day 1 (2024): Historian Hysteria

Two columns of location IDs. Part 1 sorts each column, pairs them off by
rank, and sums the distances; part 2 weights each left-hand ID by how often
it appears on the right. This is the golden day — the day the original
"(ab)Using Advent of Code as an FFI Playground" talk was built on — and the
variants in this tree are the talk's own, imported from `alycda/aoc-ffi`
and adapted to this repo's conventions rather than re-invented.

Four solves of the same puzzle share this branch with a compile-time
abstraction over them (each was built and verified independently):

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline) | `Day1` in `src/lib.rs`, ported from the real 2024 solution (`alycda/Advent-of-Code`, `refactor/alycda/2024`) |
| libc `qsort` | Rust → C | `src/qsort.rs`, `sort_via_qsort` in `src/lib.rs` |
| C++ `std::sort` | Rust → C++ behind a C shim | `src/cpp.rs`, `src/cpp_sort.cpp` |
| uthash hash table (part 2) | Rust → C | `src/uthash.rs`, `src/uthash_wrapper.{c,h}` |
| `Sorter` trait | — (compile time) | `Sorter`/`NativeSort`/`CSort`/`CppSort` in `src/lib.rs` |

## The variants

**Pure Rust.** The `aoc-ornaments` solution as actually written in 2024 —
`FromStr` unzips and rank-sorts the columns, `part1` zips, `part2` scans.
`Day1Hashmap`'s unfinished `nom_parser` is preserved as-is from upstream,
annotation included: this tree imports the golden solution, warts and all.
The sort and the part-2 count are kept as named functions
(`sort_pure_rust`, `similarity_pure_rust`) rather than folded away, so the
benches can put each backend beside them.

**libc `qsort` (`src/qsort.rs`, `--features qsort`).** The talk's headline
crossing: the column's storage handed over as pointer + length, with a
C-shaped comparator passed as a function pointer. The comparator keeps the
talk's hard-won idiom — `a - b` overflows on `i32::MAX` vs `i32::MIN` and
shipped as a live bug before the `if`/`else` form landed; the agreement
test pins exactly those extremes. No build step and no pkg-config: qsort
ships in the C standard library every target already links.

**C++ `std::sort` (`src/cpp.rs`, `--features cpp`).** C++ can't cross the
boundary as C++ — templates have no ABI and mangled names differ per
compiler — so `cpp_sort.cpp` flattens `std::sort` to one `extern "C"`
function, compiled by `build.rs` (via the `cc` crate) only when the
feature is on. The talk reached the same function through autocxx; the
port hand-writes the one signature autocxx generated under the hood,
because bindgen's libclang requirement is exactly the kind of build-time
dependency a bare CI runner doesn't have.

**uthash part 2 (`src/uthash.rs`, `--features uthash`).** Part 2's
frequency map built and queried on the C side. uthash is a *macro*
library — `HASH_ADD_INT` expands to inline C at each use site, so there is
no symbol anywhere to bind until `uthash_wrapper.c` instantiates the
macros behind three ordinary functions. The header comes from nixpkgs (no
`.pc` file — the nix cc wrapper's include injection does the finding), not
vendored.

**`Sorter` trait.** The talk's zero-cost-abstraction branch: zero-sized
marker types (`NativeSort`, `CSort`, `CppSort`) select the sort backend at
compile time through `Day1::parse_with::<S>`. Monomorphization emits a
specialized copy per backend — no vtable, no branch; choosing a backend is
a type, not a runtime decision. `FromStr` picks by feature precedence
(qsort, then cpp, then native), and every backend stays reachable by name.

## Running things

```sh
cd days/2024-12-01 && cargo run                     # pure Rust, both parts
cargo run --features qsort,cpp,uthash               # same answers, three boundaries

cargo test -p aoc-2024-12-01                        # baseline
cargo test -p aoc-2024-12-01 --features qsort,cpp,uthash   # + agreement tests

just days bench 2024-12-01                                 # parse + parts, see days/README.md
cargo bench -p aoc-2024-12-01 --bench sort --features qsort,cpp     # sort backends head to head
cargo bench -p aoc-2024-12-01 --bench lookup --features uthash      # part-2 structures head to head
```

## Benchmarks

`benches/sort.rs` races the sort backends on one fixed 1000-element
column, each iteration sorting a fresh unsorted clone (bench profile,
aarch64):

| backend | time | vs pure Rust |
|---|---|---|
| pure Rust `sort` | ~4.1 µs | — |
| C++ `std::sort` | ~5.0 µs | ~1.2× |
| libc `qsort` | ~15.7 µs | ~3.8× |

`benches/lookup.rs` races part 2's counting structures, whole job per
iteration (build the map, query it, tear it down), on 1000-entry columns
drawn from a range where matches actually occur:

| structure | time | vs naive |
|---|---|---|
| naive scan (the shipped baseline) | ~99.4 µs | — |
| std `HashMap` | ~23.2 µs | ~4.3× faster |
| `ahash` (bench-only dependency) | ~13.1 µs | ~7.6× faster |
| uthash via FFI | ~10.6 µs | ~9.4× faster |

`benches/day.rs` times the day's own pipeline; its parse row is where the
sort lives, so it is the number that moves with a backend feature on
(~30.4 µs pure → ~32.2 µs cpp → ~60.6 µs qsort at generated/1000).

## Learnings

- **The boundary's cost depends on how often you cross it, not that you
  cross it.** qsort pays an uninlinable indirect call *per comparison* and
  runs 3.8× behind Rust; uthash crosses twice in bulk (build, then
  lookups against a table that stays on the C side) and *wins* — beating
  not just the O(n²) scan but std's `HashMap` and `ahash` at this scale.
  "C through FFI is slower" is not a law; per-element crossings are.
- **`a - b` is not a comparator.** The C idiom for qsort comparators looks
  clever, overflows on extreme `i32` pairs, panics in debug, wraps in
  release — and shipped live in the talk. The test suite here pins
  `i32::MAX`/`i32::MIN` through every sort backend because of it.
- **A macro library has no ABI surface at all.** There is nothing in
  uthash to link against — the "library" only exists once a C translation
  unit instantiates its macros. Hand-written FFI to a header-only C
  library therefore means writing C, not just `extern "C"`.
- **Generated bindings are the same C shape plus a toolchain.** autocxx
  reached `std::sort` by generating the identical pointer-plus-length
  `extern "C"` shim this port hand-writes — and doing the generating cost
  a bindgen/libclang build-time dependency that a stock runner and a
  manual-path attendee don't have. Templates have no ABI either way.
- **Zero-sized types turn "which backend" into a compile-time fact.**
  `parse_with::<CSort>` and `parse_with::<NativeSort>` are separate
  monomorphized functions; the marker costs no memory and no dispatch.
  The feature flags choose a default, but the type parameter is what
  makes every backend independently nameable — which is also what the
  benches are built on.
- **Not every discovery mechanism is pkg-config.** The tcc/caca days
  probe `.pc` files; nixpkgs' uthash ships none, and the working pattern
  is one step simpler — put the package in `shell.nix`'s `buildInputs`
  and let the nix cc wrapper inject the include path. Check what the
  package actually installs before designing the probe.
