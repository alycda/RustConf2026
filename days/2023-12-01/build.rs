// Links against this day's C library — YARA, the malware-scanning engine it
// runs the calibration scan through (src/yara.rs) — via pkg-config rather
// than system-wide installs or hardcoded paths.
//
// nixpkgs ships `yara.pc`, so unlike 2021-12-02's chipmunk and duckdb there is
// nothing for shell.nix to synthesize; the probe below is the same three lines
// every C variant in this repo uses.
//
// The library is probed only when its cargo feature is enabled (cargo exposes
// enabled features to build scripts as CARGO_FEATURE_* env vars), so the
// default build needs neither pkg-config nor YARA — that is what keeps stock
// CI runners and the manual-setup path green, and it is the rule two earlier
// days shipped red before adopting.
//
// This day's build script does one thing the others don't: it compiles a C
// file. YARA's match-reading API is macros over its internal structs, which no
// FFI can call, so `yara_shim.c` does that walking and hands Rust flat
// integers — see the comment at the top of that file. Compiling it with a
// plain `Command` rather than the `cc` crate is deliberate: this build script
// already shells out to pkg-config, the workshop is about doing FFI by hand,
// and a build-dependency that has to be vendored for the offline path is a
// cost the six lines below do not carry.
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=yara_shim.c");

    // Naming files above turns off cargo's default "rerun if any tracked file
    // changed" and replaces it with exactly what is listed — so the
    // environment pkg-config resolves against has to be listed too, or cargo
    // will happily replay link flags pointing at a store path that no longer
    // exists. PATH steers which pkg-config runs, PKG_CONFIG overrides it
    // outright, CC steers the compiler, and the rest steer what pkg-config
    // finds.
    for var in [
        "PATH",
        "CC",
        "PKG_CONFIG",
        "PKG_CONFIG_PATH",
        "PKG_CONFIG_LIBDIR",
        "PKG_CONFIG_SYSROOT_DIR",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    if std::env::var_os("CARGO_FEATURE_YARA").is_none() {
        return;
    }

    let cflags = pkg_config(&["--cflags", "yara"]);
    let libs = pkg_config(&["--libs", "yara"]);

    build_shim(&cflags);

    for flag in libs.split_whitespace() {
        println!("cargo:rustc-link-arg={flag}");
    }
}

fn pkg_config(args: &[&str]) -> String {
    let output = Command::new("pkg-config")
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run pkg-config: {e}"));

    if !output.status.success() {
        panic!(
            "pkg-config could not find yara.pc ({}). Run inside the project's nix shell \
             (see shell.nix), which provides it.",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Compiles `yara_shim.c` into a static archive and links it.
///
/// Static, not a `cdylib`: this object is an implementation detail of the
/// `yara` feature and has no business being a file anyone could load. It also
/// means the Swift track's `cdylib` — which is built without this feature —
/// never acquires a symbol it has no header for.
fn build_shim(cflags: &str) {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let object = out_dir.join("yara_shim.o");
    let archive = out_dir.join("libaoc_yara_shim.a");

    // `cc` if the environment names one (cross-compiling, or a nix stdenv that
    // points it at its own wrapper), otherwise the plain name — the same
    // fallback order scripts/self-check.sh uses when it probes for a compiler.
    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());

    let status = Command::new(&compiler)
        .args(cflags.split_whitespace())
        .args(["-O2", "-fPIC", "-c"])
        .arg("yara_shim.c")
        .arg("-o")
        .arg(&object)
        .status()
        .unwrap_or_else(|e| panic!("failed to run {compiler}: {e}"));
    assert!(status.success(), "{compiler} failed to compile yara_shim.c");

    // `ar` rather than linking the object directly: cargo's link-arg ordering
    // puts our arguments before the Rust objects, and a bare .o there is
    // dropped by some linkers as unreferenced. An archive member is pulled in
    // by the symbol that needs it, whatever the order.
    let ar = std::env::var("AR").unwrap_or_else(|_| "ar".to_string());
    let status = Command::new(&ar)
        .arg("rcs")
        .arg(&archive)
        .arg(&object)
        .status()
        .unwrap_or_else(|e| panic!("failed to run {ar}: {e}"));
    assert!(status.success(), "{ar} failed to archive yara_shim.o");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=aoc_yara_shim");
}
