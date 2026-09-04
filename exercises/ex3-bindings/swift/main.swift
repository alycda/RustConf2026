// Exercise 3, Swift track.
//
// Compile & run from this directory (adjust lib extension on Linux):
//   swiftc main.swift -import-objc-header ../../ex2-c-glue/include/ex2_c_glue.h \
//       -L ../../ex2-c-glue/target/release -lex2_c_glue -o ex3 \
//   && DYLD_LIBRARY_PATH=../../ex2-c-glue/target/release ./ex3
//
// (Why is this file named main.swift? Multi-file swiftc only allows
// top-level code in main.swift — a rule you now know that most Swift
// developers learn the hard way.)

// TODO 1: your day's example input. Swift strings are UTF-16 internally —
// what happens on the way to `const char *`? (Module 3 answered this;
// now watch it happen.)
let example = "PASTE YOUR DAY'S EXAMPLE INPUT HERE"
let expectedPart1: Int64 = 0  // from the puzzle statement

// TODO 2: call ex_part1. Swift auto-bridges String → UnsafePointer<CChar>
// for C functions taking `const char *`... which hides exactly the work
// Ex 2 made you do by hand. Free lunch or hidden cost? Debrief material.
let got: Int64 = -999  // replace with the real call

guard got == expectedPart1 else {
    print("part1 = \(got), expected \(expectedPart1)")
    exit(1)
}

// TODO 3: the hostile-input contract. How do you even pass NULL from Swift?
// (Hint: the imported signature takes an Optional pointer.)

print("Ex 3 (Swift) passed.")
