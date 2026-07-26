// guards.mjs — gate guard, reservation guard, privacy/scout read guard,
// and bash write-target extraction. Used by the write-guard hook and helpers.

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { findConflicts, findSessionConflicts, reservationsPath, isHardConflict } from './reservations.mjs';
import { readConfig, resolveContext, resolvePipeline } from './state.mjs';
// xwh-4: cross-worktree foreign-hold consultation. worktree-holds.mjs imports
// only fsutil/lock/reservations.mjs — no cycle (same discipline cells.mjs's
// own findForeignHolds import documents).
import { findForeignHolds, holdsStoreCorrupt } from './worktree-holds.mjs';
// wcg-1: the shared nested/companion-checkout detection primitive combines
// isConcurrentMode() (claims.mjs, already built) with a new structural check.
// No cycle — claims.mjs imports only fsutil/lock/decisions, never guards.mjs.
// multisession-native-21 (D2/D3, invariant-15 groundwork): the workspace
// single-write-owner registry (msn-19) and session heartbeat/workspace_id
// (msn-19's session stamping) — both structurally isolated leaf modules, no
// cycle back into guards.mjs (workspace-store.mjs imports only fs/path/
// fsutil/lock; claims.mjs imports fs/path/crypto/fsutil/lock/decisions.mjs).
import { isConcurrentMode, readSession, heartbeatStale } from './claims.mjs';
import { readWorkspace, workspacePath, WorkspaceStoreError } from './workspace-store.mjs';

/** File-path patterns that must never be read without asking the human. */
export const SECRET_PATTERNS = [
  /(^|[\\/])\.env(\.[A-Za-z0-9._-]+)?$/i,
  /\.pem$/i,
  /\.key$/i,
  /(^|[\\/])id_rsa[^\\/]*$/i,
  /\.p12$/i,
  /(^|[\\/])credentials[^\\/]*$/i,
  /(^|[\\/])secrets\.[^\\/]+$/i,
];

/** Directories agents should never scout through. */
export const SCOUT_DIRS = [
  'node_modules/',
  'dist/',
  'build/',
  '.git/objects',
  'vendor/',
  'coverage/',
  '.next/',
  '__pycache__/',
];

/** Paths writable in gated phases even before execution approval. */
export const GATE_ALLOWED_PREFIXES = ['.bee/', 'docs/', 'plans/', 'AGENTS.md'];

// docs/history/ is the tech-agnostic KNOWLEDGE layer (.md only: CONTEXT.md,
// plan.md, reports, walkthrough). Executable/code files (a verify.sh, a helper
// script) never belong there — a persistent verify script lives in the project's
// own scripts (committed with the product), a disposable proof in .bee/spikes/.
// GitHub #17: agents were dropping verify.sh scripts into docs/history/<feature>/.
const HISTORY_CODE_EXTENSIONS = new Set([
  '.sh', '.bash', '.zsh', '.fish', '.ps1', '.bat', '.cmd',
  '.mjs', '.cjs', '.js', '.jsx', '.ts', '.tsx',
  '.py', '.rb', '.go', '.rs', '.java', '.php', '.pl', '.lua', '.r',
]);
function docsHistoryCodeDeny(normalized) {
  if (!normalized.startsWith('docs/history/')) return null;
  const dot = normalized.lastIndexOf('.');
  if (dot === -1) return null;
  const ext = normalized.slice(dot).toLowerCase();
  return HISTORY_CODE_EXTENSIONS.has(ext) ? ext : null;
}

// ─── scratch-shape guard (tree-hygiene D4/D5, cell th-6) ───────────────────
// One canonical scratch home (docs/specs/doctrine-layer.md Business Rules,
// decision f21efe6e): every ephemeral file bee writes for its own working
// purposes belongs in .bee/tmp/<feature-or-session>/ (feasibility code in
// .bee/spikes/<feature>/), never a tracked path. This denies a write whose
// TARGET NAME looks scratch-shaped when it lands anywhere else in the
// tracked tree — first-hit, same precedence class as direct-edit and
// docs-history-code above.
//
// The hard requirement (plan-review, decision f21efe6e): a FALSE DENY on a
// real deliverable is worse than the garbage this guard prevents. Two
// independent safety nets keep this rule narrow:
//   1. An explicit allow-list runs BEFORE any shape pattern is even
//      evaluated: the scratch homes themselves, and every known deliverable
//      store (docs/**, .bee/cells/, .bee/decisions.jsonl, the four rendered
//      plugin skill trees). Nothing under these paths is ever denied here,
//      no matter how scratch-shaped its basename looks.
//   2. The shape patterns themselves are deliberately narrow band, chosen so
//      a real project source/test file is unlikely to collide:
//        - SCRATCH_DOTFILE_RE only matches a basename that STARTS WITH "."
//          and contains debug/stress/scratch — the exact shape of the crash
//          leak this feature was filed over (.rel1710rc3_stress_debug.sh).
//          Committed project sources are essentially never dot-prefixed.
//        - SCRATCH_PREFIX_RE only matches a basename STARTING WITH
//          verdict-/probe-/digest- — bee's own scratch vocabulary, not
//          plausible deliverable naming.
//        - SCRATCH_EXT_RE (bare .tmp/.log/.bak) is the one genuinely
//          ambiguous shape — a project can legitimately commit a fixture
//          named `sample.log` or `snapshot.bak` for a test. This is the ONE
//          pattern additionally exempted whenever the path runs through a
//          recognized test/fixture directory segment (test/, tests/,
//          __tests__/, fixtures/, __fixtures__/, testdata/, examples/) — a
//          project's own `foo.log`-named source/test file is not bee
//          scratch, and this is how that distinction is drawn: by directory
//          convention, not by guessing intent from the extension alone.
const SCRATCH_HOME_PREFIXES = ['.bee/tmp/', '.bee/spikes/', '.bee/logs/', '.bee/workers/'];
// Deliverable stores that must never be false-denied: docs/** (reports,
// specs, decisions, backlog), the cell store, the decisions ledger, and the
// four rendered plugin skill trees (scripts/render_plugin_skill_trees.mjs
// TARGET_ROOTS + the two skills/ mirrors onboarding also keeps in sync).
const DELIVERABLE_PREFIXES = [
  'docs/',
  '.bee/cells/',
  '.claude-plugin/skills/',
  '.codex-plugin/skills/',
  '.claude/skills/',
  '.agents/skills/',
];
const DELIVERABLE_EXACT = new Set(['.bee/decisions.jsonl']);
const TEST_FIXTURE_DIR_RE = /(^|\/)(test|tests|__tests__|fixtures|__fixtures__|testdata|examples)(\/|$)/i;
const SCRATCH_EXT_RE = /\.(tmp|log|bak)$/i;
const SCRATCH_DOTFILE_RE = /^\.[^/]*(?:debug|stress|scratch)[^/]*$/i;
const SCRATCH_PREFIX_RE = /^(?:verdict|probe|digest)-/i;

function underAnyPrefix(normalized, prefixes) {
  return prefixes.some((prefix) => normalized === prefix.slice(0, -1) || normalized.startsWith(prefix));
}

// Returns a short kind string when `normalized` is a scratch-shaped write
// landing outside every allowed home/deliverable, else null.
function scratchShapeDeny(normalized) {
  if (underAnyPrefix(normalized, SCRATCH_HOME_PREFIXES)) return null;
  if (DELIVERABLE_EXACT.has(normalized)) return null;
  if (underAnyPrefix(normalized, DELIVERABLE_PREFIXES)) return null;

  const basename = normalized.slice(normalized.lastIndexOf('/') + 1);
  if (SCRATCH_DOTFILE_RE.test(basename)) return 'a dotfile named like a debug/stress/scratch script';
  if (SCRATCH_PREFIX_RE.test(basename)) return 'a verdict-/probe-/digest- style scratch payload';
  if (SCRATCH_EXT_RE.test(basename) && !TEST_FIXTURE_DIR_RE.test(normalized)) {
    return `a ${basename.slice(basename.lastIndexOf('.'))} scratch file`;
  }
  return null;
}

const GATED_PHASES = new Set(['exploring', 'planning', 'validating']);

// Phases where no bee work is active: never started ('idle') and finished
// ('compounding-complete', the terminal alias state.mjs already accepts as an
// idle-equivalent in startFeature). Both must hit the intake gate. Testing
// `phase === 'idle'` alone left every repo default-open the moment a feature
// closed — the gates stay approved from the closed feature, so the gated-phase
// branch never fires either, and source edits for the NEXT piece of work walked
// straight through with nothing blocking them.
const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);

// Direct hand-edits to these two files are denied in every phase, first-hit,
// before any other checkWrite logic (including GATE_ALLOWED_PREFIXES —
// `.bee/` is an allowed prefix today, so this precedence is mandatory, not
// incidental). Both files now have a validating, atomic-write CLI
// (cli-mutations plan.md: bee.mjs state, bee.mjs backlog) — a direct
// Edit/Write/Bash-redirect bypasses that validation and reintroduces the
// schema-drift class the CLIs exist to close. This does not touch the CLIs'
// own writes: hooks see tool calls (Edit/Write/MultiEdit/Bash), never the
// bee.mjs state / bee.mjs backlog child process's internal file I/O.
const DIRECT_EDIT_DENY = {
  '.bee/state.json': 'bee.mjs state set --owner <selected pre-mutation phase>, or the dedicated state gate/worker/scribing-run verb',
  '.bee/backlog.jsonl': 'bee.mjs backlog add',
  // backlog-unification D3: docs/backlog.md is the generated VIEW over the
  // .bee/backlog.jsonl fold — a hand-edit here is invisible to the fold and
  // is silently overwritten by the next `backlog render`, reintroducing the
  // exact double-truth trap D3 retired `backlog rank --write` to close. One
  // exact key is sufficient (no prefix branch): v2 has no per-item files
  // under docs/, only this single rendered path.
  'docs/backlog.md':
    'bee.mjs backlog pbi add / bee.mjs backlog pbi status / bee.mjs backlog pbi amend to change data, or bee.mjs backlog render --write to regenerate the view',
  // xwh-4: the cross-worktree coordination stores are CLI-owned too — the
  // holds ledger is mirrored/released only by bee.mjs reservations (xwh-2)
  // and the grant registry only by bee.mjs worktree register/unregister. A
  // hand edit bypasses the store lock and the atomic tmp+rename write both
  // stores rely on (worktree-holds.mjs / worktree-store.mjs).
  '.bee/runtime/cross-worktree-holds.json':
    'bee.mjs reservations reserve/release (holds are mirrored into the ledger automatically)',
  '.bee/runtime/worktree-grants.json': 'bee.mjs worktree register / unregister',
  // D12: the companion-worktree marker (worktree-store.mjs's runCompanionStart)
  // is CLI-owned the same way — a hand edit could point resolveCompanionMountedRelPath
  // at an arbitrary worktreePath/mountPath pair, granting access outside the
  // physical worktree.
  '.bee/companion-session.json': 'bee worktree new --with-companion (started/ended automatically by the companion lifecycle)',
};

function normalizeRel(relPath) {
  return String(relPath || '')
    .replace(/\\/g, '/')
    .replace(/^\.\/+/, '');
}

function underAllowedPrefix(relPath) {
  const normalized = normalizeRel(relPath);
  return GATE_ALLOWED_PREFIXES.some((prefix) => {
    if (prefix.endsWith('/')) {
      return normalized === prefix.slice(0, -1) || normalized.startsWith(prefix);
    }
    return normalized === prefix;
  });
}

// ─── intake-gate refusal message (D3, ige-2 / P46 / GH #1) ────────────────
// One shared builder for every terminal-phase ("intake gate") refusal —
// plain source writes AND the git-command denials below all funnel through
// this, so the wording fix applies everywhere the operator can hit it, not
// just the git path the incident happened to use. D3: the FIX line names the
// bookkeeping-direct-commit route and bee-hive FIRST; `guards.idle_gate` is
// mentioned LAST, as a repo-level opt-out, never as the way to finish a
// commit — the previous ordering pointed the operator straight at the
// dangerous escape, which is exactly how the incident (a7d2069) happened.
function intakeFixLine() {
  return (
    `FIX: commit or write bookkeeping directly — ${GATE_ALLOWED_PREFIXES.join(', ')} are exempt from this gate — ` +
    'or route the request through bee-hive first (classify the mode; tiny fixes stay tiny — one cell, a 2-minute ' +
    'reality check, Gate 3, go), then execute. Last resort, repo-level opt-out: ' +
    'bee config set --key guards.idle_gate --value false (re-enable with: bee config unset --key guards.idle_gate).'
  );
}

function intakeRefusal(phase, blockedDescription, extraSentence = '') {
  return (
    `bee intake gate: no bee work is active (phase: ${phase}) — ${blockedDescription} is blocked. ` +
    extraSentence +
    intakeFixLine()
  );
}

// Resolves the effective phase/gate record for a write decision: a bound
// sessionId reads through resolvePipeline's lane record; an absent one uses
// the caller's own `state` (byte-identical to the pre-fsh-8 checkWrite
// contract). An unresolvable lane binding is a typed deny, never a silent
// fallback to the default pipeline. Shared by checkWrite and
// checkGitBashCommand so both apply the SAME phase/lane semantics. `source`
// ('default'|'lane') is carried through on success (msn-21) so a caller can
// tell a plain default-pipeline write from a lane-governed one without
// re-deriving it — lane flows opt out of this cell's new workspace-ownership
// check (see checkWrite below).
//
// msn-18a (advisor-digest-slice4 binding condition 2): `controlRoot` here is
// resolvePipeline's lane/session/workflow reads are control-plane
// (resolveContext.controlRoot = mainRoot) — a worktree-bound write must find
// the SAME lane record main sees, never hard-deny with LANE_MISSING just
// because its own checkout has no `.bee/lanes/<feature>.json` of its own.
// Main/solo repos: controlRoot === root, byte-identical to the pre-msn-18a
// behavior.
//
// msn-21: `controlRoot` is now a caller-supplied, ALREADY-RESOLVED value
// (checkWrite/checkGitBashCommand resolve topology exactly once via
// resolveWriteTopology below and thread the result through every branch that
// needs it) rather than this function re-resolving resolveContext(root)
// itself — a second, independent topology walk inside the SAME checkWrite
// call (advisor-digest-slice4 binding condition 5: "resolveContext is THE
// single git-common-dir resolver", applied inside this file too).
function resolveWriteRecord(controlRoot, state, sessionId) {
  if (typeof sessionId === 'string' && sessionId.trim()) {
    const resolved = resolvePipeline(controlRoot, { sessionId });
    if (!resolved.ok) {
      return { ok: false, reason: `bee lane guard: ${resolved.reason}` };
    }
    return { ok: true, record: resolved.record, source: resolved.source };
  }
  return { ok: true, record: state, source: 'default' };
}

// resolveWriteTopology(root, controlRootOverride) — the ONE resolveContext
// call per checkWrite/checkGitBashCommand invocation, feeding every branch
// below that needs topology (resolveWriteRecord's controlRoot, the
// cross-worktree hold consultation, and the new workspace-ownership check).
// Wrapped in try/catch: an invalid linked-worktree marker
// (WorktreeLinkInvalidError) resolves to an all-null context — the SAME
// fail-open outcome the pre-msn-21 resolveHoldTopology's own try/catch
// always produced for that case (critical pattern 20260716: an over-denying
// guard must never lock a session out of its own fix).
//
// `controlRootOverride`: when the caller already resolved topology itself
// (the write-guard hook's own readHookContext/resolveRoots walk, exposed as
// ctx.controlRoot since d69d81e) and passes it through, that value wins over
// this function's own resolveContext(root).controlRoot — the adapter's
// resolution is authoritative once it exists, rather than re-derived from
// scratch a second time. Omitted (every existing library/test caller):
// byte-identical to today, controlRoot comes from resolveContext(root) alone.
function resolveWriteTopology(root, controlRootOverride) {
  let ctx;
  try {
    ctx = resolveContext(root);
  } catch {
    ctx = { controlRoot: null, workspaceRoot: null, workspaceId: null, worktreeId: null };
  }
  const override =
    typeof controlRootOverride === 'string' && controlRootOverride.trim() ? controlRootOverride.trim() : null;
  const controlRoot = override || ctx.controlRoot || root;
  return { ctx, controlRoot };
}

// ─── git write-exemption classification (D1/D3/D4, ige-2 / P46 / GH #1) ───
// Read-only git subcommands, deliberately enumerated — never inferred. Two
// of these are read-only ONLY with a specific flag: a bare `git branch
// <name>` / `git tag <name>` MUTATES (creates), so they must not match here
// without --list; `git remote` similarly needs -v/--verbose to be read-only.
const GIT_READONLY_SUBCOMMANDS = new Set([
  'status', 'log', 'diff', 'show', 'rev-parse', 'ls-files', 'check-ignore',
  'merge-base', 'rev-list', 'describe', 'blame', 'cat-file',
]);
const GIT_READONLY_FLAG_GATED = {
  branch: new Set(['--list']),
  tag: new Set(['--list']),
  remote: new Set(['-v', '--verbose']),
};

// Mutating git subcommands this exemption logic recognizes at all (D1).
// `push` is deliberately NOT a member — it never gets the bookkeeping-path
// exemption (outward-facing) and is classified separately below. Anything
// NOT in this set and NOT read-only is "unrecognized" and refused at a
// terminal phase rather than silently allowed through (fail closed).
const GIT_MUTATING_SUBCOMMANDS = new Set([
  'commit', 'add', 'rm', 'mv', 'checkout', 'restore',
  'tag', 'merge', 'reset', 'stash', 'clean', 'apply', 'cherry-pick', 'revert', 'rebase',
]);
// Subset of GIT_MUTATING_SUBCOMMANDS whose changed paths this classifier can
// actually resolve from real git state (D4). The rest (merge/reset/stash/
// clean/apply/cherry-pick/revert/rebase/tag) are structural/broad operations
// with no reliable pathspec model here — they always fail closed (today's
// refusal), never inferred safe just because they're "recognized".
const GIT_PATH_RESOLVABLE_SUBCOMMANDS = new Set(['commit', 'add', 'rm', 'mv', 'checkout', 'restore']);
const GIT_BROAD_PATHSPECS = new Set(['.', ':', ':/', './']);

function gitGlobalFlagTakesValue(token) {
  return token === '-C' || token === '-c' || token === '--git-dir' || token === '--work-tree' || token === '--namespace';
}

// Finds the FIRST top-level `git <subcommand>` invocation in `command`
// (skipping git's own global flags, e.g. `-C <dir>`), returning
// { subcommand, rest } — `subcommand` is null for a bare "git" with no
// subcommand token at all — or null when `command` contains no `git`
// invocation whatsoever. Only the first invocation is classified; a compound
// command chaining a SECOND git call is a documented limitation of this cell.
function findGitInvocation(tokens) {
  for (let i = 0; i < tokens.length; i += 1) {
    if (SEPARATORS.has(tokens[i])) continue;
    const cmd = tokens[i].replace(/\\/g, '/').split('/').pop();
    if (cmd !== 'git') continue;
    let end = i + 1;
    while (end < tokens.length && !SEPARATORS.has(tokens[end])) end += 1;
    const invocationTokens = tokens.slice(i + 1, end);
    let subcommand = null;
    let subIdx = -1;
    for (let j = 0; j < invocationTokens.length; j += 1) {
      const t = invocationTokens[j];
      if (gitGlobalFlagTakesValue(t)) { j += 1; continue; }
      if (t.startsWith('-')) continue;
      subcommand = t;
      subIdx = j;
      break;
    }
    if (subcommand === null) return { subcommand: null, rest: [] };
    return { subcommand, rest: invocationTokens.slice(subIdx + 1) };
  }
  return null;
}

function runGitCapture(cwd, args) {
  try {
    const out = execFileSync('git', args, { cwd, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] });
    return out.split(/\r?\n/).map((l) => l.trim()).filter(Boolean);
  } catch {
    return null;
  }
}

function hasGitShortFlag(tokens, letter) {
  return tokens.some((t) => /^-[a-zA-Z]+$/.test(t) && t.slice(1).includes(letter));
}

// Explicit pathspec args: everything after a literal `--`, or (when no `--`
// is present) every non-flag token. Used for add/rm/mv/checkout/restore,
// whose syntax is `git <verb> [flags] [--] <pathspec>...` — a pathspec here
// is exactly what the command names, nothing inferred.
function extractExplicitPathspecs(restTokens) {
  const dashDashIdx = restTokens.indexOf('--');
  const scanTokens = dashDashIdx === -1 ? restTokens : restTokens.slice(dashDashIdx + 1);
  if (dashDashIdx === -1) return scanTokens.filter((t) => !t.startsWith('-'));
  return scanTokens;
}

/**
 * Resolves the repo-relative paths a mutating git subcommand would actually
 * change, from REAL git state at check time (D4) — never from the command's
 * wording, a flag, or an env var. Returns null when the set cannot be proved
 * (a broad/glob pathspec, no pathspec at all where one is required, or the
 * git call itself failed) — the caller fails closed on null, exactly like a
 * proved source path.
 *
 * `commit`: a pathspec is only ever recognized AFTER a literal `--`
 * (git's own disambiguator) — never a bare trailing token, because a bare
 * trailing token after `-m`/`-c`/etc. is that flag's VALUE (the commit
 * message), not a pathspec; treating it as one would let a message's wording
 * masquerade as a path, which D4 forbids. No `--` pathspec -> resolves to the
 * STAGED index (`git diff --cached --name-only`); `-a`/`--all` (or a short
 * flag containing 'a', e.g. `-am`) folds in tracked-but-unstaged paths too.
 */
function resolveGitMutationPaths(cwd, subcommand, restTokens) {
  if (subcommand === 'commit') {
    const dashDashIdx = restTokens.indexOf('--');
    const explicitPathspecs = dashDashIdx === -1 ? [] : restTokens.slice(dashDashIdx + 1);
    const preDashDash = dashDashIdx === -1 ? restTokens : restTokens.slice(0, dashDashIdx);
    const isAll = hasGitShortFlag(preDashDash, 'a') || preDashDash.includes('--all');

    const staged = runGitCapture(cwd, ['diff', '--cached', '--name-only']);
    if (staged === null) return null;

    if (explicitPathspecs.length > 0) {
      if (explicitPathspecs.some((p) => GIT_BROAD_PATHSPECS.has(p) || p.includes('*'))) return null;
      return explicitPathspecs;
    }
    if (!isAll) return staged;
    const unstaged = runGitCapture(cwd, ['diff', '--name-only']);
    if (unstaged === null) return null;
    return Array.from(new Set([...staged, ...unstaged]));
  }

  // add / rm / mv / checkout / restore: resolve to literal pathspec args.
  const pathspecs = extractExplicitPathspecs(restTokens);
  if (pathspecs.length === 0) return null; // bare/flags-only invocation: unprovable
  if (pathspecs.some((p) => GIT_BROAD_PATHSPECS.has(p) || p.includes('*'))) return null; // broad/glob: unprovable
  return pathspecs;
}

/**
 * Git-command awareness for the intake gate (D1/D3/D4, ige-2 / P46 / GH #1).
 * Scoped ONLY to the terminal-phase intake gate — D1 says "while the phase
 * is terminal", and the Boundary this cell shipped under is explicit that
 * nothing here may reopen the gate's actual purpose. Outside a terminal
 * phase (gated phases, swarming, ...), this returns null unconditionally and
 * the caller's existing Bash-target logic is completely unaffected — the
 * fix stays confined to the one door the incident (a7d2069) walked through.
 *
 * Returns:
 *   null                          — not a git command, phase isn't
 *                                    terminal, or the idle gate is disabled:
 *                                    caller's existing logic decides.
 *   { allow: true, kind }         — read-only git, or a mutating git command
 *                                    whose actually-changed paths are ALL
 *                                    inside GATE_ALLOWED_PREFIXES.
 *   { allow: false, kind, reason } — `git push` (never exempt), an
 *                                    unrecognized subcommand (fail closed),
 *                                    or a mutating command touching a
 *                                    non-bookkeeping path (today's refusal).
 */
export function checkGitBashCommand(root, state, command, { cwd = root, sessionId = null, controlRoot: controlRootOverride = null } = {}) {
  const { controlRoot } = resolveWriteTopology(root, controlRootOverride);
  const recordResolution = resolveWriteRecord(controlRoot, state, sessionId);
  if (!recordResolution.ok) {
    return { allow: false, kind: 'lane', reason: recordResolution.reason };
  }
  const phase = recordResolution.record?.phase || 'idle';
  if (!TERMINAL_PHASES.has(phase)) return null;

  const config = readConfig(root);
  const idleGateOn = !(config.guards && config.guards.idle_gate === false);
  if (!idleGateOn) return null;

  const tokens = tokenize(command);
  const invocation = findGitInvocation(tokens);
  if (!invocation) return null;
  const { subcommand, rest } = invocation;

  if (subcommand && GIT_READONLY_SUBCOMMANDS.has(subcommand)) {
    return { allow: true, kind: 'git-read-only' };
  }
  if (subcommand && GIT_READONLY_FLAG_GATED[subcommand] && rest.some((t) => GIT_READONLY_FLAG_GATED[subcommand].has(t))) {
    return { allow: true, kind: 'git-read-only' };
  }

  if (subcommand === 'push') {
    return {
      allow: false,
      kind: 'git-push',
      reason: intakeRefusal(
        phase,
        '`git push`',
        'git push is outward-facing and is never exempted from this gate, regardless of what it would push. ',
      ),
    };
  }

  if (subcommand && GIT_MUTATING_SUBCOMMANDS.has(subcommand)) {
    const resolvedPaths = GIT_PATH_RESOLVABLE_SUBCOMMANDS.has(subcommand)
      ? resolveGitMutationPaths(cwd, subcommand, rest)
      : null;
    if (resolvedPaths === null) {
      return {
        allow: false,
        kind: 'intake',
        reason: intakeRefusal(phase, `running \`git ${subcommand}\` (its changed paths could not be proved bookkeeping-only)`),
      };
    }
    const offending = resolvedPaths.map(normalizeRel).find((p) => !underAllowedPrefix(p));
    if (offending) {
      return {
        allow: false,
        kind: 'intake',
        reason: intakeRefusal(phase, `running \`git ${subcommand}\` — it would change "${offending}"`),
      };
    }
    return { allow: true, kind: 'git-bookkeeping' };
  }

  return {
    allow: false,
    kind: 'git-unrecognized',
    reason: intakeRefusal(
      phase,
      `running \`git ${subcommand || command.trim()}\``,
      'This git subcommand is not recognized as read-only or as a modeled bookkeeping-eligible mutation, so it is refused rather than assumed safe. ',
    ),
  };
}

/**
 * Corrupt-vs-missing discriminator for the reservation store (D3 fail-closed
 * shape, panel B1). A MISSING store is today's exact open behavior — nothing
 * has ever reserved anything, so there is nothing to fail closed over. A
 * PRESENT but unparseable store is the one case that must deny rather than
 * silently read as empty: reservations.mjs's own readStore/listReservations/
 * findConflicts/findSessionConflicts stay fail-open (untouched here) because
 * they serve reads and intra-swarm nickname conflicts that must never crash a
 * whole session over one bad file; this session-aware WRITE guard is the one
 * caller that cannot afford to silently treat "corrupt" as "empty" — a stray
 * concurrent-write torn file could otherwise open every held path in the
 * repo to any session. Never called when sessionId is absent (byte-identical
 * to today in that case).
 */
function reservationStoreCorrupt(root) {
  const file = reservationsPath(root);
  if (!fs.existsSync(file)) return false; // missing store = today's open behavior
  try {
    JSON.parse(fs.readFileSync(file, 'utf8'));
    return false;
  } catch {
    return true;
  }
}

/** Expiry display for a hold-deny message, computed from the reservation's own
 * public fields only (never importing reservations.mjs's private isExpired). */
function holdExpiry(reservation) {
  const reservedMs = Date.parse(reservation?.reserved_at);
  const ttl = reservation?.ttl_seconds;
  if (!Number.isFinite(reservedMs) || !Number.isFinite(ttl) || ttl <= 0) return 'no expiry';
  return `expires ${new Date(reservedMs + ttl * 1000).toISOString()}`;
}

/** Same expiry-string convention as holdExpiry above, rebased on a
 * cross-worktree ledger hold's `mirrored_at`/`ttl_seconds` fields (same shape
 * bee.mjs's holdForeignExpiry uses for its own list rendering). */
function foreignHoldExpiry(hold) {
  const mirroredMs = Date.parse(hold?.mirrored_at);
  const ttl = hold?.ttl_seconds;
  if (!Number.isFinite(mirroredMs) || !Number.isFinite(ttl) || ttl <= 0) return 'no expiry';
  return `expires ${new Date(mirroredMs + ttl * 1000).toISOString()}`;
}

// multisession-native-14 (D4, issue #56 3.5): built-in exclusive-resource
// defaults for the cross-worktree advisory downgrade below. A path matching
// one of these globs keeps the ORIGINAL hard cross-worktree deny even though
// every other path downgrades to an advisory warning — these are the
// resources where two checkouts editing "the same logical thing" from
// different physical trees is unsafe to discover only at merge time (a
// migration ordering collision, a lockfile rewritten out from under another
// checkout's install, a release/onboarding artifact bee itself reads as an
// atomic ledger). `.bee/config.json`'s `guards.exclusive_paths` EXTENDS this
// list (never replaces it) — see isExclusivePath below.
const DEFAULT_EXCLUSIVE_PATHS = [
  // DB migration directories, any depth.
  '**/migrations/**',
  // Lockfiles — matched both at repo root and nested (monorepo packages).
  'package-lock.json',
  '**/package-lock.json',
  'yarn.lock',
  '**/yarn.lock',
  'pnpm-lock.yaml',
  '**/pnpm-lock.yaml',
  'Cargo.lock',
  '**/Cargo.lock',
  'composer.lock',
  '**/composer.lock',
  'Gemfile.lock',
  '**/Gemfile.lock',
  // Release/manifest artifacts bee itself treats as a single atomic ledger.
  // (bee.mjs's own RELEASE_MANIFEST_LINT_PATH constant — kept a literal here
  // too rather than imported, to avoid a guards.mjs -> bee.mjs cycle.)
  'docs/history/codex-harness-hardening/release-manifest.json',
  '.bee/onboarding.json',
  // Generated client directories, any depth.
  '**/generated/**',
];

/**
 * Translates a small glob vocabulary (`*` = any run of non-slash characters,
 * `**` = zero or more path segments, everything else literal) into an
 * anchored RegExp. Deliberately NOT reusing reservations.mjs's pathsOverlap —
 * that predicate answers a different question (does a broad prefix/glob
 * CONTAIN a path, for reservation/wave-scheduling containment) with only
 * trailing-`*` support; this one answers "does this glob PATTERN match this
 * exact path" for the exclusive-resource list, which needs mid-path `**`
 * (`**\/migrations/**`) that pathsOverlap was never built for.
 */
function globToRegExp(glob) {
  const normalized = String(glob || '').replace(/\\/g, '/').replace(/^\.\/+/, '');
  let pattern = '';
  let i = 0;
  while (i < normalized.length) {
    const c = normalized[i];
    if (c === '*' && normalized[i + 1] === '*') {
      let j = i + 2;
      if (normalized[j] === '/') {
        // '**/' — an optional run of whole path segments as a prefix.
        pattern += '(?:.*/)?';
        j += 1;
      } else {
        // trailing/mid '**' with no following slash — matches any remainder.
        pattern += '.*';
      }
      i = j;
      continue;
    }
    if (c === '*') {
      pattern += '[^/]*';
      i += 1;
      continue;
    }
    if ('.+^${}()|[]\\'.includes(c)) {
      pattern += `\\${c}`;
      i += 1;
      continue;
    }
    pattern += c;
    i += 1;
  }
  return new RegExp(`^${pattern}$`);
}

/**
 * True when `normalizedPath` matches any exclusive-resource glob — the
 * built-in defaults above, EXTENDED (never replaced) by
 * `.bee/config.json`'s `guards.exclusive_paths` array (documented in
 * AGENTS.md/CONTEXT.md D4: "user list extends defaults" — simpler and safer
 * than a replace, since a repo that adds one project-specific exclusive path
 * should not have to also re-declare every built-in one to keep it covered).
 * A malformed/absent config key is silently treated as an empty extension
 * list, never an error — this is a policy lookup, not a validated write.
 */
function isExclusivePath(root, normalizedPath) {
  const config = readConfig(root);
  const extra = config.guards && Array.isArray(config.guards.exclusive_paths) ? config.guards.exclusive_paths : [];
  const globs = DEFAULT_EXCLUSIVE_PATHS.concat(extra.filter((g) => typeof g === 'string' && g.trim()));
  return globs.some((glob) => globToRegExp(glob).test(normalizedPath));
}

// xwh-4/msn-21: resolves the cross-worktree HOLD topology for the write
// guard — same shape/naming as cells.mjs's resolveHoldTopology (xwh-3), now
// DERIVED from the single resolveWriteTopology(root) resolution checkWrite
// already computed (msn-21: no second resolveRoots(root) walk of its own).
// Returns `{ mainRoot, holder }` for the two topologies worth consulting:
//   - an ORDINARY checkout (ctx.worktreeId is null): holder = 'main',
//     mainRoot = controlRoot.
//   - a GRANTED linked worktree (ctx.workspaceId === ctx.worktreeId — i.e.
//     resolveContext did NOT fall back to the 'main' workspaceId default,
//     the exact same "registered" condition the old storeRoot===worktreeRoot
//     check proved): holder = ctx.workspaceId (its git-verified id),
//     mainRoot = controlRoot.
// Returns `null` for every other case — an UNGRANTED linked worktree
// (ctx.worktreeId set but ctx.workspaceId fell back to 'main': the shared
// main store's same-checkout reservation guards above already govern it
// directly) and an unresolvable/invalid checkout (ctx.workspaceRoot null —
// resolveWriteTopology's own try/catch already turned a thrown
// WorktreeLinkInvalidError into this same all-null shape) both fall through
// to `null`, which checkWrite treats as "skip the foreign-hold consultation
// entirely, byte-identical to before this cell" — FAIL-OPEN, never a deny.
// An over-denying write guard can lock every session out of its own fix
// (critical pattern 20260716), so no error path in this resolution may deny.
function resolveHoldTopology(ctx, controlRoot) {
  if (!ctx.workspaceRoot) return null;
  if (!ctx.worktreeId) return { mainRoot: controlRoot, holder: 'main' };
  if (ctx.workspaceId && ctx.workspaceId === ctx.worktreeId) {
    return { mainRoot: controlRoot, holder: ctx.workspaceId };
  }
  return null;
}

// msn-21 (deny class (b) workspace scoping): the SAME 3-mode resolution
// state.mjs's own (unexported) resolveWritePolicyMode applies for
// applyWritePolicy (msn-20) — duplicated here as a tiny, deliberately
// inline read rather than an added export, since guards.mjs's `files` scope
// for this cell is the guard lib/hook/tests only. Kept BYTE-IDENTICAL in
// logic to state.mjs's own resolver: 'observe'/'shared-disjoint' read
// straight off config.guards.write_policy; anything else (including absent)
// is the default 'isolated'.
function resolveWritePolicyMode(config) {
  const configured =
    config && config.guards && typeof config.guards.write_policy === 'string' ? config.guards.write_policy.trim() : '';
  if (configured === 'observe') return 'observe';
  if (configured === 'shared-disjoint') return 'shared-disjoint';
  return 'isolated';
}

// A session's own workspace identity, from its stamped `workspace_id`
// (msn-19, createSession) — OMITTED on every legacy/pre-msn-19 session and
// on any unreadable/missing session record, both of which read as 'main'
// here (the same default resolveContext itself uses for an ordinary
// checkout), never a throw or a guessed-blank value.
function sessionWorkspaceId(controlRoot, sessionId) {
  const session = readSession(controlRoot, sessionId);
  return (session && typeof session.workspace_id === 'string' && session.workspace_id.trim()) || 'main';
}

// msn-21 (deny class (c)): does a DIFFERENT, LIVE session already hold
// write ownership of ctx's workspace? Read-only — this never CLAIMS
// ownership itself (that stays applyWritePolicy's job, msn-20); checkWrite
// only asks whether the acting session is entitled to write here right now.
// Returns `{ blocked: false }` when the workspace has never been registered
// (WORKSPACE_MISSING — nobody has claimed it yet, matches "a solo caller
// always becomes owner" byte-identical prohibition), when nobody/only the
// acting session owns it, or when the current owner's heartbeat is stale
// (a dead owner never blocks — this function does not reclaim, it simply
// stops treating a crashed session as a live blocker, same posture
// applyWritePolicy's own isOwnerLive predicate documents). Returns
// `{ blocked: true, corrupt: true }` for a present-but-unreadable workspace
// record (fail CLOSED, matching this file's reservationStoreCorrupt/
// holdsStoreCorrupt precedent: missing is open, corrupt is denied — a torn
// record could otherwise silently misreport who owns the workspace).
// Otherwise `{ blocked: true, owner }` names the live foreign owner.
function checkWorkspaceOwnership(controlRoot, ctx, sessionId) {
  const workspaceId = (ctx && ctx.workspaceId) || 'main';
  let workspace;
  try {
    workspace = readWorkspace(controlRoot, workspaceId);
  } catch (err) {
    if (err instanceof WorkspaceStoreError && err.code === 'WORKSPACE_MISSING') {
      return { blocked: false };
    }
    return { blocked: true, corrupt: true };
  }
  const owner = workspace.write_owner_session;
  if (!owner || owner === sessionId) return { blocked: false };
  const ownerSession = readSession(controlRoot, owner);
  const live = ownerSession ? !heartbeatStale(ownerSession) : false;
  if (!live) return { blocked: false };
  return { blocked: true, owner };
}

/**
 * Gate + reservation write check.
 * - Direct-edit deny (first hit, every phase): `.bee/state.json` and
 *   `.bee/backlog.jsonl` must go through their CLI (bee.mjs state /
 *   bee.mjs backlog), never a direct Edit/Write/Bash-redirect. Checked before
 *   phase logic and before GATE_ALLOWED_PREFIXES, since `.bee/` is itself an
 *   allowed prefix in gated phases.
 * - Terminal phases (intake gate): 'idle' (never started) and
 *   'compounding-complete' (feature closed) both mean no bee work is active, so
 *   source writes are blocked until the request is routed through bee-hive.
 *   Repository-harness lesson: a default-open first move is the hole every
 *   ad-hoc edit slips through — and "the feature just closed" is a first move.
 *   Disable per repo with: bee config set --key guards.idle_gate --value false
 * - Gated phases (exploring/planning/validating): block writes outside
 *   GATE_ALLOWED_PREFIXES while approved_gates.execution is false.
 * - Swarming: deny writes that conflict with another agent's reservation
 *   (agent identity from agentName arg or BEE_AGENT_NAME env).
 * - Optional sessionId (fsh-5, D2/D4): when provided, phase and gates come
 *   from resolvePipeline(root, { sessionId }) — a bound session is governed
 *   by its lane record, an unbound/unknown session by the default record.
 *   Absent sessionId is byte-identical to today: the caller's state argument
 *   decides. A binding that cannot resolve (invalid/missing/corrupt lane) is
 *   a typed DENY — a write guard never guesses a broken binding back to the
 *   default pipeline (the wrong pipeline's gates would decide the write).
 * - Cross-session hold deny (fsh-7, D3): also gated on sessionId being
 *   present. Runs right after record resolution and BEFORE every phase-based
 *   branch below (terminal/gated/swarming) — D3 is unconditional on phase, so
 *   a write into a path another LIVE session holds is denied even in
 *   swarming with execution approved, not just in tail-reaching phases. The
 *   acting session's own holds, expired holds, and legacy session-less
 *   reservation rows never block. A present-but-corrupt reservation store
 *   fails closed with a typed {allow:false, kind:'holds-unreadable'} verdict
 *   (never a throw — the production hook is fail-open and would swallow a
 *   throw into an allow); a missing store stays open, same as today.
 * - Cross-WORKTREE hold policy (xwh-4, revised multisession-native-14 D4):
 *   right after the cross-session block, before every phase branch, and NOT
 *   gated on sessionId — checkout identity comes from
 *   resolveHoldTopology(root) (ordinary => 'main', granted worktree => its
 *   git-verified id). A path ledger-held by a DIFFERENT checkout now DENIES
 *   only when the path matches the exclusive-resource list
 *   (isExclusivePath — migrations, lockfiles, release/manifest artifacts,
 *   generated client dirs; kind 'worktree-hold', same reason shape as
 *   before). Every other cross-worktree overlap ALLOWS with a `warning`
 *   naming the holding checkout, its feature, and that `bee worktree merge`
 *   will surface any real conflict at merge time — never a silent allow
 *   (must_haves prohibition: "no silent allow without the advisory
 *   warning"). Own holds, expired/released holds, and a missing ledger never
 *   deny or warn; unresolvable/ungranted topology skips the consultation
 *   entirely (fail-open). The one unconditional deny on a broken store: a
 *   present-but-corrupt ledger => typed
 *   {allow:false, kind:'worktree-holds-unreadable'} (holdsStoreCorrupt).
 *   SAME-workspace conflicts (the cross-session hold block above, and the
 *   swarming reservation block below) are untouched by this cell — only the
 *   cross-worktree branch's policy changed.
 * - Workspace-ownership deny (msn-21, deny class (c), invariant-15
 *   groundwork): right after the phase is known, before every phase-based
 *   branch below — a write from a real session into a workspace a
 *   DIFFERENT, LIVE session already write-owns (workspace-store.mjs, msn-19)
 *   is denied, but ONLY where applyWritePolicy's 'isolated' mode governs
 *   (msn-20 D3): sessionId present, config.guards.write_policy resolves to
 *   'isolated' (the default — 'observe'/'shared-disjoint' opt out
 *   entirely), the DEFAULT pipeline (a lane-bound session is untouched —
 *   lanes keep their existing guard branches, CONTEXT.md's "Scope
 *   boundaries"), and phase !== 'swarming' (the reservation block below IS
 *   the sanctioned multi-session-in-one-checkout pattern). Never claims or
 *   reclaims ownership itself — read-only, applyWritePolicy remains the one
 *   writer. An unregistered workspace, no owner, the acting session's own
 *   ownership, or a stale/dead owner's heartbeat never blocks; a
 *   present-but-corrupt workspace record fails closed
 *   ({allow:false, kind:'workspace-unreadable'}), matching this file's
 *   holds-unreadable/worktree-holds-unreadable precedent. No new hard-block
 *   CLASS beyond the three enumerated (must_haves prohibition) — this is
 *   class (c) itself, not a fourth.
 */
export function checkWrite(root, state, relPath, agentName = null, { sessionId = null, controlRoot: controlRootOverride = null } = {}) {
  const normalized = normalizeRel(relPath);

  const directEditVerb = DIRECT_EDIT_DENY[normalized];
  if (directEditVerb) {
    return {
      allow: false,
      kind: 'direct-edit',
      reason:
        `bee direct-edit guard: "${normalized}" is CLI-owned — direct edits are blocked in every phase. ` +
        'Hand-edited state files reintroduce schema drift (the exact class the CLI validates away). ' +
        `FIX: use ${directEditVerb} instead of editing this file directly.`,
    };
  }

  const historyCodeExt = docsHistoryCodeDeny(normalized);
  if (historyCodeExt) {
    return {
      allow: false,
      kind: 'docs-history-code',
      reason:
        `bee docs-history guard: "${normalized}" writes a "${historyCodeExt}" code file into docs/history/, which is ` +
        'the tech-agnostic KNOWLEDGE layer (.md only — CONTEXT.md, plan.md, reports, walkthrough). Code never lives there. ' +
        "FIX: put a persistent verify/helper script in the project's own scripts (committed with the product) and point " +
        'the cell\'s verify command at it; put a disposable proof in .bee/spikes/<feature>/. Never docs/history.',
    };
  }

  const scratchKind = scratchShapeDeny(normalized);
  if (scratchKind) {
    return {
      allow: false,
      kind: 'scratch-shape',
      reason:
        `bee scratch-shape guard: "${normalized}" looks like ${scratchKind} landing in a tracked directory. ` +
        'Every ephemeral file bee writes for its own working purposes belongs in .bee/tmp/<feature-or-session>/ ' +
        '(feasibility code in .bee/spikes/<feature>/), never a tracked path (docs/specs/doctrine-layer.md). ' +
        'FIX: write it to .bee/tmp/ instead (or .bee/spikes/ for a feasibility proof), and let `bee tmp sweep` clear it later.',
    };
  }

  // msn-21: ONE topology resolution feeds every branch below that needs it
  // (resolveWriteRecord's controlRoot, the SAME-workspace scoping on the
  // cross-session hold check just below, the cross-worktree hold topology,
  // and the new workspace-ownership check) — see resolveWriteTopology's own
  // header for the fail-open/override contract.
  const { ctx, controlRoot } = resolveWriteTopology(root, controlRootOverride);

  const recordResolution = resolveWriteRecord(controlRoot, state, sessionId);
  if (!recordResolution.ok) {
    return { allow: false, kind: 'lane', reason: recordResolution.reason };
  }
  const record = recordResolution.record;

  if (typeof sessionId === 'string' && sessionId.trim()) {
    const acting = sessionId.trim();
    if (reservationStoreCorrupt(root)) {
      return {
        allow: false,
        kind: 'holds-unreadable',
        reason:
          `bee hold guard: the reservation store (${path.relative(root, reservationsPath(root))}) is present but ` +
          'unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as empty. ' +
          'FIX: inspect/restore the reservation store, then retry.',
      };
    }
    const holdConflicts = findSessionConflicts(root, acting, [normalized]);
    if (holdConflicts.length > 0) {
      // msn-21 (deny class (b), workspace-aware): a lease is keyed by REPO-
      // RELATIVE path in the shared control-plane store — the same path
      // string names a DIFFERENT physical file in a DIFFERENT workspace
      // (main vs. a linked worktree), so a foreign session's exact lease
      // only hard-blocks when it was taken from the SAME workspace this
      // write is happening in. A session's workspace comes from its own
      // stamped `workspace_id` (msn-19, createSession) — OMITTED on every
      // legacy/pre-msn-19 session, which reads as 'main' here, exactly
      // matching resolveContext's own 'main' default for an ordinary
      // checkout — so a solo/single-workspace repo (every existing test in
      // this file) sees byte-identical hard-block behavior: acting and
      // holder both read 'main', always match.
      const actingWorkspaceId = ctx.workspaceId || 'main';
      const sameWorkspace = holdConflicts.filter((holder) => sessionWorkspaceId(controlRoot, holder.session) === actingWorkspaceId);
      if (sameWorkspace.length > 0) {
        const holder = sameWorkspace[0];
        return {
          allow: false,
          kind: 'hold',
          reason:
            `bee cross-session hold: "${normalized}" is held by session "${holder.session}" ` +
            `(agent ${holder.agent}, cell ${holder.cell}), ${holdExpiry(holder)}. ` +
            'Wait for the hold to expire or coordinate with that session — a cross-session hold is a hard block (D3).',
        };
      }
    }
  }

  // xwh-4: cross-WORKTREE foreign-hold consultation — unconditional on phase
  // and on sessionId, same placement discipline as the cross-session block
  // above (a foreign checkout's hold denies even in swarming with execution
  // approved). Topology unresolvable/ungranted => null => skip entirely
  // (fail-open). The ONE deliberate deny on a broken store is a
  // present-but-unparseable ledger (holdsStoreCorrupt: missing=open,
  // unparseable=deny — reservationStoreCorrupt's exact semantics): silently
  // reading a torn ledger as empty would open every foreign-held path to
  // this checkout. Any other failure inside the consultation itself is
  // swallowed into an allow, never a deny (critical pattern 20260716: an
  // over-denying guard locks the session out of its own fix).
  {
    const topology = resolveHoldTopology(ctx, controlRoot);
    if (topology) {
      if (holdsStoreCorrupt(topology.mainRoot)) {
        return {
          allow: false,
          kind: 'worktree-holds-unreadable',
          reason:
            'bee cross-worktree hold guard: the shared holds ledger (.bee/runtime/cross-worktree-holds.json ' +
            'in the main checkout) is present but unreadable/corrupt — failing closed rather than silently ' +
            'treating it as empty. FIX: inspect/restore the ledger in the main checkout, then retry.',
        };
      }
      let foreign = [];
      try {
        foreign = findForeignHolds(topology.mainRoot, topology.holder, [normalized]);
      } catch {
        foreign = []; // fail-open: a consultation crash never denies
      }
      if (foreign.length > 0) {
        const hold = foreign[0];
        if (isExclusivePath(root, normalized)) {
          return {
            allow: false,
            kind: 'worktree-hold',
            reason:
              `bee cross-worktree hold: "${normalized}" is held by checkout "${hold.holder}" ` +
              `(feature ${hold.feature || 'unknown'}${hold.cell ? `, cell ${hold.cell}` : ''}), ${foreignHoldExpiry(hold)}. ` +
              'Wait for the hold to expire or coordinate with that checkout — a cross-worktree hold is a hard block.',
          };
        }
        // multisession-native-14 (D4): a NORMAL (non-exclusive) path only
        // ever downgrades to advisory here — allow, but never silently (the
        // must_haves prohibition this cell adds). The warning names the
        // holding checkout/feature/expiry, same facts the old hard deny
        // reason carried, plus the merge-time consequence: worktree-store.mjs's
        // merge fence (its own P3 drift check) is what will actually surface
        // a real collision, not this write-time check.
        return {
          allow: true,
          warning:
            `bee cross-worktree hold: "${normalized}" is also held by checkout "${hold.holder}" ` +
            `(feature ${hold.feature || 'unknown'}${hold.cell ? `, cell ${hold.cell}` : ''}), ${foreignHoldExpiry(hold)} — ` +
            'advisory only (different workspace, not an exclusive resource). ' +
            `Coordinate with that checkout if possible; otherwise "bee worktree merge" will surface any real conflict ` +
            'between the two checkouts at merge time.',
        };
      }
    }
  }

  const phase = record?.phase || 'idle';

  // msn-21 (deny class (c), invariant-15 groundwork): a write into a
  // workspace a DIFFERENT live session already owns (workspace-store.mjs,
  // msn-19) is denied — but ONLY in the exact scope applyWritePolicy's
  // 'isolated' mode governs (msn-20's own "default-path feature work"
  // reasoning applied at write time, not just at startFeature): a real
  // acting session (sessionless calls have no workspace to own — skip,
  // matching applyWritePolicy's own "nothing here ACQUIRES ownership for a
  // sessionless caller"), write_policy resolved to 'isolated' (config can
  // opt out via 'observe'/'shared-disjoint', same resolution
  // applyWritePolicy uses), the DEFAULT pipeline (recordResolution.source
  // === 'default' — "lane flows keep their existing guard branches", D2's
  // "Scope boundaries" locks lanes-as-UX unchanged), and phase !== 'swarming'
  // ("swarming reservations keep their existing guard branches" — the
  // reservation block just below IS the sanctioned multi-session-in-one-
  // checkout pattern, same enforceIsolation:false reasoning
  // applyWritePolicy's own header documents for cells claim/claim-next).
  // No new hard-block CLASS beyond the three enumerated (must_haves
  // prohibition) — this only widens WHEN class (c) fires, never adds a
  // fourth.
  if (
    typeof sessionId === 'string' &&
    sessionId.trim() &&
    recordResolution.source === 'default' &&
    phase !== 'swarming' &&
    resolveWritePolicyMode(readConfig(root)) === 'isolated'
  ) {
    const ownership = checkWorkspaceOwnership(controlRoot, ctx, sessionId.trim());
    if (ownership.blocked) {
      if (ownership.corrupt) {
        return {
          allow: false,
          kind: 'workspace-unreadable',
          reason:
            `bee workspace-ownership guard: the workspace record for "${ctx.workspaceId || 'main'}" ` +
            `(${path.relative(controlRoot, workspacePath(controlRoot, ctx.workspaceId || 'main'))}) is present but ` +
            'unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as ' +
            'unowned. FIX: inspect/restore the workspace record, then retry.',
        };
      }
      return {
        allow: false,
        kind: 'workspace-ownership',
        reason:
          `bee write-policy: workspace "${ctx.workspaceId || 'main'}" is write-owned by session "${ownership.owner}" ` +
          `— a second write-capable session defaults to isolation, never a shared write into the same checkout. ` +
          'FIX: coordinate with that session, wait for its heartbeat to go stale, or start your own feature with ' +
          '`bee.mjs state start-feature --isolate` (or set guards.auto_isolate to true in .bee/config.json) to work ' +
          'in a fresh worktree instead.',
      };
    }
  }

  if (TERMINAL_PHASES.has(phase)) {
    const config = readConfig(root);
    const idleGateOn = !(config.guards && config.guards.idle_gate === false);
    if (idleGateOn && !underAllowedPrefix(normalized)) {
      return {
        allow: false,
        kind: 'intake',
        reason: intakeRefusal(phase, `writing "${normalized}"`),
      };
    }
    return { allow: true };
  }

  if (GATED_PHASES.has(phase)) {
    const executionApproved = record?.approved_gates?.execution === true;
    if (!executionApproved && !underAllowedPrefix(normalized)) {
      return {
        allow: false,
        kind: 'gate',
        reason:
          `bee gate: phase is "${phase}" and gate "execution" is not approved — ` +
          `writing "${normalized}" is blocked. Allowed now: ${GATE_ALLOWED_PREFIXES.join(', ')}. ` +
          'Get execution approval (bee-hive) before touching source files.',
      };
    }
    return { allow: true };
  }

  if (phase === 'swarming') {
    const agent = agentName || process.env.BEE_AGENT_NAME || null;
    if (agent) {
      const conflicts = findConflicts(root, agent, [normalized]);
      if (conflicts.length > 0) {
        // multisession-native-13 (D4 — advisor consult slice 3 condition D):
        // split findConflicts' matches into a HARD set (denies, exactly as
        // before this cell) and an ADVISORY set (a declared 'intent' whose
        // glob/dir scope merely covers this write, with no exact-path
        // conflict) via reservations.mjs's isHardConflict — the SAME
        // classification reserve()'s own conflict pre-check uses, so a
        // declared intent can never hard-block through either chokepoint.
        // pathsOverlap itself (what findConflicts is built on) is left
        // completely UNCHANGED: wave-scheduling (schedule.mjs/state.mjs/
        // cells.mjs) still needs broad containment to count as "overlap".
        const hardConflicts = conflicts.filter((c) => isHardConflict(c, normalized));
        if (hardConflicts.length > 0) {
          const held = hardConflicts
            .map((c) => `${c.agent} holds "${c.path}" (cell ${c.cell})`)
            .join('; ');
          return {
            allow: false,
            kind: 'reservation',
            reason:
              `bee reservation conflict: "${normalized}" is reserved by another agent — ${held}. ` +
              'Reserve the path first or return [BLOCKED] to the orchestrator.',
          };
        }
        // Every remaining conflict is an advisory intent — allow, but surface
        // a warning (the hook prints `warning` as a non-blocking notice)
        // instead of silently dropping the information (prohibition: "no
        // hard deny from an intent record").
        const warned = conflicts
          .map((c) => `${c.agent}'s declared intent "${c.path}" (cell ${c.cell}) covers "${normalized}"`)
          .join('; ');
        return {
          allow: true,
          warning: `bee reservation intent: ${warned} — advisory only (kind: intent), not a hard block.`,
        };
      }
    }
    return { allow: true };
  }

  return { allow: true };
}

// ─── shared nested/companion checkout detection (worktree-concurrency-guard,
//     cell wcg-1; D2 as widened by supersession 0ccc1cf3) ────────────────────
// The shared detection primitive for BOTH enforcement surfaces named in D1
// (bee-write-guard.mjs's write check and bee.mjs's handleWorktreeNew). It
// answers ONE question: does `targetPath` lie inside a nested/companion-shaped
// checkout — one with its own `.git` boundary — that another concurrently-live
// session could also reach? Concurrency is part of the answer (per approach.md:
// the primitive combines isConcurrentMode() with the structural check), so a
// solo checkout is a pure no-op (D6 backward compatibility).
//
// EPIC 1 SCOPE: this is exported and tested but deliberately NOT wired into
// checkWrite or the hook dispatch yet — that wiring is Epic 2/3, a later slice.
//
// The two flagged shapes and one exclusion, with their proven baselines from
// the validating spike (docs/history/worktree-concurrency-guard/reports/
// validation-e1.md):
//   (a) VERIFIED companion mount — a `.bee/companion-session.json` marker whose
//       declared worktreePath realpath matches the live mount symlink, target
//       resolving inside that mount. Allowed unconditionally today
//       (resolveCompanionMountedRelPath, bee-write-guard.mjs:384-414); flagged
//       here so a concurrency gate can cover it. An UNVERIFIED / marker-less
//       symlink escape is NOT this shape: today's containment
//       (canonicalRelPath/describeCrossWorktreeTarget) already denies it
//       regardless of concurrency, so this primitive stays narrow and does not
//       flag it (spike case A: status 2, denied by existing containment).
//   (b) PLAIN nested `.git` physically inside this checkout's own tree — a
//       distinct git repo at an ancestor dir strictly under root. Completely
//       unguarded today (spike case B: status 0) — STR65's actual incident
//       shape, the primary gap this closes.
// EXCLUDED: a real, `.gitmodules`-REGISTERED git submodule (spike case C:
// status 0, structurally identical to case B). Since a plain nested repo and a
// registered submodule cannot be told apart by "has its own `.git`" alone, the
// exclusion keys off registration evidence (`.gitmodules` `path =` entries),
// which covers BOTH submodule shapes — a directory `.git` and an absorbed-
// gitdir `.git` FILE — because it never inspects the `.git` node itself.
// Port-D4: `opts.controlRoot` (an added field alongside `excludeSessionId`,
// not a new positional param — both call sites already pass an opts object)
// scopes the isConcurrentMode() check to the coordination root, which can
// differ from the physical `root` when linked worktrees share one controlRoot
// (resolveWriteTopology's controlRoot = override || ctx.controlRoot || root,
// bee-write-guard.mjs). Falls back to `root` when omitted so a bare call
// (main/solo checkout, controlRoot === root) is byte-identical to before.
// The filesystem walk below (realpathOrNull/findNestedCheckoutDir) stays
// root-scoped always — controlRoot is coordination-state-only, never a
// filesystem location to scan.
export function isSharedNestedCheckoutTarget(root, targetPath, opts = {}) {
  // D6: additive, fires only when a second session is concurrently live.
  // Review finding F1 (worktree-concurrency-guard-controlroot-port): strict
  // mode makes a transient/hard error reading session records (EACCES, EIO,
  // EMFILE) propagate instead of silently reading as "nobody else is live"
  // — this call is the exact fail-open path the guard's own contract (a
  // detection failure denies) must hold, so it opts in where every other
  // isConcurrentMode caller keeps its existing fail-open default.
  if (!isConcurrentMode(opts.controlRoot || root, { ...opts, strict: true })) return false;

  const rootReal = realpathOrNull(root);
  if (!rootReal) return false;
  const absTarget = path.isAbsolute(targetPath)
    ? targetPath
    : path.resolve(root, String(targetPath || ''));

  // Shape (a): a marker-verified companion mount the target resolves inside.
  if (targetInsideVerifiedCompanionMount(root, absTarget)) return true;

  // Shape (b): a plain nested `.git` strictly under root — flagged unless it is
  // a registration-verified submodule.
  const nestedDir = findNestedCheckoutDir(rootReal, absTarget);
  if (nestedDir && !isRegisteredSubmodule(rootReal, nestedDir)) return true;

  return false;
}

// wcg-3: the DIRECTORY-SCAN companion to isSharedNestedCheckoutTarget, for the
// second D1 surface (bee.mjs handleWorktreeNew). The point-check above walks UP
// from a concrete write target; `bee worktree new` has no such target — it must
// answer the complementary question BEFORE any worktree is created: does
// ANYTHING companion-eligible + shared exist ANYWHERE inside this checkout that
// another concurrently-live session could also reach? So this walks DOWN from
// root, reusing the very same companion-marker verification
// (resolveVerifiedCompanionMountReal) and submodule-registration exclusion
// (isRegisteredSubmodule) as the point-check — never a second copy of either.
// Same D6 no-op contract: a solo checkout (nobody else live) always returns
// false, so a host with no concurrency, or nothing shared, sees zero change.
// Port-D4: same opts.controlRoot shape as isSharedNestedCheckoutTarget above
// — scopes isConcurrentMode() to the coordination root, falls back to `root`
// when omitted. The directory scan below stays root-scoped.
export function hasAnySharedNestedCheckout(root, opts = {}) {
  // D6: additive, fires only when a second session is concurrently live.
  // Review finding F1: strict mode, same rationale as
  // isSharedNestedCheckoutTarget above — a detection failure must deny, not
  // silently read as "solo".
  if (!isConcurrentMode(opts.controlRoot || root, { ...opts, strict: true })) return false;

  const rootReal = realpathOrNull(root);
  if (!rootReal) return false;

  // Shape (a): a marker-verified companion mount present in this checkout.
  if (resolveVerifiedCompanionMountReal(root)) return true;

  // Shape (b): any plain nested `.git` strictly under root (excluding a
  // registration-verified submodule) — the STR65 incident shape.
  return scanForNestedCheckout(rootReal, rootReal, 0);
}

// Review finding F2 (worktree-concurrency-guard-controlroot-port): a missing
// path is expected everywhere this block walks the filesystem (an ancestor
// dir that doesn't exist yet, a dangling companion-mount target) and stays
// silent. Any OTHER errno (EACCES, EIO, EMFILE, a symlink loop) is genuinely
// undetectable state, not "nothing here" — it must propagate so the caller
// (isSharedNestedCheckoutTarget/hasAnySharedNestedCheckout) fails closed
// instead of silently reading a hard error the same as "not shared". Scoped
// to this self-contained block only (these helpers have no callers outside
// it), so nothing else in guards.mjs changes behavior.
function rethrowUnlessMissing(err) {
  if (err && err.code === 'ENOENT') return;
  throw err;
}

function realpathOrNull(p) {
  try {
    return fs.realpathSync(p);
  } catch (err) {
    rethrowUnlessMissing(err);
    return null;
  }
}

// True when real path `childReal` is `parentReal` itself or strictly nested
// under it (mirrors bee-write-guard.mjs's isUnderRoot).
function isRealUnderRoot(parentReal, childReal) {
  if (!parentReal || !childReal) return false;
  const rel = path.relative(parentReal, childReal);
  return rel === '' || (rel !== '..' && !rel.startsWith(`..${path.sep}`) && !path.isAbsolute(rel));
}

// Realpath of `absPath` when it may not fully exist yet: realpath the deepest
// existing ancestor, then re-append the unresolved suffix (mirrors
// bee-write-guard.mjs's resolveTargetRealpath). Null only if nothing resolves.
function resolveExistingRealpath(absPath) {
  let cursor = absPath;
  const unresolved = [];
  for (;;) {
    const real = realpathOrNull(cursor);
    if (real) return unresolved.length ? path.resolve(real, ...unresolved) : real;
    const parent = path.dirname(cursor);
    if (parent === cursor) return null;
    unresolved.unshift(path.basename(cursor));
    cursor = parent;
  }
}

// Replicates resolveCompanionMountedRelPath's VERIFICATION (bee-write-guard.mjs
// :384-414) as the live mount's realpath, or null: a present, parseable marker
// naming a worktreePath/mountPath pair whose declared worktreePath realpath
// matches the LIVE mount symlink's realpath. A stale or tampered marker
// (mismatch) resolves to null — never grant on where the symlink happens to
// point today. Shared by the point-check (targetInsideVerifiedCompanionMount)
// and the directory-scan (hasAnySharedNestedCheckout) so the verification lives
// in exactly one place.
function resolveVerifiedCompanionMountReal(root) {
  try {
    const raw = fs.readFileSync(path.join(root, '.bee', 'companion-session.json'), 'utf8');
    const marker = JSON.parse(raw);
    const declaredWorktreePath = marker && typeof marker === 'object' ? marker.worktreePath : undefined;
    const mountPath = marker && typeof marker === 'object' ? marker.mountPath : undefined;
    if (
      typeof declaredWorktreePath !== 'string' || !declaredWorktreePath ||
      typeof mountPath !== 'string' || !mountPath
    ) {
      return null;
    }
    const declaredReal = realpathOrNull(declaredWorktreePath);
    const liveMountReal = realpathOrNull(path.join(root, mountPath));
    if (!declaredReal || !liveMountReal || declaredReal !== liveMountReal) return null;
    return liveMountReal;
  } catch (err) {
    // F2: no marker file at all is the overwhelmingly common, legitimate
    // case (return null, unverified); a read error beyond "missing" (EACCES)
    // or a corrupt/unparseable marker (JSON.parse throwing) is ambiguous
    // state this guard treats the same as any other detection failure —
    // propagate rather than silently call it "not a companion mount".
    rethrowUnlessMissing(err);
    return null;
  }
}

// True when `absTarget` resolves inside a marker-verified companion mount.
function targetInsideVerifiedCompanionMount(root, absTarget) {
  const liveMountReal = resolveVerifiedCompanionMountReal(root);
  if (!liveMountReal) return false;
  const targetReal = resolveExistingRealpath(absTarget);
  if (!targetReal) return false;
  return isRealUnderRoot(liveMountReal, targetReal);
}

// Walks realpath'd ancestor dirs of `absTarget` from just above the target up
// toward (but never reaching) root, returning the innermost dir strictly under
// root that carries its own `.git` node — the nested checkout boundary. Null
// when none exists before root, or when the walk leaves root's real tree (the
// companion-symlink case, handled separately above).
function findNestedCheckoutDir(rootReal, absTarget) {
  let cursor = absTarget;
  for (;;) {
    const parent = path.dirname(cursor);
    if (parent === cursor) return null;
    cursor = parent;
    const cursorReal = realpathOrNull(cursor);
    if (!cursorReal) continue; // dir does not exist yet — keep climbing
    if (cursorReal === rootReal) return null; // reached root, no nested boundary
    if (!isRealUnderRoot(rootReal, cursorReal)) return null; // left root's tree
    if (hasGitNode(cursorReal)) return cursorReal;
  }
}

// Directory names never descended during the down-scan — the scout-excluded
// build/dep dirs (a nested repo in node_modules/ is a dependency's own repo,
// never a companion-eligible shared checkout) plus root's own `.git`.
const NESTED_SCAN_SKIP_DIRS = new Set([
  'node_modules', 'dist', 'build', 'vendor', 'coverage', '.next', '__pycache__', '.git',
]);
// A generous physical-depth bound on the down-scan: real nested checkouts sit
// near the top of a tree, and the scan only runs at `bee worktree new` time
// while another session is live — never on the hot path.
const NESTED_SCAN_MAX_DEPTH = 8;

// Bounded, symlink-free DFS for the FIRST companion-eligible nested checkout
// strictly under `rootReal`. Skips files AND symlinks (D2 shape (b) is a
// PHYSICAL nested repo; the symlink/companion shape is covered by
// resolveVerifiedCompanionMountReal), prunes the scout-excluded dirs, and stops
// descending at any `.git`-bearing dir — a nested repo's own contents are never
// the concern. Returns true on the first non-submodule nested checkout found.
function scanForNestedCheckout(rootReal, dir, depth) {
  if (depth > NESTED_SCAN_MAX_DEPTH) return false;
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch (err) {
    // F2: a dir that vanished mid-scan (ENOENT, benign race) just prunes
    // this branch; EACCES/EIO/EMFILE mean the scan cannot honestly claim
    // "nothing found here" and must propagate instead.
    rethrowUnlessMissing(err);
    return false;
  }
  for (const entry of entries) {
    if (!entry.isDirectory()) continue; // skips regular files AND symlinks
    if (NESTED_SCAN_SKIP_DIRS.has(entry.name)) continue;
    const child = path.join(dir, entry.name);
    if (hasGitNode(child)) {
      const childReal = realpathOrNull(child);
      if (
        childReal && childReal !== rootReal &&
        isRealUnderRoot(rootReal, childReal) &&
        !isRegisteredSubmodule(rootReal, childReal)
      ) {
        return true;
      }
      continue; // a `.git`-bearing dir — never descend into a nested repo
    }
    if (scanForNestedCheckout(rootReal, child, depth + 1)) return true;
  }
  return false;
}

// A `.git` node exists whether it is a directory (plain repo / plain submodule
// checkout) or a FILE (git worktree or absorbed-gitdir submodule) — statSync
// succeeds for both, which is exactly why "has its own `.git`" cannot by itself
// distinguish a submodule from an accidental shared repo (spike cases B vs C).
function hasGitNode(dir) {
  try {
    fs.statSync(path.join(dir, '.git'));
    return true;
  } catch (err) {
    // F2: no `.git` node is the expected, overwhelmingly common answer
    // (ENOENT); a stat failing for any other reason (EACCES on the parent)
    // is undetectable state, not evidence this dir is plain.
    rethrowUnlessMissing(err);
    return false;
  }
}

// Registration-based submodule exclusion (D2): a genuine submodule is declared
// in `<root>/.gitmodules` via a `path = <repo-relative>` entry. Keys off that
// registration, never the `.git` node shape, so a directory-`.git` and a
// file-`.git` (absorbed-gitdir) submodule are both recognized. Returns false
// when `.gitmodules` is absent (nothing registered) or unreadable.
function isRegisteredSubmodule(rootReal, nestedDirReal) {
  let content;
  try {
    content = fs.readFileSync(path.join(rootReal, '.gitmodules'), 'utf8');
  } catch (err) {
    // F2: no .gitmodules is the ordinary "nothing registered" case
    // (ENOENT); any other read failure cannot honestly be called "not a
    // submodule" — it's undetectable, so it propagates.
    rethrowUnlessMissing(err);
    return false;
  }
  for (const line of content.split(/\r?\n/)) {
    const match = /^\s*path\s*=\s*(.+?)\s*$/.exec(line);
    if (!match) continue;
    const entryReal = realpathOrNull(path.resolve(rootReal, match[1]));
    if (entryReal && entryReal === nestedDirReal) return true;
  }
  return false;
}

/**
 * Privacy/scout read check. Privacy denials carry a marker the hook prints
 * so the runtime can surface the question to the human.
 */
// checkAskUserQuestion — turn the harness's opaque "Invalid tool parameters"
// rejection of an AskUserQuestion call into a CLEAR, self-documenting deny that
// names the exact schema violation, so the agent fixes it (and a screenshot
// shows the real cause). Fail-open on any shape we cannot confidently call
// invalid — never block a question we are unsure about.
export function checkAskUserQuestion(toolInput) {
  try {
    const questions =
      toolInput && Array.isArray(toolInput.questions) ? toolInput.questions : null;
    if (!questions) return { allow: true };
    if (questions.length < 1 || questions.length > 4) {
      return {
        allow: false,
        kind: 'ask-schema',
        reason: `bee AskUserQuestion guard: ${questions.length} question(s) — the tool takes 1–4 per call. Split into separate calls.`,
      };
    }
    // ask-guard-autofix D1/D2: a header over 12 chars is FIXABLE, not a deny
    // — collect the rewrite here and keep scanning the rest of this question
    // (and later questions) for any UNFIXABLE violation, which still wins
    // (deny) over any fix collected so far.
    const headerFixes = []; // { index, oldHeader, newHeader }
    for (let i = 0; i < questions.length; i += 1) {
      const q = questions[i];
      if (!q || typeof q !== 'object') continue; // odd shape — fail open
      const where = questions.length > 1 ? ` (question ${i + 1})` : '';
      if (typeof q.header === 'string' && q.header.length > 12) {
        const truncated = q.header.slice(0, 11).trimEnd();
        headerFixes.push({ index: i, oldHeader: q.header, newHeader: `${truncated}…` });
      }
      if (Array.isArray(q.options)) {
        if (q.options.length < 2 || q.options.length > 4) {
          return {
            allow: false,
            kind: 'ask-schema',
            reason: `bee AskUserQuestion guard: ${q.options.length} option(s)${where} — each question needs 2–4 options (an "Other" free-text choice is added automatically). Fold overflow into a follow-up question.`,
          };
        }
        for (let j = 0; j < q.options.length; j += 1) {
          const o = q.options[j];
          if (!o || typeof o !== 'object') continue;
          if (typeof o.label !== 'string' || !o.label.trim()) {
            return {
              allow: false,
              kind: 'ask-schema',
              reason: `bee AskUserQuestion guard: option ${j + 1}${where} is missing a non-empty "label". Every option needs a label and a description.`,
            };
          }
          if (typeof o.description !== 'string' || !o.description.trim()) {
            return {
              allow: false,
              kind: 'ask-schema',
              reason: `bee AskUserQuestion guard: option "${o.label}"${where} is missing a non-empty "description". Every option needs a label and a description.`,
            };
          }
        }
      }
    }
    if (headerFixes.length === 0) {
      return { allow: true };
    }
    // Every violation found was fixable (no unfixable violation returned
    // early above) — deep-clone toolInput, rewrite each over-long header,
    // and report the rewrite so the caller can surface it. The original
    // toolInput is never mutated.
    const fixed = JSON.parse(JSON.stringify(toolInput));
    const notes = [];
    for (const fix of headerFixes) {
      fixed.questions[fix.index].header = fix.newHeader;
      notes.push(`header "${fix.oldHeader}" (${fix.oldHeader.length} chars) → "${fix.newHeader}"`);
    }
    return { allow: true, fixed, notes };
  } catch {
    return { allow: true }; // fail-open: never block on an unexpected shape
  }
}

export function checkRead(relPath) {
  const normalized = normalizeRel(relPath);

  if (SECRET_PATTERNS.some((pattern) => pattern.test(normalized))) {
    const question = `"${normalized}" looks like a secret/credential file. Ask the user before reading it.`;
    const marker = `@@BEE_PRIVACY@@${JSON.stringify({ file: normalized, question })}@@END@@`;
    return {
      allow: false,
      kind: 'privacy',
      reason: `bee privacy guard: ${question}`,
      marker,
    };
  }

  const scoutHit = SCOUT_DIRS.find(
    (dir) => normalized.startsWith(dir) || normalized.includes(`/${dir}`),
  );
  if (scoutHit) {
    return {
      allow: false,
      kind: 'scout',
      reason:
        `bee scout guard: "${normalized}" is inside "${scoutHit}" — generated/vendored content. ` +
        'Read the source or lockfile instead.',
    };
  }

  return { allow: true };
}

const WRITE_COMMANDS = new Set(['rm', 'mv', 'cp', 'mkdir', 'touch', 'tee']);
const SEPARATORS = new Set(['&&', '||', ';', '|', '&']);
const BROAD_TARGETS = new Set(['.', '..', '/', '~', '*', './*', '/*']);

// Character-scanning tokenizer (replaces a regex-alternation approach that
// could not get all three of these right at once):
//   - Shell separators (`;`, `&`, `|`, `&&`, `||`) are always their own
//     token, even glued to the preceding text with no space
//     (`2>/dev/null;`, `rm file.txt;`) — SEPARATORS-consuming loops below
//     (git add/mv/rm, WRITE_COMMANDS, the redirect branch) already assumed
//     separators arrive as standalone tokens; a prior regex-only fix made
//     that true for THIS case but broke the next two.
//   - Adjacent quoted/unquoted segments with NO separating whitespace merge
//     into ONE token, matching bash word-splitting (`'.bee/state'".json"`
//     is the single word `.bee/state.json`). A regex-alternation tokenizer
//     that scans quotes and bare text as independent matches (no merging)
//     splits this into two tokens; only the first ever reaches a target
//     check, so `DIRECT_EDIT_DENY` (and any other containment check) can be
//     bypassed entirely by concatenating quotes around a protected path.
//   - A backslash escapes the next character literally (`a\;b.txt` is the
//     one-argument filename `a;b.txt`, not a separator) — without this, the
//     escaped separator still splits the token, and the real filename is
//     never the one that gets checked.
// Exported so guards.test.mjs can assert byte-for-byte equivalence against
// bee-write-guard.mjs's hand-synced copy (tokenize-command.mjs) — this is
// the source of truth the comment there points back to.
export function tokenize(command) {
  const str = String(command || '');
  const tokens = [];
  let current = '';
  let hasCurrent = false;
  const flush = () => {
    if (hasCurrent) {
      tokens.push(current);
      current = '';
      hasCurrent = false;
    }
  };
  let i = 0;
  while (i < str.length) {
    const ch = str[i];
    if (ch === ' ' || ch === '\t' || ch === '\n' || ch === '\r') {
      flush();
      i += 1;
      continue;
    }
    if (ch === '\\' && i + 1 < str.length) {
      current += str[i + 1];
      hasCurrent = true;
      i += 2;
      continue;
    }
    if (ch === '"' || ch === "'") {
      const close = str.indexOf(ch, i + 1);
      const end = close === -1 ? str.length : close;
      current += str.slice(i + 1, end);
      hasCurrent = true;
      i = end + 1;
      continue;
    }
    if ((ch === '&' && str[i + 1] === '&') || (ch === '|' && str[i + 1] === '|')) {
      flush();
      tokens.push(ch + ch);
      i += 2;
      continue;
    }
    if (ch === ';' || ch === '&' || ch === '|') {
      flush();
      tokens.push(ch);
      i += 1;
      continue;
    }
    current += ch;
    hasCurrent = true;
    i += 1;
  }
  flush();
  return tokens;
}

function isFlag(token) {
  return token.startsWith('-');
}

function isBroad(target) {
  const normalized = normalizeRel(target);
  return (
    BROAD_TARGETS.has(target) ||
    BROAD_TARGETS.has(normalized) ||
    normalized.endsWith('/*') ||
    normalized.endsWith('/.') ||
    normalized === '*'
  );
}

/**
 * Extract file targets a bash command may write to (khuym patterns:
 * `sed -i`, `tee`, `rm`, `mv`, `cp`, `mkdir`, `touch`, `git add|mv|rm`,
 * redirection `>`). Returns { paths, broadWrite }.
 */
export function extractBashTargets(command) {
  const tokens = tokenize(command);
  const paths = [];
  let broadWrite = false;

  const addTarget = (target) => {
    if (!target || target === '/dev/null' || target === 'NUL') return;
    if (isBroad(target)) broadWrite = true;
    paths.push(target);
  };

  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i];

    // Redirection: "> file", ">> file", ">file", "2> file".
    // NOT a file write: fd-duplication like `2>&1`, `1>&2`, `>&2` — the target
    // starts with `&` (a file descriptor, not a filename). Treating `&1` as a
    // write blocked read-only commands at idle (guards.mjs bug, decision 0014).
    const redirect = token.match(/^\d?>{1,2}(.*)$/);
    if (redirect) {
      const inline = redirect[1];
      if (inline) {
        if (!inline.startsWith('&')) addTarget(inline);
      } else if (
        tokens[i + 1] &&
        !SEPARATORS.has(tokens[i + 1]) &&
        !tokens[i + 1].startsWith('&')
      ) {
        addTarget(tokens[i + 1]);
        i += 1;
      }
      continue;
    }

    if (SEPARATORS.has(token)) continue;

    const cmd = token.replace(/\\/g, '/').split('/').pop();

    if (cmd === 'git' && ['add', 'mv', 'rm'].includes(tokens[i + 1])) {
      // D8: `git add` only STAGES already-written content — it is not itself
      // a content mutation, so staging a CLI-owned file (DIRECT_EDIT_DENY) is
      // not a direct-edit target. `git mv`/`git rm` genuinely change what's
      // on disk, so they stay fully tracked, same as any other target.
      const gitVerb = tokens[i + 1];
      let end = i + 2;
      while (end < tokens.length && !SEPARATORS.has(tokens[end])) end += 1;
      const segment = tokens.slice(i + 2, end);
      // `-A`/`--all`/`-u`/`--update` stage every changed path, not just the
      // named ones — the reservation guard must see this as a broad write.
      if (
        gitVerb === 'add' &&
        (segment.includes('--all') || segment.includes('--update') || hasGitShortFlag(segment, 'A') || hasGitShortFlag(segment, 'u'))
      ) {
        broadWrite = true;
      }
      for (const t of segment) {
        if (!isFlag(t)) {
          const cliOwnedStageOnly = gitVerb === 'add' && Object.prototype.hasOwnProperty.call(DIRECT_EDIT_DENY, normalizeRel(t));
          if (!cliOwnedStageOnly) addTarget(t);
        }
      }
      i = end - 1;
      continue;
    }

    if (cmd === 'git' && tokens[i + 1] === 'commit') {
      // `-a`/`--all`/`-am` folds tracked-but-unstaged changes into the
      // commit — blanket staging the guard must see, same as `git add -A`.
      let end = i + 2;
      while (end < tokens.length && !SEPARATORS.has(tokens[end])) end += 1;
      const segment = tokens.slice(i + 2, end);
      if (segment.includes('--all') || hasGitShortFlag(segment, 'a')) broadWrite = true;
      i = end - 1;
      continue;
    }

    if (cmd === 'sed') {
      let inPlace = false;
      let last = i;
      const args = [];
      for (let j = i + 1; j < tokens.length && !SEPARATORS.has(tokens[j]); j += 1) {
        if (tokens[j].startsWith('-i')) inPlace = true;
        else if (!isFlag(tokens[j])) args.push(tokens[j]);
        last = j;
      }
      if (inPlace) {
        // First non-flag arg is the script; the rest are files.
        for (const file of args.slice(1)) addTarget(file);
      }
      i = last;
      continue;
    }

    if (WRITE_COMMANDS.has(cmd)) {
      let sawAny = false;
      let last = i;
      for (let j = i + 1; j < tokens.length && !SEPARATORS.has(tokens[j]); j += 1) {
        if (!isFlag(tokens[j])) {
          addTarget(tokens[j]);
          sawAny = true;
        }
        last = j;
      }
      if (cmd === 'rm' && !sawAny) broadWrite = true;
      i = last;
      continue;
    }
  }

  return { paths, broadWrite };
}

// ─── internals-reach guard (state-query-surface, cell sqs-a, D 3fbe2f79) ──
//
// Denies ONLY the inline-eval reach — `node -e`/`--eval`/`-p` whose script
// text imports/requires a `bin/lib/` or `packages/bee/lib/` module — never a
// file-based `node <path>.mjs` run (tests legitimately import lib modules
// that way, and this guard must never trap them). The reach fetches
// internals with no compatibility promise, by the worst possible path, when
// the same data is already a paved public read (`bee status --json`,
// `bee <group> --help --json`) — the FIX line always names both.
const NODE_INVOCATION_BASENAMES = new Set(['node', 'nodejs']);
const INLINE_EVAL_FLAGS = new Set(['-e', '--eval', '-p']);
const LIB_IMPORT_SPECIFIER_RE =
  /(?:\brequire\s*\(\s*|\bimport\s*\(\s*|\bimport\b[^'"()]*\bfrom\s*)['"`]([^'"`]+)['"`]/g;

/** Every inline-eval script string found in one `&&`/`;`/`|`-separated
 * command segment: the token(s) right after a bare `-e`/`--eval`/`-p` flag,
 * plus the attached `--eval=<script>` form. */
function inlineEvalScriptsInSegment(segment) {
  const scripts = [];
  for (let i = 0; i < segment.length; i += 1) {
    const token = segment[i];
    if (INLINE_EVAL_FLAGS.has(token)) {
      const next = segment[i + 1];
      if (typeof next === 'string') scripts.push(next);
      continue;
    }
    const attached = /^--eval=(.*)$/.exec(token);
    if (attached) scripts.push(attached[1]);
  }
  return scripts;
}

/** First `import(...)`/`require(...)`/`import ... from '...'` specifier in
 * `script` that resolves into a `bin/lib/` or `packages/bee/lib/` module, or
 * null when the script never reaches into either. */
function libImportSpecifierIn(script) {
  if (typeof script !== 'string' || !script) return null;
  LIB_IMPORT_SPECIFIER_RE.lastIndex = 0;
  let match;
  while ((match = LIB_IMPORT_SPECIFIER_RE.exec(script))) {
    const specifier = match[1];
    if (specifier && /(^|\/)(?:bin\/lib|packages\/bee\/lib)(\/|$)/.test(specifier)) {
      return specifier;
    }
  }
  return null;
}

/**
 * Bash-command guard (D 3fbe2f79): deny an inline-eval `node -e`/`--eval`/
 * `-p` command whose script text imports/requires a `bin/lib/` or
 * `packages/bee/lib/` module. Returns:
 *   null                    — not a node inline-eval invocation, or the
 *                              script never reaches into bin/lib or
 *                              packages/bee/lib (includes every file-based
 *                              `node <path>.mjs` run — never blocked here).
 *   { allow: false, reason } — the inline eval reaches an internal module;
 *                              `reason` names the paved read instead.
 */
export function checkBinLibImportBashCommand(command) {
  const str = typeof command === 'string' ? command : '';
  if (!str.trim()) return null;
  const tokens = tokenize(str);

  let i = 0;
  while (i < tokens.length) {
    if (SEPARATORS.has(tokens[i])) {
      i += 1;
      continue;
    }
    let end = i;
    while (end < tokens.length && !SEPARATORS.has(tokens[end])) end += 1;
    const segment = tokens.slice(i, end);
    i = end;

    const cmd = (segment[0] || '').replace(/\\/g, '/').split('/').pop();
    if (!NODE_INVOCATION_BASENAMES.has(cmd)) continue;

    for (const script of inlineEvalScriptsInSegment(segment)) {
      const specifier = libImportSpecifierIn(script);
      if (specifier) {
        return {
          allow: false,
          reason:
            `bee internals-reach guard: this inline eval imports "${specifier}" — a bin/lib/ or ` +
            'packages/bee/lib/ internal module, reached via `node -e`/`--eval`/`-p` rather than the CLI. ' +
            'Internals carry no compatibility promise and this bypasses the CLI\'s own validation. ' +
            'FIX: use the paved read instead — `bee status --json` for current state, or ' +
            '`bee <group> --help --json` for a command group\'s full schema. ' +
            '(File-based `node <path>.mjs` runs that import lib modules, e.g. tests, are unaffected.)',
        };
      }
    }
  }

  return null;
}
