#!/usr/bin/env bash

function main() {
  cp config/nvim-config.lua "$HOME/.config/stmux/"
  cargo build --release && sudo cp target/release/stmux /usr/local/bin
}

main "$@"

