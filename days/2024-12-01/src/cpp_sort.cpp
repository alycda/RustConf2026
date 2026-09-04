// The C++ side of the boundary: std::sort wrapped in a C-shaped function.
//
// C++ can't cross the FFI boundary as C++ — templates have no ABI at all
// and mangled names differ per compiler — so the shim flattens to the same
// shape qsort has: extern "C", a raw pointer and a length. The talk's
// version made the same choice for the same reason: handing a std::vector
// across would mean converting at the boundary.
#include <algorithm>
#include <cstddef>
#include <cstdint>

extern "C" void cpp_sort_i32(int32_t *data, size_t len) {
    std::sort(data, data + len);
}
