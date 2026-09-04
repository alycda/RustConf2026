// Exercise 3, Swift track — solved. The CI overlay for the attendee scaffold
// (see .github/ci/README.md): the same file with its TODOs filled, compiled
// and run the way the scaffold says, from this directory:
//   swiftc main.swift -import-objc-header ../../ex2-c-glue/include/ex2_c_glue.h \
//       -L ../../target/release -lex2_c_glue \
//       -Xlinker -rpath -Xlinker "$PWD/../../target/release" -o ex3 \
//   && ./ex3

import Foundation

// TODO 1, done: the statement example. Swift String is UTF-8 inside since
// Swift 5 (https://www.swift.org/blog/utf8-string/), so the trip to
// `const char *` is a NUL-terminated view of bytes it already has.
let example = [
    "987654321111111",
    "811111111111119",
    "234234234234278",
    "818181911112111",
].joined(separator: "\n")
let expectedPart1: Int64 = 357
let expectedPart2: Int64 = 3121910778619  // above 32 bits, on purpose

// TODO 2, done: Swift bridges String → UnsafePointer<CChar> for a C function
// taking `const char *` on its own — exactly the work Ex 2 did by hand.
let got1 = ex_part1(example)
guard got1 == expectedPart1 else {
    print("part1 = \(got1), expected \(expectedPart1)")
    exit(1)
}
let got2 = ex_part2(example)
guard got2 == expectedPart2 else {
    print("part2 = \(got2), expected \(expectedPart2)")
    exit(1)
}

// TODO 3, done: the hostile-input contract. The imported signature takes an
// Optional pointer, so nil is the null pointer.
guard ex_part1(nil) == -1 else {
    print("null input was not refused")
    exit(1)
}

print("Ex 3 (Swift) passed.")
