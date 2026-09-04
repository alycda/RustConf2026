#!/usr/bin/env bash
# Print the python to use: the repo-local venv (`just setup-python`) if it
# exists, whatever its layout, else python3.
#
# bin/ is the POSIX venv layout (and WSL2's); Scripts/ is what a native
# Windows python builds, probed with and without the .exe suffix because Git
# Bash's -x test is not consistent about executable extensions. The one home
# for this rule: scripts/self-check.sh, both justfiles and the Verify
# workflow call it rather than each carrying a copy that lags the others.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
for cand in "$root/.venv/bin/python" \
            "$root/.venv/Scripts/python.exe" \
            "$root/.venv/Scripts/python"; do
  if [ -x "$cand" ]; then
    printf '%s\n' "$cand"
    exit 0
  fi
done
printf '%s\n' python3
