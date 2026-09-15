#!/usr/bin/env bash
# release-dirt.sh — print un-allowlisted git porcelain status entries.
#
# Reads `git status --porcelain -z` and prints the NUL-separated porcelain
# entries (keeping rename/copy pairs together) whose path is NOT exactly
# `.bee/wave-ledger.jsonl` and NOT matching `.bee/lanes/*.json`.
set -euo pipefail

is_allowlisted() {
  local p="$1"
  if [ "$p" = ".bee/wave-ledger.jsonl" ]; then
    return 0
  fi
  # shellcheck disable=SC2053
  if [[ "$p" == .bee/lanes/*.json ]]; then
    return 0
  fi
  return 1
}

while IFS= read -r -d '' entry || [ -n "$entry" ]; do
  [ -z "$entry" ] && continue
  src=""
  case "${entry:0:2}" in
    R*|C*)
      IFS= read -r -d '' src
      ;;
  esac
  path="${entry:3}"
  if is_allowlisted "$path"; then
    continue
  fi
  if [ -n "$src" ]; then
    printf '%s\0%s\0' "$entry" "$src"
  else
    printf '%s\0' "$entry"
  fi
done < <(git status --porcelain -z)
