{
  description = "Using Advent of Code as an FFI Playground — pinned workshop environment";

  # The pin. shell.nix used to say `import <nixpkgs> {}`, which resolves
  # against whatever channel each attendee happens to have — the same tool
  # set, different versions, per person. The README said so itself:
  # "shell.nix is unpinned ... If the whole room needs identical versions,
  # that takes a pinned nixpkgs, not a choice of option."
  #
  # This is that pinned nixpkgs. It is also what lets the Exercise 3 version
  # floors (JDK 17+, Python 3.10+, Dart 3.0+) hold by construction instead of
  # being checked by a version-string parser nobody wants to maintain.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

  outputs = { self, nixpkgs }:
    let
      # No flake-utils: one `inputs` entry is one more thing to keep current,
      # and this is four lines.
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      eachSystem = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      # nix/shells.nix takes the raw nixpkgs set because it applies its own
      # overlay (the two darwin package fixes) before using it.
      devShells = eachSystem (system:
        import ./nix/shells.nix {
          nixpkgs = import nixpkgs { inherit system; };
        });

      formatter = eachSystem (system:
        (import nixpkgs { inherit system; }).nixpkgs-fmt);
    };
}
