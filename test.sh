#!/usr/bin/env bash

main() {
   echo -ne "item1\nitem2\nitem3" | fzf --no-multi --border --border-label " Select an option " --expect=alt-l --expect=alt-h | {
      read -r key; read -r selection

      if [[ "$key" == "alt-l" ]]; then
          echo "Kenny: $selection"
      elif [[ "$key" = "alt-h" ]]; then
          echo "Spenny: $selection"
      else
          echo "Princess: $selection"
      fi
  }
}

main "$@"

