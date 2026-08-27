// Compiles the uthash wrapper (src/uthash_wrapper.c, see src/uthash.rs) —
// but only when the `uthash` feature is enabled (cargo exposes enabled
// features to build scripts as CARGO_FEATURE_* env vars), so the default
// build compiles no C and needs neither a C toolchain nor the header:
// that's what keeps stock CI runners and the manual-setup path green.
//
// Unlike the tcc/caca days there is no pkg-config probe because there is
// nothing to probe: nixpkgs' uthash ships headers only, no .pc file and no
// library to link. Inside the project's nix shell the cc wrapper injects
// the include path for every buildInputs entry (shell.nix carries uthash),
// so the plain compile below finds <uthash.h> with no flags; off nix, put
// uthash.h on the include path via CFLAGS=-I<dir>, which the cc crate
// honors.
//
// The cc crate is deliberately a plain (not optional) build-dependency: an
// optional one couldn't be named from this script at all when disabled —
// build scripts get features as env vars, not as cfg. It's pure Rust and
// cheap to compile, and invoked only inside the gate.
fn main() {
    if std::env::var_os("CARGO_FEATURE_UTHASH").is_some() {
        cc::Build::new()
            // Always optimized, even in dev profiles: hardened glibc
            // (nix's included) warns `_FORTIFY_SOURCE requires compiling
            // with optimization` at -O0.
            .opt_level(2)
            .file("src/uthash_wrapper.c")
            .include("src")
            .compile("uthash_wrapper");
        println!("cargo:rerun-if-changed=src/uthash_wrapper.c");
        println!("cargo:rerun-if-changed=src/uthash_wrapper.h");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
