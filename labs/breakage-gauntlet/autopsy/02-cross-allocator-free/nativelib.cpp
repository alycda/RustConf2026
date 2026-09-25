// AI-generated native side: renders an answer string on the native runtime's
// heap and hands the caller a raw pointer, with an explicit release function.
//
// The contract the binding is SUPPOSED to honour:
//   - `render_answer()` allocates with the native runtime's allocator.
//   - the caller MUST return that pointer to `answer_free()` — never to any
//     other free/delete/GC. Free with the allocator that allocated.
//
// One planted flaw lives on the binding side. See REVIEW.md.
#include <cstring>
#include <cstddef>

extern "C" {

// Allocated with operator new[] — i.e. the C++ runtime's allocator, standing
// in for "a different runtime's heap than the caller's libc".
char *render_answer() {
    const char *msg = "42";
    size_t n = std::strlen(msg) + 1;
    char *buf = new char[n];
    std::memcpy(buf, msg, n);
    return buf;
}

// The ONLY correct way to release what render_answer() returned.
void answer_free(char *p) {
    delete[] p;
}

} // extern "C"
