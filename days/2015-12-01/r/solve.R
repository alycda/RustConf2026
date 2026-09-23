#!/usr/bin/env Rscript
# Exercise 3 (R track): call the Exercise 2 C API from R through `.C()`.
#
# `.C()` is R's oldest foreign-function interface, and the treaty it offers
# is unlike any other in this repo. Writing R Extensions §5.2 tabulates
# what each R vector arrives as on the C side —
#
#     integer    ->  int *
#     raw        ->  unsigned char *
#     character  ->  char **
#
# — and then says the quiet part out loud: "the compiled code should not
# return anything except through its arguments: C functions should be of
# type void."
#
# Our C API is not void. `int aoc_2015_12_01_part1(const char *, int *)`
# returns a status (0, or a negative code from the table in days/README.md),
# and `.C()` discards it. Three consequences follow,
# and each one is marked [1] [2] [3] where it shows up below:
#
#   [1] the string cannot be passed as a string,
#   [2] the status code is unobservable,
#   [3] the arguments are copied.
#
# Nothing here reads the generated header. R matches on the symbol name and
# trusts the vector modes it was handed: no declared types, no checked
# signature, no return value. That is this track's row — unread and
# unchecked, one below Kotlin, which at least keeps an interface.
#
# Run via: just days r-demo 2015-12-01 (builds the cdylib first); or
# directly once it exists: Rscript r/solve.R

# Rscript does not hand a script its own path, so dig it out of the command
# line the way python/solve.py uses __file__ — everything below is relative
# to this file, so the demo runs from any directory.
script_path <- function() {
  file <- sub("^--file=", "", grep("^--file=", commandArgs(trailingOnly = FALSE), value = TRUE))
  if (length(file) == 0L) {
    stop("run this with Rscript (it needs --file to locate the repo)", call. = FALSE)
  }
  normalizePath(file[1L])
}

DAY_DIR <- dirname(dirname(script_path()))
DAYS_DIR <- dirname(DAY_DIR)
REPO_ROOT <- dirname(DAYS_DIR)

# A cdylib takes the host's name, not Rust's choice: libaoc_2015_12_01.so on
# Linux, libaoc_2015_12_01.dylib on macOS, aoc_2015_12_01.dll (no lib prefix)
# on Windows. Searching the three filenames needs no platform check —
# whichever one cargo produced is the one that exists. Same search, same
# order (debug before release) as python/solve.py's.
load_library <- function() {
  for (profile in c("debug", "release")) {
    for (name in c("libaoc_2015_12_01.so", "libaoc_2015_12_01.dylib", "aoc_2015_12_01.dll")) {
      candidate <- file.path(DAYS_DIR, "target", profile, name)
      if (file.exists(candidate)) {
        return(dyn.load(candidate))
      }
    }
  }
  stop(
    "no libaoc_2015_12_01.{so,dylib} / aoc_2015_12_01.dll found — run: ",
    "cd days && cargo build -p aoc-2015-12-01 --lib",
    call. = FALSE
  )
}

# One call across the boundary, with all three consequences visible.
call_part <- function(lib, name, text) {
  # The symbol is looked up in the DLL we loaded, by name and by name only.
  # `.C(name, ...)` would work too — R would search every loaded DLL — but
  # naming the library is the honest spelling of what is actually being
  # trusted here, since nothing else about this call is checked.
  symbol <- getNativeSymbolInfo(name, PACKAGE = lib)

  # [3] `.C()` duplicates every argument before the call. The pointer the
  # Rust side writes through is to R's *copy*, so this variable is not the
  # one that gets written; the answer comes back in the returned list. It is
  # asserted below rather than only described.
  out <- NA_integer_

  result <- .C(
    symbol,
    # [1] The string cannot be passed as a string. A `character` vector
    # arrives as `char **` — a pointer to pointers — and the C API takes
    # `const char *`, so Rust would read a pointer as text. A `raw` vector
    # arrives as `unsigned char *`, byte for byte what the C side expects.
    # `charToRaw()` gives the bytes; the terminator R's strings do not carry
    # is appended by hand. (readBin(path, "raw") would skip the character
    # round trip; the round trip is kept because the vector *mode* is the
    # whole lesson here.)
    input = c(charToRaw(text), as.raw(0)),
    # The out-parameter: one `integer`, which is one `int *`.
    out = out,
    # NA_integer_ *is* INT_MIN once it crosses, and .C() refuses to pass an
    # NA without being told it is deliberate. Deliberate: see below.
    NAOK = TRUE
  )

  # [3] again, this time as an assertion: the local `out` is still NA after a
  # call that wrote an answer. Who allocates is the runtime, twice — R made
  # the copy and R owns it.
  stopifnot(is.na(out))

  # [2] The status code is unobservable. `.C()` called the function for its
  # side effects on the arguments and handed back the modified argument
  # list; the status code that C API was designed around went nowhere R can
  # look. The out-parameter is the only channel left, so the sentinel has to
  # be a value no real answer can be — and the plan's -999 is not one: part 1
  # is a floor, and a long enough input genuinely ends on -999. NA_integer_
  # is INT_MIN, which is not a floor this puzzle can reach and not a 1-based
  # position at all. Unchanged means the call failed; which failure it was
  # (bad input, Santa never reaching the basement, overflow, a caught panic)
  # is exactly what this interface cannot tell us. The C side promises to
  # leave the out-parameter alone on every one of them (days/README.md,
  # "C API status codes"), which is the only reason this check works.
  if (is.na(result$out)) {
    stop(
      name, ": no answer arrived — .C() discards the return value, so the ",
      "status code (-1 bad input, -2 no answer, -3 overflow, ",
      "-4 internal error; see days/README.md) went with it",
      call. = FALSE
    )
  }

  result$out
}

main <- function() {
  lib <- load_library()

  input_path <- file.path(REPO_ROOT, "inputs", "2015-12-01.txt")
  if (!file.exists(input_path)) {
    stop("no puzzle input at ", input_path, " (see .gitignore)", call. = FALSE)
  }
  text <- readChar(input_path, file.size(input_path), useBytes = TRUE)

  # The labels carry literal UTF-8 bytes, not "\\U0001F4CA" escapes. R renders
  # a \\U escape according to the *locale*: in a UTF-8 one it prints the emoji,
  # and in the C locale it prints the six-character text "<U+0001F4CA>" — so
  # the escape form makes the label depend on the environment, and CI matches
  # this line byte for byte. Written as bytes, it passes through either locale
  # unchanged (verified both ways).
  cat(sprintf("Part 1 📊(🦀): %d\n", call_part(lib, "aoc_2015_12_01_part1", text)))
  cat(sprintf("Part 2 📊(🦀): %d\n", call_part(lib, "aoc_2015_12_01_part2", text)))
}

main()
