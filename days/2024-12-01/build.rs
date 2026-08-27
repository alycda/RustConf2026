// Compiles the C++ std::sort shim (src/cpp_sort.cpp, see src/cpp.rs) — but
// only when the `cpp` feature is enabled (cargo exposes enabled features to
// build scripts as CARGO_FEATURE_* env vars), so the default build compiles
// no C++ and needs no C++ toolchain: that's what keeps stock CI runners and
// the manual-setup path green.
//
// The cc crate is deliberately a plain (not optional) build-dependency: an
// optional one couldn't be named from this script at all when disabled —
// build scripts get features as env vars, not as cfg. It's pure Rust, cheap
// to compile, and invoked only inside the gate; it picks the right C++
// compiler per platform (c++/clang++/MSVC) instead of a hand-rolled g++
// command, and links the matching C++ standard library.
fn main() {
    if std::env::var_os("CARGO_FEATURE_CPP").is_some() {
        cc::Build::new()
            .cpp(true)
            .std("c++17")
            // Always optimized, even in dev profiles: hardened glibc (nix's
            // included) warns `_FORTIFY_SOURCE requires compiling with
            // optimization` at -O0. The talk's build.rs hit the same wall
            // and landed the same -O2.
            .opt_level(2)
            .file("src/cpp_sort.cpp")
            .compile("cpp_sort");
        println!("cargo:rerun-if-changed=src/cpp_sort.cpp");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
