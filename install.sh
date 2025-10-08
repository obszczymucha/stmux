#!/usr/bin/env bash
set -euo pipefail

STMUX_CONFIG_PATH="$HOME/.config/stmux"

stderr() {
  echo "$@" >&2
}

copy_config_file() {
  local source_file="config/$1"
  local target_file="${STMUX_CONFIG_PATH}/$1"

  if [[ -f "$target_file" ]]; then
    stderr "$target_file exists. Skipping..."
    return
  fi

  stderr "Copying $source_file to ${target_file}..."
  cp "$source_file" "$target_file"
}

function main() {
  mkdir -p "$HOME/.config/stmux" && \
    "./build.sh" && \
    copy_config_file "nvim-config.lua" && \
    copy_config_file "status.toml" && \
    sudo ln -sf "$(pwd)/target/release/stmux" /usr/local/bin/stmux && \
    tmux display-message "#[fg=#8a60ab,align=centre]stmux #[fg=#e0e0e0]installed"
}

main "$@"

