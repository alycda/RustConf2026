{ pkgs, ... }:
{
  imports = [ ../home.nix ];

  # Swift track. Yes, really — see ./README.md for what works, what's old,
  # and the two runtime gotchas. Foundation is not optional: without
  # `import Foundation` in your Swift file the binary links but can't find
  # libdispatch.so at runtime.
  # Dispatch is not optional either, and its absence fails EARLIER than the
  # Foundation caveat above: swiftPackages.Foundation installs the Foundation,
  # FoundationNetworking and FoundationXML Swift modules and nothing else,
  # while the Foundation module itself requires the CoreFoundation and Dispatch
  # *clang* modules. Without them `import Foundation` does not reach the
  # runtime gotcha below — it fails to compile with
  # `missing required modules: 'CoreFoundation', 'Dispatch'`.
  # Both corelibs packages are multi-output: `out` holds the libraries and the
  # Swift modules, `dev` holds the C headers and the module maps that Swift's
  # Dispatch and Foundation modules are wrappers around. home.packages takes
  # the default output, so installing only these two gives you modules that
  # resolve and then fail to load. The .dev outputs are not optional here.
  home.packages = with pkgs; [
    swift
    swiftPackages.Foundation
    swiftPackages.Foundation.dev
    swiftPackages.Dispatch
    swiftPackages.Dispatch.dev
  ];
}
