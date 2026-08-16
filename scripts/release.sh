#!/usr/bin/env bash
# release.sh — the WHOLE release tail, in one command.
#
# Why this exists: releases 2.6.6 and 2.6.7 were committed locally and never
# tagged or pushed, so installers kept serving 2.6.5. The release commit is
# not the release — the pushed tag is what triggers release-binaries.yml,
# and the GitHub release with binaries is what installers actually consume.
# This script runs every step after the release commit and refuses to stop
# half way:
#
#   1. read the version from .claude-plugin/plugin.json (single source)
#   2. verify the version is committed at HEAD, on main
#   3. tag vX.Y.Z at HEAD (idempotent if the tag already points there)
#   4. push main and the tag
#   5. wait for the release-binaries workflow on that tag to go green
#   6. verify the GitHub release carries the binaries + SHA256SUMS
#
# Usage: scripts/release.sh            (version comes from plugin.json)
#        scripts/release.sh 2.6.8      (must match plugin.json, or it refuses)

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

log()  { printf '%s\n' "release  $*" >&2; }
fail() { printf '%s\n' "release  FAIL: $*" >&2; exit 1; }

command -v gh >/dev/null 2>&1 || fail "gh CLI not found — install it, then \`gh auth login\`"
gh auth status >/dev/null 2>&1 || fail "gh not authenticated — run \`gh auth login\`"

# ---------- 1. version: plugin.json is the single source ----------
PLUGIN_JSON=".claude-plugin/plugin.json"
VERSION="$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$PLUGIN_JSON" | head -1)"
[ -n "$VERSION" ] || fail "cannot read version from $PLUGIN_JSON"

if [ "${1:-}" != "" ] && [ "$1" != "$VERSION" ]; then
  fail "asked to release $1 but $PLUGIN_JSON says $VERSION — bump plugin.json and commit first"
fi
TAG="v$VERSION"

# ---------- 2. preconditions ----------
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
[ "$BRANCH" = "main" ] || fail "on branch $BRANCH — releases cut from main only"

# The version bump must be IN a commit, not sitting in the working tree:
# the tag points at HEAD, and HEAD is what CI builds.
git show "HEAD:$PLUGIN_JSON" | grep -q "\"version\"[[:space:]]*:[[:space:]]*\"$VERSION\"" \
  || fail "$PLUGIN_JSON at HEAD does not carry $VERSION — commit the version bump first"

if [ -n "$(git status --porcelain)" ]; then
  log "warn: working tree has uncommitted changes — they are NOT part of $TAG (tag points at HEAD)"
fi

# ---------- 3. tag (idempotent) ----------
HEAD_SHA="$(git rev-parse HEAD)"
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  TAG_SHA="$(git rev-parse "$TAG^{commit}")"
  [ "$TAG_SHA" = "$HEAD_SHA" ] || fail "$TAG already exists at $TAG_SHA, HEAD is $HEAD_SHA — a published tag never moves; bump the version instead"
  log "tag      $TAG already at HEAD"
else
  git tag "$TAG" "$HEAD_SHA"
  log "tag      $TAG -> ${HEAD_SHA:0:8}"
fi

# ---------- 4. push main + tag ----------
git push origin main
git push origin "$TAG"
log "push     main + $TAG"

# ---------- 5. wait for release-binaries on the tag ----------
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

# ---------- 6. verify the release actually carries binaries ----------
ASSETS="$(gh release view "$TAG" --json assets --jq '.assets[].name')"
printf '%s\n' "$ASSETS" | grep -q "SHA256SUMS" || fail "$TAG release has no SHA256SUMS — assets: $ASSETS"
BIN_COUNT="$(printf '%s\n' "$ASSETS" | grep -c '^bee-' || true)"
[ "$BIN_COUNT" -ge 2 ] || fail "$TAG release has $BIN_COUNT bee-* binaries (expected >= 2) — assets: $ASSETS"

log "assets   $(printf '%s' "$ASSETS" | tr '\n' ' ')"
log "OK       bee $VERSION is live — installers now serve $TAG"
