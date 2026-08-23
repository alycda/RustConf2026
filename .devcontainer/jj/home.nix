{ pkgs, ... }:
{
  imports = [ ../home.nix ];

  # Jujutsu on top of the shared profile. shell.nix's git stays available, so
  # colocated repos (jj git clone --colocate) work out of the box.
  home.packages = with pkgs; [ jujutsu ];
}
