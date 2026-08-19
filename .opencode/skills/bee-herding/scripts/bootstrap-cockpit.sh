#!/usr/bin/env bash
# bootstrap-cockpit.sh - builds the D13 cockpit/runtime layout for the
# bee-herding control loop in a herdr workspace, rooted at the MAIN checkout
# (never a worktree - D13/D17/D21).
#
# A fresh workspace ends at exactly 3 tabs / 5 panes: the workspace's own
# pre-existing root tab+pane (untouched, never repurposed), the cockpit tab
# (chat / dispatch / merge), and the runtime tab (one pane to start, filled
# up to four by the dispatch loop later). No pane this script creates is
# ever labelled - dispatch and merge name themselves on first run (D17); a
# label set from outside would describe intent, not reality.
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
# stale stop file causes, see below).
#
# The stop file is resolved against --main-root, never against this
# script's own invoker cwd (the human's shell, which need not be main-root):
# `bee herding control-loop`'s panes run with --cwd main-root, so anchoring
# here too is what keeps the stale-stop-file guard below and the loop's own
# check talking about the same file. `bee herding control-loop` is also
# started with this same --main-root, for the same reason.
#
# Not idempotent by accident: before building anything, this script refuses
# if a pane already carries the label `dispatch` anywhere in the target
# workspace - that label is only ever set by a live dispatch loop naming
# itself (D17), so its presence means a dispatch loop is already polling
# this workspace's backlog and a second one would double-poll it.
#
# The bee binary is resolved from --main-root ($MAIN_ROOT/.bee/bin/bee),
# never from THIS script's own location. control-loop.sh (its predecessor)
# resolved itself from BASH_SOURCE because it lived under one of two skill
# roots - `.claude/` (Claude Code) and `.agents/` (Codex) - and hardcoding
# either one aborted every run under the other; the vendored bee binary
# sidesteps that split entirely, since main-root's .bee/bin/bee is the one
# copy both runtimes already share.
#
# Usage:
#   bootstrap-cockpit.sh --workspace ID --main-root PATH [--no-start] [--dry-run]
#
#   --workspace ID     Required. The herdr workspace to build the layout in.
#   --main-root PATH   Required. Absolute path to the MAIN checkout.
#   --no-start         Build the layout only; launch no loop.
#   --dry-run          Print the herdr commands that would run; execute
#                      nothing (no workspace, tab, pane, or agent changes).

set -u

WORKSPACE=""
MAIN_ROOT=""
NO_START=0
DRY_RUN=0

usage() {
  cat <<'EOF'
Usage: bootstrap-cockpit.sh --workspace ID --main-root PATH [--no-start] [--dry-run]

  --workspace ID     Required. The herdr workspace to build the layout in.
  --main-root PATH   Required. Absolute path to the MAIN checkout - becomes
                      the cwd of every tab and pane this script creates.
                      `bee worktree new`/`bee worktree merge` both refuse to
                      run from inside a linked worktree, so every control
                      pane must be rooted here, never in a worktree.
  --no-start         Build the layout only; launch no agent.
  --dry-run          Print the herdr commands that would run; execute
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

if [ -z "$WORKSPACE" ]; then
  echo "bootstrap-cockpit.sh: --workspace ID is required" >&2
  usage >&2
  exit 1
fi

if [ -z "$MAIN_ROOT" ]; then
  echo "bootstrap-cockpit.sh: --main-root PATH is required - \`bee worktree new\`/\`bee worktree merge\` both refuse to run from inside a linked worktree, so every pane this script creates must be rooted at the MAIN checkout; without it the dispatch loop would fail every iteration while dutifully continuing" >&2
  usage >&2
  exit 1
fi

fail() {
  echo "bootstrap-cockpit.sh: $1" >&2
  exit 1
}

# Anchored at --main-root, not at this script's own invoker cwd (see header
# comment) - the same file `bee herding control-loop`'s panes check, since
# those panes run with --cwd main-root too.
STOP_FILE="$MAIN_ROOT/.bee/tmp/bee-herding.stop"

if [ -f "$STOP_FILE" ]; then
  echo "bootstrap-cockpit.sh: refusing to start - stop file present at $STOP_FILE; starting a loop that a stale stop file would immediately kill is the same silent-stall class of bug as a missing --main-root. Remove the stop file first if that is really what you want." >&2
  exit 1
fi

# Resolved from --main-root, not from THIS script's own location: the
# `control-loop` verb now lives inside the vendored bee binary
# ($MAIN_ROOT/.bee/bin/bee), the one copy both skill roots - `.claude/`
# (Claude Code) and `.agents/` (Codex) - already share, so there is no
# per-runtime script path left to resolve.
BEE_BIN="$MAIN_ROOT/.bee/bin/bee"

if [ "$DRY_RUN" -eq 1 ]; then
  echo "herdr tab create --workspace $WORKSPACE --cwd $MAIN_ROOT --label cockpit --no-focus"
  echo "herdr pane split <cockpit_chat_pane> --direction right --cwd $MAIN_ROOT --no-focus"
  echo "herdr pane split <cockpit_dispatch_pane> --direction down --cwd $MAIN_ROOT --no-focus"
  echo "herdr tab create --workspace $WORKSPACE --cwd $MAIN_ROOT --label runtime --no-focus"
  if [ "$NO_START" -eq 0 ]; then
    # D11: only the DISPATCH loop is started. The merge pane is created but
    # no merge loop runs in it - merge is an owner gesture, run single-shot.
    echo "herdr pane run <cockpit_dispatch_pane> \"'$BEE_BIN' herding control-loop --role dispatch --main-root '$MAIN_ROOT'\""
    echo "# (no merge loop started - D11: merge is a single-shot owner gesture, run in the merge pane on request)"
  fi
  echo "bootstrap-cockpit.sh: dry-run - no workspace, tab, pane, or agent changes were made"
  exit 0
fi

# json_result <dotted.path.under.result> - reads one herdr JSON response on
# stdin, prints the value at that path under .result, or fails loudly
# (surfacing herdr's own .error.message) if the call did not succeed.
json_result() {
  "$MAIN_ROOT/.bee/bin/bee" herding herdr-result "$1" --context bootstrap-cockpit.sh
}

# find_dispatch_pane - reads a `herdr pane list` response on stdin and
# prints the pane_id of the first pane labelled `dispatch` anywhere in the
# workspace, or nothing if there is none. That label is only ever set by a
# live dispatch loop naming itself (D17) - never by this script - so its
# presence means a dispatch loop for this workspace is already running.
# Silent (never fails the script) on any parse trouble: idempotency is a
# refuse-if-sure check, not a reason to block a bootstrap over a herdr
# response shape mismatch.
find_dispatch_pane() {
  "$MAIN_ROOT/.bee/bin/bee" herding herdr-pane-id --label dispatch
}

# Refuse when a dispatch loop already owns this workspace - see header
# comment and find_dispatch_pane above. Read-only (`pane list`), so this
# runs before anything is created.
EXISTING_DISPATCH_JSON=$(herdr pane list --workspace "$WORKSPACE") || fail "herdr pane list --workspace $WORKSPACE failed (idempotency check)"
EXISTING_DISPATCH_PANE=$(printf '%s' "$EXISTING_DISPATCH_JSON" | find_dispatch_pane)
if [ -n "$EXISTING_DISPATCH_PANE" ]; then
  fail "refusing to start - a pane labelled 'dispatch' already exists in workspace $WORKSPACE (pane $EXISTING_DISPATCH_PANE); bootstrap is not idempotent and a second run would start a second dispatch loop polling the same backlog.
  If that loop is still running, stop it first: create the stop file at $STOP_FILE and let it exit.
  If it is already stopped or dead, the label is simply left over - a label is pane metadata that outlives the process that set it. Clear it with 'herdr pane close $EXISTING_DISPATCH_PANE' or 'herdr pane rename $EXISTING_DISPATCH_PANE --clear', and remove the stop file if you created one. Stopping alone is NOT enough to get past this check."
fi

# The cockpit tab: chat is its root pane, created directly by `tab create`
# (never a repurposed pre-existing tab). Splitting right then splitting the
# right pane down yields chat / dispatch / merge (D13); every call carries
# --cwd main-root and no --label, so none of the three panes is named by
# this script.
COCKPIT_JSON=$(herdr tab create --workspace "$WORKSPACE" --cwd "$MAIN_ROOT" --label cockpit --no-focus) || fail "herdr tab create --label cockpit failed"
CHAT_PANE=$(printf '%s' "$COCKPIT_JSON" | json_result root_pane.pane_id) || exit 1

DISPATCH_JSON=$(herdr pane split "$CHAT_PANE" --direction right --cwd "$MAIN_ROOT" --no-focus) || fail "herdr pane split (dispatch) failed"
DISPATCH_PANE=$(printf '%s' "$DISPATCH_JSON" | json_result pane.pane_id) || exit 1

MERGE_JSON=$(herdr pane split "$DISPATCH_PANE" --direction down --cwd "$MAIN_ROOT" --no-focus) || fail "herdr pane split (merge) failed"
MERGE_PANE=$(printf '%s' "$MERGE_JSON" | json_result pane.pane_id) || exit 1

# The runtime tab: one pane to start (its own root pane, rooted at
# main-root), filled up to D5's cap of four by the dispatch loop later.
RUNTIME_JSON=$(herdr tab create --workspace "$WORKSPACE" --cwd "$MAIN_ROOT" --label runtime --no-focus) || fail "herdr tab create --label runtime failed"
RUNTIME_TAB=$(printf '%s' "$RUNTIME_JSON" | json_result tab.tab_id) || exit 1

echo "bootstrap-cockpit.sh: layout built in workspace $WORKSPACE - cockpit ($CHAT_PANE chat, $DISPATCH_PANE dispatch, $MERGE_PANE merge), runtime tab $RUNTIME_TAB"

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
herdr pane run "$DISPATCH_PANE" "'$BEE_BIN' herding control-loop --role dispatch --main-root '$MAIN_ROOT'" >/dev/null || fail "could not start the dispatch loop in pane $DISPATCH_PANE"

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
