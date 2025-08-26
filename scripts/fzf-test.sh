#!/usr/bin/env bash

main() {
  echo -ne "item1\nitem2\nitem3" | \
  fzf \
  --no-multi \
  --border \
  --border-label " Select an option " \
  --expect=alt-h \
  --expect=left \
  --expect=alt-l \
  --expect=right | {
    read -r key; read -r selection

    if [[ "$key" == "alt-h" || "$key" == "left" ]]; then
        echo "Kenny: $selection"
    elif [[ "$key" = "alt-l" || "$key" == "right" ]]; then
        echo "Spenny: $selection"
    else
        echo "Princess: $selection"
    fi
  }
}

main "$@"

