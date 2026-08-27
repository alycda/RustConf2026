// Links against vectorscan (the Hyperscan fork, see src/hyperscan.rs) and
// compiles+links ICU's regex shim (see src/icu_shim.c) — both via
// pkg-config rather than a system-wide install: nixpkgs' `vectorscan` and
// `icu` packages ship libhs.pc / icu-i18n.pc / icu-uc.pc, and the dev
// shell's `pkg-config` setup hook points pkg-config at them automatically,
// so `pkg-config --cflags/--libs <name>` is enough without hardcoding any
// nix store path here.
//
// Each library is probed only when its cargo feature is enabled (cargo
// exposes enabled features to build scripts as CARGO_FEATURE_* env vars,
// not as `cfg`s), so the default build needs neither pkg-config nor the
// libraries — that is what keeps stock CI runners and the manual-setup path
// green. `cc` stays an unconditional build-dependency for the same reason:
// the code below has to *compile* either way; only running it is gated.
use std::process::Command;

fn pkg_config(args: &[&str]) -> String {
    let output = Command::new("pkg-config")
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run pkg-config: {e}"));

    if !output.status.success() {
        panic!(
            "pkg-config {args:?} failed ({}). Run inside the project's nix shell (see \
             shell.nix), which provides the `vectorscan` and `icu` packages.",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn main() {
    if std::env::var_os("CARGO_FEATURE_ICU").is_some() {
        // The ICU shim (src/icu_shim.c) needs ICU's real headers to compile —
        // see that file's header comment for why it exists at all.
        let mut build = cc::Build::new();
        for flag in pkg_config(&["--cflags", "icu-i18n"]).split_whitespace() {
            if let Some(include) = flag.strip_prefix("-I") {
                build.include(include);
            }
        }
        // Nix's cc wrapper bakes in `-O2 -D_FORTIFY_SOURCE=3` by default; cc-rs
        // then appends `-O0` for cargo's dev profile, and gcc takes the last
        // `-O` flag, leaving `_FORTIFY_SOURCE` set without the optimization it
        // requires — glibc's headers then #warning about exactly that mismatch.
        // Forcing -O2 back on for this one tiny file satisfies the actual
        // requirement rather than disabling the hardening flag to route
        // around it; the crate's own debug/release profile is unaffected.
        build.opt_level(2);
        build.file("src/icu_shim.c").compile("icu_shim");

        for flag in pkg_config(&["--libs", "icu-i18n", "icu-uc"]).split_whitespace() {
            println!("cargo:rustc-link-arg={flag}");
        }
    }

    if std::env::var_os("CARGO_FEATURE_HYPERSCAN").is_some() {
        for flag in pkg_config(&["--libs", "libhs"]).split_whitespace() {
            println!("cargo:rustc-link-arg={flag}");
        }
    }

    println!("cargo:rerun-if-changed=src/icu_shim.c");
    println!("cargo:rerun-if-changed=build.rs");
}
