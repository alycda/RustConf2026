// Links against this day's two C libraries — Chipmunk2D, the rigid-body
// physics engine it dead-reckons the submarine through (src/chipmunk.rs), and
// DuckDB, the analytical database it folds the course in (src/duckdb.rs) — via
// pkg-config rather than system-wide installs or hardcoded paths.
//
// Neither is shipped with a .pc file by nixpkgs, which is unusual; every other
// C library this repo has linked came with one. shell.nix synthesizes both and
// lets pkg-config's setup hook put them on PKG_CONFIG_PATH, so the loop below
// stays the same three lines every C variant in this repo uses, for both
// libraries and for whatever comes next. Fixing the gap on the nix side rather
// than here is deliberate: a build script that knows about include paths is a
// build script that has to be re-taught them on every machine.
//
// Each library is probed only when its cargo feature is enabled (cargo exposes
// enabled features to build scripts as CARGO_FEATURE_* env vars), so the
// default build needs neither pkg-config nor either library — that's what
// keeps stock CI runners and the manual-setup path green.
use std::process::Command;

fn main() {
    let variants = [
        ("CARGO_FEATURE_CHIPMUNK", "chipmunk"),
        ("CARGO_FEATURE_DUCKDB", "duckdb"),
    ];

    for (feature, name) in variants {
        if std::env::var_os(feature).is_none() {
            continue;
        }

        let libs = Command::new("pkg-config")
            .args(["--libs", name])
            .output()
            .unwrap_or_else(|e| panic!("failed to run pkg-config: {e}"));

        if !libs.status.success() {
            panic!(
                "pkg-config could not find {name}.pc ({}). Run inside the project's nix shell \
                 (see shell.nix), which synthesizes it — nixpkgs ships neither this day's \
                 chipmunk.pc nor its duckdb.pc.",
                String::from_utf8_lossy(&libs.stderr).trim()
            );
        }

        for flag in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
            println!("cargo:rustc-link-arg={flag}");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");

    // Naming build.rs above turns off cargo's default "rerun if any tracked
    // file changed" and replaces it with exactly what is listed — so the
    // environment pkg-config resolves against has to be listed too, or cargo
    // will happily replay link flags pointing at a store path that no longer
    // exists. PATH steers which pkg-config runs, PKG_CONFIG overrides it
    // outright, and the other three steer what it finds.
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
