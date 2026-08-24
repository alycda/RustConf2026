// Exercise 3, Dart track (hand-written route, dart:ffi).
//
// Needs Dart SDK 3.0+ and `dart pub add ffi` in a package, or run the
// worked pattern in ../../days/2024-03/dart/ first to see the shape.
// Run:  dart run ex3.dart   (from a package with ffi in pubspec)

import 'dart:ffi' as ffi;
import 'dart:io' show Platform, exit;
import 'package:ffi/ffi.dart';

// TODO 1: typedef pair for each function — one describing the C signature,
// one describing the Dart view of it. This duplication IS the binding.
// typedef ExPart1Native = ffi.Int64 Function(ffi.Pointer<Utf8>);
// typedef ExPart1Dart = int Function(ffi.Pointer<Utf8>);

void main() {
  // TODO 2: load the library (../../ex2-c-glue/target/release/,
  // .dylib on macOS / .so on Linux) and look up ex_part1.

  // TODO 3: Dart String → C string is manual here: toNativeUtf8() allocates,
  // and YOU free it (calloc.free in a try/finally). No auto-bridging —
  // Dart makes you do what Swift hid. Which do you prefer, and why?
  final example = 'PASTE YOUR DAY\'S EXAMPLE INPUT HERE';
  const expectedPart1 = 0; // from the puzzle statement

  print('Ex 3 (Dart) — fill in the TODOs, then delete this line.');
}
