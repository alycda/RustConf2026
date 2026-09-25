/* Proof harness for the read-past-end variant of the missing-NUL flaw.
 *
 * This is the OUT-OF-CONTRACT sibling of the embedded-NUL bug: here the buffer
 * is never terminated at all, so a C consumer that trusts the terminator reads
 * off the end of the allocation. Under AddressSanitizer this is a deterministic
 * heap-buffer-overflow — never a chance segfault.
 *
 * NOTE: forging a non-terminated buffer like this is exactly what the DUEL's
 * safety rule forbids (it is UB roulette, not a boundary lesson). It is allowed
 * HERE, and only here, because CI + ASan turn it into a deterministic,
 * educational diagnostic instead of a coin-flip crash.
 */
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* A C consumer that assumes its input is NUL-terminated (the whole contract). */
static long consume(const char *s) {
    return (long)strlen(s);   /* walks to a NUL that isn't there */
}

int main(void) {
    /* Three digit bytes, deliberately WITHOUT a NUL terminator. */
    char *buf = (char *)malloc(3);
    buf[0] = '1';
    buf[1] = '2';
    buf[2] = '3';
    long n = consume(buf);       /* reads past the 3-byte allocation */
    printf("strlen read %ld bytes (should never be trusted)\n", n);
    free(buf);
    return 0;
}
