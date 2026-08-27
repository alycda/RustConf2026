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
//                 include/aoc_2024_12_03.h — but it maps Kotlin types to C
//                 types by convention (String -> const char*,
//                 LongByReference -> a 64-bit slot, Int <- int), so there
//                 is no second, native-typed spelling to keep in sync.
//
// That convention is the trade: JNA is the least typing of the three and the
// least checked — the JVM has no unsigned 64-bit type at all, so the
// header's uint64_t crosses as the same eight bytes read back through a
// signed Long. Harmless here (this day's sums sit far below 2^63) but
// exactly the kind of quiet reinterpretation cffi reading the real header
// would at least have had to write down.
//
// Run via: just days kotlin-demo 2024-12-03 (regenerates the header, builds
// the cdylib, fetches JNA, compiles and runs this).

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.ptr.LongByReference
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

// int aoc_2024_12_03_part1(const char *input, uint64_t *out_sum);
// int aoc_2024_12_03_part2(const char *input, uint64_t *out_sum);
interface Aoc20241203 : Library {
    fun aoc_2024_12_03_part1(input: String, outSum: LongByReference): Int
    fun aoc_2024_12_03_part2(input: String, outSum: LongByReference): Int
}

/// This jar's own directory: days/<day>/kotlin. Located rather than assumed,
/// so the script works whatever directory it is invoked from — the same
/// property python/solve.py gets from `__file__` and dart/solve.dart from
/// `Platform.script`.
private fun kotlinDir(): File =
    File(Aoc20241203::class.java.protectionDomain.codeSource.location.toURI()).parentFile

private fun dayDir(): File = kotlinDir().parentFile

private fun loadLibrary(): Aoc20241203 {
    val daysDir = dayDir().parentFile
    for (profile in listOf("debug", "release")) {
        // A cdylib takes the host's extension, not Rust's choice: .so on
        // Linux, .dylib on macOS. Same two-loop search as python/solve.py.
        for (ext in listOf("so", "dylib")) {
            val candidate = File(daysDir, "target/$profile/libaoc_2024_12_03.$ext")
            if (candidate.exists()) {
                return Native.load(candidate.absolutePath, Aoc20241203::class.java)
            }
        }
    }
    err.println(
        "no libaoc_2024_12_03.{so,dylib} found — run: cd days && cargo build -p aoc-2024-12-03 --lib"
    )
    kotlin.system.exitProcess(1)
}

/// Runs one of the C API's out-param/status-code functions. A nonzero status
/// is an error the C side already classified — report it and exit nonzero
/// rather than printing a phantom answer.
private fun call(name: String, fn: (String, LongByReference) -> Int, text: String): Long {
    val slot = LongByReference()
    val status = fn(text, slot)
    if (status != 0) {
        err.println("$name failed with status $status (-1: input was null or not valid UTF-8)")
        kotlin.system.exitProcess(1)
    }
    return slot.value
}

fun main() {
    val lib = loadLibrary()

    val inputFile = File(dayDir().parentFile, "inputs/2024-12-03.txt")
    if (!inputFile.exists()) {
        err.println("no puzzle input at ${inputFile.path} (see days/.gitignore)")
        kotlin.system.exitProcess(1)
    }
    val text = inputFile.readText()

    out.println("Part 1 ☕️(🦀): ${call("part1", lib::aoc_2024_12_03_part1, text)}")
    out.println("Part 2 ☕️(🦀): ${call("part2", lib::aoc_2024_12_03_part2, text)}")
}
