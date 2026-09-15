// Compiles the day's C and C++ sides — each only when its feature is
// enabled (cargo exposes enabled features to build scripts as
// CARGO_FEATURE_* env vars), so the default build compiles nothing and
// needs no C or C++ toolchain: that's what keeps stock CI runners and the
// manual-setup path green.
//
// Two of the three gates below need no pkg-config probe, unlike the tcc/caca
// days, because neither side has anything to probe: the C++ shim is this
// crate's own source, and nixpkgs' uthash ships headers only — no .pc file
// and no library to link. The `lapack` gate is the exception and the only
// real link on this day; its own comment says why it probes and why it does
// not panic when the probe comes up empty. Inside
// the project's nix shell the cc wrapper injects the include path for
// every buildInputs entry, so the plain compiles below find <uthash.h>
// with no flags — but uthash is in shell.nix's `full` list, not the shell
// attendees get by default, so that means `nix-shell --arg full true`. Off
// nix, put it on the include path via CFLAGS=-I<dir>, which the cc crate
// honors.
//
// The cc crate is deliberately a plain (not optional) build-dependency: an
// optional one couldn't be named from this script at all when disabled —
// build scripts get features as env vars, not as cfg. It's pure Rust and
// cheap to compile, invoked only inside the gates; it picks the right
// compiler per platform (cc/c++/clang/MSVC) and links the matching C++
// standard library for the shim.
//
// Both invocations force -O2 even in dev profiles: hardened glibc (nix's
// included) warns `_FORTIFY_SOURCE requires compiling with optimization`
// at -O0. The talk's build.rs hit the same wall and landed the same -O2.
use std::process::Command;

fn main() {
    if std::env::var_os("CARGO_FEATURE_CPP").is_some() {
        cc::Build::new()
            .cpp(true)
            .std("c++17")
            .opt_level(2)
            .file("src/cpp_sort.cpp")
            .compile("cpp_sort");
        println!("cargo:rerun-if-changed=src/cpp_sort.cpp");
    }

    if std::env::var_os("CARGO_FEATURE_UTHASH").is_some() {
        cc::Build::new()
            .opt_level(2)
            .file("src/uthash_wrapper.c")
            .include("src")
            .compile("uthash_wrapper");
        println!("cargo:rerun-if-changed=src/uthash_wrapper.c");
        println!("cargo:rerun-if-changed=src/uthash_wrapper.h");
    }

    // LAPACK's DLASRT (src/lapack.rs) is the one variant on this day that
    // links a library instead of compiling a source file, so it is the one
    // with anything to discover. The probe is the same shape
    // days/2021-12-02/build.rs uses for chipmunk and duckdb — ask pkg-config
    // for the link flags, forward them verbatim — with one deliberate
    // difference: a failed probe is not fatal here.
    //
    // That day panics because its two libraries are reachable *only* through
    // the .pc files shell.nix synthesizes; off nix there is nothing to find,
    // and saying so beats a linker error. LAPACK is the opposite kind of
    // dependency. nixpkgs ships lapack.pc (in lapack's dev output, so
    // `lapack` in shell.nix's `full` list is the whole setup), but every
    // distro also drops liblapack.so where the linker already looks, and
    // macOS resolves `dlasrt_` out of the Accelerate framework — Accelerate
    // exports the LAPACK routines under their Fortran symbol names, which is
    // why the Rust side needs no `-framework Accelerate` special case
    // [unverified: no mac here to run it on]. So "pkg-config had nothing to
    // say" is not the same as "the library is missing", and the honest
    // fallback is to ask for -llapack and let the linker be the one to
    // complain if it really is absent.
    if std::env::var_os("CARGO_FEATURE_LAPACK").is_some() {
        let probe = Command::new("pkg-config")
            .args(["--libs", "lapack"])
            .output();

        match probe {
            Ok(libs) if libs.status.success() => {
                for flag in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
                    println!("cargo:rustc-link-arg={flag}");
                }
            }
            // Either pkg-config is not installed or it does not know this
            // library. Both land on the same default, which is what a plain
            // `cc -llapack` would have done.
            _ => println!("cargo:rustc-link-lib=lapack"),
        }
    }

    println!("cargo:rerun-if-changed=build.rs");

    // Naming build.rs above turns off cargo's default "rerun if any tracked
    // file changed" and replaces it with exactly what is listed — so the
    // environment the probe resolves against has to be listed too, or cargo
    // will replay link flags pointing at a store path that no longer exists.
    // PATH steers which pkg-config runs, PKG_CONFIG overrides it outright,
    // and the other three steer what it finds. 2021-12-02 learned this first;
    // the list is repeated here rather than shared because the failure it
    // prevents is silent and the fix is five lines.
    for var in [
        "PATH",
        "PKG_CONFIG",
        "PKG_CONFIG_PATH",
        "PKG_CONFIG_LIBDIR",
        "PKG_CONFIG_SYSROOT_DIR",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }
}
