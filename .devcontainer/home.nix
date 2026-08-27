{ pkgs, lib, config, ... }:
{
  # Allow unfree packages (needed for claude-code)
  nixpkgs.config.allowUnfree = true;

  # Let home-manager manage itself
  programs.home-manager.enable = true;

  # Core tools available everywhere
  home.packages = with pkgs; [
    helix
    claude-code
    less  # bookworm-slim ships no pager; jj and git both expect one

    # rust-analyzer's toolchain floor, and ONLY that.
    #
    # shell.nix remains the workshop toolchain: inside the dev shell its
    # entries come first on PATH, so `just verify`, CI and every terminal
    # command keep using exactly the versions shell.nix pins. These are the
    # fallback for one process that never enters that shell — VS Code's
    # extension host.
    #
    # The extension host's PATH is the user nix profile plus system dirs (see
    # the devcontainer.json remoteEnv comment), and cargo lived only in the
    # dev shell — so rust-analyzer spawned `cargo metadata` and got
    # "No such file or directory (os error 2)", then fell back to spawning
    # `rustc` and got the same. The whole workspace failed to load: no
    # completion, no diagnostics, in the editor the workshop hands attendees.
    #
    # mkhl.direnv plus direnv.restart.automatic was the intended fix and
    # still helps, but it is a race — the extension host can start
    # rust-analyzer before direnv has finished evaluating `use nix`, and a
    # cold nix-direnv cache makes that window large. This makes the failure
    # impossible instead of unlikely.
    rustc
    cargo
  ];

  # preserve claude authentication and history (possibly redundant with devcontainer volume mount)
  home.activation.preserveClaude = lib.hm.dag.entryAfter ["writeBoundary"] ''
    mkdir -p $HOME/.claude
  '';

  # direnv with nix-direnv for fast flake loading
  programs.direnv = {
    enable = true;
    nix-direnv.enable = true;
  };

  # Enable bash so home-manager can add direnv hook
  programs.bash = {
    enable = true;

    # initExtra is added to .bashrc for interactive shells
    initExtra = ''
      # Source session vars for non-login shells
      if [[ ! -v __HM_SESS_VARS_SOURCED ]]; then
        . "$HOME/.nix-profile/etc/profile.d/hm-session-vars.sh"
      fi
    '';
  };

  # Set globally
  home.sessionVariables = {
    EDITOR = "hx";
    VISUAL = "code";
    # cheat config comes from shell.nix (CHEAT_CONFIG_PATH via mkShell)
  };

  home.stateVersion = "24.05";
  home.username = "root";  # Since you're running as root in the container
  home.homeDirectory = "/root";
}
