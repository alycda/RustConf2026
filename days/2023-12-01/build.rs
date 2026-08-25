// Links against this day's C library — espeak-ng, the speech synthesiser it
// hears the calibration digits through (src/espeak.rs) — via pkg-config
// rather than system-wide installs or hardcoded paths.
//
// nixpkgs ships espeak-ng.pc, so unlike 2021-12-02's chipmunk and duckdb
// there is nothing for shell.nix to synthesize; the probe below is the same
// three lines every C variant in this repo uses.
//
// The library is probed only when its cargo feature is enabled (cargo exposes
// enabled features to build scripts as CARGO_FEATURE_* env vars), so the
// default build needs neither pkg-config nor espeak-ng — that is what keeps
// stock CI runners and the manual-setup path green, and it is the rule two
// earlier days shipped red before adopting.
use std::process::Command;

fn main() {
    let variants = [("CARGO_FEATURE_ESPEAK", "espeak-ng")];

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
                 (see shell.nix), which provides it.",
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
