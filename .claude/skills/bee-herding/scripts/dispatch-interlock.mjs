#!/usr/bin/env node
'use strict';

/**
 * dispatch-interlock.mjs — the hard enable-interlock the dispatch role MUST
 * clear before it builds any dispatchable set (D10, bee-herding).
 *
 * SUPERSEDED (R6a). The live path is the bee binary: `.bee/bin/bee herding interlock [--main-root PATH]`.
 * This file is retained ONLY as the fixture the Node suites
 * packages/bee/tests/test_herding{,_cli}.mjs still execute; nothing in the
 * instruction layer names it any more, and no agent should run it. It is
 * deleted together with the Node engine and those suites in R6.
 *
 * Historical usage: `<node> dispatch-interlock [--main-root PATH]`.
 *
 * Emits exactly one JSON object on stdout:
 *   {enabled, marker, main_root, reason}
 * Exit codes:
 *   0  enabled  — the owner-created enable marker exists; dispatch MAY proceed
 *   3  disabled — no enable marker; dispatch MUST build nothing this iteration
 *   1  error    — main-root could not be resolved (cannot decide safely)
 *
 * WHY THIS EXISTS (D10, measured, not predicted). The plan once reasoned that
 * while the backlog format and the dispatch role's prose disagreed, "the loop
 * selects nothing — the safe failure." That was empirically wrong: the lane
 * classifier anchors on the Status *value*, not column position, so it already
 * parses this repo's table — which is exactly why an adversarial review got
 * 8 of 8 real rows through it. The only remaining gap is prose a cold model
 * re-interprets ~1,440 times a day. Worse, this repo's ORDINARY post-exploring
 * state IS the dispatchable state: a feature that finishes exploring flips its
 * row to `in-flight` with its slug, a CONTEXT.md, no worktree and no cells —
 * every one of D1's four conditions — so exploring MANUFACTURES dispatchable
 * rows as a side effect of normal operation. Two such rows qualified the day
 * this was written, one of them installer/release work.
 *
 * So dispatch does not get to decide its own first minute from a coin flip on
 * a column header. It refuses to build a dispatchable set at all unless the
 * owner has explicitly, durably said "yes, run" by creating the marker:
 *
 *     touch <main-root>/.bee/tmp/bee-herding.enable
 *
 * Removing the file disables dispatch again at the next iteration boundary.
 * This is the ONE gate every other defect needs the loop already running to
 * reach; this one decides whether it runs at all.
 *
 * The marker lives beside the stop marker under .bee/tmp/ (already gitignored,
 * already this feature's home for control-plane gestures). It is an OWNER
 * gesture, never created by any agent or by this script — this script only
 * ever reads it.
 */

import { existsSync } from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const ENABLE_BASENAME = 'bee-herding.enable';

function parseArgs(argv) {
  const args = argv.slice(2);
  let mainRoot = null;
  for (let i = 0; i < args.length; i += 1) {
    if (args[i] === '--main-root') {
      mainRoot = args[i + 1] ?? null;
      i += 1;
    }
  }
  return { mainRoot };
}

// Resolve the MAIN checkout root the same way bootstrap's §1 does: the shared
// .git common dir, correct whether invoked from main or a linked worktree.
// An explicit --main-root always wins.
function resolveMainRoot(explicit) {
  if (explicit) return explicit;
  try {
    const gitCommonDir = execFileSync(
      'git',
      ['rev-parse', '--path-format=absolute', '--git-common-dir'],
      { encoding: 'utf8' },
    ).trim();
    if (!gitCommonDir) return null;
    // strip a trailing "/.git" (or ".git") to get the worktree root
    return path.dirname(gitCommonDir);
  } catch {
    return null;
  }
}

function emit(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

function main() {
  const { mainRoot: explicit } = parseArgs(process.argv);
  const mainRoot = resolveMainRoot(explicit);

  if (!mainRoot) {
    emit({
      enabled: false,
      marker: null,
      main_root: null,
      reason: 'could not resolve the MAIN checkout root (no --main-root given and `git rev-parse --git-common-dir` failed) — refusing to enable dispatch',
    });
    process.exit(1);
  }

  const marker = path.join(mainRoot, '.bee', 'tmp', ENABLE_BASENAME);

  if (existsSync(marker)) {
    emit({
      enabled: true,
      marker,
      main_root: mainRoot,
      reason: `owner enable marker present (${marker}) — dispatch may build a dispatchable set this iteration`,
    });
    process.exit(0);
  }

  emit({
    enabled: false,
    marker,
    main_root: mainRoot,
    reason: `no owner enable marker at ${marker} — dispatch MUST NOT build a dispatchable set (D10). The owner enables the loop with: touch ${marker}`,
  });
  process.exit(3);
}

main();
