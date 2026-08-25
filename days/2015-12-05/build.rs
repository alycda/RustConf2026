// Links against vectorscan (the Hyperscan fork, see src/hyperscan.rs) via
// pkg-config rather than a system-wide install: nixpkgs' `vectorscan`
// package ships a libhs.pc, and the dev shell's `pkg-config` setup hook
// points pkg-config at it automatically, so `pkg-config --libs libhs` is
// enough without hardcoding any nix store path here.
use std::process::Command;

fn main() {
    let libs = Command::new("pkg-config")
        .args(["--libs", "libhs"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run pkg-config: {e}"));

    if !libs.status.success() {
        panic!(
            "pkg-config could not find libhs.pc ({}). Run inside the project's nix shell \
             (see shell.nix), which provides the `vectorscan` package.",
            String::from_utf8_lossy(&libs.stderr).trim()
        );
    }

    for flag in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
        println!("cargo:rustc-link-arg={flag}");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
