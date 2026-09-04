// Exercise 3, Kotlin track (JNA).
//
// Needs JDK 17+, kotlinc, and the JNA jar:
//   curl -fsSL -o jna.jar https://repo1.maven.org/maven2/net/java/dev/jna/jna/5.14.0/jna-5.14.0.jar
// Run from this directory:
//   kotlinc -script ex3.kts -classpath jna.jar \
//       -J-Djna.library.path=../../ex2-c-glue/target/release

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
