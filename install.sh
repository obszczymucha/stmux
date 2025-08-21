#!/usr/bin/env bash
set -euo pipefail

function main() {
  cargo build --release && cp config/nvim-config.lua "${TMUX_USER_HOME}/.config/stmux/" && sudo cp target/release/stmux /usr/local/bin && su - "$TMUX_USER" -c 'tmux display-message "#[fg=#8a60ab,align=centre]stmux #[fg=#e0e0e0]ready"'
}

main "$@"

