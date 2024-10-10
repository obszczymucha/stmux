#!/usr/bin/env bash

function main() {
  cargo build --release && cp config/nvim-config.lua "/home/alien/.config/stmux/" && cp target/release/stmux /usr/local/bin && su - alien -c 'tmux display-message "#[fg=#8a60ab,align=centre]stmux #[fg=#e0e0e0]ready"'
}

main "$@"

