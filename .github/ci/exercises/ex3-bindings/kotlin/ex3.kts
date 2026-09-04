// Exercise 3, Kotlin track (JNA) — solved. The CI overlay for the attendee
// scaffold (see .github/ci/README.md): the same file with its TODOs filled,
// run the way the scaffold says, from this directory:
//   kotlinc -script ex3.kts -classpath "$jna" \
//       -J-Djna.library.path=../../target/debug -J-Djna.encoding=UTF-8

import com.sun.jna.Library
import com.sun.jna.Native

// TODO 1, done: method names match the exported symbols in the generated
// header exactly. `String?` is the point — a nullable parameter is how a
// null pointer can be handed across from Kotlin. JNA encodes the string with
// jna.encoding, whose default is the platform charset (native.encoding) — not
// UTF-8: cp1252 on Windows, ASCII in a LANG=C shell, and a non-ASCII byte
// becomes `?` before the C side ever sees it, silently. The run line pins
// -Djna.encoding=UTF-8, the encoding the C side reads. (The modified-UTF-8
// story from Module 3 is JNI's, not JNA's.)
interface Ex2Library : Library {
    fun ex_part1(input: String?): Long
    fun ex_part2(input: String?): Long
}

val lib = Native.load("ex2_c_glue", Ex2Library::class.java)

// TODO 2, done: the statement example and both answers.
val example = listOf(
    "987654321111111",
    "811111111111119",
    "234234234234278",
    "818181911112111",
).joinToString("\n")
val expectedPart1 = 357L
val expectedPart2 = 3121910778619L // above 32 bits, on purpose

// TODO 3, done: call, check, and prove the null contract.
val got1 = lib.ex_part1(example)
if (got1 != expectedPart1) {
    println("part1 = $got1, expected $expectedPart1")
    kotlin.system.exitProcess(1)
}
val got2 = lib.ex_part2(example)
if (got2 != expectedPart2) {
    println("part2 = $got2, expected $expectedPart2")
    kotlin.system.exitProcess(1)
}
if (lib.ex_part1(null) != -1L) {
    println("null input was not refused")
    kotlin.system.exitProcess(1)
}

println("Ex 3 (Kotlin) passed.")
