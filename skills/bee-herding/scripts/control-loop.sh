#!/usr/bin/env bash
# control-loop.sh - bounded, stoppable poll-act loop for the bee-herding
# control agents (dispatch / merge).
#
# Each iteration invokes a FRESH headless claude session (no continuation
# across iterations): the cold start per iteration is what keeps the
# context profile flat no matter how many iterations run.
#
# STOPPING. The loop exits cleanly the moment the human's stop gesture
# appears: create the stop file at <main-root>/.bee/tmp/bee-herding.stop.
# The file is checked BOTH before and after every iteration, so a stop
# created while an iteration is running takes effect at that iteration's
# boundary rather than a full interval later. NOTE: the stop file stops this
# CONTROL loop only. It does NOT stop working agents already running in
# runtime panes - those are independent claude sessions in their own
# worktrees; stop them by closing their panes (`herdr pane close <pane_id>`)
# or talking to them directly. See SKILL.md "Stop and resume" / README.md.
#
# The stop file is resolved to an ABSOLUTE path, anchored at --main-root
# when given, else at `git rev-parse --show-toplevel`, else at the cwd this
# script happens to run from. This matters because bootstrap-cockpit.sh and
# this loop can run from different cwds (the human's shell vs. the pane's
# --cwd main-root) - a relative path would let the two silently disagree
# about which file means "stop".
#
# BOUNDED, not unbounded (D4/D12). Three independent ceilings keep a cold
# loop from running away when the world is broken:
#   - a DEFAULT iteration cap applies even when --max-iterations is omitted,
#     so no invocation is ever truly unbounded;
#   - a CONSECUTIVE-FAILURE ceiling with backoff exits the loop when the
#     iteration keeps failing (e.g. a missing `claude` binary that would
#     otherwise 127-retry forever, or a control pane wedged on a permission
#     prompt) instead of hammering it every interval; and
#   - each iteration runs under a `timeout` and, for the real claude call,
#     under a per-invocation TURN ceiling (--max-turns) so a single control
#     iteration's spend is bounded, not just its count.
# A SINGLE failed iteration is still reported and tolerated - the loop only
# gives up after --max-consecutive-failures in a row.
#
# All numeric inputs are validated as positive integers before the loop
# starts. A missing flag value, or a non-numeric one, is a hard error that
# refuses to start - never a silent spin or a hot loop.
#
# CONTROL-PANE PERMISSION SURFACE (D7-FINAL). The control panes are NOT
# started with --permission-mode bypassPermissions. They carry an ENUMERATED
# command surface (--allowedTools) sized to exactly what each role measurably
# does - dispatch creates a worktree and registers a grant; merge aborts a
# stuck merge on main, writes markers, and runs merge-with-cleanup. This is
# deliberately NOT "read-only": read-only would stall both roles every
# interval on their first write. The working agents dispatch spawns keep
# bypassPermissions - that posture is the owner's recorded accepted risk,
# see SKILL.md "Accepted risk". If either role gains a new command, its
# allowlist below must grow with it or that role will silently stall.
#
# CONFIG-DRIVEN SPAWN COMMAND (D4, i54-closeout-4). The real claude invocation
# below is config-driven, not hardcoded: an optional `.bee/config.json`
# `herding.control_command` (a JSON array of argv-token strings) replaces it
# when present. Each token may carry the placeholders {PROMPT}, {MODEL},
# {MAX_TURNS}, {ALLOWED_TOOLS}, substituted per-token (never by concatenating
# into one string and re-splitting or `eval`-ing it - that is the
# shell-injection-prone shape this design deliberately avoids). With NO
# `herding.control_command` key, the built default is BYTE-EQUIVALENT to the
# pre-D4 hardcoded command - zero behavior change without explicit config.
# See `read_command_template` and `run_iteration` below, and SKILL.md's
# "Herding runtime adapter" section for the config shape and a codex example.
# This key is independent of `herding.agent_command`, which SKILL.md's
# dispatch role (§8) reads for the WORKING agent's spawn argv - a different
# process, spawned via `herdr agent start`, not this script.
#
# Usage:
#   control-loop.sh --role dispatch|merge [--main-root PATH] [--interval N]
#                    [--timeout N] [--max-iterations N]
#                    [--max-consecutive-failures N] [--turn-ceiling N]
#                    [--once] [--command CMD]
#
#   --role dispatch|merge   Which control agent this loop drives; selects
#                           the prompt file sent to claude.
#   --main-root PATH        Absolute path to the MAIN checkout. Anchors the
#                           stop file so it means the same thing here and in
#                           bootstrap-cockpit.sh. Defaults to
#                           `git rev-parse --show-toplevel`, else cwd.
#   --interval N            Seconds between iterations (>=1). Default: 60.
#   --timeout N             Seconds before an iteration is killed and
#                           counted as a failed iteration (>=1). Default: 900.
#   --max-iterations N      Stop after N iterations (>=1). Omit to use the
#                           default cap below - the loop is never unbounded.
#   --max-consecutive-failures N
#                           Give up after N consecutive failed iterations
#                           (>=1). Default: 20.
#   --turn-ceiling N        --max-turns passed to each real claude control
#                           invocation (>=1). Default: 50.
#   --once                  Run exactly one iteration then exit. Test-only.
#   --command CMD           Test-only. Evaluated as a shell string via
#                           `bash -c`, run instead of invoking claude. Lets
#                           tests exercise the loop mechanics without
#                           spawning a real agent.

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROLE=""
MAIN_ROOT=""
INTERVAL=60
TIMEOUT=900
MAX_ITERATIONS=""
MAX_CONSECUTIVE_FAILURES=20
TURN_CEILING=50
ONCE=0
TEST_COMMAND=""

# A loop with no explicit --max-iterations is still bounded: this cap keeps a
# cold, unattended process from running literally forever if every other
# guard somehow lets it. Large enough never to bite a healthy loop.
DEFAULT_MAX_ITERATIONS=10000

# Backoff on consecutive failures is capped so it never grows without bound.
BACKOFF_CAP=600

usage() {
  cat <<'EOF'
Usage: control-loop.sh --role dispatch|merge [--main-root PATH] [--interval N] [--timeout N] [--max-iterations N] [--max-consecutive-failures N] [--turn-ceiling N] [--once] [--command CMD]

  --role dispatch|merge   Which control agent this loop drives (selects the
                          prompt file sent to claude).
  --main-root PATH        Absolute path to the MAIN checkout; anchors the
                          stop file. Defaults to `git rev-parse
                          --show-toplevel`, else cwd.
  --interval N            Seconds between iterations (>=1). Default: 60.
  --timeout N             Seconds before an iteration is killed and counted
                          as a failed iteration (>=1). Default: 900.
  --max-iterations N      Stop after N iterations (>=1). Omit for the default
                          cap; the loop is never unbounded.
  --max-consecutive-failures N
                          Give up after N consecutive failures (>=1). Def: 20.
  --turn-ceiling N        --max-turns for each real claude call (>=1). Def: 50.
  --once                  Run exactly one iteration then exit. Test-only.
  --command CMD           Test-only: evaluated as a shell string via
                          `bash -c`, run instead of invoking claude.
EOF
}

die() {
  echo "control-loop.sh: $1" >&2
  usage >&2
  exit 1
}

# need_value FLAG REMAINING - refuse a value-taking flag that has no value
# (a trailing flag, or one immediately followed by another). This is the
# fix for the trailing-flag spin: `shift 2` on a single remaining argument
# used to fail silently under `set -u` with no `set -e`, leaving $# stuck at
# 1 and the while-loop spinning at 100% CPU forever. Now it is a hard error.
need_value() {
  # $1 = flag name, $2 = number of args still on the line (i.e. $#)
  if [ "$2" -lt 2 ]; then
    die "$1 requires a value"
  fi
}

require_positive_int() {
  # $1 = flag/label, $2 = value. Rejects empty, non-numeric, and zero -
  # a zero interval is a hot loop, exactly the defect this guards against.
  case "$2" in
    ''|*[!0-9]*)
      die "$1 must be a positive integer (got: '$2')"
      ;;
  esac
  if [ "$2" -lt 1 ]; then
    die "$1 must be a positive integer (got: '$2')"
  fi
}

while [ $# -gt 0 ]; do
  case "$1" in
    --role)
      need_value "$1" "$#"; ROLE="$2"; shift 2 ;;
    --main-root)
      need_value "$1" "$#"; MAIN_ROOT="$2"; shift 2 ;;
    --interval)
      need_value "$1" "$#"; INTERVAL="$2"; shift 2 ;;
    --timeout)
      need_value "$1" "$#"; TIMEOUT="$2"; shift 2 ;;
    --max-iterations)
      need_value "$1" "$#"; MAX_ITERATIONS="$2"; shift 2 ;;
    --max-consecutive-failures)
      need_value "$1" "$#"; MAX_CONSECUTIVE_FAILURES="$2"; shift 2 ;;
    --turn-ceiling)
      need_value "$1" "$#"; TURN_CEILING="$2"; shift 2 ;;
    --command)
      need_value "$1" "$#"; TEST_COMMAND="$2"; shift 2 ;;
    --once)
      ONCE=1; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "control-loop.sh: unknown argument: $1" >&2
      usage >&2
      exit 1 ;;
  esac
done

if [ -z "$ROLE" ]; then
  die "--role dispatch|merge is required"
fi

case "$ROLE" in
  dispatch|merge) ;;
  *)
    echo "control-loop.sh: unknown role '$ROLE' (expected dispatch or merge)" >&2
    exit 1 ;;
esac

# Every numeric input is validated before the loop can start - a non-numeric
# interval used to turn the 60s loop into a hot loop (sleep errored out
# instantly and the loop continued at once). Now it refuses to start.
require_positive_int "--interval" "$INTERVAL"
require_positive_int "--timeout" "$TIMEOUT"
require_positive_int "--max-consecutive-failures" "$MAX_CONSECUTIVE_FAILURES"
require_positive_int "--turn-ceiling" "$TURN_CEILING"
if [ -n "$MAX_ITERATIONS" ]; then
  require_positive_int "--max-iterations" "$MAX_ITERATIONS"
else
  MAX_ITERATIONS="$DEFAULT_MAX_ITERATIONS"
fi

PROMPT_FILE="$SCRIPT_DIR/../references/${ROLE}-prompt.md"

# resolve_main_root - an absolute path both this loop and
# bootstrap-cockpit.sh can agree on: --main-root when given, else the repo
# root, else cwd. Never a bare relative path - the whole point is that the
# stop file means the same file regardless of the invoker's cwd.
resolve_main_root() {
  if [ -n "$MAIN_ROOT" ]; then
    printf '%s\n' "$MAIN_ROOT"
    return 0
  fi
  local top
  top="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || true)"
  if [ -n "$top" ]; then
    printf '%s\n' "$top"
    return 0
  fi
  pwd
}

STOP_FILE="$(resolve_main_root)/.bee/tmp/bee-herding.stop"

# read_command_template KEY - reads herding.<KEY> from <main-root>/.bee/config.json
# as a JSON array of argv-token strings, and prints each token on its OWN
# line. This is the shell-injection-safe read path: tokens travel as
# discrete, already-split argv elements from JSON straight into a bash array
# (one `read` per line in the caller) - never concatenated into one string
# and re-split by the shell, and never passed through `eval`. A token
# containing a newline is rejected (the line-per-token protocol cannot carry
# it) - config authors keep newlines out of individual argv tokens, which
# every real command name/flag/value already does. A missing config file, a
# missing `herding.<KEY>` key, a non-array value, or any rejected token all
# print nothing and exit 0 - the caller then falls back to its own hardcoded
# default, which is exactly today's behavior (D4: no config keys =>
# byte-equivalent command).
read_command_template() {
  local key="$1"
  local config_path
  config_path="$(resolve_main_root)/.bee/config.json"
  [ -f "$config_path" ] || return 0
  node -e '
    var NL = String.fromCharCode(10);
    var raw;
    try { raw = require("fs").readFileSync(process.argv[2], "utf8"); } catch (e) { process.exit(0); }
    var cfg;
    try { cfg = JSON.parse(raw); } catch (e) { process.exit(0); }
    var tmpl = cfg && cfg.herding && cfg.herding[process.argv[1]];
    var ok = Array.isArray(tmpl) && tmpl.length > 0 &&
      tmpl.every(function (t) { return typeof t === "string" && t.indexOf(NL) === -1; });
    if (!ok) { process.exit(0); }
    tmpl.forEach(function (t) { console.log(t); });
  ' "$key" "$config_path"
}

# substitute_placeholders CMD_ARRAY_NAME - replaces the {PROMPT}, {MODEL},
# {MAX_TURNS}, {ALLOWED_TOOLS} placeholders inside EACH element of the named
# bash array, in place, one token at a time. Per-token substitution (never a
# join-then-split) means a value containing spaces, quotes, or shell
# metacharacters (e.g. $PROMPT's free-form text) lands as the literal
# content of that one argv element and can never spill into, or be
# re-parsed as, another argument or a shell operator.
substitute_placeholders() {
  local -n arr_ref="$1"
  local i
  for i in "${!arr_ref[@]}"; do
    arr_ref[$i]="${arr_ref[$i]//\{PROMPT\}/$PROMPT}"
    arr_ref[$i]="${arr_ref[$i]//\{MODEL\}/sonnet}"
    arr_ref[$i]="${arr_ref[$i]//\{MAX_TURNS\}/$TURN_CEILING}"
    arr_ref[$i]="${arr_ref[$i]//\{ALLOWED_TOOLS\}/$ALLOWED_TOOLS}"
  done
}

# allowed_tools_for ROLE - the ENUMERATED control-pane command surface
# (D7-FINAL). Sized to exactly what each role measurably does; NOT
# "read-only" (that stalls both roles on their first write every interval),
# and NOT bypassPermissions (that is the working agents' posture, not the
# control panes'). Comma-joined for claude's --allowedTools.
allowed_tools_for() {
  case "$1" in
    dispatch)
      # herdr           : every pane/tab/agent action (§1-§8)
      # bee.mjs         : status, worktree list, cells list, AND worktree new
      #                   (+grant registration) - the write that makes this
      #                   not read-only
      # classify-lane   : §6 key-1 lane-safety script (both skill roots)
      # dispatch-interlock : §5 owner-enable gate (both skill roots)
      # git rev-parse/status/-C : §0/§4 read-only worktree state checks
      # Read            : docs/backlog.md, docs/history/<slug>/CONTEXT.md, JSON
      printf '%s' \
        'Bash(herdr:*),Bash(node .bee/bin/bee.mjs:*),Bash(node .claude/skills/bee-herding/scripts/classify-lane.mjs:*),Bash(node .agents/skills/bee-herding/scripts/classify-lane.mjs:*),Bash(node .claude/skills/bee-herding/scripts/dispatch-interlock.mjs:*),Bash(node .agents/skills/bee-herding/scripts/dispatch-interlock.mjs:*),Bash(git rev-parse:*),Bash(git status:*),Bash(git -C:*),Read'
      ;;
    merge)
      # herdr           : pane current/rename/layout/list/send-text/close
      # bee.mjs         : worktree list/status, cells list, AND worktree merge
      #                   --cleanup (the merge + cleanup writes)
      # git             : `git -C <main> rev-parse --verify MERGE_HEAD` and
      #                   `git -C <main> merge --abort` (a real write to main),
      #                   plus per-worktree status/rev-parse - broad git, by
      #                   measured necessity
      # ls/mkdir/touch  : red-stop marker check + write under .bee/tmp
      # Read            : JSON outputs, worktree state
      printf '%s' \
        'Bash(herdr:*),Bash(node .bee/bin/bee.mjs:*),Bash(git:*),Bash(ls:*),Bash(mkdir:*),Bash(touch:*),Read'
      ;;
  esac
}

echo "role=${ROLE} interval=${INTERVAL}s timeout=${TIMEOUT}s max-iterations=${MAX_ITERATIONS} max-consecutive-failures=${MAX_CONSECUTIVE_FAILURES} turn-ceiling=${TURN_CEILING}"

run_iteration() {
  if [ -n "$TEST_COMMAND" ]; then
    timeout -k 30s "${TIMEOUT}s" bash -c "$TEST_COMMAND"
    return $?
  fi

  if [ ! -f "$PROMPT_FILE" ]; then
    echo "control-loop.sh: prompt file not found: $PROMPT_FILE" >&2
    return 1
  fi

  PROMPT="$(cat "$PROMPT_FILE")"
  ALLOWED_TOOLS="$(allowed_tools_for "$ROLE")"

  # CONFIG-DRIVEN COMMAND (i54-closeout-4/D4). Optional `.bee/config.json`
  # `herding.control_command` (a JSON array of argv tokens) replaces the
  # hardcoded invocation below; absent, invalid, or non-array => the default
  # array is used, which is BYTE-EQUIVALENT to the pre-D4 hardcoded command.
  # NOT bypassPermissions (D7-FINAL): the enumerated --allowedTools surface
  # above is the control-pane posture. --max-turns bounds per-iteration spend
  # (D12). --model sonnet is D4's fixed control/working model, carried into a
  # custom template via the {MODEL} placeholder.
  local -a CMD=()
  while IFS= read -r _tok; do
    CMD+=("$_tok")
  done < <(read_command_template control_command)

  if [ "${#CMD[@]}" -eq 0 ]; then
    CMD=(claude -p "$PROMPT" --model sonnet --max-turns "$TURN_CEILING" --allowedTools "$ALLOWED_TOOLS")
  else
    substitute_placeholders CMD
  fi

  timeout -k 30s "${TIMEOUT}s" "${CMD[@]}"
  return $?
}

stop_requested() {
  [ -f "$STOP_FILE" ]
}

# Backoff sleep after a failed iteration: grows with the consecutive-failure
# count, capped at BACKOFF_CAP, so a persistently failing world is polled ever
# more slowly instead of hammered every interval.
backoff_sleep() {
  local consecutive="$1"
  local secs=$(( INTERVAL * consecutive ))
  if [ "$secs" -gt "$BACKOFF_CAP" ]; then
    secs="$BACKOFF_CAP"
  fi
  sleep "$secs"
}

count=0
consecutive_failures=0
while true; do
  # Stop check BEFORE the iteration (D5): a stop created between cycles ends
  # the loop before any further work.
  if stop_requested; then
    echo "control-loop.sh: stop file found at $STOP_FILE; exiting"
    exit 0
  fi

  if [ "$count" -ge "$MAX_ITERATIONS" ]; then
    echo "control-loop.sh: reached max-iterations ($MAX_ITERATIONS); exiting" >&2
    exit 0
  fi

  run_iteration
  rc=$?
  if [ "$rc" -eq 0 ]; then
    consecutive_failures=0
  else
    consecutive_failures=$(( consecutive_failures + 1 ))
    if [ "$rc" -eq 124 ]; then
      echo "control-loop.sh: iteration timed out after ${TIMEOUT}s; failed iteration ${consecutive_failures}/${MAX_CONSECUTIVE_FAILURES}" >&2
    else
      echo "control-loop.sh: iteration failed with exit code $rc; failed iteration ${consecutive_failures}/${MAX_CONSECUTIVE_FAILURES}" >&2
    fi
  fi

  count=$(( count + 1 ))

  # Stop check AFTER the iteration (D5): a stop created DURING the iteration
  # takes effect now, not a full interval later.
  if stop_requested; then
    echo "control-loop.sh: stop file found at $STOP_FILE; exiting"
    exit 0
  fi

  # Consecutive-failure ceiling (D4): give up rather than 127-retry a broken
  # world forever. Checked before the sleep so the exit is prompt.
  if [ "$consecutive_failures" -ge "$MAX_CONSECUTIVE_FAILURES" ]; then
    echo "control-loop.sh: $consecutive_failures consecutive failed iterations (ceiling $MAX_CONSECUTIVE_FAILURES); giving up" >&2
    exit 1
  fi

  if [ "$ONCE" -eq 1 ]; then
    exit 0
  fi

  if [ "$count" -ge "$MAX_ITERATIONS" ]; then
    echo "control-loop.sh: reached max-iterations ($MAX_ITERATIONS); exiting" >&2
    exit 0
  fi

  if [ "$consecutive_failures" -gt 0 ]; then
    backoff_sleep "$consecutive_failures"
  else
    sleep "$INTERVAL"
  fi
done
