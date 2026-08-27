/* Thin C shim around ICU's regex API (uregex.h), compiled by build.rs via
 * the `cc` crate and called from Rust — see src/icu.rs for why.
 *
 * ICU's C symbols are version-renamed at link time (uregex_open becomes
 * uregex_open_78, tied to this exact ICU build) via macros in its own
 * headers, so hand-written Rust `unsafe extern "C" { fn uregex_open(...); }`
 * declarations would need to hardcode that suffix and break on every ICU
 * upgrade. Compiling this file lets the real header's renaming macros do
 * their job as intended; the two functions below export under names we
 * chose ourselves, so Rust never needs to know ICU's version at all.
 */
#include <unicode/uregex.h>
#include <unicode/utext.h>

/* Number of non-overlapping matches of `pattern` in `text` (both UTF-8,
 * NUL-terminated), or -1 on any ICU error. */
int aoc_icu_regex_count(const char *pattern, const char *text) {
    UErrorCode status = U_ZERO_ERROR;
    URegularExpression *re = uregex_openC(pattern, 0, NULL, &status);
    if (U_FAILURE(status)) {
        return -1;
    }

    UText *utext = utext_openUTF8(NULL, text, -1, &status);
    if (U_FAILURE(status)) {
        uregex_close(re);
        return -1;
    }
    uregex_setUText(re, utext, &status);

    int count = 0;
    while (U_SUCCESS(status) && uregex_findNext(re, &status)) {
        count++;
    }

    int failed = U_FAILURE(status);
    utext_close(utext);
    uregex_close(re);
    return failed ? -1 : count;
}

/* 1 if `pattern` matches anywhere in `text`, 0 if not, -1 on any ICU error. */
int aoc_icu_regex_find(const char *pattern, const char *text) {
    UErrorCode status = U_ZERO_ERROR;
    URegularExpression *re = uregex_openC(pattern, 0, NULL, &status);
    if (U_FAILURE(status)) {
        return -1;
    }

    UText *utext = utext_openUTF8(NULL, text, -1, &status);
    if (U_FAILURE(status)) {
        uregex_close(re);
        return -1;
    }
    uregex_setUText(re, utext, &status);

    int found = uregex_find(re, 0, &status) ? 1 : 0;

    int failed = U_FAILURE(status);
    utext_close(utext);
    uregex_close(re);
    return failed ? -1 : found;
}
