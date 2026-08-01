#!/usr/bin/env bash
# Claude Code status line
# Segments: cwd | branch | model [effort] | ctx X% | 5h: X% | 7d: X%

input=$(cat)

# jq with PATH fallback (GUI launches don't inherit shell PATH)
JQ=$(command -v jq || true)
for cand in /opt/homebrew/bin/jq /usr/local/bin/jq /usr/bin/jq; do
  [ -n "$JQ" ] && break
  [ -x "$cand" ] && JQ="$cand"
done
[ -z "$JQ" ] && { printf '%b\n' '\033[31mstatusline: jq not found\033[0m'; exit 0; }

cwd=$(echo "$input"    | "$JQ" -r '.cwd // empty')
model=$(echo "$input"  | "$JQ" -r '.model.display_name // empty')
effort=$(echo "$input" | "$JQ" -r '.effort.level // empty')
remaining=$(echo "$input" | "$JQ" -r '.context_window.remaining_percentage // empty')
five_pct=$(echo "$input"  | "$JQ" -r '.rate_limits.five_hour.used_percentage // empty')
week_pct=$(echo "$input"  | "$JQ" -r '.rate_limits.seven_day.used_percentage // empty')

branch=""
[ -n "$cwd" ] && branch=$(git -C "$cwd" --no-optional-locks symbolic-ref --short HEAD 2>/dev/null)

dim='\033[2m'; bright='\033[1m'; yellow='\033[33m'; red='\033[31m'
green='\033[32m'; cyan='\033[36m'; reset='\033[0m'
sep="${dim} | ${reset}"

line="${dim}${cwd}${reset}"
[ -n "$branch" ] && line="${line}${sep}${bright}${cyan}${branch}${reset}"

if [ -n "$model" ]; then
  seg="${dim}${model}${reset}"
  [ -n "$effort" ] && [ "$effort" != "medium" ] && seg="${seg} ${dim}[${effort}]${reset}"
  line="${line}${sep}${seg}"
fi

if [ -n "$remaining" ]; then
  r=$(printf '%.0f' "$remaining")
  # Red = past the ~65%-used handoff mark, not routine work.
  if   [ "$r" -gt 35 ]; then c="${green}"
  elif [ "$r" -ge 20 ]; then c="${yellow}"
  else c="${red}"; fi
  line="${line}${sep}${c}ctx: ${r}%${reset}"
fi

if [ -n "$five_pct" ]; then
  u=$(printf '%.0f' "$five_pct")
  [ "$u" -ge 70 ] && c="${bright}${yellow}" || c="${dim}"
  line="${line}${sep}${c}5h: ${u}%${reset}"
fi

if [ -n "$week_pct" ]; then
  w=$(printf '%.0f' "$week_pct")
  [ "$w" -ge 70 ] && c="${bright}${yellow}" || c="${dim}"
  line="${line}${sep}${c}7d: ${w}%${reset}"
fi

# Per-model token/cost (main session + subagents) — fail-open, never breaks the line.
# The native binary is preferred when the host carries one (rust-port): same
# stdin/stdout contract, same shared signature cache, no interpreter startup.
# R6 CUTOVER: the `node statusline-usage.mjs` fallback that stood beside it is
# gone with the runtime it needed. The leg is silenced and optional — a
# statusline must never be the reason a prompt fails to render, so a host with
# no binary simply renders the line without the usage segment.
usage_seg=""
SELF_DIR=$(dirname "${BASH_SOURCE[0]}")
for bee_bin in "$SELF_DIR/../bee" "$SELF_DIR/../bee.exe" "$SELF_DIR/../../bee" "$SELF_DIR/../../bee.exe"; do
  if [ -x "$bee_bin" ]; then
    usage_seg=$(echo "$input" | "$bee_bin" dev statusline 2>/dev/null)
    break
  fi
done
[ -n "$usage_seg" ] && line="${line}\n${yellow}${usage_seg}${reset}"

printf '%b\n' "$line"
