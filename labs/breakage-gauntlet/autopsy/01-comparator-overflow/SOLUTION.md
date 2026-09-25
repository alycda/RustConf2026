# Autopsy 01 — SOLUTION

## The flaw

`sortlib.c`, `cmp_i32`:

```c
return x - y;          /* overflows for |x - y| > INT_MAX */
```

The qsort comparator subtracts two `int`s. When the two values are far apart —
e.g. `2000000000 - (-2000000000)` — the result (`4000000000`) does not fit in
`int`, so it wraps to a *negative* number. qsort is told "x comes before y"
when in truth x is the larger, and the sort order is corrupted. The reported
minimum is wrong.

## Family

**C comparator `a - b` overflows — compare, don't subtract.**
(the reference card, "7 · Footguns"; the same comparator the `qsort` lane of `days/2024-12-01` is built around.)

## The one hostile input

```python
smallest_reading([2_000_000_000, -2_000_000_000, 5])
# expected: -2000000000
# actual:    2000000000
```

The puzzle's tiny example (`[5, 2, 9, 1, 7]`) hides it because every pairwise
difference is small enough to fit in `int` — the overflow never happens, so the
sort is correct and the code looks trustworthy.

## Why it compiles fine

`x - y` is perfectly legal C; the types line up and the return type is `int`.
Nothing is undefined *at compile time*. The undefined behavior (signed overflow)
only happens at runtime, and only for inputs whose spread exceeds `INT_MAX` —
which the ASCII-scale test fixture never produces.

## How CI proves it (`check.sh`, real output)

```
== [1/2] wrong-answer proof (plain build) ==
smallest_reading = 2000000000 (correct = -2000000000)
FLAW MANIFESTED: comparator overflow produced the wrong minimum.

== [2/2] UBSan proof (-fsanitize=undefined) ==
sortlib.c:16:14: runtime error: signed integer overflow:
    2000000000 - -2000000000 cannot be represented in type 'int'

FLAW CONFIRMED: comparator a-b overflow.
```

The plain build shows the *silent* failure (wrong answer, exit 0 in the field).
`-fsanitize=undefined -fno-sanitize-recover=undefined` makes it *deterministic*:
the subtraction traps and aborts, every run.

## The fix

Compare, don't subtract:

```c
return (x > y) - (x < y);   /* -1 / 0 / +1, no arithmetic on the range */
```

(In a JVM/Kotlin JNA binding the identical bug is `Comparator { a, b -> a - b }`
on 32-bit `Int` — fixed with `Integer.compare(a, b)` / `a.compareTo(b)`. Same
family, different runtime.)
