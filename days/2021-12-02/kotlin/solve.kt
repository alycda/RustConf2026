// Exercise 3 (Kotlin/JNA track): call the Exercise 2 C API from Kotlin via
// JNA (Java Native Access).
//
// Where the three tracks differ, and it is a real difference:
//
//   python/cffi   reads the real cbindgen header at runtime and derives its
//                 declarations from it — one source of truth.
//   dart:ffi      needs every signature written twice, once in C-ish native
//                 types and once in Dart types.
//   JNA (here)    needs each function written once, as an ordinary Kotlin
//                 method on an interface. It cannot read the header either —
//                 the names and arity below are hand-transcribed from
//                 include/aoc_2021_12_02.h — but it maps Kotlin types to C
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
// Run via: just days kotlin-demo 2021-12-02 (regenerates the header, builds
// the cdylib, fetches JNA, compiles and runs this).

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.ptr.IntByReference
import java.io.File
import java.io.PrintStream
import java.nio.charset.StandardCharsets

/// The JVM picks `System.out`'s encoding from the console/locale, not from
/// the source file, so this track was the first to print the repo's own
/// `Part N (crab)` label with the emoji replaced by `?` — they do not survive
/// an ASCII-defaulted stdout, which is what a shell with no UTF-8 locale
/// gives you. Wrapping both streams in an explicitly UTF-8 PrintStream fixes
/// it for anyone who runs the jar, rather than passing `-Dstdout.encoding` in
/// the just recipe and leaving a direct `java -cp ...` invocation broken.
private val out = PrintStream(System.out, true, StandardCharsets.UTF_8)
private val err = PrintStream(System.err, true, StandardCharsets.UTF_8)

// int aoc_2021_12_02_part1(const char *input, int32_t *out_product);
// int aoc_2021_12_02_part2(const char *input, int32_t *out_product);
interface Aoc20211202 : Library {
    fun aoc_2021_12_02_part1(input: String, outProduct: IntByReference): Int
    fun aoc_2021_12_02_part2(input: String, outProduct: IntByReference): Int
}

/// This jar's own directory: days/<day>/kotlin. Located rather than assumed,
/// so the script works whatever directory it is invoked from — the same
/// property python/solve.py gets from `__file__` and dart/solve.dart from
/// `Platform.script`.
private fun kotlinDir(): File =
    File(Aoc20211202::class.java.protectionDomain.codeSource.location.toURI()).parentFile

private fun dayDir(): File = kotlinDir().parentFile

private fun loadLibrary(): Aoc20211202 {
    val daysDir = dayDir().parentFile
    for (profile in listOf("debug", "release")) {
        // A cdylib takes the host's name, not Rust's choice: libaoc_2021_12_02.so
        // on Linux, libaoc_2021_12_02.dylib on macOS, aoc_2021_12_02.dll (no lib prefix)
        // on Windows. Same three-name search as python/solve.py — no platform
        // check, whichever file cargo produced is the one that exists.
        for (name in listOf("libaoc_2021_12_02.so", "libaoc_2021_12_02.dylib", "aoc_2021_12_02.dll")) {
            val candidate = File(daysDir, "target/$profile/$name")
            if (candidate.exists()) {
                return Native.load(candidate.absolutePath, Aoc20211202::class.java)
            }
        }
    }
    err.println(
        "no libaoc_2021_12_02.{so,dylib} / aoc_2021_12_02.dll found — run: cd days && cargo build -p aoc-2021-12-02 --lib"
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
                "or not a valid course; -3: the course overflowed an int32_t)"
        )
        kotlin.system.exitProcess(1)
    }
    return slot.value
}

fun main() {
    val lib = loadLibrary()

    val inputFile = File(dayDir().parentFile, "inputs/2021-12-02.txt")
    if (!inputFile.exists()) {
        err.println("no puzzle input at ${inputFile.path} (see days/.gitignore)")
        kotlin.system.exitProcess(1)
    }
    val text = inputFile.readText()

    out.println("Part 1 ☕️(🦀): ${call("part1", lib::aoc_2021_12_02_part1, text)}")
    out.println("Part 2 ☕️(🦀): ${call("part2", lib::aoc_2021_12_02_part2, text)}")
}
