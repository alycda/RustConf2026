/* Proof harness for the cross-allocator-free flaw.
 *
 * Reproduces exactly what flawed_binding.py does: take the pointer
 * render_answer() allocated with the native runtime's allocator (operator
 * new[]) and release it with the C runtime's free(). Built with
 * AddressSanitizer, this is a deterministic alloc-dealloc-mismatch — never a
 * chance segfault. Run with ASAN_OPTIONS=alloc_dealloc_mismatch=1 (check.sh
 * sets it).
 */
#include <stdio.h>
#include <stdlib.h>

char *render_answer(void);
void answer_free(char *p);   /* the correct release path we deliberately skip */

int main(void) {
    char *p = render_answer();
    printf("answer = %s\n", p);
    free(p);                 /* <-- THE FLAW: wrong allocator */
    printf("freed with libc free (should never reach here under ASan)\n");
    return 0;
}
