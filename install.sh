#!/usr/bin/env bash
set -euo pipefail

function main() {
  local user="$USER"
  local user_home="$HOME"
  cargo build --release && cp config/nvim-config.lua "${user_home}/.config/stmux/" && sudo cp target/release/stmux /usr/local/bin && su - "$user" -c 'tmux display-message "#[fg=#8a60ab,align=centre]stmux #[fg=#e0e0e0]ready"'
}

main "$@"

