/* Proof harness for the comparator-overflow flaw.
 *
 * Two ways to make the planted flaw deterministic:
 *   1. WRONG ANSWER  — built plain, asserts smallest_reading() returns the
 *      wrong minimum for a full-range i32 input. Exit 0 == flaw manifested.
 *   2. UBSAN         — built with -fsanitize=undefined -fno-sanitize-recover;
 *      the `x - y` subtraction traps as signed-integer-overflow and aborts.
 *
 * check.sh runs both. See SOLUTION.md.
 */
#include <stdio.h>
#include <stddef.h>

int smallest_reading(const int *vals, size_t n);

int main(void) {
    int readings[] = {2000000000, -2000000000, 5};
    int correct_min = -2000000000;

    int got = smallest_reading(readings, 3);
    printf("smallest_reading = %d (correct = %d)\n", got, correct_min);

    /* The flaw MANIFESTS when the answer is wrong. Exit 0 confirms the break. */
    if (got != correct_min) {
        printf("FLAW MANIFESTED: comparator overflow produced the wrong minimum.\n");
        return 0;
    }
    fprintf(stderr, "flaw did NOT manifest (comparator looks fixed)\n");
    return 1;
}
