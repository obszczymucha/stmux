#!/usr/bin/env bash

main() {
  cargo build --release
  local result=$?

  if [[ "$#" -ne 1 || "$1" != "--notify" ]]; then exit; fi

  if [[ "$result" -eq 0 ]]; then
    tmux display-message "#[fg=#8a60ab,align=centre]stmux #[fg=#e0e0e0]rebuilt"
  else
    tmux display-message "#[fg=#8a60ab,align=centre]stmux #[fg=#e0e0e0]build #[fg=#e00000]error"
  fi
}

main "$@"

