# Swift on Linux, in a container, without the glibc fight

Swift on Linux has a reputation, earned via the official swift.org tarballs:
they're built against one distro's glibc and anything else is a linker fight.
This container sidesteps that entirely — nixpkgs ships Swift built against
Nix's own glibc, and on aarch64-linux (and x86_64-linux) it comes straight
from the binary cache: ~660 MiB download, 3 GiB unpacked, **nothing compiles
from source**.

## Status 2026-08-17: fixed — and it took four different failures

Read this before the "Verified" section below, which it supersedes. The track
is green now — `days/2024-12-03/uniffi/build-and-test.sh swift` passes in this
container — but it did not start that way, and the route matters more than the
destination.

It began with a two-line hello-world failing:

```sh
printf 'import Foundation\nprint("hi")\n' > /tmp/t.swift && swiftc /tmp/t.swift -o /tmp/t1
# error: no such module 'Foundation'
```

No UniFFI, no bridging header, no repo environment involved — it fails the
same inside the repo's direnv shell and in `/tmp` with direnv unloaded. Three
distinct errors came up while chasing it, in this order. Each one is a
different problem, and the progression is the useful part:

**1. `no such module 'Foundation'`.** nixpkgs installs corelibs Foundation
into the *profile* (`~/.nix-profile/lib/swift/linux/<arch>/Foundation.swiftmodule`),
while `swiftc` — which resolves to
`/nix/store/…-swift-wrapper-5.10.1/bin/swiftc` — searches only its own
toolchain path. The module is on disk and invisible. Passing
`-I ~/.nix-profile/lib/swift/linux/<arch>` gets past it.

**2. `missing required modules: 'CoreFoundation', 'Dispatch'`.** Foundation's
Swift module requires two *clang* modules that live in
swift-corelibs-libdispatch, and `swiftPackages.Foundation` does not pull it
in: the profile held exactly `Foundation`, `FoundationNetworking` and
`FoundationXML` and nothing else. No flag fixes a module that was never
installed, so `home.nix` now installs `swiftPackages.Dispatch` too.

**3. `cannot load underlying module for 'Dispatch'`.** The Swift `Dispatch`
module resolves, but the C library it wraps cannot be loaded — and
`-Xcc -I~/.nix-profile/include` did not help, because that directory did not
exist. Both corelibs packages are **multi-output**: `out` holds the libraries
and Swift modules, `dev` holds the C headers and their module maps. `home.nix`
installs the default output, so the modules were present and the headers they
wrap were nowhere on disk. Fixed by installing
`swiftPackages.Foundation.dev` and `swiftPackages.Dispatch.dev` too.

**4. `cannot find -lBlocksRuntime` / `-ldispatch` / `-lswiftDispatch`.** Now it
compiles and fails to *link*: the Swift module lives in `lib/swift/…` and the
shared objects live in plain `lib/` of the same package. Compiling and linking
want different directories, and both errors name the same module — which is why
failure 4 reads like failure 3 not being fixed.

Two more were the harness's fault rather than the container's, and are fixed in
`days/2024-12-03/uniffi/`: Swift permits top-level statements only in a file
named `main.swift` (so the harness uses `@main`), and the generated wrapper's
filenames follow the namespace, so the script finds them instead of spelling
them.

### What this means

The **"Verified" section below is not reproducible as written** — its example
could not have compiled with `import Foundation` present, and caveat 1 there
describes a *runtime* libdispatch failure, a strictly later stage than failure 1
above. nixpkgs moved underneath it, which that section already names as the
expected trigger. It is kept as the record of what was claimed and when.

The generalizable lesson, which is now a book chapter
(`book/src/boundary.md`): a module resolving is not the same as its underlying
library loading, which is not the same as that library being linkable, which is
not the same as it being findable at run time. Four questions, four
directories, one module name in every error message.

### If it rots again

The flags in the day library's `build-and-test.sh` are the fragile part: they
name profile-relative paths, which is a packaging detail rather than a Swift
contract. A sturdier route, if this needs revisiting, is to build through
nixpkgs' Swift stdenv adapter or `swiftpm`, which wire the corelibs search
paths themselves — this only bites a hand-rolled `swiftc` invocation from a
home-manager profile. Failing that, this file's escape hatch (bottom) still
stands.

### What the day library carries

`days/2024-12-03/uniffi/build-and-test.sh swift` holds the `-I`, `-L` and
`-Xcc` flags for failures 1, 3 and 4, plus discovery of the generated
filenames. They are harness-side compensation for a packaging layout, and they
are not wrong on their own terms: Foundation genuinely is not on `swiftc`'s
default search path in this profile.

## Verified (2026-08, aarch64-linux container, nixpkgs unstable) — SUPERSEDED

See the status section above: this is retained as the record of what was
claimed, not as instructions.

The full workshop FFI shape works:

```sh
cc -shared -fPIC librust.c -o librust_add.so       # stand-in for the Rust cdylib
swiftc -import-objc-header bridge.h main.swift -L. -lrust_add -o ffi_swift
LD_LIBRARY_PATH="$PWD" ./ffi_swift                  # → ffi says 42
```

## Known caveats — read before filing "it's broken"

1. **`import Foundation` is load-bearing.** A Swift file that only uses the
   bridging header links fine but dies at runtime with
   `libdispatch.so: cannot open shared object file`. libdispatch is pulled in
   transitively (via libswift_Concurrency), and the Nix swift wrapper only
   embeds the rpath to it when Foundation is actually imported. Add
   `import Foundation` to your main file and it resolves.
2. **`LD_LIBRARY_PATH="$PWD"` to run.** Your own freshly built dylib isn't on
   any rpath; the Swift runtime libs are, so only the current directory needs
   adding.
3. **It's Swift 5.10.1.** Current Swift is 6.x; nixpkgs lags. Fine for the
   FFI exercise (C interop hasn't moved), not fine if you want Swift 6
   concurrency-mode anything.
4. The `glibc not found for 'aarch64-unknown-linux-gnu'` warning from swiftc
   is cosmetic — binaries build and run.

If any of this rots (nixpkgs bumping swift is the likely trigger), and fixing
it isn't fun: delete this folder and let the justfile's swift.org pointer be
the answer again. This README doubles as the tombstone inscription.
