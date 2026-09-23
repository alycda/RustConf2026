/* Exercise 2 C harness, filled in — the CI overlay for the attendee
 * scaffold (see .github/ci/README.md). A C program calling the Rust
 * through the generated header, with 2025 day 3's example. */
#include <assert.h>
#include <stdio.h>
#include "../../include/ex2_c_glue.h"

int main(void) {
    /* The statement example is four lines; a C literal needs the \n spelled
     * out, and adjacent literals concatenate. */
    const char *example =
        "987654321111111\n"
        "811111111111119\n"
        "234234234234278\n"
        "818181911112111";

    long long expected_part1 = 357;
    long long expected_part2 = 3121910778619LL; /* above 32 bits, on purpose */

    /* Hostile-input contract first, so an unsolved Ex 1 cannot hide it. */
    assert(ex_part1(NULL) == -1);
    assert(ex_part1("\xff\xfe not utf-8") == -1);

    long long got = ex_part1(example);
    printf("part1(example) = %lld (expected %lld)\n", got, expected_part1);
    assert(got == expected_part1);

    long long got2 = ex_part2(example);
    printf("part2(example) = %lld (expected %lld)\n", got2, expected_part2);
    assert(got2 == expected_part2);

    printf("All Ex 2 checks passed.\n");
    return 0;
}
