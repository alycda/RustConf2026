// The C-callable surface over uthash, declared for both sides: build.rs
// compiles uthash_wrapper.c against this header, and src/uthash.rs binds
// the same three signatures by hand.
//
// A wrapper is not optional here: uthash is a macro library — HASH_ADD_INT
// and friends expand to inline C at each use site, so there is no symbol in
// any object file to bind to until a C translation unit instantiates them.
// That is the teaching point of this variant: "header-only" C libraries
// still cost you a real C compile step at the boundary.
#pragma once
#include <uthash.h>
#include <stdint.h>
#include <stdlib.h>

// Hash table entry structure for uthash
struct hash_entry {
    int32_t key;        // the key (number from input)
    int32_t count;      // frequency count
    UT_hash_handle hh;  // makes this structure hashable
};

// Create and populate a frequency map from an array
struct hash_entry *uthash_build_frequency_map(const int32_t *arr, size_t len);

// Look up a key and return its count (0 when absent)
int32_t uthash_lookup(struct hash_entry *hash_table, int32_t key);

// Free the entire hash table
void uthash_destroy(struct hash_entry *hash_table);
