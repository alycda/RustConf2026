"""`aoc_day()` for Buck2 — the same day shape as tools/aoc.bzl, in the other
build system, so the two can be compared on identical work.

Deliberately a parallel file rather than a shared one. The two Starlark
dialects look alike and are not: Bazel wants `outs` and `$@`, Buck2 wants
`out` and `$OUT`; Bazel's rust_test recompiles a crate through
`crate = ":lib"`, Buck2's takes its own srcs and deps; Bazel calls the
feature list `crate_features`, Buck2 calls it `features`. A shared
abstraction over that would hide exactly the differences this evaluation
exists to measure.

Targets for a day `2015-12-06` with `c_api = True`:

    //days/2015-12-06:lib      rust_library
    //days/2015-12-06:test     rust_test
    //days/2015-12-06:bin      rust_binary
    //days/2015-12-06:cdylib   rust_library, preferred_linkage = "shared"
    //days/2015-12-06:header   genrule, cbindgen output
    //days/2015-12-06:c_api    cxx_library carrying header + shared object
"""

EDITION = "2024"

def _crate_ident(day):
    return "aoc_" + day.replace("-", "_")

def aoc_day(
        day,
        deps = [],
        dev_deps = [],
        c_api = False,
        crate_name = None,
        features = [],
        variant = None,
        variant_features = [],
        variant_cc_deps = [],
        cc_deps = [],
        visibility = ["PUBLIC"]):
    """See tools/aoc.bzl for the argument semantics; they match."""
    if crate_name == None:
        if len(day) != 10 or day[4] != "-" or day[7] != "-":
            fail("day must be YYYY-MM-DD, got '%s'" % day)
        ident = _crate_ident(day)
    else:
        ident = crate_name

    lib_srcs = native.glob(["src/**/*.rs"], exclude = ["src/main.rs"])

    native.rust_library(
        name = "lib",
        srcs = lib_srcs + ["Cargo.toml"],
        crate = ident,
        crate_root = "src/lib.rs",
        edition = EDITION,
        env = {"CARGO_MANIFEST_DIR": "."},
        features = features,
        visibility = visibility,
        deps = deps + cc_deps,
    )

    # Buck2 has no `crate = ":lib"` equivalent: a rust_test is its own
    # compilation from the same sources, so the dependency list is restated
    # rather than inherited. That is more duplication than the Bazel macro
    # needs, and it is also why the feature list has to be repeated here —
    # the same trap that cost 4 silently-missing tests on the Bazel side.
    # Cargo.toml in srcs, and CARGO_MANIFEST_DIR pointed at it, for the same
    # reason the Bazel macro needs compile_data: rstest's proc macro opens
    # the manifest at expansion time. The two build systems fail differently
    # — Bazel sets the variable but sandboxes the file away, Buck2 does not
    # set the variable at all — and neither is discoverable except by
    # hitting it.
    native.rust_test(
        name = "test",
        srcs = lib_srcs + ["Cargo.toml"],
        crate = ident,
        crate_root = "src/lib.rs",
        edition = EDITION,
        env = {"CARGO_MANIFEST_DIR": "."},
        features = features,
        deps = deps + dev_deps + cc_deps,
    )

    if variant:
        native.rust_test(
            name = "test_" + variant,
            srcs = lib_srcs + ["Cargo.toml"],
            crate = ident,
            crate_root = "src/lib.rs",
            edition = EDITION,
            env = {"CARGO_MANIFEST_DIR": "."},
            features = features + variant_features,
            deps = deps + dev_deps + cc_deps + variant_cc_deps,
        )

    if native.glob(["src/main.rs"]):
        # rules_rust sets CARGO_MANIFEST_DIR for every Rust compile; buck2
        # sets nothing, so every day's main.rs — which builds its input path
        # with concat!(env!("CARGO_MANIFEST_DIR"), "/../../inputs/…") — fails
        # to COMPILE here, not merely to find its file. Supplying a value is
        # enough to build.
        #
        # Neither overlay makes that path resolve correctly at runtime: the
        # value is a build-tree path under Bazel and a buck-out path here,
        # and `../../inputs` from either lands nowhere. Making `:bin`
        # actually read inputs/<day>.txt needs main.rs to stop deriving the
        # path at compile time — a first-party source change, and out of
        # scope for an evaluation that promised not to touch the workshop.
        native.rust_binary(
            name = "bin",
            srcs = ["src/main.rs"] + lib_srcs + ["Cargo.toml"],
            crate_root = "src/main.rs",
            edition = EDITION,
            env = {"CARGO_MANIFEST_DIR": "."},
            visibility = visibility,
            deps = deps + cc_deps,
        )

    if not c_api:
        return

    native.rust_library(
        name = "cdylib",
        srcs = lib_srcs,
        crate = ident,
        crate_root = "src/lib.rs",
        edition = EDITION,
        features = features,
        # The cdylib, in Buck2's vocabulary. There is no crate-type list
        # here the way Cargo.toml carries `crate-type = ["rlib", "cdylib"]`;
        # linkage is a property of how the target is consumed.
        preferred_linkage = "any",
        visibility = visibility,
        deps = deps + cc_deps,
    )

    # Same aim as the Bazel genrule and for the same documented reason:
    # src/c_api.rs, not the crate root, so a day that also imports a C
    # library does not get somebody else's API redeclared into its header.
    # `srcs` is again the enforcement — cbindgen may read these two files.
    native.genrule(
        name = "header",
        srcs = [
            "src/c_api.rs",
            "cbindgen.toml",
        ],
        out = "%s.h" % ident,
        cmd = " ".join([
            "$(exe //third-party/rust:cbindgen-cbindgen)",
            "--config $SRCDIR/cbindgen.toml",
            "--output $OUT",
            "$SRCDIR/src/c_api.rs",
        ]),
        visibility = visibility,
    )

    # header_namespace = "" is required, and is a real difference from
    # Bazel. cc_library's `includes = ["include"]` puts the generated header
    # on the include path under its own name; Buck2 instead namespaces
    # exported headers by package path by default, so without this the C
    # source would have to say
    # `#include "days/2015-12-06/aoc_2015_12_06.h"` — which is not what
    # cbindgen's include_guard, or any of the repo's existing tracks, expect.
    # `:cdylib` alone is NOT consumable from C. A buck2 rust_library hands a
    # native consumer its Rust-lang output — the default output of that
    # target is a .rmeta — and the actual shared object lives behind the
    # `[cdylib]` SUBTARGET. Depending on the target instead of the subtarget
    # links cleanly and then fails with `undefined symbol:
    # aoc_2015_12_06_part1`, which is a confusing way to learn this.
    #
    # prebuilt_cxx_library is what turns that subtarget back into something
    # cxx rules understand. Bazel needed none of this: rules_rust's
    # rust_shared_library provides CcInfo directly, so `deps = [":cdylib"]`
    # on a cc_library was the whole story.
    # `shared_lib = ":cdylib[cdylib]"` links fine and then dies at startup
    # with `libdays_2015-12-06_cdylib.so: cannot open shared object file` —
    # buck2 puts the .so in buck-out and gives the test binary no RUNPATH to
    # find it. That is the same -L-is-not-rpath lesson days/justfile spends
    # twenty lines on, arriving from a third direction.
    #
    # The staticlib subtarget sidesteps loading altogether, which is the
    # right answer for a test harness. Bazel's cc_test needed none of this:
    # rules_rust's shared library arrived through CcInfo and Bazel's runfiles
    # put it where the binary could find it.
    native.prebuilt_cxx_library(
        name = "cdylib_native",
        shared_lib = ":cdylib[cdylib]",
        preferred_linkage = "shared",
        visibility = visibility,
    )

    # And then the loader still cannot find it. Linking against the shared
    # library resolves every symbol and produces a binary that dies at
    # startup with `cannot open shared object file`, because buck2 gives a
    # cxx_test no RUNPATH into buck-out and no runfiles tree.
    #
    # So the consumer has to be handed a directory to search — which is the
    # `-rpath` half of the lesson days/justfile documents at length, except
    # that here it is spelled LD_LIBRARY_PATH and is the harness's problem
    # rather than the linker's. Bazel's runfiles handled this invisibly.
    native.genrule(
        name = "cdylib_dir",
        out = "libdir",
        cmd = "mkdir -p $OUT && cp $(location :cdylib[cdylib]) $OUT/",
        visibility = visibility,
    )

    native.cxx_library(
        name = "c_api",
        exported_headers = {"%s.h" % ident: ":header"},
        header_namespace = "",
        visibility = visibility,
        exported_deps = [":cdylib_native"],
    )
