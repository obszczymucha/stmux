#!/usr/bin/env bash

main() {
   echo -e "item1\nitem2\nitem3" | fzf --expect=alt-l | {
      read -r key
      read -r selection

      if [ "$key" = "alt-l" ]; then
          echo "Kenny: $selection"
      else
          echo "Princess: $selection"
      fi
  }
}

main "$@"

