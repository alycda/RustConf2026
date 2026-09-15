{ pkgs, ... }:
{
  imports = [ ../home.nix ];

  # wasm track (Exercise 4, and the 2015-12-01 variant): what the track
  # installs on its own, kept out of shell.nix so the other tracks never
  # download it. The target's std and the linker are NOT here — shell.nix's
  # rustc has wasm32-unknown-unknown built in and shell.nix carries lld — so
  # this is only the consumer side.
  #
  # nodejs_22: the floor scripts/self-check.sh probes and the version CI
  # runs. wasm-bindgen-cli: the channel's version is the version
  # days/Cargo.toml pins the crate to (exactly — the CLI refuses any other),
  # so a channel bump here is a pin bump there; `just days wasm-demo` checks
  # the two agree before building. wasm-pack: for the other recipe, which
  # fetches its own matching CLI and needs the network to do it.
  home.packages = with pkgs; [ nodejs_22 wasm-bindgen-cli wasm-pack ];
}
