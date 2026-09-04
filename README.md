# Steps -1 & 0: Get Your Environment Working

Before the workshop can start, your machine needs to be able to run it.
That's all this is: **step -1** provisions the machine, **step 0** picks the
language you'll pair with Rust. Nothing else happens until both work.

One file declares the tools: [`shell.nix`](./shell.nix). But not all of it is
required — the workshop needs five. The rest is my own workflow, put in
the box in case it's useful to you, and safe to ignore if it isn't.

| | Tools | Do you need it? |
|---|---|---|
| **Required** | `rustc`, `cargo`, `cbindgen`, a C compiler, `just` | Yes. This is the whole contract — `just check` verifies the first four, and `just` proves itself by running it. |
| **Recommended** | `cheat` | Cheatsheets for the FFI patterns we'll hit — `just cheats`. |

How you get the required five onto your machine is up to you. Pick one:

## Option 1: macOS / Linux — Nix

1. Install Nix. The [Determinate installer](https://install.determinate.systems)
   is the least fuss:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
   ```

   (The [upstream installer](https://nixos.org/download) works too.)

2. Open a new shell, `cd` into this repo, and run:

   ```sh
   nix-shell
   ```

   First run downloads the toolchain; after that it's instant.

3. **Optional, recommended:** install [direnv](https://direnv.net) (plus
   [nix-direnv](https://github.com/nix-community/nix-direnv)) and run
   `direnv allow` once in this directory. That's what [`.envrc`](./.envrc)
   is for — the environment then loads automatically every time you enter
   the repo, no `nix-shell` needed.

## Option 2: Windows — WSL2

Nix doesn't run natively on Windows, but WSL2 is Linux, and Linux is fine.

1. In an admin PowerShell: `wsl --install` (Ubuntu by default), then reboot
   and open your distro.
2. Follow **Option 1** from inside WSL. If the Nix installer complains about
   systemd, enable it: add `[boot]` / `systemd=true` to `/etc/wsl.conf`,
   then `wsl --shutdown` and reopen.
3. Clone the repo *inside* WSL (e.g. `~/RustConf2026`), not on `/mnt/c` —
   the Windows filesystem bridge is slow enough to hurt.

## Option 3: Docker — VS Code devcontainer

Don't want Nix (or anything) installed on your machine? The container does it
all for you.

1. Install [Docker](https://docs.docker.com/get-docker/) and
   [VS Code](https://code.visualstudio.com) with the
   [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers).
2. Open this repo in VS Code and choose **"Reopen in Container"** when
   prompted. There is more than one container to choose from, so VS Code will
   show a picker — **"FFI playground (git)"** is the standard choice;
   **"(jj)"** is the identical container with [Jujutsu](https://jj-vcs.github.io)
   added for those who prefer it. VS Code will ask for an `ANTHROPIC_API_KEY` — it's declared as a
   secret in [`.devcontainer/devcontainer.json`](./.devcontainer), and nothing
   in the workshop reads it. Leave it blank unless you want Claude Code in the
   container.
3. Wait. The first build installs Nix and
   [home-manager](https://github.com/nix-community/home-manager) inside the
   container ([`.devcontainer/`](./.devcontainer) has the details) — it takes
   a few minutes. Subsequent opens are fast.

## Option 4: 💀 Entirely Manual

No Nix. No Docker. Just you, your package manager, and consequences.

Install each tool yourself. **Required — install all five:**

| Tool | What it's for | Where to get it |
|------|---------------|-----------------|
| [rustc + cargo](https://rustup.rs) | the workshop's core: building Rust FFI libraries | rustup |
| [cbindgen](https://github.com/mozilla/cbindgen) | generating C headers from Rust | `cargo install cbindgen` (0.28 or newer — `just check` verifies) |
| C compiler | compiling/linking against your Rust libraries | Xcode CLT (`xcode-select --install`), apt/dnf `gcc` |
| [just](https://github.com/casey/just) | task runner — `just check`, `just setup-<track>` | `cargo install just`, brew, or a [release binary](https://github.com/casey/just/releases) — needs ≥ 1.31; apt/dnf ship older versions that cannot parse this repo's justfile |

**Everything else is my workflow, not your homework.** Skip the whole table
and the workshop still works:

| Tool | What it's for | Where to get it |
|------|---------------|-----------------|
| [cheat](https://github.com/cheat/cheat) | cheatsheets for the FFI patterns (`just cheats`) — recommended | brew, go install, release binaries |

You won't get the tool set for free — that's the part `shell.nix` hands you,
and here it's on you. Versions are on you either way: `shell.nix` is
unpinned (`import <nixpkgs> {}`), so every path resolves whatever nixpkgs your
channel points at. If the whole room needs identical versions, that takes a
pinned nixpkgs, not a choice of option. When your versions drift from everyone
else's, the skull emoji becomes self-explanatory. But it works.

## Step 0: Pick a language track

Machine provisioned? Then step -1 is done — step 0 is choosing which
language you'll call Rust *from* in Exercise 3. Pick **one** — you don't
need them all:

```sh
just setup-python   # repo-local venv with cffi (python3 comes from shell.nix)
just setup-swift    # Swift    (installer on macOS; pointers on Linux)
just setup-kotlin   # Kotlin/JNA — Java Native Access (JDK 17+ + kotlinc; brew on macOS, sdkman on Linux)
just setup-dart     # Dart     (brew tap on macOS; pointers on Linux)
```

Python note: after `just setup-python`, activate the venv with
`source .venv/bin/activate` so the check below sees it. (💀 manual-setup
folks: `shell.nix` isn't feeding you a `python3`, so bring your own, 3.10+.)

## Did it work?

From the repo root (inside `nix-shell`, direnv, the container, or your
hand-rolled environment):

```sh
just check
```

It verifies the required toolchain (Rust, C compiler + linker, `cbindgen`) —
including compiling and linking a real C executable, since a broken SDK path can
hide behind an installed compiler — and reports which optional language
tracks are ready. All required rows green = step -1 done; your chosen track
ready = step 0 done. The other `○` rows can stay grey forever.

Bonus points:

```sh
just           # lists all available recipes
just cheats    # cheatsheets for the FFI patterns we'll hit
```

The book is published, so you never need to build it locally.

## If it didn't work

Open an issue: https://github.com/alycda/RustConf2026/issues. Do it before
the session rather than during it — a broken environment is much cheaper to
fix the day before.

See you at step 1.
