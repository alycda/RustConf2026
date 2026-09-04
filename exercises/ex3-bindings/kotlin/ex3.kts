// Exercise 3, Kotlin track (JNA).
//
// Needs JDK 17+, kotlinc, and the JNA jar — the same pinned, checksummed
// 5.17.0 the days/ recipes use. If you ran `just days kotlin-demo` as
// prework, ../../../days/2024-12-01/kotlin/jna-5.17.0.jar is that file;
// otherwise:
//   jna=jna-5.17.0.jar
//   curl -fsSL -o "$jna" "https://repo1.maven.org/maven2/net/java/dev/jna/jna/5.17.0/$jna"
//   echo "b3a9408e7c51e08ef0e3bfcc08f443f6ec0f6191ba8cd7c18d53d2b22e5bdbc0  $jna" | sha256sum -c -
//   (a Mac without coreutils: shasum -a 256 -c -)
// Run from this directory (the exercises are one cargo workspace, so the
// Ex 2 cdylib is in ../../target/, not inside ex2-c-glue/; JNA maps the
// bare name to libex2_c_glue.so / .dylib / ex2_c_glue.dll itself):
//   kotlinc -script ex3.kts -classpath "$jna" \
//       -J-Djna.library.path=../../target/release

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
