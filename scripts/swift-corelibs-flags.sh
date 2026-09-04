#!/usr/bin/env bash
# Print the swiftc flags that find Foundation in the repo's Swift devcontainer,
# or nothing anywhere else.
#
# There, nixpkgs installs corelibs (Foundation, Dispatch) into the user
# profile rather than beside the compiler, and swiftc searches only its own
# toolchain path: bare, it fails with `no such module 'Foundation'`. Three
# directories answer three failures — the Swift modules (-I), the clang
# module maps (-Xcc -I), the shared objects (-L) — and the -rpath pair is what
# lets the binary *load* after it links (see days/justfile's swift-demo for
# the ladder, and .devcontainer/swift/README.md). On a Mac or a swift.org
# toolchain the layout is absent and this prints nothing, so callers can
# splice it unconditionally: swiftc … $(scripts/swift-corelibs-flags.sh) …
#
# Printed on one line, unquoted at the call site on purpose: bash 3.2 (a
# stock Mac) has no mapfile, and nix profile paths carry no spaces.
set -euo pipefail
profile="$HOME/.nix-profile"
arch="$(uname -m)"
if [ -d "$profile/lib/swift/linux/$arch" ]; then
  printf '%s ' \
    -I "$profile/lib/swift/linux/$arch" \
    -Xcc "-I$profile/include" \
    -L "$profile/lib/swift/linux" -L "$profile/lib" \
    -Xlinker -rpath -Xlinker "$profile/lib/swift/linux" \
    -Xlinker -rpath -Xlinker "$profile/lib"
fi
echo
