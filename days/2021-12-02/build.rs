// Links against Chipmunk2D (the 2D rigid-body physics engine this day
// dead-reckons the submarine through, see src/chipmunk.rs) via pkg-config
// rather than a system-wide install or a hardcoded path.
//
// nixpkgs' `chipmunk` is the one library in this repo that ships no .pc file
// of its own — shell.nix synthesizes `chipmunk.pc` next to it and lets
// pkg-config's setup hook put it on PKG_CONFIG_PATH, so the probe below is
// the same three lines every other C variant here uses. Fixing the gap on
// the nix side rather than here is deliberate: a build script that knows
// about include paths is a build script that has to be re-taught them on
// every machine.
//
// The probe runs only when the cargo feature is enabled (cargo exposes
// enabled features to build scripts as CARGO_FEATURE_* env vars), so the
// default build needs neither pkg-config nor the library — that's what keeps
// stock CI runners and the manual-setup path green.
use std::process::Command;

fn main() {
    let variants = [("CARGO_FEATURE_CHIPMUNK", "chipmunk")];

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
                 (see shell.nix), which synthesizes it — nixpkgs' chipmunk does not ship one.",
                String::from_utf8_lossy(&libs.stderr).trim()
            );
        }

        for flag in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
            println!("cargo:rustc-link-arg={flag}");
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}
