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
  # nixpkgs' duckdb ships include/duckdb.h and lib/libduckdb.so but no
  # lib/pkgconfig/duckdb.pc, so `pkg-config --libs duckdb` fails even with the
  # package in buildInputs. days/2021-12-02/build.rs probes pkg-config the way
  # every C variant in this repo does; teaching it a second, duckdb-only
  # discovery mechanism would make that one day the exception. Synthesizing
  # the missing file instead keeps the build script uniform — writeTextDir
  # puts it at $out/lib/pkgconfig/, which pkg-config's setup hook adds to
  # PKG_CONFIG_PATH like any other package's.
  #
  # Note the two prefixes: duckdb splits its headers and its shared object
  # across the `dev` and `lib` outputs, so one `prefix=` would resolve only
  # half of it.
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
    # 2021-12-02 folds the course in SQL through DuckDB
    # (days/2021-12-02/src/duckdb.rs) via FFI — an in-process analytical
    # database doing the work of three integer accumulators. No system-wide
    # install needed; duckdbPc above supplies the .pc nixpkgs doesn't ship.
    duckdb duckdbPc pkg-config
  ];

  CHEAT_CONFIG_PATH = cheatConf;

  # nixpkgs ships the std sources separately from rustc, so without this
  # rust-analyzer logs "can't load standard library" and goto-def/completion
  # stop at the edge of std — which in an FFI workshop means ffi::CString.
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
}
