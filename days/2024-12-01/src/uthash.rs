//! Hand-written FFI binding to the uthash wrapper (uthash_wrapper.c),
//! compiled by build.rs when the `uthash` feature is on — the part-2 hash
//! table from the talk (alycda/aoc-ffi), ported.
//!
//! uthash itself has no symbols to bind: it is a header of macros, and
//! HASH_ADD_INT / HASH_FIND_INT expand to inline C at each use site. The
//! wrapper is the translation unit that instantiates them behind three
//! ordinary C functions, and those three are all this module knows about.

/// Opaque handle to the wrapper's `struct hash_entry` chain. Rust never
/// looks inside — the UT_hash_handle layout stays uthash's business.
#[repr(C)]
struct HashEntry {
    _opaque: [u8; 0],
}

unsafe extern "C" {
    fn uthash_build_frequency_map(arr: *const i32, len: usize) -> *mut HashEntry;
    fn uthash_lookup(hash_table: *mut HashEntry, key: i32) -> i32;
    fn uthash_destroy(hash_table: *mut HashEntry);
}

/// Part 2's similarity score with the frequency map built and queried on
/// the C side: weight each left-hand ID by its count in the right column.
pub fn similarity(left: &[i32], right: &[i32]) -> i32 {
    // SAFETY: the slice pointers are valid for `len` reads and the wrapper
    // only reads them during the call; the table pointer stays owned here
    // from build to destroy, and a NULL table (empty `right`) is a valid
    // empty table to every uthash operation.
    unsafe {
        let hash_table = uthash_build_frequency_map(right.as_ptr(), right.len());

        let result = left.iter().map(|&n| n * uthash_lookup(hash_table, n)).sum();

        uthash_destroy(hash_table);

        result
    }
}
