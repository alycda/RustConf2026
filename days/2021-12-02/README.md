# Day 2: Dive!

Each line of the input is a command — `forward X`, `down X`, `up X` — and the
answer is the submarine's final horizontal position times its final depth.
Part 2 reinterprets `down`/`up` as adjustments to an `aim` that `forward` then
applies to depth. A parse and a fold over three integers.

That shape is why this day carries two libraries that have no business being
here, and why neither is quite a joke. The puzzle's entire state is a position
and an orientation — which is what a rigid body *is*. And part 2's `aim` is a
running total — which is what a SQL window function *is*. Both libraries are
three-plus orders of magnitude too large for the job, and both genuinely say
the thing.

Five solves of the same puzzle live in this branch (each was built and verified
independently):

| Variant | Direction | Files |
|---|---|---|
| Pure Rust | — (baseline) | `dead_reckon_pure_rust`, `dead_reckon_with_aim_pure_rust` in `src/lib.rs` |
| Chipmunk2D solver | Rust → C | `src/chipmunk.rs`, `dead_reckon_via_chipmunk`/`dead_reckon_with_aim_via_chipmunk` in `src/lib.rs` |
| DuckDB (SQL) | Rust → C | `src/duckdb.rs`, `dead_reckon_via_duckdb`/`dead_reckon_with_aim_via_duckdb` in `src/lib.rs` |
| cbindgen C API | Rust → C (exported) | `src/c_api.rs`, `cbindgen.toml` |
| Kotlin via JNA | C → Kotlin | `kotlin/solve.kt` |

## The variants

**Pure Rust.** `dead_reckon_pure_rust` and `dead_reckon_with_aim_pure_rust` are
the puzzle solved the ordinary way — a `fold` over a `Position`. Kept as real
functions rather than deleted once the C versions existed, so `benches/dive.rs`
can race all three and so the C API has something to build on that doesn't
depend on which backend is compiled in.

**Chipmunk2D (`src/chipmunk.rs`).** The submarine becomes a `cpBody` in a
`cpSpace`, and the puzzle's three integers stop existing in Rust entirely:

| puzzle | Chipmunk |
|---|---|
| horizontal | `cpBodyGetPosition(body).x` |
| depth | `cpBodyGetPosition(body).y` |
| aim | `cpBodyGetAngle(body)` |

Chipmunk has no "move by" call, so each command sets a velocity and the space
is stepped for exactly one unit of time. Part 2 is the half that earns the
engine: `down`/`up` set an *angular* velocity and let the solver integrate it
into the body's rotation, and `forward` reads that rotation back to build its
linear velocity — `aim` is never a variable Rust owns.

**DuckDB (`src/duckdb.rs`).** An in-process analytical database — columnar
storage, a vectorized engine, a query optimizer, all built for scanning
billions of rows — pointed at a thousand rows of `forward 5`. The course is
bulk-loaded through DuckDB's appender into `course (idx, cmd, x)`, and both
parts are one query each. Part 2 is the half that earns the database:

```sql
SUM(CASE cmd WHEN 'down' THEN x WHEN 'up' THEN -x ELSE 0 END)
  OVER (ORDER BY idx ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW)
```

That is the puzzle's aim rule, stated once, and it is arguably clearer than the
fold in `lib.rs`.

Both libraries are cargo features, off by default. With both compiled in,
`Solution` routes to Chipmunk — see `src/lib.rs` for why — but each keeps its
own `pub fn`s and its own tests, so the one `cargo run` doesn't exercise still
fails the suite if it regresses.

**cbindgen C API (`src/c_api.rs`, Exercise 2).** The direction reverses: Rust
exposes itself *as* a C library. Two `extern "C"` functions,
`aoc_2021_12_02_part1`/`part2`, built around out-parameters and status codes
rather than `Result`, because a panic unwinding across an `extern "C"` frame is
undefined behavior. They are built on the pure-Rust functions specifically —
Exercise 2 is about the export direction, and wiring it to either library would
make the header's meaning depend on a cargo feature. `just days bindgen
2021-12-02` generates the header (not committed — see `days/.gitignore`).

**Kotlin via JNA (`kotlin/solve.kt`, Exercise 3).** Loads the built `cdylib`
and calls straight in. JNA needs each function written once, as an ordinary
method on an interface extending `Library`, and maps Kotlin types to C types by
convention (`String` → `const char *`, `IntByReference` → `int32_t *`). `just
days kotlin-demo 2021-12-02` regenerates the header, builds the `cdylib`,
fetches the pinned JNA jar, compiles and runs it (needs `just setup-kotlin`
once).

## Running things

```sh
cd days/2021-12-02 && cargo run                        # pure Rust
cargo run -p aoc-2021-12-02 --features chipmunk        # through the physics engine
cargo run -p aoc-2021-12-02 --features duckdb          # through the database
cargo test -p aoc-2021-12-02 --features chipmunk,duckdb  # 22 tests, every backend

just days bench 2021-12-02                             # criterion: parse + both parts
cargo bench -p aoc-2021-12-02 --bench dive --features chipmunk,duckdb   # three-way

just days bindgen 2021-12-02                           # regenerate include/aoc_2021_12_02.h
just days kotlin-demo 2021-12-02                       # build + header + run kotlin/solve.kt
```

## Benchmarks

`benches/dive.rs` races all three implementations on a fixed synthetic course
of 1000 commands — real puzzle-input scale (bench profile, aarch64, criterion
medians):

| | pure Rust | Chipmunk2D | DuckDB |
|---|---|---|---|
| `dead_reckon` (part 1) | ~755 ns | ~44.8 µs (~59x) | ~5.14 ms (~6,800x) |
| `dead_reckon_with_aim` (part 2) | ~886 ns | ~54.1 µs (~61x) | ~5.41 ms (~6,100x) |

Three orders of magnitude between each column, for three completely different
reasons — which is the whole reason to have run both libraries rather than
picking one.

**Chipmunk: the boundary is free, the library is not.** 44.8 µs over 1000
commands is ~45 ns per `cpSpaceStep`. The throwaway C program written to
validate the approach before any Rust existed — no FFI at all, just Chipmunk —
measured **56 ns** per step for the same work. The Rust-to-C crossing is inside
the noise of the thing it crosses into. Every microsecond is the engine's
broadphase and constraint solver running over an empty space with one body in
it, exactly as they would for ten thousand colliding ones.

**DuckDB: the cost isn't the work, it's the getting ready.** The per-solve
numbers above bury the interesting one, so `benches/dive.rs` measures the query
separately against an already-loaded database:

| | time | share of a full solve |
|---|---|---|
| part 1 (two aggregates) | ~247 µs | ~4.8% |
| part 2 (window function) | ~483 µs | ~8.9% |

**Roughly 95% of a DuckDB solve happens before any question is asked** —
`duckdb_open`, connect, `CREATE TABLE`, bulk-load. The analytics the database
exists for is the cheapest part of using it. And only once the queries are
isolated do the two parts stop looking alike: the window function costs
**1.96x** the two plain aggregates, a real measurement of what `OVER (ORDER BY
...)` adds to a scan, and one that is invisible in the per-solve column.

**Against the parse, the day inverts.** `benches/day.rs` times parsing 1000
commands at ~17.9 µs against a ~775 ns pure-Rust solve — parsing costs 23x what
solving does, and the fold is a rounding error. Through Chipmunk the solve
becomes ~2.5x the parse; through DuckDB, ~287x. The cheap half of this day
became the expensive half twice over, and a single end-to-end number would have
shown none of it.

Both generated inputs draw commands uniformly with amounts 1..=9, which is the
right *shape* and the wrong *distribution*: a real input's `down`/`up` don't
cancel, so its aim climbs and its part-2 depth is far larger. The numbers these
inputs produce are not puzzle answers. The near-cancellation is deliberate for
a second reason — see the overflow Learning below.

## Learnings

- **Two libraries in a row shipped no `.pc` file — and the fix belongs in the
  environment, not the build script.** nixpkgs' `chipmunk` and `duckdb` both
  ship real headers and real shared objects and no `lib/pkgconfig/*.pc`, so
  `pkg-config --libs` fails for both even with the packages in `buildInputs`.
  The tempting fix is to teach `build.rs` to find the headers some other way —
  and then to teach it a *second* other way for the second library. Instead
  `shell.nix` synthesizes both files with `writeTextDir` and lets pkg-config's
  setup hook put them on `PKG_CONFIG_PATH`; `build.rs` keeps one loop and one
  discovery mechanism. (DuckDB needed one extra wrinkle: it splits headers and
  shared object across nix's `dev` and `lib` outputs, so its `.pc` needs two
  prefixes where Chipmunk's needs one.) Contrast day 1's `figlet`, which had
  no linkable library at all — a real dead end. Telling the two apart is a
  five-minute check: is there a `.so` *and* a header, or isn't there?
- **Where the cost lives is library-specific, and the average hides it.** Both
  variants here are "slow", and they are slow in opposite shapes. Chipmunk's
  cost is uniform per command — a thousand steps, ~45 ns each, nothing to
  amortize. DuckDB's is ~95% one-time setup, and its actual query is under 5%
  of the solve. A single ratio per library would have called both "much slower
  than Rust" and said nothing true about either. The bench had to be built to
  separate setup from work before the difference existed as a number.
- **The FFI boundary was never the cost — on this day.** ~45 ns/step through
  Rust FFI against ~56 ns/step from a pure-C program doing the same thing.
  Whatever the 59x buys, "the crossing" is not it — which is the opposite of
  2015-12-01's lesson, where a JIT compile per call meant the boundary *was*
  the entire number. Two days, two FFI variants, opposite answers to "what does
  this actually cost", and neither is guessable from reading the code.
- **The absurd library sometimes states the problem better than the sensible
  code does.** Part 2's aim is a running total, and `SUM(...) OVER (ORDER BY
  idx)` says that in one line, more directly than the mutable fold in
  `lib.rs`. That is the sharpest version of "almost useful" this repo has hit:
  not a library bent to fit the puzzle, but one whose vocabulary happens to
  contain the exact sentence the puzzle needed — at roughly six thousand times
  the price of saying it by hand.
- **Using Chipmunk well meant switching most of it off.** The answers are exact
  `i32`s only because gravity is zero and damping is 1.0, which reduces
  `cpSpaceStep`'s integrator to `position += velocity * dt`. Turn either back
  on and the physics engine starts doing physics, and the puzzle stops having
  an answer. The library was useful precisely in the configuration where it
  does the least.
- **The thing that makes a physics engine worth using is the thing you have to
  defeat.** Velocity persists across steps — that *is* the simulation. Here it
  means a body told `forward 5` keeps moving 5 forever, so every command has to
  zero both velocities before setting its own. It is pinned as its own test,
  because it fails by producing a plausible wrong number rather than by
  crashing.
- **Both libraries needed a fresh object per solve, and both would have failed
  silently without one.** Chipmunk's is obvious once stated (a body carries its
  position). DuckDB's is not: a *connection* is not an isolation boundary, the
  *database* is. Two connections to one `duckdb_database` share a catalog, so
  the second `CREATE TABLE course` fails as "already exists" and the next query
  reads the previous course's rows — which is exactly what the first scratch
  program did while cheerfully reporting the right answers for the wrong table.
  Whatever a library calls its handle, find out which level actually isolates.
- **A deprecated convenience API is a fork in the road, not a warning to
  ignore.** DuckDB's `duckdb_value_int64` and friends are documented as
  scheduled for removal; the supported path is columnar — fetch a chunk, take
  the column's vector, read its data pointer, and check a *separate* validity
  bitmask that is null when every row is valid. Taking the supported path cost
  more binding surface and was the only way to meet the data model DuckDB
  actually has. (It also matters that `SUM` over no rows is `NULL`, which
  arrives as a *successful* result, not an error — an empty course is a real
  input, and `COALESCE` is what makes it a zero.)
- **Three backends, three different ways for `i32` to be the wrong type.**
  Chipmunk computes in `f64`, so its answer is checked for integrality *and*
  range. DuckDB computes in 128-bit `HUGEINT` — `SUM` over an `INTEGER` column
  widens, which is easy to miss — so the query casts `::BIGINT` to get a value
  the FFI read can hold at all, and only then is the narrowing checked. Plain
  Rust computes in `i32` and simply panics. The same puzzle answer needed three
  different guards depending on what was holding it.
- **A columnar FFI read has no type tag to check against.** The DuckDB result
  is a raw data pointer you `cast::<i64>()`; if the column is really `HUGEINT`,
  you read its low half and get the right answer modulo 2^64 with no error
  anywhere. That is the wrong answer most likely to be believed. The fix is to
  make the *query* promise the width, not the Rust.
- **This day's C API needed a guard the others didn't, because of arithmetic.**
  Part 2's answer on a genuine input already lands within ~10% of `i32::MAX`,
  and a C caller isn't limited to genuine inputs: `forward 100000\ndown 100000`
  overflows, and overflow panics in the dev profile — straight into UB across
  an `extern "C"` frame. So the arithmetic runs inside `catch_unwind` and
  reports `-3`. It also constrains the benchmarks: the generated inputs' aim
  has to nearly cancel, or the bench itself would panic instead of reporting a
  time. An FFI contract can be forced on you by the puzzle's *number range*,
  not just by its types.
- **`catch_unwind` stops the unwind, not the noise.** A C caller that overflows
  gets a tidy `-3` — and also gets `attempt to multiply with overflow` on
  stderr from Rust's default panic hook, which fires before the unwind begins.
  Silencing it means installing a process-global `panic::set_hook`, which a
  library has no business doing to its host. So the wart is documented rather
  than fixed by reaching outside the crate's own scope.
- **cbindgen's scope has to be aimed, and this day proves why twice.** The
  crate contains two modules full of `extern "C"` *imports* — Chipmunk's
  handful and DuckDB's twenty-odd. Pointed at the crate root, cbindgen would
  redeclare both third-party C APIs inside our header. Pointed at
  `src/c_api.rs`, it emits exactly the two functions meant to be exported.
- **JNA is the least typing and the least checking of the three tracks.** cffi
  (2015-12-01) reads the real cbindgen header and derives its declarations from
  it. dart:ffi (2015-12-05) needs every signature written twice, in native and
  Dart types. JNA needs each function written once, as a plain Kotlin method,
  and marshals by convention — which is pleasant right up until a signature
  changes, because nothing checks the interface against the header. A renamed
  export fails loudly at load time; a changed *signature* doesn't fail at all.
- **A track's runtime can quietly break a repo convention.** The Kotlin track
  printed `Part 1 ?(?)` on its first end-to-end run: the JVM takes
  `System.out`'s encoding from the console/locale, not from the source file, so
  an ASCII-defaulted stdout dropped the emoji the other two tracks print fine.
  Fixed inside `solve.kt` with an explicit UTF-8 `PrintStream` rather than with
  a `-D` flag in the just recipe, so running the jar by hand behaves the same
  way the recipe does.
