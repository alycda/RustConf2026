#!/usr/bin/env bash
# Copy stdin to the clipboard — from a Mac shell or from inside a Linux
# devcontainer that has no clipboard tool and no display.
#
# Native tools first (pbcopy on macOS; wl-copy / xclip when a display is
# reachable). Otherwise OSC 52: the escape sequence that asks the terminal
# emulator itself to set the clipboard. The emulator runs on the host, so this
# works through docker, ssh and tmux (`set -s set-clipboard on`) with nothing
# installed on the far side. Honoured by kitty, iTerm2, WezTerm, Alacritty,
# Ghostty and VS Code's integrated terminal; a terminal that ignores it
# prints nothing and copies nothing — check with a paste.
#
#   cheat demo/cc | scripts/clip.sh
#   just -n cc 2>&1 | scripts/clip.sh        # from exercises/ex2-c-glue
set -euo pipefail
data="$(cat)"
if command -v pbcopy >/dev/null; then
    printf '%s' "$data" | pbcopy
elif [ -n "${WAYLAND_DISPLAY:-}" ] && command -v wl-copy >/dev/null; then
    printf '%s' "$data" | wl-copy
elif [ -n "${DISPLAY:-}" ] && command -v xclip >/dev/null; then
    printf '%s' "$data" | xclip -selection clipboard
else
    # OSC 52: ESC ] 52 ; c ; <base64> BEL — to the terminal, not stdout, so a
    # pipeline's stdout stays clean. Inside tmux, wrap it in a DCS passthrough
    # so tmux hands it to the outer terminal.
    b64="$(printf '%s' "$data" | base64 | tr -d '\n')"
    if [ -n "${TMUX:-}" ]; then
        printf '\033Ptmux;\033\033]52;c;%s\a\033\\' "$b64" > /dev/tty
    else
        printf '\033]52;c;%s\a' "$b64" > /dev/tty
    fi
fi
