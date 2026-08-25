// Compiles src/icu_shim.c (see its header comment for why a shim exists at
// all) against ICU's real headers, then links libicui18n/libicuuc via
// pkg-config — nixpkgs' `icu` package ships icu-i18n.pc/icu-uc.pc, and the
// dev shell's `pkg-config` setup hook points pkg-config at them
// automatically, so `pkg-config --cflags/--libs` is enough without
// hardcoding any nix store path here.
use std::process::Command;

fn pkg_config(args: &[&str]) -> String {
    let output = Command::new("pkg-config")
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run pkg-config: {e}"));

    if !output.status.success() {
        panic!(
            "pkg-config {args:?} failed ({}). Run inside the project's nix shell (see \
             shell.nix), which provides the `icu` package.",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    let mut build = cc::Build::new();
    for flag in pkg_config(&["--cflags", "icu-i18n"]).split_whitespace() {
        if let Some(include) = flag.strip_prefix("-I") {
            build.include(include);
        }
    }
    build.file("src/icu_shim.c").compile("icu_shim");

    for flag in pkg_config(&["--libs", "icu-i18n", "icu-uc"]).split_whitespace() {
        println!("cargo:rustc-link-arg={flag}");
    }

    println!("cargo:rerun-if-changed=src/icu_shim.c");
    println!("cargo:rerun-if-changed=build.rs");
}
