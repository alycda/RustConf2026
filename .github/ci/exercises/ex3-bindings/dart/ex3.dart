// Exercise 3, Dart track (dart:ffi) — solved. The CI overlay for the
// attendee scaffold (see .github/ci/README.md): the same file with its TODOs
// filled, run the way the scaffold says:  dart pub get && dart run ex3.dart

import 'dart:ffi' as ffi;
import 'dart:io' show File, Platform, exit, stderr;
import 'package:ffi/ffi.dart';

// TODO 1, done: one typedef describing the C signature, one describing the
// Dart view of it. This duplication IS the binding.
typedef ExPartNative = ffi.Int64 Function(ffi.Pointer<Utf8>);
typedef ExPartDart = int Function(ffi.Pointer<Utf8>);

// As shipped: one cargo workspace, three filenames, no platform check.
String _libraryPath() {
  final exercises = File(Platform.script.toFilePath()).parent.parent.parent;
  for (final profile in ['debug', 'release']) {
    for (final name in ['libex2_c_glue.so', 'libex2_c_glue.dylib', 'ex2_c_glue.dll']) {
      final path = '${exercises.path}/target/$profile/$name';
      if (File(path).existsSync()) return path;
    }
  }
  stderr.writeln('no Ex 2 library found — run ../../ex2-c-glue/build-and-test.sh first');
  exit(1);
}

// TODO 3, done: Dart String → C string is manual. toNativeUtf8() allocates
// (NUL-terminated, UTF-8), and the caller frees it — in a finally, so a
// throw on the way out does not leak.
int _call(ExPartDart fn, String text) {
  final input = text.toNativeUtf8();
  try {
    return fn(input);
  } finally {
    calloc.free(input);
  }
}

void main() {
  // TODO 2, done: open the library and look up both exports.
  final lib = ffi.DynamicLibrary.open(_libraryPath());
  final part1 = lib.lookupFunction<ExPartNative, ExPartDart>('ex_part1');
  final part2 = lib.lookupFunction<ExPartNative, ExPartDart>('ex_part2');

  final example = [
    '987654321111111',
    '811111111111119',
    '234234234234278',
    '818181911112111',
  ].join('\n');
  const expectedPart1 = 357;
  const expectedPart2 = 3121910778619; // above 32 bits, on purpose

  final got1 = _call(part1, example);
  if (got1 != expectedPart1) {
    print('part1 = $got1, expected $expectedPart1');
    exit(1);
  }
  final got2 = _call(part2, example);
  if (got2 != expectedPart2) {
    print('part2 = $got2, expected $expectedPart2');
    exit(1);
  }

  // The hostile-input contract from the Dart side: nullptr is the null
  // pointer, and the C side answers in band.
  if (part1(ffi.nullptr) != -1) {
    print('null input was not refused');
    exit(1);
  }

  print('Ex 3 (Dart) passed.');
}
