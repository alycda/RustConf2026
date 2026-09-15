{ pkgs, ... }:
{
  imports = [ ../home.nix ];

  # Godot track: the engine itself, which is the only thing the track needs
  # that the base container lacks — gdext is a cargo dependency and builds
  # with the rustc ../home.nix already installs (1.94 is its floor; nixpkgs'
  # rustc is past it). Everything runs headless here: `just days godot-demo
  # 2015-12-01` and the CI cell both drive `godot --headless`, and there is
  # no display in a container to open an editor on anyway.
  #
  # nixpkgs installs three names — `godot`, `godot4` and `godot4.<minor>` —
  # so scripts/godot-bin.sh finds it with no $GODOT override. It comes from
  # the binary cache (nothing compiles), but it is not small: ~1.3 GiB
  # closure on aarch64-linux, most of it the X11/Wayland/audio libraries a
  # headless run never touches. That is why the engine lives in this variant
  # and not in shell.nix, the same rule that keeps R and gfortran out of the
  # default shell.
  #
  # "Fontconfig error: Cannot load default config file" on every run is
  # cosmetic — the engine prints it before --headless takes effect and
  # carries on. The channel's Godot is 4.6.x, which is the `api-4-6` floor
  # days/2015-12-01/Cargo.toml declares; a newer channel loads the same
  # extension (GDExtension is forward compatible within 4.x).
  home.packages = with pkgs; [ godot ];
}
