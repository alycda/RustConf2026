{ pkgs ? import <nixpkgs> {} }:

let
  # The jj devcontainer exports WORKSHOP_HOME_NIX=<...>/.devcontainer/jj/home.nix
  # via containerEnv; impure eval reads it here. jj sheets live with that
  # variant (.devcontainer/jj/cheat/) and only its container sees them —
  # attendees in the other containers don't get sheets for a tool they lack.
  isJJContainer = pkgs.lib.hasInfix "/jj/" (builtins.getEnv "WORKSHOP_HOME_NIX");

  # cheat config, dotfiles-style (alycda/dotfiles tools/cheat/conf.nix): the
  # sheets are copied into the nix store and the config points there, so the
  # setup survives fresh containers and nix-direnv's cached-env replay — a
  # shellHook-generated file in /tmp would not. Sheet edits re-copy on the
  # next prompt because .envrc watches .cheat; plain nix-shell users
  # re-enter the shell instead.
  cheatPaths = [
    # builtins.path with an explicit name: interpolating ./.cheat directly
    # would store it under its basename, and store names starting with a
    # period are rejected before Nix 2.20 — killing the whole shell, required
    # toolchain included, on distro-packaged Nix.
    { name = "ffi-playground"; path = builtins.path { path = ./.cheat; name = "ffi-playground-cheat"; }; tags = "[]"; }
  ] ++ pkgs.lib.optionals (isJJContainer && builtins.pathExists ./.devcontainer/jj/cheat) [
    { name = "jj"; path = ./.devcontainer/jj/cheat; tags = "[ jj ]"; }
  ];
  cheatConf = pkgs.writeText "ffi-playground-cheat-conf.yml" (''
    colorize: true
    style: monokai
    formatter: terminal256
    pager: less -FRX
    cheatpaths:
  '' + pkgs.lib.concatMapStrings (p: ''
    - name: ${p.name}
      path: ${p.path}
      tags: ${p.tags}
      readonly: true
  '') cheatPaths);
  # nixpkgs ships neither of this day's two C libraries with a pkg-config
  # file: chipmunk has include/chipmunk/*.h and lib/libchipmunk.so, duckdb has
  # include/duckdb.h and lib/libduckdb.so, and `pkg-config --libs <name>` fails
  # for both even with the packages in buildInputs.
  #
  # days/2021-12-02/build.rs probes pkg-config for every library it links.
  # Teaching it two library-specific discovery mechanisms would make that one
  # build script the exception in a repo where they all look alike, so the
  # missing files are synthesized here instead: writeTextDir puts each at
  # $out/lib/pkgconfig/, which pkg-config's setup hook adds to PKG_CONFIG_PATH
  # like any other package's. Two libraries, two gaps, one technique.
  chipmunkPc = pkgs.writeTextDir "lib/pkgconfig/chipmunk.pc" ''
    prefix=${pkgs.chipmunk}
    Name: chipmunk
    Description: Chipmunk2D rigid body physics
    Version: ${pkgs.chipmunk.version}
    Cflags: -I''${prefix}/include
    Libs: -L''${prefix}/lib -lchipmunk -lm
  '';

  # Note the two prefixes here and only one above: duckdb splits its headers
  # and its shared object across the `dev` and `lib` outputs, so a single
  # `prefix=` would resolve half of it.
  duckdbPc = pkgs.writeTextDir "lib/pkgconfig/duckdb.pc" ''
    Name: duckdb
    Description: DuckDB in-process analytical database
    Version: ${pkgs.duckdb.version}
    Cflags: -I${pkgs.duckdb.dev}/include
    Libs: -L${pkgs.duckdb.lib}/lib -lduckdb
  '';
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    # required workshop toolchain (verified by `just check`); mkShell's stdenv
    # already provides the C compiler and linker. `just` is required too — it
    # is how attendees invoke everything.
    rustc cargo rust-cbindgen just
    # clippy and rustfmt ship separately from cargo in nixpkgs, so without
    # them `cargo fmt` / `cargo clippy` are "no such command" in this shell —
    # and .github/workflows/rust.yml gates on both. A CI check an attendee
    # cannot run before pushing is a check they only ever meet as a red X.
    clippy rustfmt
    # recommended: cheatsheets for the FFI patterns (`just cheats`)
    cheat
    # safety net: python3 for the Python track; git so pure/minimal shells
    # (and jj colocated clones) get a current git (no verification needed)
    python3 git
# 2015-12-01 banners its answer through libcaca's FIGlet engine
    # (days/2015-12-01/src/caca.rs) and JIT-compiles a C function with
    # libtcc at runtime (days/2015-12-01/src/tcc.rs), both via FFI — no
    # system-wide installs needed, `pkg-config` picks up caca.pc and
    # libtcc.pc automatically via its setup hook.
    libcaca tinycc pkg-config
    # 2015-12-05 scans lines two ways, both via FFI: through vectorscan
    # (the maintained Hyperscan fork, days/2015-12-05/src/hyperscan.rs) and
    # through ICU's regex engine via a small C shim
    # (days/2015-12-05/src/icu_shim.c, src/icu.rs) — `pkg-config` picks up
    # libhs.pc / icu-i18n.pc / icu-uc.pc automatically via its setup hook.
    vectorscan icu
    # 2021-12-02 solves the same puzzle two absurd ways, both via FFI and both
    # off by default as cargo features: it dead-reckons the submarine through
    # Chipmunk2D's rigid-body solver (days/2021-12-02/src/chipmunk.rs) and
    # folds the course in SQL through DuckDB (days/2021-12-02/src/duckdb.rs).
    # No system-wide installs needed; the two *Pc entries above supply the .pc
    # files nixpkgs doesn't ship, so nothing here is hardcoded in build.rs.
    chipmunk chipmunkPc duckdb duckdbPc
    # 2023-12-01 solves the same puzzle two absurd ways, both via FFI and both
    # off by default as cargo features: it finds the calibration digits with
    # YARA, the malware-scanning engine (days/2023-12-01/src/yara.rs), and it
    # tries to hear them through espeak-ng, the speech synthesiser
    # (days/2023-12-01/src/espeak.rs). nixpkgs ships yara.pc and espeak-ng.pc,
    # so unlike 2021-12-02's chipmunk and duckdb there is nothing to
    # synthesize here — build.rs finds both through pkg-config's setup hook
    # with no hardcoded store path, and espeak's own dictionaries come from the
    # data directory compiled into the library.
    yara espeak-ng
  ];

  CHEAT_CONFIG_PATH = cheatConf;

  # nixpkgs ships the std sources separately from rustc, so without this
  # rust-analyzer logs "can't load standard library" and goto-def/completion
  # stop at the edge of std — which in an FFI workshop means ffi::CString.
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
}
