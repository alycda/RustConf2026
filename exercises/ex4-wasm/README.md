# Exercise 4: One Boundary, No C in It (wasm) — [??s]

## Goal

Your Ex 2 wrapper, built for `wasm32-unknown-unknown` instead of your
machine, called from TypeScript with **nothing generated** — and the one
export the new runtime needs that no C caller ever did.

Afternoon / bonus material: this block has not been in front of a room
yet, so it carries no timing. It slots after Ex 3, and it is the answer
to "try another language" when the other language has no C ABI at all.

## Route

Raw. The `.wasm` cargo produces exports your `extern "C"` functions as
they are; Node's built-in `WebAssembly` API calls them. No wasm-bindgen,
no npm package on the Rust side, no header read by anyone — the module's
own export section is the header now, and `build.sh` prints it.

The generated route (wasm-bindgen writing the glue you are about to write
by hand) is the bonus at the end, and the worked reference for it is
[`../../days/2015-12-01/wasm`](../../days/2015-12-01/wasm/).

## Tasks

0. `../ex2-c-glue` green first, and `just setup-wasm` once (the target
   for rustup users, Node 22 for everyone). Then, **before implementing
   anything**, `just exercises wasm` and read what happens: the crate's
   `todo!()` is Ex 2's step 0 again with a different ending — not an
   abort, a `RuntimeError: unreachable`, and an instance that is still
   there afterwards, answering. Decide for yourself which is worse.
1. `src/lib.rs`, TODO 0: paste your Ex 2 `ex_part1` body in. It compiles
   for the new target with no edits. That is the first lesson.
2. `src/lib.rs`, TODO 1: `ex_alloc` and `ex_free`. Read the doc comments —
   they say why a caller with no allocator needs the module to lend one,
   and why `free` takes the size back.
3. `wasm/src/ex4.ts`, TODO 2 to 5, top to bottom: read how the module is
   loaded, write the string into linear memory yourself (the NUL too),
   call and get a `BigInt` back, prove the hostile-input contract from
   this side.
4. `just exercises wasm` until it prints `Ex 4 (wasm) passed.`

## Key Concepts

- **The header is inside the module.** `WebAssembly.Module.exports` and
  `.imports`, readable before anything runs, with wasm types: `i32`
  where the C header said `const char *`, `i64` where it said `int64_t`.
  Names cannot drift; the C types did not survive.
- **When the caller has no allocator, the callee lends one.** JavaScript
  cannot allocate inside linear memory. `ex_alloc`/`ex_free` are the
  callee-allocates contract from the reference card, met as a
  precondition of calling at all — and `free` wants the size because a
  wasm module carries no `malloc` header to recover it from.
- **`i64` is a `BigInt`.** A JavaScript number cannot hold one, so the
  answer comes back as `5n`, the sentinel as `-1n`, and `5 === 5n` is
  false. Types disagree here too; this runtime says so in the type.
- **A trap is not an abort.** A panic traps the module; nothing unwinds,
  nothing is cleaned up, and the instance keeps answering the next call.
  Ex 2's process death was the kinder failure.
- **Two channels for what goes wrong, and the compiler keeps them
  apart.** The sentinel is a typed failure a caller can match on; the
  trap is a defect `catchAll` cannot see. That is `Result` versus
  `panic!`, rebuilt on the JavaScript side — the distinction the C ABI
  erased in Ex 2 and Ex 3.
- **`memory.buffer` detaches.** A view made before an `ex_alloc` can be
  dead after it. Make the view after the borrow, every time.

## Bonus: the generated lap

`cd ../../days/2015-12-01 && just days wasm-demo 2015-12-01`. Same
solver, `#[wasm_bindgen]` exports, and a tool writing the glue TODO 3
and 4 made you write. Then read `wasm/src/boundary.ts` there: the
fifteen lines the generator still cannot write, because the boundary
does not carry the distinction they restore.

The debrief question you're collecting an answer to: **what did the
wasm runtime need that the C header could not say?**
