// Exercise 3 (Swift track): call the Exercise 2 C API from Swift.
//
// Where the four tracks differ, and it is a real difference:
//
//   python/cffi   reads the real cbindgen header at runtime and derives its
//                 declarations from it — one source of truth, checked when
//                 the script starts.
//   dart:ffi      needs every signature written twice, once in native types
//                 and once in Dart types, hand-transcribed from the header.
//   JNA (Kotlin)  needs each function written once, as an ordinary method on
//                 an interface, and marshals by convention — the least
//                 typing and the least checking of the four.
//   Swift (here)  needs no signature at all. `module.modulemap` points clang
//                 at the generated header, `import AocDay` brings the two
//                 functions in as ordinary typed Swift functions, and the
//                 compiler checks every call against the real header. This
//                 is the only track where a header that changed shape is a
//                 build failure rather than something you find out about at
//                 runtime, if at all.
//
// The trade is where the library gets found. The other three `dlopen` a path
// they compute at runtime, so one script covers both cargo profiles. Swift
// links against the cdylib, so "debug or release" is a linker flag, not an
// `if` — see the `swift-demo` recipe in days/justfile, which passes -L and
// -rpath for both. That is the honest shape of a compiled track: the search
// still happens, it just happens once, earlier, and by the build.
//
// Run via: just days swift-demo 2024-12-01 (regenerates the header, builds
// the cdylib, compiles and runs this).

import Foundation
import AocDay

/// This binary's own directory — days/<day>/swift/.build, one level deeper
/// than the source, because that is where the recipe puts the executable:
/// `.build/` is Swift's own convention for build output and is already
/// ignored repo-wide, where a bare `solve` next to `solve.swift` would match
/// no existing rule and get committed.
///
/// `Bundle.main.bundlePath`, not `#filePath`: `#filePath` is baked in at
/// compile time as verbatim whatever path swiftc was given, so a compiler
/// invoked with a relative path leaves a relative path in the binary.
/// `bundlePath` asks the runtime where the executable actually is — the
/// same answer Kotlin gets from `codeSource.location` and Dart from
/// `Platform.script`.
private let buildDir = URL(fileURLWithPath: Bundle.main.bundlePath)
private let dayDir = buildDir.deletingLastPathComponent().deletingLastPathComponent()
private let daysDir = dayDir.deletingLastPathComponent()

private func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data("\(message)\n".utf8))
    exit(1)
}

/// Runs one of the C API's out-param/status-code functions. A nonzero status
/// is an error the C side already classified — report it and exit nonzero
/// rather than printing a phantom answer.
///
/// `withCString` is what makes the `const char *` safe: it hands the callee a
/// NUL-terminated buffer that is guaranteed live for the duration of the
/// closure and no longer, which is exactly the contract the header states.
/// A Swift `String` is not NUL-terminated and has no stable address, so
/// passing one some other way is how this call would go wrong quietly.
private func call(
    _ name: String,
    _ function: (UnsafePointer<CChar>?, UnsafeMutablePointer<Int32>?) -> Int32,
    _ text: String
) -> Int32 {
    var slot: Int32 = 0
    let status = text.withCString { function($0, &slot) }
    guard status == 0 else {
        fail(
            "\(name) failed with status \(status) (-1: input was null, not valid UTF-8, "
                + "or not two integers per line; -2: a total overflowed an int32_t)")
    }
    return slot
}

let inputPath = daysDir.deletingLastPathComponent().appendingPathComponent("inputs/2024-12-01.txt")
guard let text = try? String(contentsOf: inputPath, encoding: .utf8) else {
    fail("no puzzle input at \(inputPath.path) (see .gitignore)")
}

print("Part 1 🕊️(🦀): \(call("part1", aoc_2024_12_01_part1, text))")
print("Part 2 🕊️(🦀): \(call("part2", aoc_2024_12_01_part2, text))")
