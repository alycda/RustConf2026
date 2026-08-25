// Links against DuckDB (the in-process analytical database this day folds the
// course in, see src/duckdb.rs) via pkg-config rather than a system-wide
// install or a hardcoded path.
//
// nixpkgs' `duckdb` ships no .pc file of its own — shell.nix synthesizes
// `duckdb.pc` next to it and lets pkg-config's setup hook put it on
// PKG_CONFIG_PATH, so the probe below is the same three lines any other C
// variant would use. Fixing the gap on the nix side rather than here is
// deliberate: a build script that knows about include paths is a build script
// that has to be re-taught them on every machine.
//
// The probe runs only when the cargo feature is enabled (cargo exposes
// enabled features to build scripts as CARGO_FEATURE_* env vars), so the
// default build needs neither pkg-config nor the library — that's what keeps
// stock CI runners and the manual-setup path green.
use std::process::Command;

fn main() {
    let variants = [("CARGO_FEATURE_DUCKDB", "duckdb")];

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
                 (see shell.nix), which synthesizes it — nixpkgs' duckdb does not ship one.",
                String::from_utf8_lossy(&libs.stderr).trim()
            );
        }

        for flag in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
            println!("cargo:rustc-link-arg={flag}");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}
