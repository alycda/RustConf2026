// Exercise 3 (Dart track): call the Exercise 2 C API from Dart via
// dart:ffi.
//
// Unlike python/solve.py's cffi (which reads the real cbindgen header at
// runtime and derives its declarations from it — one source of truth),
// dart:ffi has no equivalent for a plain script: DynamicLibrary.open +
// lookupFunction need the C signature written as Dart types up front,
// hand-transcribed from include/aoc_2024_12_01.h below. (A generator,
// package:ffigen, exists for larger APIs — pulling it in for two
// functions would be more machinery than the thing it generates.)
//
// Run via: just days dart-demo 2024-12-01 (fetches ffi, builds the
// cdylib, runs this); or directly once the cdylib exists:
//   cd dart && dart pub get && dart run solve.dart

import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

// int aoc_2024_12_01_part1(const char *input, int32_t *out_distance);
// int aoc_2024_12_01_part2(const char *input, int32_t *out_score);
typedef _PartFnNative = Int32 Function(
    Pointer<Utf8> input, Pointer<Int32> outValue);
typedef _PartFnDart = int Function(
    Pointer<Utf8> input, Pointer<Int32> outValue);

Directory _dayDir() => File(Platform.script.toFilePath()).parent.parent;

DynamicLibrary _loadLibrary() {
  final daysDir = _dayDir().parent;
  for (final profile in ['debug', 'release']) {
    // A cdylib takes the host's name, not Rust's choice: libaoc_2024_12_01.so on
    // Linux, libaoc_2024_12_01.dylib on macOS, aoc_2024_12_01.dll (no lib prefix) on
    // Windows. Same three-name search as python/solve.py — no platform
    // check, whichever file cargo produced is the one that exists.
    for (final name in ['libaoc_2024_12_01.so', 'libaoc_2024_12_01.dylib', 'aoc_2024_12_01.dll']) {
      final path = '${daysDir.path}/target/$profile/$name';
      if (File(path).existsSync()) {
        return DynamicLibrary.open(path);
      }
    }
  }
  stderr.writeln(
      'no libaoc_2024_12_01.{so,dylib} / aoc_2024_12_01.dll found — run: cd days && cargo build -p aoc-2024-12-01 --lib');
  exit(1);
}

/// Runs one of the C API's out-param/status-code functions. A nonzero
/// status is an error the C side already classified — report it and exit
/// nonzero rather than printing a phantom answer.
int _call(_PartFnDart fn, String name, String text) {
  final inputPtr = text.toNativeUtf8();
  final outValue = calloc<Int32>();
  int status;
  int value;
  try {
    status = fn(inputPtr, outValue);
    value = outValue.value;
  } finally {
    calloc.free(inputPtr);
    calloc.free(outValue);
  }

  if (status != 0) {
    stderr.writeln('$name failed with status $status '
        '(-1: input was null, not valid UTF-8, or not two integers per '
        'line; -2: a total overflowed an int32_t)');
    exit(1);
  }
  return value;
}

void main() {
  final lib = _loadLibrary();
  final part1 =
      lib.lookupFunction<_PartFnNative, _PartFnDart>('aoc_2024_12_01_part1');
  final part2 =
      lib.lookupFunction<_PartFnNative, _PartFnDart>('aoc_2024_12_01_part2');

  final inputFile = File('${_dayDir().parent.parent.path}/inputs/2024-12-01.txt');
  if (!inputFile.existsSync()) {
    stderr.writeln(
        'no puzzle input at ${inputFile.path} (see .gitignore)');
    exit(1);
  }
  final text = inputFile.readAsStringSync();

  print('Part 1 🎯(🦀): ${_call(part1, 'part1', text)}');
  print('Part 2 🎯(🦀): ${_call(part2, 'part2', text)}');
}
