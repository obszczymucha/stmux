#!/usr/bin/env bash
WINDOW_NAME=
TMUX_USER=alien

function build_and_install() {
  echo "Building and installing..." >&2
  ./install.sh
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
      build_and_install
      echo "Back to listening..." >&2
      ;;
    *)
      ;;
  esac
}

function cleanup() {
  su - "$TMUX_USER" -c "tmux rename-window ""$WINDOW_NAME"" && tmux set automatic-rename on"
}

function main() {
  WINDOW_NAME=$(su - "$TMUX_USER" -c "tmux display-message -p '#W'")
  su - "$TMUX_USER" -c "tmux rename-window listener"
  listen
}

trap cleanup EXIT SIGINT
main "$@"

