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
