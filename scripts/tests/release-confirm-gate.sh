#!/usr/bin/env bash
# scripts/tests/release-confirm-gate.sh — regression test for release.sh --no-test gate
#
# Asserts the three paths of the --no-test confirm gate:
#   (1) --no-test alone on a TTY prints all four checklist items and exits 1
#       before the authorization check is reached.
#   (2) --no-test --confirm on a TTY prints no checklist refusal and reaches
#       the authorization failure.
#   (3) --no-test with no TTY prints the checklist and reaches the authorization
#       failure.
#
# All three paths run with BEE_DISPATCH_ID unset so no path can tag, commit,
# push, or bump: every path stops at or before the authorization refusal.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RELEASE_SH="$ROOT/scripts/release.sh"

[ -x "$RELEASE_SH" ] || { echo "FAIL: $RELEASE_SH not found or not executable" >&2; exit 1; }

# Ensure BEE_DISPATCH_ID is unset for the test run
unset BEE_DISPATCH_ID

INITIAL_HEAD="$(git -C "$ROOT" rev-parse HEAD)"

# Clean up any test tag on exit if ever created
cleanup() {
  git -C "$ROOT" tag -d v99.99.99 >/dev/null 2>&1 || true
}
trap cleanup EXIT

# -----------------------------------------------------------------------------
# Path 1: --no-test alone on a TTY
# Uses `script -qec` to allocate a pty.
# Must print all 4 checklist items, print refusal, and exit 1 before authorization.
# -----------------------------------------------------------------------------
echo "Testing Path 1: --no-test on TTY without --confirm..."
set +e
OUT_1="$(script -qec "bash \"$RELEASE_SH\" 99.99.99 --no-test" /dev/null 2>&1)"
STATUS_1=$?
set -e

if [ "$STATUS_1" -ne 1 ]; then
  echo "FAIL Path 1: expected exit 1, got $STATUS_1" >&2
  exit 1
fi

for item in \
  "1. The declared test suite was run and passed on the base commit." \
  "2. No unverified code changes exist in the release commit or working tree." \
  "3. The published release tag and GitHub binaries cannot be retracted once pushed." \
  "4. The operator accepts full responsibility for releasing unverified code."
do
  if ! printf '%s\n' "$OUT_1" | grep -Fq "$item"; then
    echo "FAIL Path 1: missing checklist item: $item" >&2
    exit 1
  fi
done

if ! printf '%s\n' "$OUT_1" | grep -q -- "--no-test requires --confirm on interactive runs"; then
  echo "FAIL Path 1: missing interactive refusal message" >&2
  exit 1
fi

if printf '%s\n' "$OUT_1" | grep -q "release authorization required"; then
  echo "FAIL Path 1: reached authorization check when it should have exited at option-parse gate" >&2
  exit 1
fi

# -----------------------------------------------------------------------------
# Path 2: --no-test --confirm on a TTY
# Uses `script -qec` to allocate a pty.
# Must print no checklist refusal and reach authorization failure.
# -----------------------------------------------------------------------------
echo "Testing Path 2: --no-test --confirm on TTY..."
set +e
OUT_2="$(script -qec "bash \"$RELEASE_SH\" 99.99.99 --no-test --confirm" /dev/null 2>&1)"
STATUS_2=$?
set -e

if [ "$STATUS_2" -ne 1 ]; then
  echo "FAIL Path 2: expected exit 1, got $STATUS_2" >&2
  exit 1
fi

if printf '%s\n' "$OUT_2" | grep -q -- "--no-test requires --confirm"; then
  echo "FAIL Path 2: unexpected checklist refusal printed" >&2
  exit 1
fi

if ! printf '%s\n' "$OUT_2" | grep -q "release authorization required"; then
  echo "FAIL Path 2: did not reach authorization check" >&2
  exit 1
fi

if ! printf '%s\n' "$OUT_2" | grep -q "nothing was changed"; then
  echo "FAIL Path 2: authorization check did not state 'nothing was changed'" >&2
  exit 1
fi

# -----------------------------------------------------------------------------
# Path 3: --no-test with no TTY
# Runs non-interactively (stderr redirected to file).
# Must print the checklist and reach authorization failure.
# -----------------------------------------------------------------------------
echo "Testing Path 3: --no-test with no TTY..."
TMP_3="$(mktemp)"
set +e
bash "$RELEASE_SH" 99.99.99 --no-test >"$TMP_3" 2>&1
STATUS_3=$?
set -e
OUT_3="$(cat "$TMP_3")"
rm -f "$TMP_3"

if [ "$STATUS_3" -ne 1 ]; then
  echo "FAIL Path 3: expected exit 1, got $STATUS_3" >&2
  exit 1
fi

for item in \
  "1. The declared test suite was run and passed on the base commit." \
  "2. No unverified code changes exist in the release commit or working tree." \
  "3. The published release tag and GitHub binaries cannot be retracted once pushed." \
  "4. The operator accepts full responsibility for releasing unverified code."
do
  if ! printf '%s\n' "$OUT_3" | grep -Fq "$item"; then
    echo "FAIL Path 3: missing checklist item: $item" >&2
    exit 1
  fi
done

if ! printf '%s\n' "$OUT_3" | grep -q "proceeding unconfirmed"; then
  echo "FAIL Path 3: missing 'proceeding unconfirmed' message" >&2
  exit 1
fi

if printf '%s\n' "$OUT_3" | grep -q -- "--no-test requires --confirm"; then
  echo "FAIL Path 3: unexpected refusal on non-interactive run" >&2
  exit 1
fi

if ! printf '%s\n' "$OUT_3" | grep -q "release authorization required"; then
  echo "FAIL Path 3: did not reach authorization check" >&2
  exit 1
fi

if ! printf '%s\n' "$OUT_3" | grep -q "nothing was changed"; then
  echo "FAIL Path 3: authorization check did not state 'nothing was changed'" >&2
  exit 1
fi

# -----------------------------------------------------------------------------
# Safety verification across all paths
# Ensure no tag, commit, push, or mutation occurred.
# -----------------------------------------------------------------------------
if git -C "$ROOT" rev-parse -q --verify "refs/tags/v99.99.99" >/dev/null; then
  echo "FAIL: tag v99.99.99 was created" >&2
  exit 1
fi

CURRENT_HEAD="$(git -C "$ROOT" rev-parse HEAD)"
if [ "$CURRENT_HEAD" != "$INITIAL_HEAD" ]; then
  echo "FAIL: HEAD changed during test ($INITIAL_HEAD -> $CURRENT_HEAD)" >&2
  exit 1
fi

echo "All 3 release confirm gate paths verified successfully."
