# Step -1: Machine Provisioning

Before anything else, your machine has to be able to run the workshop.
Everything lives in a single file, `nix/shells.nix`, reached through either
`flake.nix` or `shell.nix` — but only five tools are *required*: `rustc`,
`cargo`, `cbindgen`, a C compiler, and `just`. The one extra in the box, `cheat`, is cheatsheets for the FFI
patterns we'll hit — useful, and safe to ignore.

That is all `nix develop` gives you, on purpose. Some of the sample days link
real C libraries — a physics engine, a database, a malware scanner — behind
cargo features that are off by default, and those libraries are a far bigger
download than the workshop itself. They live in their own shell,
`nix develop .#full`, so nobody pays for them just to get to the
starting line.

Those versions are pinned. `flake.lock` names one nixpkgs revision, so this
path gives the whole room the same tool *versions* and not merely the same
tools — which is also what makes the language-track floors in step 0 true
by construction rather than something a script has to go looking for.

How the required five get onto your machine is your choice; the
[repo README](https://github.com/alycda/RustConf2026#readme) walks each
path in detail. In brief:

## The four paths

1. **macOS / Linux — Nix.** Install Nix (the
   [Determinate installer](https://install.determinate.systems) is the least
   fuss), then `nix develop` in the repo root — or `nix-shell`, if your
   installation doesn't have flakes enabled. Both land on the same pinned
   toolchain. Optionally add [direnv](https://direnv.net) + nix-direnv so the
   environment loads automatically on `cd` — that's what the repo's `.envrc`
   is for, and it picks whichever entry point your Nix supports.

2. **Windows — WSL2.** Nix doesn't run natively on Windows, but WSL2 is
   Linux. `wsl --install`, then follow the Nix path inside your distro.
   Clone the repo inside WSL, not on `/mnt/c`.

3. **Docker — VS Code devcontainer.** Nothing on your machine but Docker.
   "Reopen in Container" builds Nix + home-manager inside; first build takes
   minutes, later opens are fast.

4. **💀 Entirely manual.** Install every tool yourself with your package
   manager of choice. It works, but versions are on you.

## Why a self-check exists

The point of this workshop is FFI, not fighting your linker. The most common
way an environment *looks* fine but isn't: the compiler binary exists, but
the SDK paths behind it are broken (a macOS upgrade is the classic cause).
`command -v cc` passes; actually compiling fails.

So verification isn't "is the tool installed" — it's "does the toolchain do
its job":

```sh
just check
```

This runs `scripts/self-check.sh`, which verifies every required tool *and*
compiles and links a real C executable before declaring your machine ready. Red
rows come with the fix command inline.

Once the required rows are green, step -1 — steps 1 and 2 of the README — is done. The `○` rows belong to
the next step.
