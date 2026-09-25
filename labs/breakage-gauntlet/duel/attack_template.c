/* Station A — Hostile-Caller Duel: your attack harness.
 *
 * This mirrors exercises/ex2-c-glue/tests/c/test_glue.c — same include, same
 * shape — except now you are the ATTACKER. Your job: hand ex_part1 the nastiest
 * IN-CONTRACT input you can and see whether the defender's glue holds.
 *
 * Build it the way the defender builds their own harness (see
 * exercises/ex2-c-glue/build-and-test.sh):
 *
 *     cc attack.c path/to/libex2_c_glue.{dylib,so} -o attack && ./attack
 *
 * You get the defender's include/ex2_c_glue.h and their compiled cdylib.
 * You do NOT get their src/lib.rs — attack the contract, not the source.
 */
#include <stdio.h>
#include "../../../exercises/ex2-c-glue/include/ex2_c_glue.h"
/* ^ adjust this path to wherever the defender's generated header actually is. */

/* THE SAFETY RULE, in code form:
 *   Every `input` below must be a well-formed, NUL-terminated C string, or NULL.
 *   A C string literal is automatically NUL-terminated for you — good.
 *   Do NOT hand-forge a char* that isn't terminated. That is out-of-contract,
 *   voids the point, and is UB roulette. If you're building a pointer by hand,
 *   stop: you've left the contract.
 */

static void probe(const char *label, const char *input) {
    /* A NULL defender-guard skip will usually crash HERE, inside ex_part1. */
    long long got = ex_part1(input);
    printf("  %-28s -> %lld\n", label, got);
}

int main(void) {
    printf("Hostile-caller duel: four in-contract attack classes\n");

    /* ---- Class 1: NULL --------------------------------------------------- */
    /* Contract says null is legal input. A correct defender returns -1.
     * A defender who skipped the null check dereferences it -> crash. */
    probe("class 1: NULL", NULL);

    /* ---- Class 2: invalid UTF-8 ----------------------------------------- */
    /* NUL-terminated (the literal terminates it), but 0xFF/0xFE are never
     * valid UTF-8. Correct defender returns -1; a skipper may read garbage. */
    probe("class 2: invalid UTF-8", "\xff\xfe not valid utf-8");

    /* ---- Class 3: embedded NUL ------------------------------------------ */
    /* A LEGAL C string that ends early: C sees only "12". If your day's answer
     * depends on the bytes after the \0, the defender silently returns a
     * TRUNCATED (wrong) answer -> catalog family "missing/embedded NUL".
     * Swap in an input where truncation actually changes the answer. */
    probe("class 3: embedded NUL", "12\000034");

    /* ---- Class 4: absurd length ----------------------------------------- */
    /* Well-formed and NUL-terminated, just enormous. Watch for slow paths,
     * integer width assumptions, or allocation surprises on the defender side.
     * (Kept small here so the template runs instantly; scale it up on the day.)
     */
    {
        static char big[1 << 16];      /* 64 KiB, stack-safe as static */
        for (size_t i = 0; i < sizeof(big) - 1; i++) big[i] = 'a';
        big[sizeof(big) - 1] = '\0';   /* ALWAYS terminate. In contract. */
        probe("class 4: absurd length", big);
    }

    /* ---- Your turn ------------------------------------------------------- */
    /* Craft the one input the footgun catalog predicts will break THIS
     * defender's glue. Paste your day's example, then mutate it in-contract.
     * Score only breaks the catalog names (reference card, "7 · Footguns"). */

    printf("Done. Now adjudicate each line against RULES.md's scoring table.\n");
    return 0;
}
