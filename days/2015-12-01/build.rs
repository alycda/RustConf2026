// Links against libcaca (the FIGlet-font banner renderer, see src/caca.rs)
// and libtcc (the Tiny C Compiler's JIT backend, see src/tcc.rs) via
// pkg-config rather than system-wide installs: nixpkgs' `libcaca` and
// `tinycc` packages ship caca.pc and libtcc.pc, and the dev shell's
// `pkg-config` setup hook points pkg-config at them automatically, so
// `pkg-config --libs <name>` is enough without hardcoding any nix store
// path here. Both packages live in shell.nix's `full` list — the shell
// attendees get by default carries neither, because nothing they run needs
// them; `nix-shell --arg full true` is the one that does.
//
// Each library is probed only when its cargo feature is enabled (cargo
// exposes enabled features to build scripts as CARGO_FEATURE_* env vars),
// so the default build needs neither pkg-config nor the libraries — that's
// what keeps stock CI runners and the manual-setup path green.
use std::process::Command;

fn main() {
    let variants = [
        ("CARGO_FEATURE_CACA", "caca"),
        ("CARGO_FEATURE_TCC", "libtcc"),
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
                 with the C libraries — `nix-shell --arg full true` (see shell.nix), which \
                 provides it.",
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
