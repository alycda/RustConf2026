/* Exercise 2 C harness — provided; you fill in the expected values.
 *
 * This is the moment of truth: a C program calling YOUR Rust through
 * YOUR generated header. */
#include <assert.h>
#include <stdio.h>
#include "../../include/ex2_c_glue.h"

int main(void) {
    /* TODO: paste your day's example input as a C string literal.
     * Watch out — this is where encoding reality bites: C string literals
     * with embedded newlines need \n, and your day's example is probably
     * multi-line. */
    const char *example = "PASTE EXAMPLE INPUT HERE";

    /* TODO: replace 0 with the expected answers from the puzzle statement. */
    long long expected_part1 = 0;

    long long got = ex_part1(example);
    printf("part1(example) = %lld (expected %lld)\n", got, expected_part1);
    assert(got == expected_part1);

    /* Hostile-input contract — these two should already pass once your
     * null/UTF-8 checks are in (no edits needed): */
    assert(ex_part1(NULL) == -1);
    assert(ex_part1("\xff\xfe not utf-8") == -1);

    printf("All Ex 2 checks passed.\n");
    return 0;
}
