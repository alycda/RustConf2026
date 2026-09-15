{
  description = "Using Advent of Code as an FFI Playground — the workshop shells, pinned";

  # The pin. shell.nix used to say `import <nixpkgs> {}`, which resolves
  # against whatever channel each machine happens to carry: the README said
  # "versions are on you", and this repo's own docs proved it in one day —
  # R 4.5.3 here and 4.6.1 on the flake registry, Godot 4.6.3 and 4.7.2,
  # gfortran 15.2 and 15.3, all "current nixpkgs" on the same afternoon.
  # flake.lock names one revision, and shell.nix reads that lock, so the
  # non-flake entry points (`nix-shell`, direnv's `use nix`, the
  # devcontainers, CI's ffi job) resolve against the same revision without
  # enabling flakes anywhere. `nix develop` is the flake-native door onto
  # the same two shells.
  #
  # A release branch, not unstable: bumps happen when someone runs
  # `nix flake update` and reads the diff, and a stable branch keeps those
  # bumps to security and bug fixes. 26.05 carries everything the tracks
  # need — rustc past gdext's 1.94 floor, Godot 4.6 (the `api-4-6` floor
  # days/2015-12-01/Cargo.toml declares), R, gfortran, LAPACK.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs = { self, nixpkgs }:
    let
      # No flake-utils: one more input is one more thing to keep current,
      # and this is three lines.
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      eachSystem = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      # shell.nix stays the single definition of both shells; these are the
      # same derivations `nix-shell` and `nix-shell --arg full true` build
      # (verified: identical .drv paths), reached through the flake.
      devShells = eachSystem (system:
        let pkgs = import nixpkgs { inherit system; };
        in {
          default = import ./shell.nix { nixpkgs = pkgs; };
          full = import ./shell.nix { nixpkgs = pkgs; full = true; };
        });
    };
}
