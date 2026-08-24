// Links against libcaca (the FIGlet-font banner renderer, see src/caca.rs)
// and libtcc (the Tiny C Compiler's JIT backend, see src/tcc.rs) via
// pkg-config rather than system-wide installs: nixpkgs' `libcaca` and
// `tinycc` packages ship caca.pc and libtcc.pc, and the dev shell's
// `pkg-config` setup hook points pkg-config at them automatically, so
// `pkg-config --libs <name>` is enough without hardcoding any nix store
// path here.
use std::process::Command;

fn main() {
    for name in ["caca", "libtcc"] {
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
}
