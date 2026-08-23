{ pkgs, ... }:
{
  imports = [ ../home.nix ];

  # Flutter/Dart track: the Flutter SDK's bin/ also ships `dart`, which is
  # all the Dart track and `just check` actually probe for. Do NOT add
  # pkgs.dart alongside — both provide bin/dart and the profile build fails
  # on the collision. Heads up: ~930 MiB download, ~3.5 GiB unpacked (all
  # binary-cache hits on aarch64-linux — nothing compiles).
  home.packages = with pkgs; [ flutter ];
}
