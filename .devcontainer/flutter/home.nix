{ pkgs, ... }:
{
  imports = [ ../home.nix ];

  # Dart track: `dart` is all the track and `just check` probe for — solve.dart,
  # ex3.dart and `dart pub` need the SDK, not Flutter. This variant used to
  # install pkgs.flutter for its bin/dart; flutter 3.47 now depends on aapt,
  # which nixpkgs builds only for x86_64-linux and aarch64-darwin, so on an
  # Apple Silicon Mac (aarch64-linux inside the container) activation refused
  # to evaluate and the container came up with no toolchain at all. Do NOT add
  # pkgs.flutter back alongside — both provide bin/dart and the profile build
  # fails on the collision. The directory keeps its `flutter` name so the
  # README's --config paths stay valid.
  home.packages = with pkgs; [ dart ];
}
