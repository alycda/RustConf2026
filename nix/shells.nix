# Every dev shell this repo offers, as one function of `nixpkgs`.
#
# Imported by BOTH entry points — flake.nix (`nix develop`) and shell.nix
# (`nix-shell`, which .github/workflows/rust.yml's `ffi` job and
# .vscode/tasks.json both drive) — so the two paths cannot describe
# different environments. flake.lock pins the nixpkgs they resolve against.
#
# Shells:
#
#   default            the five required tools plus the small conveniences
#   full               the above plus every C library the days' default-off
#                      cargo features link against
#   python kotlin
#   swift dart         the above plus ONE Exercise 3 language track, pinned
#
# The default/full split exists for the room, and the reasoning is unchanged
# from when this was one file: attendees who clone on-site pay for the
# default shell over venue Wi-Fi, and none of the C libraries is on the path
# through the exercises. Every one sits behind a cargo feature that is off by
# default, and every build.rs skips its pkg-config probe unless that feature
# is on (days/2021-12-02/build.rs is the pattern). So the download they
# cannot avoid stays as small as the workshop's own contract — which is what
# scripts/self-check.sh verifies and what book/src/nix.md advertises.
#
# Measured on nixpkgs-unstable, 2026-09-06, via `nix-build --dry-run` —
# i.e. BEFORE this file was pinned, so the split is the reason the two shells
# exist rather than a current reading of the locked revision. Worth re-taking
# on a cold store against flake.lock; docs/flake-handoff.md tracks that.
# 726.5 MiB for the default shell against 1.4 GiB for full. The libraries are
# not a rounding error next to the Rust toolchain; they are roughly the other
# half of the download. Most of that half is one package: espeak-ng costs
# ~600 MiB on its own, because nixpkgs' build wants audio output and so drags
# in libpulseaudio and most of ffmpeg — for a day that only calls
# espeak_TextToPhonemes and friends. duckdb is a distant second at ~73 MiB;
# every other library here is under 20 MiB. If the full shell ever needs to
# get cheaper, an audio-less espeak-ng is the whole game.
#
# The per-track shells are Exercise 3's "pick ONE language track". Entering
# one IS the setup: the floors the README states (JDK 17+, Python 3.10+,
# Dart 3.0+) hold because the pin says so, not because a script went looking.
# The `just setup-<track>` recipes remain the answer off Nix, where nothing
# can promise a version.
#
# A track shell builds on `default`, not on `full` — Exercise 3 calls a day's
# cbindgen C API, which needs no system library. Someone who wants both can
# combine them by hand; see docs/flake-handoff.md.
{ nixpkgs }:

let
  # Two nixpkgs packages are broken on darwin (details at each override). The
  # fixes are applied as an overlay so that `pkgs.chipmunk` and `pkgs.tinycc`
  # *are* the fixed packages everywhere below: a plain let-binding shadowing
  # them inside `with pkgs; [ ... ]` works too, but a later `pkgs.chipmunk`
  # anywhere in this file would silently reach the unfixed one, and on darwin
  # that is issue #1 again with the fix sitting thirty lines above it.
  #
  # Each override carries a tripwire. The channel is unpinned, so the day
  # nixpkgs fixes the package the override keeps forcing a source build for
  # no reason; the warnIf fires on that day, instead of the comments quietly
  # describing a nixpkgs that no longer exists.
  pkgs = nixpkgs.extend (final: prev: let inherit (prev) lib; in {
    chipmunk =
      lib.warnIf
        (!lib.any (d: (d.pname or "") == "glfw") prev.chipmunk.buildInputs)
        "shell.nix: nixpkgs' chipmunk no longer depends on glfw2; drop the override"
        (darwinFixes.chipmunk prev);
    tinycc =
      if !prev.stdenv.hostPlatform.isDarwin then prev.tinycc else
      lib.warnIf
        (!lib.any (i: lib.hasInfix (builtins.placeholder "lib") (i.text or "")) prev.tinycc.pkgconfigItems)
        "shell.nix: nixpkgs' tinycc ships a libtcc.pc with real paths; drop the override"
        (darwinFixes.tinycc prev);
  });

  darwinFixes = {
    # nixpkgs' tinycc writes its libtcc.pc through makePkgconfigItem, which only
    # knows how to defer `placeholder "out"`; the item uses `placeholder "lib"`
    # and `placeholder "dev"` too, and those survive into the installed file as
    # bare 52-character hashes: `-L/0sra2y…/lib -Wl,--rpath /0sra2y…/lib`. Linux
    # never noticed — the cc wrapper already passes the real -L for every
    # buildInput, and GNU ld swallows the bogus path as --rpath's argument. On
    # macOS clang rejects the stray positional path before ld runs, so
    # 2015-12-01's `tcc` feature cannot link. The rewritten item uses the
    # `@lib@`/`@dev@` forms that copyPkgconfigItems' substituteAllInPlace does
    # resolve.
    #
    # Except libtcc.dylib's install name is `@rpath/libtcc.dylib` — tinycc
    # skips the fixDarwinDylibNames hook the rest of nixpkgs runs, so a test
    # binary links fine and then aborts at load with "Library not loaded".
    # The hook is added here; it rewrites the id to the store path in fixup.
    #
    # darwin only (the overlay above): Linux loads libtcc fine through the cc
    # wrapper and gets tcc from cache.nixos.org; a source build there costs
    # ~65 s with tinycc's Linux-only installCheck. On darwin it is ~15 s.
    tinycc = prev: prev.tinycc.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ prev.fixDarwinDylibNames ];
      pkgconfigItems = [
        (prev.makePkgconfigItem {
          name = "libtcc";
          inherit (old) version;
          description = "Tiny C compiler backend";
          # The variables are the single source for the paths; pkg-config
          # expands ${libdir} itself. The rpath is one token — the upstream
          # item's `-Wl,--rpath <path>` is two, and build.rs splits on
          # whitespace — so a link that bypasses nixpkgs' wrapped linker (a
          # ~/.cargo/config.toml `linker =`, say) still finds the dylib.
          cflags = [ "-I\${includedir}" ];
          libs = [ "-L\${libdir}" "-Wl,-rpath,\${libdir}" "-ltcc" ];
          variables = {
            prefix = "@out@";
            includedir = "@dev@/include";
            libdir = "@lib@/lib";
          };
        })
      ];
    });

    # nixpkgs' chipmunk declares `platforms = unix` but pulls glfw2, libglut
    # and the X11 stack into buildInputs — all of it for the `chipmunk_demos`
    # binary, none of it for libchipmunk. glfw2 is `platforms = linux`, so on
    # aarch64-darwin the evaluator refuses the whole package, and with it the
    # whole shell (issue #1). The override drops the demo and its inputs;
    # Chipmunk's own CMakeLists offers BUILD_DEMOS for exactly this. The
    # library then builds on macOS from source in about a minute — nothing in
    # cache.nixos.org has this derivation, on either platform.
    chipmunk = prev: prev.chipmunk.overrideAttrs (old: {
      buildInputs = [ ];
      cmakeFlags = (old.cmakeFlags or [ ]) ++ [ "-DBUILD_DEMOS=OFF" ];
      postInstall = "";
    });
  };

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
    # builtins.path with an explicit name: interpolating ../.cheat directly
    # would store it under its basename, and store names starting with a
    # period are rejected before Nix 2.20 — killing the whole shell, required
    # toolchain included, on distro-packaged Nix.
    { name = "ffi-playground"; path = builtins.path { path = ../.cheat; name = "ffi-playground-cheat"; }; tags = "[]"; }
  ] ++ pkgs.lib.optionals (isJJContainer && builtins.pathExists ../.devcontainer/jj/cheat) [
    { name = "jj"; path = ../.devcontainer/jj/cheat; tags = "[ jj ]"; }
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

  # The two derivations below are referenced only from the `full` list, and
  # Nix is lazy, so the default shell never evaluates them — no chipmunk or
  # duckdb path is realised, let alone downloaded.
  #
  # nixpkgs ships neither of 2021-12-02's two C libraries with a pkg-config
  # file: chipmunk has include/chipmunk/*.h and lib/libchipmunk.so, duckdb has
  # include/duckdb.h and lib/libduckdb.so, and `pkg-config --libs <name>` fails
  # for both even with the packages in buildInputs.
  #
  # days/2021-12-02/build.rs probes pkg-config for every library it links.
  # Teaching it two library-specific discovery mechanisms would make that one
  # build script the exception in a repo where they all look alike, so the
  # missing files are synthesized here instead: writeTextDir puts each at
  # $out/lib/pkgconfig/, which pkg-config's setup hook adds to PKG_CONFIG_PATH
  # like any other package's. Two libraries, two gaps, one technique — and a
  # third .pc, libtcc's, fixed by a different one above: nixpkgs ships that
  # file, just with the wrong contents, so it is overridden rather than added.
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

  # The workshop shell: everything an attendee needs for Exercises 1-3, and
  # nothing whose absence they'd only discover by opting into a feature.
  workshop = with pkgs; [
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
    # Here rather than in `full`, despite having no consumer in this list:
    # every C-backed day's build.rs shells out to pkg-config, and the panic
    # naming shell.nix and the missing .pc only happens if pkg-config runs at
    # all. Leave it out of the default shell and an attendee who flips a
    # feature on gets "failed to run pkg-config: No such file or directory"
    # instead of the message telling them which shell to be in.
    pkg-config
  ];

  # The C libraries behind the days' cargo features. None of these is reachable
  # from `just check`, the exercises, or any default `cargo build` — enabling
  # the feature is the only way to need them, and `--arg full true` is how you
  # get them. .github/workflows/rust.yml's `ffi` job is the CI side of that.
  cLibraries = with pkgs; [
    # 2015-12-01 banners its answer through libcaca's FIGlet engine
    # (days/2015-12-01/src/caca.rs) and JIT-compiles a C function with
    # libtcc at runtime (days/2015-12-01/src/tcc.rs), both via FFI — no
    # system-wide installs needed, `pkg-config` picks up caca.pc and
    # libtcc.pc automatically via its setup hook.
    libcaca tinycc
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
    # uthash for days/2024-12-01's C-hash-table variant (cargo feature
    # `uthash`, off by default). Header-only and no .pc file, so no
    # pkg-config: the cc wrapper injects the include path for every
    # buildInputs entry, which is how that day's build.rs finds <uthash.h>.
    uthash
  ];
in
let
  # The shell every other one is built from.
  mkWorkshopShell = { extra ? [ ], env ? { } }:
    pkgs.mkShell (env // {
      buildInputs = workshop ++ extra;

      CHEAT_CONFIG_PATH = cheatConf;

      # nixpkgs ships the std sources separately from rustc, so without this
      # rust-analyzer logs "can't load standard library" and goto-def/completion
      # stop at the edge of std — which in an FFI workshop means ffi::CString.
      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    });

  # WORKSHOP_TRACK is what scripts/self-check.sh reads to tell "pinned by
  # this shell" apart from "found on PATH, version unverified". Nothing else
  # should depend on it.
  mkTrack = { track, packages ? [ ], env ? { } }:
    mkWorkshopShell {
      extra = packages;
      env = { WORKSHOP_TRACK = track; } // env;
    };

  # python3 with cffi already in it. On this path `just setup-python` is
  # unnecessary — no venv to create, nothing to pip-install — which is the
  # point: the 3.10+ floor is satisfied by construction.
  pythonWithCffi = pkgs.python3.withPackages (ps: [ ps.cffi ]);

  # nixpkgs' swift is Linux-only in practice; on Darwin the toolchain comes
  # from the Xcode CLT, which is what `just setup-swift` and the README have
  # always said. Rather than fail to evaluate on a mac, the swift shell there
  # is the base shell plus a pointer at the CLT.
  swiftAvailable = pkgs.stdenv.hostPlatform.isLinux && (pkgs ? swift);
in
{
  default = mkTrack { track = "none"; };

  # `--arg full true`'s replacement. shell.nix still accepts that flag and
  # maps it here, so .github/workflows/rust.yml's `ffi` job is untouched.
  full = mkTrack {
    track = "none";
    packages = cLibraries;
  };

  python = mkTrack {
    track = "python";
    packages = [ pythonWithCffi ];
  };

  kotlin = mkTrack {
    track = "kotlin";
    # jdk17, not `jdk`: the README and the self-check hint both state a
    # JDK 17+ floor, and naming the version is what makes the floor real.
    # A newer default JDK would still satisfy it, but then the floor would be
    # whatever nixpkgs happened to default to — the drift the pin removes.
    packages = with pkgs; [ jdk17 kotlin ];

    # Putting jdk17 on PATH is not enough, and this was measured rather than
    # assumed: `kotlinc` honours JAVA_HOME over PATH, so in a shell where the
    # ambient environment already exports one — a distro JDK, the
    # devcontainer, brew's openjdk, sdkman — `java -version` reports the
    # pinned 17 while kotlinc quietly compiles against the system JVM.
    # Observed as `kotlinc-jvm 2.1.20 (JRE 21.0.10+7-Ubuntu-124.04)` sitting
    # next to `openjdk version "17.0.17"`. A pinned track that silently defers
    # to whatever JDK the attendee happens to have is the failure this pin
    # exists to remove, so the shell names the JVM explicitly.
    env.JAVA_HOME = "${pkgs.jdk17}";
  };

  swift = mkTrack {
    track = "swift";
    packages = pkgs.lib.optionals swiftAvailable [ pkgs.swift ];
    env = pkgs.lib.optionalAttrs (!swiftAvailable) {
      shellHook = ''
        echo "note: nixpkgs' Swift toolchain is not available for this system."
        echo "      Install the Xcode Command Line Tools instead: xcode-select --install"
        echo "      (Everything else in this shell is the pinned workshop toolchain.)"
      '';
    };
  };

  dart = mkTrack {
    track = "dart";
    packages = [ pkgs.dart ];
  };
}
