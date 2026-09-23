// Exercise 3, Kotlin track (JNA).
//
// Needs JDK 17+, kotlinc, and the JNA jar — the same pinned, checksummed
// 5.17.0 the days/ recipes use, from the one script that owns the pin:
//   jna="$(../../../scripts/fetch-jna.sh)"
// (it prints the jar's path: $JNA_JAR in the kotlin devcontainer, otherwise
// a cached, verified download)
// Run from this directory (the exercises are one cargo workspace, so the
// Ex 2 cdylib is in ../../target/, not inside ex2-c-glue/; JNA maps the
// bare name to libex2_c_glue.so / .dylib / ex2_c_glue.dll itself):
//   kotlinc -script ex3.kts -classpath "$jna" \
//       -J-Djna.library.path=../../target/debug -J-Djna.encoding=UTF-8

import com.sun.jna.Library
import com.sun.jna.Native

// TODO 1: declare the interface JNA should bind. Method names must match
// the exported symbols in ../../ex2-c-glue/include/ex2_c_glue.h exactly.
// JNA maps Kotlin String → const char* for you... using which encoding?
// (Check jna.encoding — the default has bitten Android teams before.
// That's the modified-UTF-8 story from Module 3.)
interface Ex2Library : Library {
    // fun ex_part1(input: String?): Long
}

val lib = Native.load("ex2_c_glue", Ex2Library::class.java)

// TODO 2: example input + expected answer from your puzzle statement.
val example = "PASTE YOUR DAY'S EXAMPLE INPUT HERE"
val expectedPart1 = 0L

// TODO 3: call it, check it, and prove the null contract (String? lets you
// pass null — what comes back?).

println("Ex 3 (Kotlin) — fill in the TODOs, then delete this line.")
