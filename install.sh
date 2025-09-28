#!/usr/bin/env bash
set -euo pipefail

function main() {
  mkdir -p "$HOME/.config/stmux"
  "./build.sh" && cp config/nvim-config.lua "$HOME/.config/stmux/" && sudo ln -sf "$(pwd)/target/release/stmux" /usr/local/bin/stmux && tmux display-message "#[fg=#8a60ab,align=centre]stmux #[fg=#e0e0e0]installed"
}

main "$@"

