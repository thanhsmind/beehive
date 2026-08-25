#!/usr/bin/env bash
# release.sh — the WHOLE release, in one command.
#
# Why this exists: releases 2.6.6 and 2.6.7 were committed locally and never
# tagged or pushed, so installers kept serving 2.6.5. The release commit is
# not the release — the pushed tag is what triggers release-binaries.yml,
# and the GitHub release with binaries is what installers actually consume.
#
# This script used to own only the TAIL of that (tag → push → wait → verify)
# and REFUSED unless somebody had already bumped both plugin manifests, run
# the regen chain and made the release commit by hand. That prologue lived as
# a checklist in CLAUDE.md, and a checklist walked by hand is a step that gets
# skipped. Hand it a version and it now owns the whole thing, in this order:
#
#   1. preconditions — main, gh, semver, strictly newer, tag free, tree clean
#      (every one of them refuses BEFORE a single byte is written)
#   2. bump BOTH .claude-plugin/plugin.json and .codex-plugin/plugin.json —
#      onboarding reads them as ONE tuple, so both move or neither does
#   3. `bee dev regen` — render-skill-trees, onboard --apply,
#      release-manifest --write; called, never reimplemented here
#   4. TEST GATE: the declared suite, read from .bee/config.json
#      `commands.test`, runs BEFORE anything is tagged. Tagging first is how a
#      red build becomes a published tag, and a published tag never moves.
#   5. the release commit, path-scoped to the files this script itself wrote
#   6. tag vX.Y.Z at HEAD (idempotent if the tag already points there)
#   7. push main and the tag
#   8. wait for the release-binaries workflow on that tag to go green
#   9. verify the GitHub release carries the binaries + SHA256SUMS
#
# Anything that aborts between step 2 and step 5 puts the tree back the way it
# found it, so a failed release leaves nothing half-bumped behind.
#
# Usage: scripts/release.sh                  (no bump — version from plugin.json)
#        scripts/release.sh 2.23.0           (bump → regen → test → commit → tail)
#        scripts/release.sh 2.23.0 --no-test (skips step 4, loudly; own the risk)
#        scripts/release.sh 2.23.0 -m "Release 2.23.0: one-command release"
#
# Re-running with a version that is ALREADY committed is safe and idempotent:
# it says so and drops straight into the tail, so a run that died waiting on
# CI just picks the release back up.
#
# The last line of a finished release is the `OK` line. Nothing else exits 0.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

log()  { printf '%s\n' "release  $*" >&2; }
fail() { printf '%s\n' "release  FAIL: $*" >&2; exit 1; }

PLUGIN_JSON=".claude-plugin/plugin.json"
CODEX_JSON=".codex-plugin/plugin.json"
BEE_CONFIG=".bee/config.json"
BEE_BIN=".bee/bin/bee"

# ---------- 0. arguments ----------
# No `--help`: a release is done only when the `OK` line prints, so this
# script has exactly one successful exit and never returns 0 early. The usage
# block above is the documentation.
ARG_VERSION=""
RUN_TESTS=1
COMMIT_SUBJECT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --no-test)
      RUN_TESTS=0; shift ;;
    -m|--message)
      [ $# -ge 2 ] || fail "$1 needs a commit subject — e.g. -m \"Release 2.23.0\""
      COMMIT_SUBJECT="$2"; shift 2 ;;
    -*)
      fail "unknown option $1 — usage: scripts/release.sh [VERSION] [--no-test] [-m SUBJECT]" ;;
    *)
      [ -z "$ARG_VERSION" ] \
        || fail "two versions given ($ARG_VERSION and $1) — one release at a time"
      ARG_VERSION="$1"; shift ;;
  esac
done

command -v gh >/dev/null 2>&1 || fail "gh CLI not found — install it, then \`gh auth login\`"
gh auth status >/dev/null 2>&1 || fail "gh not authenticated — run \`gh auth login\`"

# ---------- 1. version: plugin.json is the single source ----------
read_version() { sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -1; }

VERSION="$(read_version "$PLUGIN_JSON")"
[ -n "$VERSION" ] || fail "cannot read version from $PLUGIN_JSON"

# ---------- 2. preconditions ----------
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
[ "$BRANCH" = "main" ] || fail "on branch $BRANCH — releases cut from main only"

HEAD_VERSION="$(git show "HEAD:$PLUGIN_JSON" 2>/dev/null | read_version /dev/stdin || true)"

# ---------- 3. bump + regen + test + commit (only with a VERSION argument) ----------
if [ -n "$ARG_VERSION" ] && [ "$ARG_VERSION" = "$HEAD_VERSION" ]; then
  # Idempotent re-run: the release commit already exists. Never re-bump, never
  # re-commit — just resume the tail (this is the path a failed CI wait takes).
  log "version  $ARG_VERSION is already committed at HEAD — skipping bump, resuming the release tail"
elif [ -n "$ARG_VERSION" ]; then
  # Every check below refuses with zero mutations. Nothing is written until
  # all of them have passed.

  # Leading zeros are rejected on purpose: bash reads 08 as octal, and semver
  # forbids them anyway.
  [[ "$ARG_VERSION" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] \
    || fail "\"$ARG_VERSION\" is not a valid version — expected MAJOR.MINOR.PATCH (e.g. 2.23.0); nothing was changed"

  if [ "$ARG_VERSION" = "$VERSION" ]; then
    fail "$PLUGIN_JSON already says $ARG_VERSION but HEAD says ${HEAD_VERSION:-<none>} — commit or discard that pending bump first, then re-run; nothing was changed"
  fi

  # Strictly greater, field by field as numbers. `sort -V` is not used: it
  # cannot tell "greater" from "equal", and equal is exactly what must refuse.
  version_gt() {
    local IFS=. i
    # shellcheck disable=SC2206
    local -a new=($1) cur=($2)
    for i in 0 1 2; do
      [ "${new[i]}" -gt "${cur[i]}" ] && return 0
      [ "${new[i]}" -lt "${cur[i]}" ] && return 1
    done
    return 1
  }
  version_gt "$ARG_VERSION" "$VERSION" \
    || fail "$ARG_VERSION is not newer than $VERSION — a release only moves forward, and a published number never comes back; nothing was changed"

  # A tag that exists ANYWHERE is published. Today's HEAD comparison happens
  # far downstream, after files would already have been written; this refuses
  # first, so a taken version costs nothing.
  NEW_TAG="v$ARG_VERSION"
  ! git rev-parse -q --verify "refs/tags/$NEW_TAG" >/dev/null \
    || fail "$NEW_TAG already exists locally — a published tag never moves; pick a higher version. Nothing was changed"
  REMOTE_TAG="$(git ls-remote --tags origin "refs/tags/$NEW_TAG" 2>/dev/null)" \
    || fail "cannot reach origin to check whether $NEW_TAG is already published — \"cannot tell\" is not \"free\"; nothing was changed"
  [ -z "$REMOTE_TAG" ] \
    || fail "$NEW_TAG already exists on origin — a published tag never moves; pick a higher version. Nothing was changed"

  # The commit this script makes must carry the bump and the regen output and
  # NOTHING else, so any dirt at all refuses — named path by path.
  DIRT="$(git status --porcelain)"
  if [ -n "$DIRT" ]; then
    printf '%s\n' "$DIRT" >&2
    fail "working tree is not clean (paths above) — the release commit must carry only the bump and the regen output; commit or stash first. Nothing was changed"
  fi

  [ -f "$BEE_CONFIG" ] || fail "$BEE_CONFIG not found — cannot read the declared test command; nothing was changed"
  [ -x "$BEE_BIN" ] || fail "$BEE_BIN not found or not executable — cannot run the regen chain; nothing was changed"

  # ----- from here on the tree gets written; put it back on any abort -----
  BACKUP_DIR="$(mktemp -d)"
  cp "$PLUGIN_JSON" "$BACKUP_DIR/claude-plugin.json"
  cp "$CODEX_JSON" "$BACKUP_DIR/codex-plugin.json"

  restore_tree() {
    cp "$BACKUP_DIR/claude-plugin.json" "$PLUGIN_JSON" 2>/dev/null || true
    cp "$BACKUP_DIR/codex-plugin.json" "$CODEX_JSON" 2>/dev/null || true
    # The tree was verified clean above, so every tracked change since then is
    # this script's own — undoing them restores exactly what we found.
    git checkout -- . 2>/dev/null || true
    rm -rf "$BACKUP_DIR"
    # Untracked leftovers (a brand-new file the regen chain rendered) are
    # NAMED, never deleted: a sibling worker's untracked file can appear in
    # the same window, and no restore is worth eating someone else's work.
    local left
    left="$(git status --porcelain 2>/dev/null || true)"
    if [ -n "$left" ]; then
      log "warn: these paths were left behind — review them:"
      printf '%s\n' "$left" >&2
    fi
    log "restored the tree to its pre-release state"
  }
  trap restore_tree EXIT

  # Targeted edit of the version field only: everything else in the manifest —
  # description, keywords, the codex interface block — is copied byte for byte.
  bump_version_field() {
    local file="$1" new="$2" tmp
    tmp="$(mktemp)"
    awk -v new="$new" '
      !bumped && /"version"[[:space:]]*:[[:space:]]*"[^"]*"/ {
        sub(/"version"[[:space:]]*:[[:space:]]*"[^"]*"/, "\"version\": \"" new "\"")
        bumped = 1
      }
      { print }
    ' "$file" >"$tmp"
    if ! grep -q "\"version\"[[:space:]]*:[[:space:]]*\"$new\"" "$tmp"; then
      rm -f "$tmp"
      fail "could not write version $new into $file"
    fi
    cat "$tmp" >"$file"
    rm -f "$tmp"
  }

  # Both manifests or neither: onboarding reads them as one authoritative
  # tuple and a mismatch blocks it.
  bump_version_field "$PLUGIN_JSON" "$ARG_VERSION"
  bump_version_field "$CODEX_JSON" "$ARG_VERSION"
  log "bump     $VERSION -> $ARG_VERSION in $PLUGIN_JSON + $CODEX_JSON"

  # The regen chain is bee's own verb (render-skill-trees, then onboard
  # --repo-root . --apply, then release-manifest --write, stopping on the
  # first red). Call it — a reimplementation here would drift from it.
  log "regen    $BEE_BIN dev regen"
  "$BEE_BIN" dev regen >&2 \
    || fail "\`bee dev regen\` went RED — the release stops here"

  # Snapshot what to commit NOW, before the long test run: everything dirty at
  # this moment is this script's own work, because the tree was clean above.
  CHANGED=()
  while IFS= read -r -d '' entry; do
    case "${entry:0:2}" in
      R*|C*) IFS= read -r -d '' src && CHANGED+=("$src") ;;
    esac
    CHANGED+=("${entry:3}")
  done < <(git status --porcelain -z)
  [ ${#CHANGED[@]} -gt 0 ] \
    || fail "the bump and the regen chain changed no files — nothing to release"

  # ----- the test gate: a red suite tags nothing -----
  if [ "$RUN_TESTS" -eq 1 ]; then
    # Read from .bee/config.json, never hardcoded: CI reads the same field, so
    # this gate can never drift from what the project declares.
    if command -v jq >/dev/null 2>&1; then
      TEST_CMD="$(jq -r '.commands.test // empty' "$BEE_CONFIG")"
    elif command -v python3 >/dev/null 2>&1; then
      TEST_CMD="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("commands",{}).get("test",""))' "$BEE_CONFIG")"
    else
      fail "need jq or python3 to read \`commands.test\` from $BEE_CONFIG — install one, or re-run with --no-test"
    fi
    [ -n "$TEST_CMD" ] \
      || fail "no \`commands.test\` in $BEE_CONFIG — record it there, or re-run with --no-test"
    log "test     $TEST_CMD"
    bash -c "$TEST_CMD" >&2 \
      || fail "the declared test suite went RED — nothing tagged, nothing pushed; a published tag is the one thing a release can never take back"
  else
    log "warn: --no-test — the declared suite did NOT run; you are about to tag code that nothing proved"
  fi

  # ----- the release commit, path-scoped -----
  # Never `git commit -a`, and never `git add -A`: a sibling worker's in-flight
  # edit must never be swept into a release commit. Only the snapshot above.
  SUBJECT="${COMMIT_SUBJECT:-Release $ARG_VERSION}"
  git add -- "${CHANGED[@]}"
  git commit -m "$SUBJECT" -- "${CHANGED[@]}" >&2
  trap - EXIT
  rm -rf "$BACKUP_DIR"
  log "commit   $SUBJECT (${#CHANGED[@]} paths)"

  VERSION="$ARG_VERSION"
fi

TAG="v$VERSION"

# The version bump must be IN a commit, not sitting in the working tree:
# the tag points at HEAD, and HEAD is what CI builds.
git show "HEAD:$PLUGIN_JSON" | grep -q "\"version\"[[:space:]]*:[[:space:]]*\"$VERSION\"" \
  || fail "$PLUGIN_JSON at HEAD does not carry $VERSION — pass the version to this script (\`scripts/release.sh $VERSION\`) and it bumps, regens, tests and commits it for you"

if [ -n "$(git status --porcelain)" ]; then
  log "warn: working tree has uncommitted changes — they are NOT part of $TAG (tag points at HEAD)"
fi

# ---------- 4. tag (idempotent) ----------
HEAD_SHA="$(git rev-parse HEAD)"
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  TAG_SHA="$(git rev-parse "$TAG^{commit}")"
  [ "$TAG_SHA" = "$HEAD_SHA" ] || fail "$TAG already exists at $TAG_SHA, HEAD is $HEAD_SHA — a published tag never moves; bump the version instead"
  log "tag      $TAG already at HEAD"
else
  git tag "$TAG" "$HEAD_SHA"
  log "tag      $TAG -> ${HEAD_SHA:0:8}"
fi

# ---------- 5. push main + tag ----------
git push origin main
git push origin "$TAG"
log "push     main + $TAG"

# ---------- 6. wait for release-binaries on the tag ----------
# The run appears a few seconds after the tag push; poll for it, then watch.
RUN_ID=""
for _ in $(seq 1 30); do
  RUN_ID="$(gh run list --workflow release-binaries.yml --branch "$TAG" --limit 1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || true)"
  [ -n "$RUN_ID" ] && break
  sleep 5
done
[ -n "$RUN_ID" ] || fail "no release-binaries run found for $TAG after 150s — check GitHub Actions"

log "ci       watching release-binaries run $RUN_ID for $TAG"
gh run watch "$RUN_ID" --exit-status >/dev/null \
  || fail "release-binaries run $RUN_ID went RED — no release until it is green: gh run view $RUN_ID"

# ---------- 7. verify the release actually carries binaries ----------
ASSETS="$(gh release view "$TAG" --json assets --jq '.assets[].name')"
printf '%s\n' "$ASSETS" | grep -q "SHA256SUMS" || fail "$TAG release has no SHA256SUMS — assets: $ASSETS"
BIN_COUNT="$(printf '%s\n' "$ASSETS" | grep -c '^bee-' || true)"
[ "$BIN_COUNT" -ge 2 ] || fail "$TAG release has $BIN_COUNT bee-* binaries (expected >= 2) — assets: $ASSETS"

log "assets   $(printf '%s' "$ASSETS" | tr '\n' ' ')"
log "OK       bee $VERSION is live — installers now serve $TAG"
