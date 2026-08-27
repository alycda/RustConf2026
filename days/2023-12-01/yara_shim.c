/* The one part of YARA's C API that an FFI cannot reach.
 *
 * Everything else this day calls — yr_initialize, yr_compiler_create,
 * yr_compiler_add_string, yr_compiler_get_rules, yr_rules_scan_mem and the
 * destructors — is an ordinary exported symbol, and src/yara.rs declares each
 * one in an `unsafe extern "C"` block with opaque pointer types. Reading the
 * *matches* is not an exported symbol. `yr_rule_strings_foreach` and
 * `yr_string_matches_foreach` are preprocessor macros that walk YR_RULE,
 * YR_STRING and YR_MATCH by field, and those structs are built out of
 * DECLARE_REFERENCE, whose layout depends on how libyara was configured.
 * Rust cannot call a macro, and hand-transcribing three internal structs to
 * chase it would be a binding that compiles against one build of YARA and
 * silently reads garbage from the next.
 *
 * So the macro-walking stays in C, where the header defines it, and hands
 * Rust a flat array of plain integers. Six lines of C to avoid a whole class
 * of version-skew bug; build.rs compiles this into a static archive when the
 * `yara` feature is on, and nothing here is compiled otherwise.
 *
 * The Rust side keeps the puzzle. This file does not know what a calibration
 * value is — it reports (offset, which string) pairs and stops.
 */

#include <stddef.h>
#include <stdint.h>
#include <yara.h>

/* One reported occurrence: where it started, and which of the rule's strings
 * it was. `string_index` is YR_STRING.idx, which is declaration order within
 * the compiled rules — src/yara.rs writes the rule text, so it owns the
 * mapping from index back to digit value. */
typedef struct
{
  uint64_t offset;
  uint32_t string_index;
} aoc_yara_match;

typedef struct
{
  aoc_yara_match *out;
  size_t cap;
  size_t count;
  int overflow;
} aoc_collector;

static int aoc_on_rule(
    YR_SCAN_CONTEXT *context,
    int message,
    void *message_data,
    void *user_data)
{
  if (message != CALLBACK_MSG_RULE_MATCHING)
    return CALLBACK_CONTINUE;

  YR_RULE *rule = (YR_RULE *) message_data;
  aoc_collector *collector = (aoc_collector *) user_data;
  YR_STRING *string;
  YR_MATCH *match;

  yr_rule_strings_foreach(rule, string)
  {
    yr_string_matches_foreach(context, string, match)
    {
      if (collector->count >= collector->cap)
      {
        collector->overflow = 1;
        return CALLBACK_ABORT;
      }
      /* base is 0 for a scan_mem of a single block, but adding it is what
       * makes the offset absolute rather than block-relative, which is the
       * contract the field names describe. */
      collector->out[collector->count].offset =
          (uint64_t) match->base + (uint64_t) match->offset;
      collector->out[collector->count].string_index = string->idx;
      collector->count++;
    }
  }

  return CALLBACK_CONTINUE;
}

/* Scans `data` and writes every occurrence of every string in `rules` into
 * `out`, in whatever order YARA reports them — which is grouped by string,
 * NOT sorted by offset. The caller sorts, or takes a min and a max.
 *
 * Returns 0 on success, -1 if there were more matches than `cap` (in which
 * case *out_count is not meaningful), or YARA's own nonzero error code.
 * `out_count` receives the number of entries written. */
int aoc_yara_collect(
    YR_RULES *rules,
    const uint8_t *data,
    size_t length,
    aoc_yara_match *out,
    size_t cap,
    size_t *out_count)
{
  aoc_collector collector = {out, cap, 0, 0};

  int result = yr_rules_scan_mem(
      rules,
      data,
      length,
      SCAN_FLAGS_REPORT_RULES_MATCHING,
      aoc_on_rule,
      &collector,
      0);

  if (collector.overflow)
    return -1;
  if (result != ERROR_SUCCESS)
    return result;

  *out_count = collector.count;
  return 0;
}
