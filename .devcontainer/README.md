# Running these devcontainers

One variant per track. Pick one — they are alternatives, not layers:

| Variant | For |
|---|---|
| `git/` | the base. Everything except a track-specific toolchain |
| `jj/` | the base plus jujutsu, and the jj cheatsheets |
| `kotlin/` | the Kotlin/JNA track — kotlinc, JDK 17, and `JNA_JAR` |
| `flutter/` | the Dart track — the Dart SDK (`dart`; the name predates dropping the Flutter SDK, which no longer builds on aarch64-linux) |

The Swift track's container arrives later in the history, with its own README;
add a row here when it does.

## With VS Code

Open the repo, accept the "Reopen in Container" prompt, and pick a variant from
the list. Nothing else to do.

## Without VS Code

Use the [devcontainer CLI](https://github.com/devcontainers/cli). It reads the
same `devcontainer.json` files, so you get the same container — you just drive
it from a terminal.

```sh
npm install -g @devcontainers/cli

devcontainer up   --workspace-folder . --config .devcontainer/git/devcontainer.json
devcontainer exec --workspace-folder . --config .devcontainer/git/devcontainer.json bash
```

**`--config` is not optional here.** The CLI looks for
`.devcontainer/devcontainer.json` or `.devcontainer.json` and this repo has
neither — only the variant directories above. Without `--config` it will
not find a configuration at all. Swap `git` for whichever variant you want, and
pass the same `--config` to every subsequent command.

### What you get, and what you don't

The CLI honours everything that builds the container: the base image, the Nix
feature, the mounts, `containerEnv` (which is how the variants select their own
`home.nix`), `remoteEnv` (the `PATH` fix that puts the home-manager profile
where tools can find it), and both `onCreateCommand` and `postStartCommand` —
so `setup.sh` and `poststart.sh` run exactly as they do under VS Code, and
home-manager installs your profile.

What it does not apply is the `customizations.vscode` block: **8 extensions and
5 editor settings**, which are meaningless without VS Code. Two consequences
worth knowing:

- Nothing loads direnv for you. Run `direnv allow` once, yourself.
- There is no rust-analyzer, no `files.exclude`. If you want an editor, run one
  outside the container against the same directory, or use `hx`, which
  home-manager installs inside it.

### The API key

The `ANTHROPIC_API_KEY` secret is prompted for by VS Code. The CLI takes a file
instead:

```sh
echo '{"ANTHROPIC_API_KEY":"sk-..."}' > /tmp/dc-secrets.json
devcontainer up --workspace-folder . \
  --config .devcontainer/git/devcontainer.json \
  --secrets-file /tmp/dc-secrets.json
```

Skip it if you are not using Claude Code in the container; the variable is
simply unset and nothing else cares.

## Why not just `docker run`?

Because `setup.sh` starts with `nix-channel --add`, and nothing in this repo
installs Nix. That comes from the `ghcr.io/devcontainers/features/nix:1`
*feature*, which the devcontainer tooling applies for you. A hand-written
`docker run` would have to reproduce that feature, and then keep doing so for
every variant config, none of which have `extends` between them. The CLI is the shortcut.

## No Docker at all?

GitHub Codespaces reads these same files, so the repo opens in a browser with
nothing installed locally. It needs a decent connection, which makes it the
opposite trade from the offline kit — but if your machine is fighting you on
workshop day, it is the fastest way to a working environment.
