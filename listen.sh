#!/usr/bin/env bash

function build_and_release() {
  echo "Building and releasing..." >&2
  cargo build --release && cp config/nvim-config.lua "/home/alien/.config/stmux/" && cp target/release/stmux /usr/local/bin && su - alien -c 'tmux display-message " #[fg=#8a60ab]stmux #[default]released."'
}

function listen() {
  echo "Listening..." >&2
  # The 4 digit regex deals with temporary neovim files.
  inotifywait -mqre create,close_write,delete,move --format "%e %w%f" --exclude '/[0-9]{4}$' . | while read -r event filename; do
    local name
    name=$(echo "$filename" | sed -E 's/^\.\///g')

    if [[ "$name" != *~ ]]; then
      on_change "$event" "$name"
    fi
  done
}

function on_change() {
  local event="$1"
  local filename="$2"

  # check if the file is a lua file
  if [[ "$filename" != *.lua && "$filename" != *.rs ]]; then
    return
  fi
   
  case "$event" in
    "CLOSE_WRITE,CLOSE")
      echo "File modified: $filename" >&2
      build_and_release
      echo "Back to listening..." >&2
      ;;
    *)
      ;;
  esac
}

function main() {
  listen
}

main "$@"

