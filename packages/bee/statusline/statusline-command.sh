#!/usr/bin/env bash
# Claude Code status line
# Segments: cwd | branch | model [effort] | ctx X% | 5h: X% | 7d: X%

input=$(cat)
[ -z "${input//[[:space:]]/}" ] && exit 0

# jq with PATH fallback (GUI launches don't inherit shell PATH)
JQ=$(command -v jq || true)
for cand in /opt/homebrew/bin/jq /usr/local/bin/jq /usr/bin/jq; do
  [ -n "$JQ" ] && break
  [ -x "$cand" ] && JQ="$cand"
done
[ -z "$JQ" ] && exit 0

# Unparseable payload check: fail-open, exit silently without error
echo "$input" | "$JQ" empty 2>/dev/null || exit 0

cwd=$(echo "$input"    | "$JQ" -r '.cwd // empty' 2>/dev/null)
model=$(echo "$input"  | "$JQ" -r '.model.display_name // empty' 2>/dev/null)
effort=$(echo "$input" | "$JQ" -r '.effort.level // empty' 2>/dev/null)
remaining=$(echo "$input" | "$JQ" -r '.context_window.remaining_percentage // empty' 2>/dev/null)
five_pct=$(echo "$input"  | "$JQ" -r '.rate_limits.five_hour.used_percentage // empty' 2>/dev/null)
week_pct=$(echo "$input"  | "$JQ" -r '.rate_limits.seven_day.used_percentage // empty' 2>/dev/null)

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
  r=$(printf '%.0f' "$remaining" 2>/dev/null)
  # Red = past the ~65%-used handoff mark, not routine work.
  if   [ "$r" -gt 35 ] 2>/dev/null; then c="${green}"
  elif [ "$r" -ge 20 ] 2>/dev/null; then c="${yellow}"
  else c="${red}"; fi
  line="${line}${sep}${c}ctx: ${r}%${reset}"
fi

if [ -n "$five_pct" ]; then
  u=$(printf '%.0f' "$five_pct" 2>/dev/null)
  [ "$u" -ge 70 ] 2>/dev/null && c="${bright}${yellow}" || c="${dim}"
  line="${line}${sep}${c}5h: ${u}%${reset}"
fi

if [ -n "$week_pct" ]; then
  w=$(printf '%.0f' "$week_pct" 2>/dev/null)
  [ "$w" -ge 70 ] 2>/dev/null && c="${bright}${yellow}" || c="${dim}"
  line="${line}${sep}${c}7d: ${w}%${reset}"
fi

# Per-model token/cost (main session + subagents) — fail-open, never breaks the line.
# The native binary is preferred when the host carries one (rust-port): same
# stdin/stdout contract, same shared signature cache, no interpreter startup.
# R6 CUTOVER: the `node statusline-usage.mjs` fallback that stood beside it is
# gone with the runtime it needed. The leg is silenced and optional — a
# statusline must never be the reason a prompt fails to render, so a host with
# no binary simply renders the line without the usage segment.
#
# The lookup is the hook wiring's resolver in shell (onboard::hooks_wiring):
# this file is VENDORED to <repo>/.claude/, so its own directory says nothing
# about where the binary is — only the host's project directory does. The
# cheap candidates go first; `--git-common-dir` runs only after they miss,
# because a linked worktree materialises tracked files only and the vendored
# binary is untracked, so from a worktree the binary lives in the MAIN
# checkout that git-common-dir's parent names.
usage_seg=""
BEE=""
for cand in "${CLAUDE_PROJECT_DIR:-.}/.bee/bin/bee" "${CLAUDE_PROJECT_DIR:-.}/.bee/bin/bee.exe"; do
  [ -n "$BEE" ] && break
  [ -x "$cand" ] && BEE="$cand"
done
if [ -z "$BEE" ]; then
  g=$(git -C "${CLAUDE_PROJECT_DIR:-.}" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)
  if [ -n "$g" ]; then
    for cand in "$g/../.bee/bin/bee" "$g/../.bee/bin/bee.exe"; do
      [ -n "$BEE" ] && break
      [ -x "$cand" ] && BEE="$cand"
    done
  fi
fi
[ -n "$BEE" ] || BEE=$(command -v bee || true)
[ -n "$BEE" ] && usage_seg=$(echo "$input" | "$BEE" dev statusline 2>/dev/null)
[ -n "$usage_seg" ] && line="${line}\n${yellow}${usage_seg}${reset}"

printf '%b\n' "$line"
