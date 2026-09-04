// Exercise 3, Swift track.
//
// Compile & run from this directory. The exercises are one cargo workspace,
// so the Ex 2 cdylib is in ../../target/, not inside ex2-c-glue/; -l names
// no extension, so the same line works on macOS and Linux. The -rpath is
// what lets ./ex3 find the library at run time on both — an environment
// variable would be DYLD_* on one and LD_* on the other. The script line
// prints nothing on a Mac or a swift.org toolchain, and the Foundation paths
// the repo's Swift devcontainer needs; `just exercises swift` is the same
// command:
//   swiftc main.swift -import-objc-header ../../ex2-c-glue/include/ex2_c_glue.h \
//       $(../../../scripts/swift-corelibs-flags.sh) \
//       -L ../../target/release -lex2_c_glue \
//       -Xlinker -rpath -Xlinker "$PWD/../../target/release" -o ex3 \
//   && ./ex3
//
// (Why is this file named main.swift? Multi-file swiftc only allows
// top-level code in main.swift — a rule you now know that most Swift
// developers learn the hard way.)

// exit() is libc, not the Swift standard library. It only resolved before
// because cbindgen's default header pulls in <stdlib.h>; say it explicitly
// so the file still compiles against a header that doesn't.
import Foundation

// TODO 1: your day's example input. Swift String has been UTF-8 inside
// since Swift 5 (https://www.swift.org/blog/utf8-string/ — short, worth
// reading); UTF-16 lives in the NSString bridge. So what does the trip to
// `const char *` still cost? (Module 3 answered this; now watch it happen.)
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
