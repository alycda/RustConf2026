/* Calls 2015-12-06's cbindgen surface from C.
 *
 * The include below resolves because //days/2015-12-06:c_api sets
 * includes = ["include"] on the *generated* header — the file does not
 * exist in the source tree and is never committed (.gitignore), exactly as
 * under `just days bindgen 2015-12-06`. The difference is that here it is a
 * build output with a declared consumer, so touching src/c_api.rs
 * invalidates this translation unit and touching src/lib.rs does not. */
#include <stdio.h>
#include <string.h>

#include "aoc_2015_12_06.h"

static int expect(const char *what, int rc, unsigned int got, unsigned int want) {
    if (rc != 0) {
        fprintf(stderr, "%s: returned %d, expected 0\n", what, rc);
        return 1;
    }
    if (got != want) {
        fprintf(stderr, "%s: got %u, expected %u\n", what, got, want);
        return 1;
    }
    return 0;
}

int main(void) {
    int failures = 0;
    int rc = 0;
    unsigned int value = 0;

    const char *part1_input =
        "turn on 0,0 through 999,999\n"
        "toggle 0,0 through 999,0\n"
        "turn off 499,499 through 500,500";
    /* The call is sequenced before the check on purpose: passing it as an
     * argument alongside `value` would leave the read of `value`
     * unsequenced against the write the callee performs. */
    rc = aoc_2015_12_06_part1(part1_input, &value);
    failures += expect("part1", rc, value, 998996);

    const char *part2_input =
        "turn on 0,0 through 0,0\n"
        "toggle 0,0 through 999,999";
    value = 0;
    rc = aoc_2015_12_06_part2(part2_input, &value);
    failures += expect("part2", rc, value, 2000001);

    /* The refusal path, from C rather than from the Rust-side test: a null
     * input is a status code, not a panic unwinding across the boundary. */
    value = 7;
    if (aoc_2015_12_06_part1(NULL, &value) != -1 || value != 7) {
        fprintf(stderr, "null input: expected -1 and an untouched out_value\n");
        failures++;
    }

    if (failures == 0) {
        printf("2015-12-06 C harness: ok\n");
    }
    return failures;
}
