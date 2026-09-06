{ pkgs, lib, ... }:
{
  imports = [ ../home.nix ];

  # Jujutsu on top of the shared profile. shell.nix's git stays available, so
  # colocated repos (jj git clone --colocate) work out of the box.
  home.packages = with pkgs; [ jujutsu ];

  # JJ_USER / JJ_EMAIL come from the devcontainer secrets prompt (see
  # ./devcontainer.json). A secret left blank arrives as an EMPTY variable, not
  # an absent one, and jj's env layer (ConfigSource::EnvOverrides) outranks
  # every config file — user, repo and workspace alike. So an empty JJ_USER
  # does not mean "unconfigured": it means an empty author name that silently
  # beats a later `jj config set --user user.name`, with nothing in the output
  # to say why the setting had no effect. Drop the empties instead.
  #
  # This is .bashrc, so it covers interactive shells — where jj is used — and
  # not VS Code's extension host, which keeps the empty variable for jjk and
  # visualjj. Filling both prompts avoids that corner entirely.
  programs.bash.initExtra = lib.mkAfter ''
    [[ -n ''${JJ_USER:-} ]] || unset JJ_USER
    [[ -n ''${JJ_EMAIL:-} ]] || unset JJ_EMAIL
  '';
}
