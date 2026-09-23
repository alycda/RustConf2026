#!/usr/bin/env Rscript
# The R track's generated lap: the same puzzle through extendr
# (`src/extendr.rs`, cargo feature `extendr`) instead of through `.C()`.
#
# Read `solve.R` first — this script only makes sense as its counterweight.
# There, `.C()` discards the C function's return value, so a failure is
# detectable only as "the out-parameter never changed" and the status code
# the C API classified it with is gone. Here the boundary is R's other interface,
# `.Call()`: SEXPs both ways, and `#[extendr]` generates the wrapper. A
# Rust `Result<i32, String>` arrives as an R error *condition*, which is a
# thing R already knows how to handle — `tryCatch`, a message, a class.
#
# Still no header, and still no package: this calls the generated
# `wrap__<name>` symbols directly rather than building an R package around
# the metadata extendr also emits, which keeps the comparison to one file
# against one file.
#
# Run via: just days r-extendr-demo 2015-12-01 (builds with the feature —
# needs R installed, since extendr links libR at build time).

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

# Same three-name search as solve.R — same cdylib, in fact; the extendr
# wrappers are extra exported symbols in it, not a second library.
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
    "no cdylib found — run: cd days && cargo build -p aoc-2015-12-01 --lib --features extendr",
    call. = FALSE
  )
}

lib <- load_library()

# `#[extendr] fn part1` exports `wrap__part1`. If this lookup fails, the
# cdylib was built without the feature.
wrapper <- function(name) {
  symbol <- tryCatch(
    getNativeSymbolInfo(paste0("wrap__", name), PACKAGE = lib),
    error = function(e) {
      stop(
        "no wrap__", name, " in the cdylib — it was built without the extendr ",
        "feature. Run: cd days && cargo build -p aoc-2015-12-01 --lib --features extendr",
        call. = FALSE
      )
    }
  )
  function(...) .Call(symbol, ...)
}

part1 <- wrapper("part1")
part2 <- wrapper("part2")
boundary_panic <- wrapper("boundary_panic")

input_path <- file.path(REPO_ROOT, "inputs", "2015-12-01.txt")
if (!file.exists(input_path)) {
  stop("no puzzle input at ", input_path, " (see .gitignore)", call. = FALSE)
}
# A character vector this time, not a raw one with a NUL bolted on: .Call()
# hands over the SEXP itself, and extendr reads the string out of it. The
# NUL-appending in solve.R was .C()'s price, not R's.
text <- readChar(input_path, file.size(input_path), useBytes = TRUE)

cat(sprintf("Part 1 📊(🦀): %d\n", part1(text)))

# Part 2 is the one that can fail — Santa may never reach the basement, the
# -2 the C API answers with and .C() throws away. Through extendr it is an
# error condition, so the caller can say WHICH failure it was.
#
# What the generator hid, visible here if you watch stderr: extendr 0.9.0
# implements the Err arm by *panicking* with the message (the trace names
# extendr-api's own into_robj.rs) and turning that into the R error. The
# condition R receives is clean; the route it took to get here is a Rust
# panic, on an ordinary domain error the Rust side handled as data. Every
# generated boundary has a seam like this one; the value of doing the raw
# route first is knowing where to look for it.
answer <- tryCatch(
  part2(text),
  error = function(e) {
    cat("Part 2 📊(🦀): no answer —", conditionMessage(e), "\n")
    NULL
  }
)
if (!is.null(answer)) {
  cat(sprintf("Part 2 📊(🦀): %d\n", answer))
}

# The panic exhibit. Exercise 2's C API cannot let a panic out at all (it is
# UB across `extern "C"`, so c_api.rs is written so nothing can panic) and
# the wasm track's answer is a trap that takes the instance with it. Here
# extendr's generated wrapper catches the unwind and raises an ordinary R
# error: catchable, classed, and survivable. The stderr noise above this
# line is Rust's own panic handler printing on the way past — the condition
# is the part R can act on.
caught <- tryCatch(
  boundary_panic(),
  error = function(e) sprintf("%s: %s", class(e)[1L], conditionMessage(e))
)
cat("Panic across the boundary:", caught, "\n")
cat("...and the R session is still running.\n")
