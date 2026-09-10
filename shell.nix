# nix-shell entry point, pinned to the same nixpkgs as flake.nix.
#
# This file exists because not everything speaks flakes:
# .github/workflows/rust.yml's `ffi` job runs
# `nix-shell ../shell.nix --arg full true`, .vscode/tasks.json runs
# `nix-shell --command`, the upstream Nix installer does not enable
# `nix-command flakes` (the Determinate installer the README recommends
# does), and .envrc falls back to `use nix` when they are off.
#
# It is a shim, not a second definition. The revision comes out of
# flake.lock and the shell contents come out of nix/shells.nix, so a
# non-flake caller gets the same rustc, the same cbindgen and the same JDK
# as everyone else. Change the pin by updating flake.lock
# (`nix flake update`), never by editing this file.
#
#   nix-shell                    the workshop shell (five required tools)
#   nix-shell --arg full true    the above plus every C library the days'
#                                default-off cargo features link against
#   nix-shell -A python          + python3 with cffi, no venv needed
#   nix-shell -A kotlin          + JDK 17 and kotlinc
#   nix-shell -A swift           + the Swift toolchain (Linux; CLT on macOS)
#   nix-shell -A dart            + the Dart SDK
#
# `--arg full true` is kept verbatim so the `ffi` CI job needs no edit; it is
# the same environment as `nix develop .#full`. `--arg nixpkgs ...` still
# overrides the pin for anyone who needs to test against another channel.
# The `let` wraps the function rather than sitting inside it: a function
# argument's default is evaluated in the scope OUTSIDE the function, so a
# `pinnedNixpkgs` bound in the body is not visible to `nixpkgs ? ...` and
# every caller dies with `undefined variable 'pinnedNixpkgs'`.
let
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  nixpkgsLock = lock.nodes.nixpkgs.locked;

  # narHash from flake.lock is the hash of the unpacked tree, which is
  # exactly what fetchTarball checks — so both entry points are pinned by the
  # same value and `nix flake update` moves them together.
  pinnedNixpkgs = builtins.fetchTarball {
    url = "https://github.com/${nixpkgsLock.owner}/${nixpkgsLock.repo}/archive/${nixpkgsLock.rev}.tar.gz";
    sha256 = nixpkgsLock.narHash;
  };
in
{
  nixpkgs ? import pinnedNixpkgs { },
  full ? false,
}:

let
  shells = import ./nix/shells.nix { inherit nixpkgs; };
in
# A derivation, not an attribute set. `nix-shell` on a file that evaluates to
# a set fails outright with "nix-shell requires a single derivation", which
# would break the `ffi` CI job, .vscode/tasks.json, and the bare `nix-shell`
# the README documents. So the selected shell is the value, and the rest ride
# along as extra attributes — `//` keeps `type = "derivation"`, so
# `nix-shell -A kotlin` still resolves.
(if full then shells.full else shells.default) // {
  inherit (shells) default full python kotlin swift dart;
}
