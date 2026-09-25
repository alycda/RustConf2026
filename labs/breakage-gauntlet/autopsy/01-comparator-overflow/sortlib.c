/* AI-generated native helper for a "closest pair of readings" answer.
 *
 * The task: given a list of sensor readings (signed 32-bit), sort them and
 * return the smallest reading. Looks fine. Compiles clean. Passes on the
 * puzzle's tiny ASCII example. Ships.
 *
 * One planted flaw. See REVIEW.md.
 */
#include <stddef.h>
#include <stdlib.h>

/* The classic C-qsort comparator footgun: subtract instead of compare. */
static int cmp_i32(const void *a, const void *b) {
    int x = *(const int *)a;
    int y = *(const int *)b;
    return x - y;          /* <-- overflows for |x - y| > INT_MAX */
}

/* Sort ascending and return the minimum (the first element after sort). */
int smallest_reading(const int *vals, size_t n) {
    if (n == 0) return 0;
    int *scratch = (int *)malloc(n * sizeof(int));
    for (size_t i = 0; i < n; i++) scratch[i] = vals[i];
    qsort(scratch, n, sizeof(int), cmp_i32);
    int result = scratch[0];
    free(scratch);
    return result;
}
