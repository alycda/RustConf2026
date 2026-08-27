// Compiles the day's C and C++ sides — each only when its feature is
// enabled (cargo exposes enabled features to build scripts as
// CARGO_FEATURE_* env vars), so the default build compiles nothing and
// needs no C or C++ toolchain: that's what keeps stock CI runners and the
// manual-setup path green.
//
// No pkg-config probes, unlike the tcc/caca days, because neither side has
// anything to probe: the C++ shim is this crate's own source, and nixpkgs'
// uthash ships headers only — no .pc file and no library to link. Inside
// the project's nix shell the cc wrapper injects the include path for
// every buildInputs entry (shell.nix carries uthash), so the plain
// compiles below find <uthash.h> with no flags; off nix, put it on the
// include path via CFLAGS=-I<dir>, which the cc crate honors.
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

    println!("cargo:rerun-if-changed=build.rs");
}
