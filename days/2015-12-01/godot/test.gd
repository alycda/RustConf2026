# The Godot track's consumer: GDScript calling Rust across a GDExtension.
#
# Run it: `just days godot-demo 2015-12-01` (builds the cdylib with the
# `godot` feature, copies it where aoc.gdextension says, runs this headless).
# Or directly, from this directory, once the library is in lib/:
#
#     godot --headless -s test.gd          # `godot4` on distros that rename it
#
# It is both the test and the demo. The assertions below are the statement
# examples days/2015-12-01/src/lib.rs asserts on, so a green run means
# GDScript got the same numbers Rust did; then, only if you have dropped your
# own input at <repo>/inputs/2015-12-01.txt, it prints the day's answers in
# the repo's usual shape. CI runs it without one.
#
# `extends SceneTree` is what `-s` wants: the script *is* the main loop, so
# there is no scene, no node and no .tscn anywhere in this project. A
# RefCounted class needs none of that.

extends SceneTree

var failures := 0


# Not assert(). GDScript's assert() is compiled out of release engine builds,
# and the binary from godotengine.org is a release build — an assertion that
# disappears exactly where CI runs is not an assertion. This one is an if.
func check(condition: bool, what: String) -> void:
	if condition:
		print("  ok   %s" % what)
	else:
		failures += 1
		printerr("  FAIL %s" % what)


func check_eq(actual: Variant, expected: Variant, what: String) -> void:
	check(actual == expected, "%s (got %s, want %s)" % [what, actual, expected])


func _init() -> void:
	# If the extension failed to load, this is where you find out — every
	# other line below would fail with "Identifier not declared".
	if not ClassDB.class_exists("Aoc20151201"):
		printerr("Aoc20151201 is not registered — the GDExtension did not load.")
		printerr("Check lib/ has the cdylib aoc.gdextension names for this platform,")
		printerr("and that `cargo build --features godot` actually ran.")
		quit(1)
		return

	var day := Aoc20151201.new()

	print("part1 — the floor Santa ends up on")
	# The nine part-1 cases from src/lib.rs's rstest table, same inputs, same
	# answers. -1 and -3 are the ones worth having: a negative answer is what
	# a naive in-band error convention eats, and this boundary returns it as
	# an ordinary int.
	check_eq(day.part1("(())"), 0, "(())")
	check_eq(day.part1("()()"), 0, "()()")
	check_eq(day.part1("((("), 3, "(((")
	check_eq(day.part1("(()(()("), 3, "(()(()(")
	check_eq(day.part1("))((((("), 3, "))(((((")
	check_eq(day.part1("())"), -1, "())")
	check_eq(day.part1("))("), -1, "))(")
	check_eq(day.part1(")))"), -3, ")))")
	check_eq(day.part1(")())())"), -3, ")())())")

	print("part2 — Option, spelled nil")
	check_eq(day.part2(")"), 1, ")")
	check_eq(day.part2("()())"), 5, "()())")
	# The C API answers this one with status -2. Here the absence *is* the
	# value: GDScript has no Option, and null is where one goes.
	check_eq(day.part2("((("), null, "((( is nil, not 0")
	check(day.part2("(((") == null, "((( compares equal to null")

	print("part2_status — the C API's convention, natively")
	# An Array as the out-parameter, because GDScript has no `int *`, and
	# @GlobalScope.Error as the return — the same two-channel design
	# src/c_api.rs exports, landing in a runtime that already speaks it.
	var out: Array = []
	check_eq(day.part2_status("()())", out), OK, "status for ()()) is OK")
	check_eq(out.size(), 1, "one answer written through the out-parameter")
	check_eq(out[0], 5, "and it is 5")

	var missing: Array = []
	check_eq(day.part2_status("(((", missing), ERR_DOES_NOT_EXIST, "status for ((( is ERR_DOES_NOT_EXIST")
	check(missing.is_empty(), "nothing written on failure")

	print("utf8_len — UTF-32 on this side, UTF-8 on the other")
	# Godot's String is UTF-32: one code point per 32-bit unit, no surrogate
	# pairs, no variable width. length() counts those units; the number Rust
	# reports is the length of the UTF-8 it was transcoded into. ASCII hides
	# the difference entirely, which is why the second case is the one that
	# matters.
	check_eq("()()".length(), 4, "GDScript: ()() is 4 code points")
	check_eq(day.utf8_len("()()"), 4, "Rust: ()() is 4 UTF-8 bytes")
	check_eq("(é)".length(), 3, "GDScript: (é) is 3 code points")
	check_eq(day.utf8_len("(é)"), 4, "Rust: (é) is 4 UTF-8 bytes — é costs two")
	check_eq("(🦀)".length(), 3, "GDScript: (🦀) is 3 code points — no surrogate pair")
	check_eq(day.utf8_len("(🦀)"), 6, "Rust: (🦀) is 6 UTF-8 bytes")

	print("part2_unwrapped — the third panic semantics")
	check_eq(day.part2_unwrapped("()())"), 5, "no panic when there is an answer")
	# The next call panics inside Rust. Exercise 2's C API cannot do this at
	# all (unwinding across extern "C" is UB); wasm turns it into a trap that
	# kills the instance. gdext does a third thing: it catches the unwind and
	# the engine prints the Rust panic message as an ERROR — the one you can
	# see in the output just above this line, which is expected here and is
	# not a test failure — and then execution continues.
	print("  (an engine ERROR for the caught Rust panic should appear here)")
	var after_panic: Variant = day.part2_unwrapped("(((")
	# Deliberately printed, not asserted. gdext 0.5.5 does not write the
	# return slot when it catches a panic, so what the caller reads is
	# whatever was in that slot already: 0 from a fresh one, and — as here,
	# after the successful call above — the *previous* call's answer. It is
	# not the type's default, it is not nil, and it is not detectable from
	# GDScript. That is the actual hazard of "caught and logged", and pinning
	# a number on it in CI would be pinning an implementation detail.
	print("  note  the value after a caught panic is %s — stale, not a default" % after_panic)
	# What is worth asserting is that the engine, the extension and the
	# instance all survived it. This is the claim the track makes.
	check_eq(day.part1("(())"), 0, "the same instance still answers after a panic")
	check_eq(day.part2("((("), null, "and still returns nil correctly")

	solve_real_input(day)

	print("")
	if failures == 0:
		print("godot track: all checks passed")
		quit(0)
	else:
		printerr("godot track: %d check(s) failed" % failures)
		quit(1)


# The demo half. Puzzle inputs are never committed (see .gitignore), so this
# is skipped in CI and everywhere else that has not got one — the same rule
# every other track's script follows, just with the print at the end instead
# of the whole point.
func solve_real_input(day: Object) -> void:
	# res:// stops at the project directory, so the repo root has to be
	# reached through the real filesystem path behind it.
	var project_dir := ProjectSettings.globalize_path("res://")
	var input_path := project_dir.path_join("../../../inputs/2015-12-01.txt").simplify_path()
	if not FileAccess.file_exists(input_path):
		print("")
		print("no input at %s — skipping the real puzzle (see .gitignore)" % input_path)
		return

	var text := FileAccess.get_file_as_string(input_path)
	print("")
	print("Part 1 🎮(🦀): %d" % day.part1(text))
	var position: Variant = day.part2(text)
	if position == null:
		printerr("Part 2 🎮(🦀): nil — Santa never enters the basement")
		failures += 1
	else:
		print("Part 2 🎮(🦀): %d" % position)
