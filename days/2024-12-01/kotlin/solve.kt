// Exercise 3 (Kotlin/JNA track): call the Exercise 2 C API from Kotlin via
// JNA (Java Native Access).
//
// Where the tracks differ, and it is a real difference:
//
//   python/cffi   reads the real cbindgen header at runtime and derives its
//                 declarations from it — one source of truth.
//   dart:ffi      needs every signature written twice, once in C-ish native
//                 types and once in Dart types.
//   JNA (here)    needs each function written once, as an ordinary Kotlin
//                 method on an interface. It cannot read the header either —
//                 the names and arity below are hand-transcribed from
//                 include/aoc_2024_12_01.h — but it maps Kotlin types to C
//                 types by convention (String -> const char*,
//                 IntByReference -> int32_t*, Int <- int), so there is no
//                 second, native-typed spelling to keep in sync.
//
// That convention is the trade: JNA is the least typing of the three and the
// least checked. Nothing here verifies that the header still says what this
// file assumes — a renamed export fails at load time with UnsatisfiedLink,
// but a *changed signature* is found at runtime, if at all. cffi reading the
// real header would have caught it at declaration time.
//
// Run via: just days kotlin-demo 2024-12-01 (regenerates the header, builds
// the cdylib, fetches JNA, compiles and runs this).

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.ptr.IntByReference
import java.io.File
import java.io.PrintStream
import java.nio.charset.StandardCharsets

/// The JVM picks `System.out`'s encoding from the console/locale, not from
/// the source file, so the repo's `Part N (crab)` labels come out as `?`
/// under an ASCII-defaulted stdout. Wrapping both streams in an explicitly
/// UTF-8 PrintStream fixes it for anyone who runs the jar, rather than
/// passing `-Dstdout.encoding` in the just recipe and leaving a direct
/// `java -cp ...` invocation broken.
private val out = PrintStream(System.out, true, StandardCharsets.UTF_8)
private val err = PrintStream(System.err, true, StandardCharsets.UTF_8)

// int aoc_2024_12_01_part1(const char *input, int32_t *out_distance);
// int aoc_2024_12_01_part2(const char *input, int32_t *out_score);
interface Aoc20241201 : Library {
    fun aoc_2024_12_01_part1(input: String, outValue: IntByReference): Int
    fun aoc_2024_12_01_part2(input: String, outValue: IntByReference): Int
}

/// This jar's own directory: days/<day>/kotlin. Located rather than assumed,
/// so the script works whatever directory it is invoked from — the same
/// property python/solve.py gets from `__file__` and dart/solve.dart from
/// `Platform.script`.
private fun kotlinDir(): File =
    File(Aoc20241201::class.java.protectionDomain.codeSource.location.toURI()).parentFile

private fun dayDir(): File = kotlinDir().parentFile

private fun loadLibrary(): Aoc20241201 {
    val daysDir = dayDir().parentFile
    for (profile in listOf("debug", "release")) {
        // A cdylib takes the host's name, not Rust's choice: libaoc_2024_12_01.so
        // on Linux, libaoc_2024_12_01.dylib on macOS, aoc_2024_12_01.dll (no lib prefix)
        // on Windows. Same three-name search as python/solve.py — no platform
        // check, whichever file cargo produced is the one that exists.
        for (name in listOf("libaoc_2024_12_01.so", "libaoc_2024_12_01.dylib", "aoc_2024_12_01.dll")) {
            val candidate = File(daysDir, "target/$profile/$name")
            if (candidate.exists()) {
                return Native.load(candidate.absolutePath, Aoc20241201::class.java)
            }
        }
    }
    err.println(
        "no libaoc_2024_12_01.{so,dylib} / aoc_2024_12_01.dll found — run: cd days && cargo build -p aoc-2024-12-01 --lib"
    )
    kotlin.system.exitProcess(1)
}

/// Runs one of the C API's out-param/status-code functions. A nonzero status
/// is an error the C side already classified — report it and exit nonzero
/// rather than printing a phantom answer.
private fun call(name: String, fn: (String, IntByReference) -> Int, text: String): Int {
    val slot = IntByReference()
    val status = fn(text, slot)
    if (status != 0) {
        err.println(
            "$name failed with status $status (-1: input was null, not valid UTF-8, " +
                "or not two integers per line; -2: a total overflowed an int32_t)"
        )
        kotlin.system.exitProcess(1)
    }
    return slot.value
}

fun main() {
    val lib = loadLibrary()

    val inputFile = File(dayDir().parentFile, "inputs/2024-12-01.txt")
    if (!inputFile.exists()) {
        err.println("no puzzle input at ${inputFile.path} (see days/.gitignore)")
        kotlin.system.exitProcess(1)
    }
    val text = inputFile.readText()

    out.println("Part 1 ☕️(🦀): ${call("part1", lib::aoc_2024_12_01_part1, text)}")
    out.println("Part 2 ☕️(🦀): ${call("part2", lib::aoc_2024_12_01_part2, text)}")
}
