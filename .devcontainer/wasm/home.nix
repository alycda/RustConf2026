{ pkgs, ... }:
let
  # wasm-bindgen's CLI refuses a module built with any other patch version
  # of the crate, so the version that matters is the one days/Cargo.lock
  # pins — read from there, not copied here, so this file cannot disagree
  # with it. days/Cargo.toml's `wasm-bindgen = "=0.2.x"` is the pin; the
  # lockfile is where cargo resolved it, and `builtins.fromTOML` reads it.
  lock = builtins.fromTOML (builtins.readFile ../../days/Cargo.lock);
  pinned = (pkgs.lib.findFirst (p: p.name == "wasm-bindgen") null lock.package).version;

  # nixpkgs keeps every recent CLI release as its own attribute —
  # wasm-bindgen-cli_0_2_121 and a dozen-odd siblings — hashes maintained
  # upstream and built by Hydra, so the pinned version comes from the
  # binary cache whichever channel this profile is built from. The bare
  # `wasm-bindgen-cli` is whatever that channel calls current, which was
  # 0.2.121 on the channel this track was written against and 0.2.127 on
  # nixpkgs-unstable the same week: the first attendee to reopen the
  # variant met `just days wasm-demo` refusing to run. Same store path as
  # the default when the channel agrees, the right older build when it
  # does not.
  attr = "wasm-bindgen-cli_${builtins.replaceStrings [ "." ] [ "_" ] pinned}";
  wasmBindgenCli = pkgs.${attr} or (throw ''
    ${attr}: this nixpkgs has no wasm-bindgen-cli at ${pinned}, the version
    days/Cargo.lock pins. Either bump days/Cargo.toml's wasm-bindgen pin to a
    version it does carry (`nix eval --impure --expr 'builtins.filter (n:
    builtins.match "wasm-bindgen-cli_.*" n != null) (builtins.attrNames
    (import <nixpkgs> {}))'` lists them) and `cargo update -p wasm-bindgen`, or run
    `cargo install wasm-bindgen-cli --version ${pinned} --locked` for this
    one profile.
  '');
in
{
  imports = [ ../home.nix ];

  # wasm track (Exercise 4, and the 2015-12-01 variant): what the track
  # installs on its own, kept out of shell.nix so the other tracks never
  # download it. The target's std and the linker are NOT here — shell.nix's
  # rustc has wasm32-unknown-unknown built in and shell.nix carries lld — so
  # this is only the consumer side.
  #
  # nodejs_22: the floor scripts/self-check.sh probes and the version CI
  # runs. wasm-bindgen-cli: at the lockfile's version, see above; `just
  # days wasm-demo` still checks the two agree before building, which is
  # the message you get if this file and the lockfile ever drift. wasm-pack:
  # for the other recipe, which fetches its own matching CLI and needs the
  # network to do it.
  home.packages = with pkgs; [ nodejs_22 wasmBindgenCli wasm-pack ];
}
