// Links against libcaca (the FIGlet-font banner renderer, see src/caca.rs)
// via pkg-config rather than a system-wide install: nixpkgs' `libcaca`
// package ships a caca.pc, and the dev shell's `pkg-config` setup hook
// points pkg-config at it automatically, so `pkg-config --libs caca` is
// enough without hardcoding any nix store path here.
use std::process::Command;

fn main() {
    let libs = Command::new("pkg-config")
        .args(["--libs", "caca"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run pkg-config: {e}"));

    if !libs.status.success() {
        panic!(
            "pkg-config could not find caca.pc ({}). Run inside the project's nix shell \
             (see shell.nix), which provides the `libcaca` package.",
            String::from_utf8_lossy(&libs.stderr).trim()
        );
    }

    for flag in String::from_utf8_lossy(&libs.stdout).split_whitespace() {
        println!("cargo:rustc-link-arg={flag}");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
