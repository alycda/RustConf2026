// Exercise 3, Dart track (dart:ffi).
//
// Needs Dart SDK 3.7+. pubspec.yaml beside this file declares the one
// dependency (package:ffi); worked reference for the shape:
// ../../../days/2024-12-01/dart/solve.dart.
// Run from this directory:  dart pub get && dart run ex3.dart

import 'dart:ffi' as ffi;
import 'dart:io' show File, Platform, exit, stderr;
import 'package:ffi/ffi.dart';

// TODO 1: typedef pair for each function — one describing the C signature,
// one describing the Dart view of it. This duplication IS the binding.
// typedef ExPart1Native = ffi.Int64 Function(ffi.Pointer<Utf8>);
// typedef ExPart1Dart = int Function(ffi.Pointer<Utf8>);

// The exercises are one cargo workspace, so the Ex 2 cdylib lands in
// ../../target/ (not inside ex2-c-glue/), and it takes the host's name, not
// Rust's: libex2_c_glue.so on Linux, libex2_c_glue.dylib on macOS,
// ex2_c_glue.dll (no lib prefix) on Windows. Searching the three needs no
// platform check — whichever file cargo produced is the one that exists.
String _libraryPath() {
  final exercises = File(Platform.script.toFilePath()).parent.parent.parent;
  for (final profile in ['release', 'debug']) {
    for (final name in ['libex2_c_glue.so', 'libex2_c_glue.dylib', 'ex2_c_glue.dll']) {
      final path = '${exercises.path}/target/$profile/$name';
      if (File(path).existsSync()) return path;
    }
  }
  stderr.writeln('no Ex 2 library found — run ../../ex2-c-glue/build-and-test.sh first');
  exit(1);
}

void main() {
  // TODO 2: open the library at _libraryPath() and look up ex_part1.

  // TODO 3: Dart String → C string is manual here: toNativeUtf8() allocates,
  // and YOU free it (calloc.free in a try/finally). No auto-bridging —
  // Dart makes you do what Swift hid. Which do you prefer, and why?
  final example = 'PASTE YOUR DAY\'S EXAMPLE INPUT HERE';
  const expectedPart1 = 0; // from the puzzle statement

  print('Ex 3 (Dart) — fill in the TODOs, then delete this line.');
}
