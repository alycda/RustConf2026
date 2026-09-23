// Exercise 3 (Dart track): call the Exercise 2 C API from Dart via
// dart:ffi.
//
// Unlike python/solve.py's cffi (which reads the real cbindgen header at
// runtime and derives its declarations from it — one source of truth),
// dart:ffi has no equivalent for a plain script: DynamicLibrary.open +
// lookupFunction need the C signature written as Dart types up front,
// hand-transcribed from include/aoc_2015_12_05.h below. (A generator,
// package:ffigen, exists for larger APIs — pulling it in for two
// functions would be more machinery than the thing it generates.)
//
// Run via: just days dart-demo 2015-12-05 (fetches ffi, builds the
// cdylib, runs this); or directly once the cdylib exists:
//   cd dart && dart pub get && dart run solve.dart

import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

// int aoc_2015_12_05_part1(const char *input, unsigned int *out_count);
// int aoc_2015_12_05_part2(const char *input, unsigned int *out_count);
typedef _CountFnNative = Int32 Function(
    Pointer<Utf8> input, Pointer<Uint32> outCount);
typedef _CountFnDart = int Function(
    Pointer<Utf8> input, Pointer<Uint32> outCount);

Directory _dayDir() => File(Platform.script.toFilePath()).parent.parent;

DynamicLibrary _loadLibrary() {
  final daysDir = _dayDir().parent;
  for (final profile in ['debug', 'release']) {
    // A cdylib takes the host's name, not Rust's choice: libaoc_2015_12_05.so on
    // Linux, libaoc_2015_12_05.dylib on macOS, aoc_2015_12_05.dll (no lib prefix) on
    // Windows. Same three-name search as python/solve.py — no platform
    // check, whichever file cargo produced is the one that exists.
    for (final name in ['libaoc_2015_12_05.so', 'libaoc_2015_12_05.dylib', 'aoc_2015_12_05.dll']) {
      final path = '${daysDir.path}/target/$profile/$name';
      if (File(path).existsSync()) {
        return DynamicLibrary.open(path);
      }
    }
  }
  stderr.writeln(
      'no libaoc_2015_12_05.{so,dylib} / aoc_2015_12_05.dll found — run: cd days && cargo build -p aoc-2015-12-05 --lib');
  exit(1);
}

/// Runs one of the C API's out-param/status-code functions. A nonzero
/// status is an error the C side already classified — report it and exit
/// nonzero rather than printing a phantom answer.
int _call(_CountFnDart fn, String name, String text) {
  final inputPtr = text.toNativeUtf8();
  final outCount = calloc<Uint32>();
  int status;
  int count;
  try {
    status = fn(inputPtr, outCount);
    count = outCount.value;
  } finally {
    calloc.free(inputPtr);
    calloc.free(outCount);
  }

  if (status != 0) {
    stderr.writeln('$name failed with status $status '
        '(-1: input was null or not valid UTF-8)');
    exit(1);
  }
  return count;
}

void main() {
  final lib = _loadLibrary();
  final part1 =
      lib.lookupFunction<_CountFnNative, _CountFnDart>('aoc_2015_12_05_part1');
  final part2 =
      lib.lookupFunction<_CountFnNative, _CountFnDart>('aoc_2015_12_05_part2');

  final inputFile = File('${_dayDir().parent.parent.path}/inputs/2015-12-05.txt');
  if (!inputFile.existsSync()) {
    stderr.writeln(
        'no puzzle input at ${inputFile.path} (see .gitignore)');
    exit(1);
  }
  final text = inputFile.readAsStringSync();

  print('Part 1 🎯(🦀): ${_call(part1, 'part1', text)}');
  print('Part 2 🎯(🦀): ${_call(part2, 'part2', text)}');
}
