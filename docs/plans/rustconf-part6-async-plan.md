# Part 6: async, across an ABI that has no such thing

Written 2026-09-24, stacked on the Part 5 plan (PR #8), which is stacked on
`wasm` (PR #4). PR #7's status-code table is assumed merged underneath all of it.
Status: **plan, nothing built.** Decisions the owner has already made: 2015-12-06 is
the day; the exercise covers rungs A, B and C; Godot is in; and not blocking the
main thread is the motivation (Swift's main actor, Android's main thread).
2015-12-06's timing was measured, not estimated. Anything about a runtime I could not
run here is marked `[verify]`.

## Why this exists

The C ABI has functions that return. It has no futures, promises, coroutines or
tasks. "Async across FFI" is always built out of plain calls, in one of three
shapes:

1. **Block on someone else's thread.** The host is async; Rust stays synchronous.
2. **A completion callback.** Rust calls back into the host, from a thread the host
   did not create.
3. **A job handle.** Start, poll, cancel, free.

Each shape moves a different part of the contract out of the type system: which
thread code runs on, how long an input has to live, how often a callback fires, and
what cancel means. As in Part 5, **the pain points are the curriculum.**

## The motivation, stated honestly

Advent of Code is CPU-bound and pure. Nothing here waits on I/O, so async is not
about throughput. It is about **the thread that must not stall**:

- **Swift**: the main actor, and the main thread under it, runs UI. `[verify]` each
  of the following before it goes on a slide:
  - iOS's watchdog kills an app that blocks the main thread too long at launch
    (`0x8badf00d`).
  - Xcode reports hangs.
  - Swift 6 strict concurrency refuses to compile a call from a background context
    into `@MainActor`-isolated code. The compiler enforces the *hop back*, not the
    *don't block*: calling a slow synchronous function from `@MainActor` compiles
    without a word.
- **Kotlin on Android**: blocking the main thread for long enough produces an ANR
  ("Application Not Responding") dialog; the input-dispatch timeout is 5 s. Before
  that, Choreographer logs "Skipped N frames! The application may be doing too much
  work on its main thread." `[verify]` both, and the frame threshold for the log.
  StrictMode catches disk and network on the main thread, not CPU work.
  - **The Kotlin track is JVM, not Android.** On a plain JVM "main" is not special,
    so the track models it: a single-thread executor plays the main thread, with a
    heartbeat the harness checks (see "The heartbeat" below). The Android behavior
    is lecture material and an optional demo, not a CI cell.
- **Godot**: `_process` runs once per frame. A solver called from it freezes the
  game for exactly as long as it runs. It is the most visible stall in the repo, so
  it is the opening demo (see "Godot").
- **Dart/Flutter**: the UI runs on the main isolate's event loop. The track is plain
  Dart, with the same heartbeat harness.
- **JavaScript**: the main thread runs the event loop. A synchronous wasm call
  blocks it.

**Measured** (a scratch crate against this tree, this container, 300 instructions
with random rectangles, roughly a real input's scale): `Day::run` takes **225–245 ms
per part in debug and 72–81 ms in release.** At 60 fps a frame is 16.7 ms, so one
blocking call drops about 4 to 15 frames. That is visible jank. **It is nowhere near
Android's 5 s ANR timeout**, and the lecture must not imply otherwise. If a demo
needs an ANR-scale stall, repeat the input: time grows linearly with the
instruction count, so about 20× the instructions is needed to reach seconds even in
debug `[verify]`.

## The day: 2015-12-06

- **It exists and has a C API** (after #7, with the lints, the `-3`/`-4` split, and a
  solver passed in as a function pointer, which is the injection point Part 6 needs).
  It also has a Dart track.
- **The instruction loop gives natural checkpoints.** `Day::run` walks one
  instruction at a time, so:
  - progress is `instructions done / total`;
  - cancellation is a flag checked once per instruction, so it is cooperative and
    bounded (the worst case is one instruction's rectangle, under 1 ms in release at
    the scale above `[verify]`);
  - the test gate (below) sits at the same checkpoint.
- **What changes in the day:** `run` gains a checkpoint parameter, something like
  `run_with(rule, &mut dyn FnMut(done, total) -> ControlFlow<()>)`, and the existing
  `run` calls it with a closure that never breaks. `Solution::part1`/`part2` keep
  their behavior, and a test pins that the two agree. That is the Liskov check for
  the new entry point.

## The ladder

Signatures are proposals, written as cbindgen would render them.

### Rung A: the host makes it async; Rust is unchanged

Each runtime calls the existing blocking `aoc_2015_12_06_part1`/`part2` off its main
thread:

| Track | Mechanism | `[verify]` |
|---|---|---|
| Python | `asyncio.to_thread` | cffi releases the GIL for the duration of the call (without that, "async" gives no parallelism) |
| Dart | `Isolate.run` | a `Pointer` crosses isolates only as an address; allocate and free in the same isolate |
| Kotlin/JVM | `ExecutorService` + `CompletableFuture` (stdlib; no coroutines jar to fetch, unlike JNA's) | JNA calls from a pool thread need no setup |
| Swift | `Task.detached`, then `await MainActor.run` for the result | no `@Sendable` warning for a `String` input under the pinned toolchain's concurrency checking |
| TypeScript/wasm | the module instantiated in a `node:worker_threads` Worker; results by `postMessage` | the Worker instance has its own linear memory; nothing is shared |
| Godot | `WorkerThreadPool`, result delivered with `call_deferred` | see "Godot" |

- **Teaches: most of the time this is the right answer**, and the lecture says so.
  Rust does not need to know it is being called asynchronously.
- **The pain: you cannot cancel a running C call.** Cancelling in the host stops the
  *waiting*, not the *working*. The worker thread burns on, and its result arrives
  for a caller that has gone. Demo: cancel after 10 ms, and watch the CPU stay busy
  for the next 200.
- **wasm:** the Worker instance is a separate module with separate memory. It is
  Part 4's allocator story again, per worker.

### Rung B: a completion callback from a Rust thread

```c
typedef void (*aoc_done_cb)(void *user_data, int status, uint32_t value);
int aoc_2015_12_06_start_cb(const char *input, int part,
                            aoc_done_cb cb, void *user_data, AocJob **out_job);
```

This is the dangerous rung. Its contract has four clauses, each one a demo and each
one tested:

1. **The input is copied before `start_cb` returns.** Every `c_api.rs` so far says
   the input must stay valid "for the duration of the call". An async call outlives
   its own call, so that sentence stops being enough. A caller that frees its buffer
   right after `start_cb` returns must still get the right answer.
2. **The callback fires exactly once**, on success, error, cancellation and caught
   panic. That is a Liskov postcondition, and breaking it does not crash: the host
   waits forever.
3. **It fires on a thread the host did not create**, and each runtime has its own
   rule for that. All `[verify]` against the pinned versions:
   - **Python/cffi:** the callback acquires the GIL on entry. That works, but it runs
     off the event loop, so it must hand off with `loop.call_soon_threadsafe`.
   - **Dart:** an ordinary Dart callback invoked from a foreign thread crashes the
     VM. `NativeCallable.listener` (Dart 3.1+) exists for exactly this: it queues
     the call onto the isolate's event loop and returns immediately. The callback's
     arguments are copied, which is fine for `(status, value)`.
   - **Kotlin/JNA:** JNA attaches the foreign thread to the JVM for the call.
     Whether it detaches afterwards is configurable. Attaching every time costs
     overhead; staying attached leaks threads the JVM believes are its own.
   - **Swift:** a `@convention(c)` closure cannot capture context, so the context
     travels in `user_data` as `Unmanaged<Box>.passRetained`, and the callback
     balances it with `takeRetainedValue`. That retain/release pair is a new
     ownership contract. The result must hop to the main actor before it touches
     anything UI-shaped, and Swift 6 checking enforces that hop.
   - **TypeScript/wasm: rung B does not exist.** `wasm32-unknown-unknown` has no
     threads without atomics and a `SharedArrayBuffer`, which in turn needs a
     nightly `build-std` toolchain `[verify]`. The TS exercise does A and C and
     reads B's README section as "why not". That is a fact about the runtime, and
     the exercise says so rather than hiding it.
   - **Godot:** see "Godot". `call_deferred` is the hop back to the main thread.
4. **A panic in the worker is a hang, not an abort.** Rust 1.81's
   abort-at-`extern "C"` guarantee does not reach here: a spawned thread has no
   `extern "C"` frame, so the panic ends the thread quietly and clause 2 is broken.
   The morning's loud failure (abort) becomes a silent one (hang), which is worse.
   The fix is `catch_unwind` around the thread body, delivering `-4` through the
   callback, with an injected-panic test in the same shape as #7's.

Also part of the contract, and documented:

- **The callback must not call back into this library on the same job.** That is
  reentrancy. Name it; a test calls `aoc_2015_12_06_free` from inside the callback
  and expects a documented result, not a deadlock.
- **`out_job` exists so rung B can be cancelled** with rung C's `cancel`/`free`. Rung
  B is rung C with the result pushed instead of polled.

### Rung C: a job handle — start, poll, cancel, free

```c
typedef struct AocJob AocJob;
int  aoc_2015_12_06_start(const char *input, int part, AocJob **out_job);
int  aoc_2015_12_06_poll(AocJob *job, int *out_done, uint32_t *out_value,
                         uint32_t *out_permille);
void aoc_2015_12_06_cancel(AocJob *job);
void aoc_2015_12_06_free(AocJob *job);   /* cancels, then joins */
```

- **No foreign-thread callbacks.** The host polls from its own loop: an asyncio
  task, a Dart `Timer.periodic`, a Swift `Task` with `Task.sleep`, a Kotlin
  `ScheduledExecutorService`, a JS `setInterval` talking to a Worker, and Godot's
  `_process`. That last one is the most natural fit in the whole ladder, because a
  game loop *is* a poll loop.
- **The #7 lesson, applied: "pending" is never a status code.** Every track treats
  any nonzero status as an error. #7 made that a written rule, and the Hyrum rubric
  says it is depended on whether written or not. A pending job returning `1` would
  turn every existing consumer's "still running" into an error. So a pending poll
  returns `0` with `*out_done = 0`, and the job's own status arrives from the poll
  that sees `*out_done = 1`.
- **Cancellation needs its own code: `-5 cancelled`.** It is an outcome the caller
  asked for, not a bug (`-4`) and not bad input (`-1`). It becomes a new row in
  `days/README.md`, added in the same commit as the first function that returns it.
- **`free` while running cancels and joins; it never detaches.** A detached thread
  that outlives `free`, and then the library itself (a `dlclose`, or process exit),
  executes unmapped code. That is Liskov's history constraint: nothing happens
  after `free`. The join is bounded by one checkpoint (Power of Ten rule 2), and the
  doc comment says so.
- **Cancel after done is a no-op; cancel twice is a no-op; poll after free is
  undefined** (a use-after-free, as for any handle). The first two are tested; the
  third is documented.

## Godot

The opening demo, and a rung-by-rung companion, not an exercise track:

- **The stall:** a scene with a spinning sprite and an FPS readout calls the
  blocking solver from `_process`. It freezes for 225 ms in debug. Everyone in the
  room sees the problem before anyone explains it.
- **Rung A:** `WorkerThreadPool` runs the solver, and `call_deferred` delivers the
  result to the main thread. The sprite keeps spinning.
- **Rung B:** the Rust worker thread must not touch engine objects. The callback
  queues the result, and `_process` or `call_deferred` picks it up. `[verify]` what
  gdext 0.5.5 allows off the main thread, and whether its `experimental-threads`
  feature is needed.
- **Rung C:** `_process` calls `poll` once per frame, and a progress bar reads
  `out_permille`. It is the cleanest fit in the ladder.
- **Plumbing:** 2015-12-06 has no Godot extension today. It would mirror 2015-12-01's
  (`src/godot.rs` behind a `godot` feature, `godot/aoc.gdextension`, the same
  gdext 0.5.5 / `api-4-6` pin and the MSRV note that goes with it), and
  `just days godot-demo 2015-12-06`.
- **The headless check:** Godot's headless mode still runs `_process`, so CI can
  assert that the frame counter kept advancing while a gated job was held (see "The
  heartbeat"). A blocking call advances it by zero frames. `[verify]` on the Verify
  job's Godot cell.

## The heartbeat: testing "did not block the main thread"

Timing assertions are flaky, so the harness never asserts on wall time. Instead:

- **Test builds get a gate.** A test-only cargo feature (`test-gate`) makes the
  solver wait at its first checkpoint until a gate is opened, through an export
  like `aoc_2015_12_06_test_gate_open()` that only exists with the feature.
- **The main loop opens the gate after it has ticked k times** (a heartbeat timer in
  each track's harness: asyncio, Dart `Timer`, the Kotlin "main" executor, Swift's
  main actor, a JS interval, Godot `_process`).
- **If the main thread is blocked by the call, it never ticks, the gate never opens,
  and the job never finishes.** The harness's bounded timeout turns that deadlock
  into a failure with a message naming the blocked thread. With rungs A, B and C the
  heartbeat ticks, the gate opens, and the answer arrives.

This makes "did not block" a deterministic, bounded assertion rather than a
measurement. The measured frame drops are printed as a demo figure, never asserted.

## Tracks and scope

| Track | A | B | C |
|---|---|---|---|
| Python | exercise | exercise: `call_soon_threadsafe` | exercise: asyncio poll task |
| Dart | exercise | exercise: `NativeCallable.listener` | exercise: `Timer.periodic` |
| Kotlin/JVM | exercise | exercise: JNA thread attach, result to the "main" executor | exercise: scheduled poll |
| Swift | exercise | exercise: `Unmanaged` context, hop to `MainActor` | exercise: `Task` poll loop |
| TypeScript/wasm | exercise: Worker | **not possible** (no threads); reading only | exercise: Worker + interval |
| Godot | demo | demo | demo |
| Fortran, R | – | – | – |

- **R:** R is single-threaded, and calling into R from a foreign thread is fatal
  `[verify]`. One line in the R README pointing here, and nothing more.
- **Fortran:** out of scope. There is nothing distinctive to show that another track
  does not already show.

**The exercise** (`exercises/ex6-*`) takes the attendee's own Ex 1 solver, as Part 5
does. Cooperative cancellation needs the solver's cooperation, so the scaffold has
one TODO that is the lesson: **add a checkpoint to your own loop.** A solver with no
checkpoint can only be cancelled before it starts or ignored after it finishes. The
exercise makes the attendee find that out. The CI overlay is 2025 day 3 again, with
a checkpoint per line.

## Changes to #7's table

- **Add `-5 cancelled`.**
- **Write down that "pending" is never a status**, so no later function adds
  `1 = pending` and breaks every `!= 0` check.
- **Extend the input-lifetime sentence** in the `# Safety` convention: for a function
  that starts work, the input is copied before return, and the caller may free it
  immediately.

All three land in `days/README.md` in the same commit as the first function that
relies on them.

## Tests the rungs need from the start

- **No `sleep` in any test.** Progress is controlled by gates, as in "The heartbeat".
- **Exactly once**, asserted with a counter on every path: success, bad input, cancel
  before start, cancel mid-run, cancel after done, a panic injected in the worker, and
  `free` mid-run (the callback fires with `-5` before `free` returns).
- **Input copy:** free the input immediately after `start`, with the buffer
  pre-filled with a garbage pattern, and assert the answer is still correct. A
  use-after-free then reads garbage rather than a lucky leftover.
- **`run_with` agrees with `run`** on the statement examples and on the 300-line
  generated input: the new entry point is a stand-in for the old one.
- **Reentrancy:** `free` from inside the callback has a documented, tested outcome.
- **Bounded joins:** every harness wait has a timeout, and a timeout fails with the
  job's last `out_permille` so a stuck job reports where it stuck.
- **loom** for the done/cancel handoff: `[verify]` whether it earns its dependency
  for one `AtomicBool` and one `Mutex`. If not, the gate tests carry it.

## Cost, risk, open questions

- **Size:** bigger than Part 5. Three rungs × five tracks, plus the Godot extension
  and the heartbeat harness in every track.
- **Flakiness** is the main risk, and the gate/heartbeat design exists to remove it.
  Any test that needs `sleep` means the design is wrong, not the test.
- **The wasm hole in rung B** is permanent under the current target. If threads on
  wasm (`wasm32-wasip1-threads`, or atomics on `unknown-unknown`) become practical
  on a stable toolchain, revisit it. `[verify]` the state of both.
- **Android itself** is not a track. Adding one means an emulator in CI, which costs
  more than the rest of Part 6 combined. The JVM track plus the lecture carries the
  point.

## Order

1. After #8: `run_with` on 2015-12-06 with its agreement test. Nothing else changes.
2. Rung C in `c_api.rs`, with `-5` and the pending rule in the README, and the
   exactly-once and bounded-join tests. It comes first because it needs no
   foreign-thread callbacks.
3. The `test-gate` feature and a heartbeat harness in one track (Python, since
   asyncio has the tooling), proving the deterministic "did not block" check.
4. Rung B, with the panic-in-worker and reentrancy tests.
5. Worked references: the Dart track (it exists for this day) and Python, rungs A to
   C.
6. The Godot extension for 2015-12-06 and its demo scene.
7. `exercises/ex6-*`: scaffold with the checkpoint TODO, overlay, `just` recipes,
   Verify cells with the heartbeat.
8. The book chapter, last.

## Still open

1. **The Kotlin "main thread":** a single-thread executor (recommended: no new
   dependency), or pull in kotlinx-coroutines for `Dispatchers.Main`-style code that
   reads like Android. The latter needs a main dispatcher implementation on a
   plain JVM, which is another jar.
2. **An optional Android demo** on the presenter's device: worth it for the ANR
   dialog on screen, or is the lecture enough?
3. **The ANR-scale input**: ship a generator for a 20× input in the day's bench
   helpers, or only describe it?
