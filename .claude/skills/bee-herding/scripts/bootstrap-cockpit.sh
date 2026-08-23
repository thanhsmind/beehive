#!/usr/bin/env bash
# bootstrap-cockpit.sh - builds the D13 cockpit/runtime layout for the
# bee-herding control loop, rooted at the MAIN checkout (never a worktree -
# D13/D17/D21).
#
# Every pane action here goes through the transport-neutral `bee herding
# pane …` verbs (cockpit D2). This script therefore reads the same, and
# builds the same layout, wherever the control loop runs; nothing below
# speaks a raw pane-manager command.
#
# TRANSPORT - the one comment that names the two of them. The verbs read
# `herding.transport` from $MAIN_ROOT/.bee/config.json and act on whichever
# it names, so this script only needs the key for the ONE thing that
# genuinely differs, the origin of the chat pane:
#   - the default transport (herdr) has real workspace objects, so
#     --workspace is required and the cockpit tab's own root pane becomes
#     the chat pane (a pre-existing pane is never repurposed);
#   - tmux has no workspace object - the workspace IS the caller's session
#     (D3) - so --workspace is accepted and ignored, and the chat pane is
#     the pane this script was run from (`bee herding pane current`).
# Run with --dry-run to see exactly which of the two you will get.
#
# A fresh layout ends at the cockpit surface (chat / dispatch / merge) plus
# a runtime tab (one pane to start, filled up to four by the dispatch loop
# later). No pane this script creates is ever labelled - dispatch and merge
# name themselves on first run (D17); a label set from outside would
# describe intent, not reality.
#
# MERGE IS A GESTURE, NOT A LOOP (D11). This script starts ONLY the dispatch
# loop. The merge PANE is still created (the owner runs the single-shot merge
# gesture in it on request - `bee herding control-loop --role merge --once`,
# or the merge role via bee-herding), but no unattended merge loop is launched: the
# risk this feature most needed to shed - unattended, unsupervised merges
# into main - is retired by keeping a human present when anything lands in
# main. Graduating merge back to a loop is a later decision, on evidence.
#
# --main-root is required and becomes the cwd of every tab and pane this
# script creates: `bee worktree new`/`bee worktree merge` both refuse to run
# from inside a linked worktree, so the control panes must be rooted at the
# MAIN checkout - without this, every dispatch iteration would fail forever
# while the loop dutifully continued (the same silent-stall class of bug a
# stale stop file causes, see below). It is also passed to every pane verb,
# so the verbs resolve the same config this script read instead of guessing
# from the invoker's cwd.
#
# The stop file is resolved against --main-root, never against this
# script's own invoker cwd (the human's shell, which need not be main-root):
# `bee herding control-loop`'s panes run with --cwd main-root, so anchoring
# here too is what keeps the stale-stop-file guard below and the loop's own
# check talking about the same file. `bee herding control-loop` is also
# started with this same --main-root, for the same reason. That stop-file
# guard is this script's ONLY pre-flight; main-clean and the bypass level
# are the bootstrap ROLE's pre-flights, checked before this script runs.
#
# Not idempotent by accident: before building anything, this script refuses
# if a pane already carries the label `dispatch` - that label is only ever
# set by a live dispatch loop naming itself (D17), so its presence means a
# dispatch loop is already polling this backlog and a second one would
# double-poll it.
#
# The bee binary is resolved from --main-root ($MAIN_ROOT/.bee/bin/bee),
# never from THIS script's own location. control-loop.sh (its predecessor)
# resolved itself from BASH_SOURCE because it lived under one of two skill
# roots - `.claude/` (Claude Code) and `.agents/` (Codex) - and hardcoding
# either one aborted every run under the other; the vendored bee binary
# sidesteps that split entirely, since main-root's .bee/bin/bee is the one
# copy both runtimes already share - and it is the same binary that carries
# the pane verbs.
#
# Usage:
#   bootstrap-cockpit.sh --main-root PATH [--workspace ID] [--no-start] [--dry-run]
#
#   --main-root PATH   Required. Absolute path to the MAIN checkout.
#   --workspace ID     The workspace to build the layout in. Required on a
#                      transport that has workspace objects; ignored where
#                      the caller's own session IS the workspace.
#   --no-start         Build the layout only; launch no loop.
#   --dry-run          Print the bee verb lines that would run; execute
#                      nothing (no workspace, tab, pane, or agent changes).

set -u

WORKSPACE=""
MAIN_ROOT=""
NO_START=0
DRY_RUN=0

usage() {
  cat <<'EOF'
Usage: bootstrap-cockpit.sh --main-root PATH [--workspace ID] [--no-start] [--dry-run]

  --main-root PATH   Required. Absolute path to the MAIN checkout - becomes
                      the cwd of every tab and pane this script creates.
                      `bee worktree new`/`bee worktree merge` both refuse to
                      run from inside a linked worktree, so every control
                      pane must be rooted here, never in a worktree.
  --workspace ID     The workspace to build the layout in. Required on a
                      transport that has workspace objects; accepted and
                      ignored where the caller's own session IS the
                      workspace. Run with --dry-run to see which applies.
  --no-start         Build the layout only; launch no agent.
  --dry-run          Print the bee verb lines that would run; execute
                      nothing (no workspace, tab, pane, or agent changes).
EOF
}

# Refuse a value-taking flag with no value rather than let `shift 2` fail
# silently under `set -u` and spin the while-loop at 100% CPU forever (the
# same trailing-flag defect fixed in `bee herding control-loop`'s own flag
# parser).
need_value() {
  # $1 = flag name, $2 = number of args still on the line ($#)
  if [ "$2" -lt 2 ]; then
    echo "bootstrap-cockpit.sh: $1 requires a value" >&2
    usage >&2
    exit 1
  fi
}

while [ $# -gt 0 ]; do
  case "$1" in
    --workspace)
      need_value "$1" "$#"; WORKSPACE="$2"; shift 2
      ;;
    --main-root)
      need_value "$1" "$#"; MAIN_ROOT="$2"; shift 2
      ;;
    --no-start)
      NO_START=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "bootstrap-cockpit.sh: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [ -z "$MAIN_ROOT" ]; then
  echo "bootstrap-cockpit.sh: --main-root PATH is required - \`bee worktree new\`/\`bee worktree merge\` both refuse to run from inside a linked worktree, so every pane this script creates must be rooted at the MAIN checkout; without it the dispatch loop would fail every iteration while dutifully continuing" >&2
  usage >&2
  exit 1
fi

fail() {
  echo "bootstrap-cockpit.sh: $1" >&2
  exit 1
}

# read_transport <config-path> - prints the configured transport name, or
# nothing when the file, the object, or the key is absent. Deliberately a
# text scrape and not a parse: the answer only picks between the two shapes
# below, and anything unrecognized falls back to the default transport,
# whose --workspace demand is the loud branch.
read_transport() {
  [ -f "$1" ] || return 0
  tr -d ' \t\r\n' < "$1" \
    | sed -n 's/.*"herding":{.*"transport":"\([A-Za-z0-9_-]*\)".*/\1/p' \
    | head -n 1
}

# See the TRANSPORT comment at the top: the key decides only whether a
# workspace id is demanded and where the chat pane comes from.
case "$(read_transport "$MAIN_ROOT/.bee/config.json")" in
  tmux)
    WORKSPACE_REQUIRED=0
    CHAT_FROM_CURRENT=1
    ;;
  *)
    WORKSPACE_REQUIRED=1
    CHAT_FROM_CURRENT=0
    ;;
esac

if [ "$WORKSPACE_REQUIRED" -eq 1 ] && [ -z "$WORKSPACE" ]; then
  echo "bootstrap-cockpit.sh: --workspace ID is required on this transport - it has workspace objects, so the layout has to be built inside one of them" >&2
  usage >&2
  exit 1
fi

# Passed through to every verb that takes it. Empty on a transport whose
# workspace is the caller's own session, where the verbs resolve it.
WS_ARGS=""
if [ -n "$WORKSPACE" ]; then
  WS_ARGS="--workspace $WORKSPACE"
fi

# Anchored at --main-root, not at this script's own invoker cwd (see header
# comment) - the same file `bee herding control-loop`'s panes check, since
# those panes run with --cwd main-root too.
STOP_FILE="$MAIN_ROOT/.bee/tmp/bee-herding.stop"

if [ -f "$STOP_FILE" ]; then
  echo "bootstrap-cockpit.sh: refusing to start - stop file present at $STOP_FILE; starting a loop that a stale stop file would immediately kill is the same silent-stall class of bug as a missing --main-root. Remove the stop file first if that is really what you want." >&2
  exit 1
fi

# Resolved from --main-root, not from THIS script's own location: the
# `control-loop` verb and the pane verbs both live inside the vendored bee
# binary ($MAIN_ROOT/.bee/bin/bee), the one copy both skill roots -
# `.claude/` (Claude Code) and `.agents/` (Codex) - already share, so there
# is no per-runtime script path left to resolve.
BEE_BIN="$MAIN_ROOT/.bee/bin/bee"

if [ "$DRY_RUN" -eq 1 ]; then
  echo "$BEE_BIN herding pane-id --label dispatch --main-root $MAIN_ROOT"
  if [ "$CHAT_FROM_CURRENT" -eq 1 ]; then
    echo "$BEE_BIN herding pane current --main-root $MAIN_ROOT"
  else
    # Unquoted on purpose: WS_ARGS is empty on a transport with no
    # workspace object, and word splitting is what drops it cleanly.
    # shellcheck disable=SC2086
    echo $BEE_BIN herding pane tab-create $WS_ARGS --cwd $MAIN_ROOT --label cockpit --main-root $MAIN_ROOT
  fi
  echo "$BEE_BIN herding pane split <cockpit_chat_pane> --direction right --cwd $MAIN_ROOT --main-root $MAIN_ROOT"
  echo "$BEE_BIN herding pane split <cockpit_dispatch_pane> --direction down --cwd $MAIN_ROOT --main-root $MAIN_ROOT"
  # shellcheck disable=SC2086 # see the cockpit tab-create line above
  echo $BEE_BIN herding pane tab-create $WS_ARGS --cwd $MAIN_ROOT --label runtime --main-root $MAIN_ROOT
  if [ "$NO_START" -eq 0 ]; then
    # D11: only the DISPATCH loop is started. The merge pane is created but
    # no merge loop runs in it - merge is an owner gesture, run single-shot.
    echo "$BEE_BIN herding pane run <cockpit_dispatch_pane> \"'$BEE_BIN' herding control-loop --role dispatch --main-root '$MAIN_ROOT'\" --main-root $MAIN_ROOT"
    echo "# (no merge loop started - D11: merge is a single-shot owner gesture, run in the merge pane on request)"
  fi
  echo "bootstrap-cockpit.sh: dry-run - no workspace, tab, pane, or agent changes were made"
  exit 0
fi

# verb_result <dotted.path.under.result> - reads one pane-verb envelope on
# stdin, prints the value at that path under .result, or fails loudly
# (surfacing the envelope's own .error.message) if the call did not succeed.
verb_result() {
  "$BEE_BIN" herding result "$1" --context bootstrap-cockpit.sh
}

# Refuse when a dispatch loop already owns this cockpit - see the header
# comment. `pane-id` is read-only, so this runs before anything is created.
# A miss is a typed refusal (exit 1, error code `not_found`), which is why
# the probe is quiet and a failing probe simply means "no such pane". Any
# other trouble reads the same way on purpose: idempotency is a
# refuse-if-sure check, never a reason to block a bootstrap over a response
# shape mismatch.
EXISTING_DISPATCH_PANE=$(
  "$BEE_BIN" herding pane-id --label dispatch --main-root "$MAIN_ROOT" 2>/dev/null \
    | verb_result pane_id 2>/dev/null
) || EXISTING_DISPATCH_PANE=""

if [ -n "$EXISTING_DISPATCH_PANE" ]; then
  fail "refusing to start - a pane labelled 'dispatch' already exists (pane $EXISTING_DISPATCH_PANE); bootstrap is not idempotent and a second run would start a second dispatch loop polling the same backlog.
  If that loop is still running, stop it first: create the stop file at $STOP_FILE and let it exit.
  If it is already stopped or dead, the label is simply left over - a label is pane metadata that outlives the process that set it. Clear it with 'bee herding pane close $EXISTING_DISPATCH_PANE' or 'bee herding pane rename $EXISTING_DISPATCH_PANE --clear', and remove the stop file if you created one. Stopping alone is NOT enough to get past this check."
fi

# The chat pane. Where it comes from is the one transport-shaped choice in
# this script (see the TRANSPORT comment): a cockpit tab's own fresh root
# pane, or the pane the human ran this from. Either way it is never a
# pre-existing pane repurposed out from under someone else's work.
if [ "$CHAT_FROM_CURRENT" -eq 1 ]; then
  CHAT_PANE=$("$BEE_BIN" herding pane current --main-root "$MAIN_ROOT" | verb_result pane_id) \
    || fail "bee herding pane current failed - could not resolve the pane to use as chat"
else
  # shellcheck disable=SC2086 # WS_ARGS is an intentional flag+value pair
  CHAT_PANE=$("$BEE_BIN" herding pane tab-create $WS_ARGS --cwd "$MAIN_ROOT" --label cockpit --main-root "$MAIN_ROOT" | verb_result pane_id) \
    || fail "bee herding pane tab-create --label cockpit failed"
fi
[ -n "$CHAT_PANE" ] || fail "no chat pane id came back - refusing to split panes off an unknown pane"

# Splitting the chat pane right, then splitting that new pane down, yields
# chat / dispatch / merge (D13). Every call carries --cwd main-root and no
# label, so none of the three panes is named by this script.
DISPATCH_PANE=$("$BEE_BIN" herding pane split "$CHAT_PANE" --direction right --cwd "$MAIN_ROOT" --main-root "$MAIN_ROOT" | verb_result pane_id) \
  || fail "bee herding pane split (dispatch) failed"
[ -n "$DISPATCH_PANE" ] || fail "no dispatch pane id came back from the split"

MERGE_PANE=$("$BEE_BIN" herding pane split "$DISPATCH_PANE" --direction down --cwd "$MAIN_ROOT" --main-root "$MAIN_ROOT" | verb_result pane_id) \
  || fail "bee herding pane split (merge) failed"

# The runtime tab: one pane to start (its own root pane, rooted at
# main-root), filled up to D5's cap of four by the dispatch loop later.
# `tab-create` answers with that root pane's id on either transport, and
# that id is also the handle the dispatch loop splits the other three off.
# shellcheck disable=SC2086 # WS_ARGS is an intentional flag+value pair
RUNTIME_PANE=$("$BEE_BIN" herding pane tab-create $WS_ARGS --cwd "$MAIN_ROOT" --label runtime --main-root "$MAIN_ROOT" | verb_result pane_id) \
  || fail "bee herding pane tab-create --label runtime failed"

echo "bootstrap-cockpit.sh: layout built - cockpit ($CHAT_PANE chat, $DISPATCH_PANE dispatch, $MERGE_PANE merge), runtime tab rooted at pane $RUNTIME_PANE"

if [ "$NO_START" -eq 1 ]; then
  echo "bootstrap-cockpit.sh: --no-start - layout built, no agent launched"
  exit 0
fi

# Reachability, not a path check: control-loop is now a verb inside the
# vendored bee binary, not a standalone script, so "is it there" means "does
# invoking the verb work" rather than "does a file exist at a path".
if ! "$BEE_BIN" herding control-loop --help >/dev/null 2>&1; then
  fail "bee herding control-loop is not reachable via $BEE_BIN - layout was built but the dispatch loop was not started. Re-run onboarding (or your repo's bee-vendoring step) so .bee/bin/bee is present and up to date at $MAIN_ROOT, then re-run bootstrap-cockpit.sh."
fi

# D11: ONLY the dispatch loop is started. `pane run` types the command into
# the already-created pane and presses Enter; it does not block on the loop
# it starts. Dispatch is the low-authority half - worst case it starts work
# in an isolated worktree; nothing lands in main from it.
"$BEE_BIN" herding pane run "$DISPATCH_PANE" "'$BEE_BIN' herding control-loop --role dispatch --main-root '$MAIN_ROOT'" --main-root "$MAIN_ROOT" >/dev/null \
  || fail "could not start the dispatch loop in pane $DISPATCH_PANE"

# The merge pane ($MERGE_PANE) is intentionally left idle: merge is an owner
# GESTURE, not a loop (D11). Unattended merge is where the risk concentrated -
# it alone carries the merge-authority hard gate, the long stop-latency
# window, and the execute-agent-code-via-verify exposure - so nothing lands
# in main without a human present. Run merge single-shot in the merge pane
# when you want to retire finished worktrees, e.g.:
#   '$BEE_BIN' herding control-loop --role merge --main-root '$MAIN_ROOT' --timeout 5400 --once
# (the large --timeout is because a merge iteration runs `bee worktree merge`,
# whose wall clock is its own verify plus the shared verify-flock queue;
# killing one mid-merge leaves main holding a staged uncommitted merge, since
# bee's abort-and-prove path is a JS `finally` SIGTERM never runs.)
echo "bootstrap-cockpit.sh: dispatch loop started in pane $DISPATCH_PANE; merge pane $MERGE_PANE left idle (merge is a single-shot owner gesture, D11 - run 'bee herding control-loop --role merge --once' there on request)"
