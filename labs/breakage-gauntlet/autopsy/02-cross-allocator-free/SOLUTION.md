# Autopsy 02 — SOLUTION

## The flaw

`flawed_binding.py`, in `get_answer()`:

```python
libc.free(ptr)              # cross-allocator free
# correct:  native.answer_free(ptr)
```

`render_answer()` allocated the buffer with the native runtime's allocator
(`operator new[]`, standing in for "a heap that is not your libc's"). The
binding releases it with libc `free()`. The allocator that frees is not the
allocator that allocated.

## Family

**Free with the allocator that allocated — never across runtimes.**
(the reference card, "7 · Footguns".)

## The one hostile input

There isn't one — **every** call is already wrong. That is what makes this
flaw dangerous: no crafted input is needed, and on many machines it silently
"works" because the two allocators happen to share the same underlying pages
today. It becomes a crash only when the heaps diverge (a different platform, a
different build, a future allocator) — i.e. by luck, which is why you must
*never* rely on a chance segfault to catch it.

## Why it compiles fine

`ptr` is a `void*` on the Python side; `libc.free` accepts any pointer. Nothing
in the type system records *which allocator owns this pointer* — that ownership
lives only in the human-readable contract, which the AI binding ignored. It is
a perfectly well-typed, perfectly compilable mistake.

## How CI proves it (`check.sh`, real output)

```
== ASan proof (native new[] freed with libc free) ==
==NNNNN==ERROR: AddressSanitizer: alloc-dealloc-mismatch (operator new [] vs free) on 0x...
FLAW CONFIRMED: free with the wrong allocator across runtimes.
```

AddressSanitizer (`-fsanitize=address`, with `alloc_dealloc_mismatch=1`) turns
the maybe-crash into a guaranteed abort on every run — the deterministic judge
the attendee toolchain deliberately lacks.

## The fix

Return the pointer to the allocator that owns it:

```python
native.answer_free(ptr)     # the release path the library documented
```

General rule: whichever side allocates must expose (and be given back) its own
free. Never assume `free`/`delete`/GC is interchangeable across an FFI boundary.
