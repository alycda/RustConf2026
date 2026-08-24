#!/usr/bin/env bash
set -e

# Mark repo as safe, see: https://code.visualstudio.com/docs/sourcecontrol/faq#_why-is-vs-code-warning-me-that-the-git-repository-is-potentially-unsafe (because of nix)
git config --global --add safe.directory "$PWD"
