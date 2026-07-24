// worktree-store.mjs — worktree-feature-parallelism S2: the NEW decision +
// replay logic proven by .bee/spikes/worktree-feature-parallelism/seam-proof.mjs
// (5/5 passed). This module is NOT YET WIRED — nothing in production imports
// it. Git-worktree CLASSIFICATION (ordinary / linked-valid / linked-invalid)
// already lives in production `resolveRoots` and is intentionally NOT
// duplicated here; this module only picks up from a `classification` object
// that the wire-in slice will have `resolveRoots` produce.
//
// Wire-in plan (future slice, not this one): `resolveRoots` starts returning
// `{ kind, id, mainRoot, worktreeRoot }` (same shape `classify()` builds in
// the spike), and the caller does:
//
//   const grants = readGrants(path.join(mainRoot, '.bee'));
//   const decision = decideWorktreeStore(classification, { grants });
//
// Zero deps beyond node: built-ins, EXCEPT `releaseAllForHolder` (xwh-2,
// imported below from worktree-holds.mjs) and `withStoreLock` (hardening-4b,
// imported below from lock.mjs) — the two intentional exceptions.
// releaseAllForHolder is wired into performCleanup below so a removed
// worktree's mirrored cross-worktree holds are released alongside its grant.
// withStoreLock serializes writeGrant/removeGrant/createFeatureWorktree/
// mergeFeatureWorktree (+ performCleanup, which joins its caller's held
// lock rather than re-acquiring) under one 'worktree-admin' lock on the
// MAIN store — the audit finding was that two concurrent worktree admin ops
// (e.g. `new` racing `merge`, or two `register`s) could interleave their
// read-check-write of runtime/worktree-grants.json and the worktree
// lifecycle itself. No cycle: neither worktree-holds.mjs nor lock.mjs import
// this module. Node 18+.

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { releaseAllForHolder } from './worktree-holds.mjs';
import { withStoreLock } from './lock.mjs';

// ---------------------------------------------------------------------------
// readGrants — load the MAIN store's grant registry.
// ---------------------------------------------------------------------------

/**
 * Reads <mainStoreRoot>/runtime/worktree-grants.json.
 *
 * `mainStoreRoot` is the MAIN checkout's `.bee` directory (never a
 * worktree's own `.bee`) — that asymmetry is the whole security point, see
 * decideWorktreeStore below. Returns `{}` on any missing file, unreadable
 * file, or malformed JSON — this never throws, because a throw here would
 * propagate into a fail-open hook and become a silent allow.
 */
export function readGrants(mainStoreRoot) {
  const grantsFile = path.join(mainStoreRoot, 'runtime', 'worktree-grants.json');
  try {
    const parsed = JSON.parse(fs.readFileSync(grantsFile, 'utf8'));
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

// ---------------------------------------------------------------------------
// decideWorktreeStore — the NEW decision layer. PURE: no fs, no throw.
// ---------------------------------------------------------------------------

/**
 * Decides which `.bee` store a write should land in, given a checkout's git
 * CLASSIFICATION (already computed by production `resolveRoots`) and the
 * grants already read from the MAIN store's registry.
 *
 * `classification` shape (what the wire-in slice will feed):
 *   { kind: 'ordinary' | 'linked-valid' | 'linked-invalid',
 *     id?: string, mainRoot?: string, worktreeRoot?: string }
 *
 * `options.grants` is the object returned by `readGrants(mainStoreRoot)` —
 * always read from the MAIN checkout's store. This function takes grants as
 * a plain argument and never reads any filesystem itself: structurally it
 * cannot see a worktree's own `.bee/`, because nothing about `worktreeRoot`
 * is ever used to locate a registry to read. A worktree could write any
 * self-claiming marker it likes inside its own `.bee/runtime/` and this
 * function would never look there — the only trusted grant source is the
 * `grants` object the caller supplies, which the wire-in plan always sources
 * from the main store. This is the security property proven by spike case 4
 * ("self-written grant marker is ignored").
 *
 * Returns one of:
 *   { ok: true,  kind: 'ordinary',              storeRoot }
 *   { ok: false, reason: 'WORKTREE_LINK_INVALID' }
 *   { ok: true,  kind: 'linked-valid-granted',   storeRoot, id }
 *   { ok: true,  kind: 'linked-valid-default',   storeRoot, id }
 *
 * NEVER throws — an invalid/unrecognized classification also collapses to a
 * typed deny rather than an exception, for the same fail-open-hook reason
 * readGrants never throws.
 */
export function decideWorktreeStore(classification, { grants } = {}) {
  try {
    const safeGrants = grants && typeof grants === 'object' ? grants : {};
    const kind = classification && classification.kind;

    if (kind === 'ordinary') {
      // Own store: the checkout's own .bee. worktreeRoot is absent for a
      // true ordinary checkout, so fall back to mainRoot — kept simple and
      // documented per the brief, rather than requiring callers to always
      // populate both fields identically.
      const root = classification.worktreeRoot ?? classification.mainRoot;
      return { ok: true, kind: 'ordinary', storeRoot: path.join(root, '.bee') };
    }

    if (kind === 'linked-invalid') {
      // Typed deny, never a throw: this models a fail-open hook where an
      // uncaught exception would otherwise collapse to a silent allow.
      return { ok: false, reason: 'WORKTREE_LINK_INVALID' };
    }

    if (kind === 'linked-valid') {
      const { id, mainRoot, worktreeRoot } = classification;
      if (safeGrants[id] === true) {
        return {
          ok: true,
          kind: 'linked-valid-granted',
          storeRoot: path.join(worktreeRoot, '.bee'),
          id,
        };
      }
      // Not granted -> P40 default: fall back to the main store.
      return {
        ok: true,
        kind: 'linked-valid-default',
        storeRoot: path.join(mainRoot, '.bee'),
        id,
      };
    }

    // Unrecognized classification kind: fail closed, typed, no throw.
    return { ok: false, reason: 'WORKTREE_LINK_INVALID' };
  } catch {
    return { ok: false, reason: 'WORKTREE_LINK_INVALID' };
  }
}

// ---------------------------------------------------------------------------
// replayLog — pure, deterministic, idempotent projection over an event log.
// ---------------------------------------------------------------------------

/**
 * Sorts `events` by `(ts, id)`, dedups by `id` (last one after the sort
 * wins), and folds the result into a plain object keyed by id. Pure: does
 * not mutate `events`, performs no I/O, and calling it twice with the same
 * input yields byte-identical (JSON.stringify-equal) output.
 */
export function replayLog(events) {
  const sorted = [...events].sort((a, b) => {
    if (a.ts !== b.ts) return a.ts - b.ts;
    return String(a.id).localeCompare(String(b.id));
  });

  const map = new Map();
  for (const ev of sorted) {
    map.set(ev.id, ev); // dedup by id: last wins after the sort above
  }

  const ids = [...map.keys()].sort();
  const state = {};
  for (const id of ids) state[id] = map.get(id);
  return state;
}

// ---------------------------------------------------------------------------
// writeGrant / removeGrant / listGrants — MAIN store grant registry mutators
// (worktree-feature-parallelism Slice A: the wire-in above is read-only —
// nothing before this point could ever put a `true` into the registry.
// These are the write-side companion to readGrants: readGrants itself is
// left completely untouched, still the fail-open, read-only primitive
// resolveRoots depends on).
// ---------------------------------------------------------------------------

function grantsFile(mainStoreRoot) {
  return path.join(mainStoreRoot, 'runtime', 'worktree-grants.json');
}

function writeGrantsFileAtomic(mainStoreRoot, grants) {
  const file = grantsFile(mainStoreRoot);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const tmp = `${file}.tmp`;
  fs.writeFileSync(tmp, `${JSON.stringify(grants, null, 2)}\n`, 'utf8');
  fs.renameSync(tmp, file);
}

// hardening-4b: the UNLOCKED core write — used directly by callers that
// already hold the 'worktree-admin' lock themselves (createFeatureWorktree,
// mergeFeatureWorktree/performCleanup below), so they never try to
// re-acquire the same lock they're already inside (withStoreLock is not
// reentrant — a nested acquisition by the same call stack would simply wait
// out its own hold and time out). The exported `writeGrant` wraps this with
// the lock for every OTHER, standalone caller (bee.mjs's `worktree
// register`).
function writeGrantCore(mainStoreRoot, id) {
  const next = { ...readGrants(mainStoreRoot), [id]: true };
  writeGrantsFileAtomic(mainStoreRoot, next);
  return next;
}

// hardening-4b: same unlocked/locked split as writeGrantCore/writeGrant.
function removeGrantCore(mainStoreRoot, id) {
  const existing = readGrants(mainStoreRoot);
  if (!(id in existing)) return existing;
  const next = { ...existing };
  delete next[id];
  writeGrantsFileAtomic(mainStoreRoot, next);
  return next;
}

/**
 * Merges `{ [id]: true }` into <mainStoreRoot>/runtime/worktree-grants.json,
 * preserving every other entry already on disk. Creates the runtime/ dir and
 * the grants file if either is missing. Atomic write (tmp file + rename),
 * the same pattern fsutil.mjs's writeJsonAtomic uses (not imported directly,
 * to keep this module's zero-deps-beyond-node-builtins contract intact
 * beyond the two documented exceptions in the module header).
 *
 * hardening-4b: serialized under withStoreLock(mainRoot, 'worktree-admin')
 * — `mainStoreRoot` is always `<mainRoot>/.bee` at every call site in this
 * codebase (see the module header's wire-in note), so `path.dirname` recovers
 * `mainRoot` for the lock without a second parameter.
 */
export async function writeGrant(mainStoreRoot, id) {
  return withStoreLock(path.dirname(mainStoreRoot), 'worktree-admin', () => writeGrantCore(mainStoreRoot, id));
}

/**
 * Deletes `id` from the MAIN store's grant registry. A no-op (returns the
 * registry unchanged, no write) when `id` was never present or the file
 * does not exist yet.
 *
 * hardening-4b: serialized under withStoreLock(mainRoot, 'worktree-admin'),
 * same rationale as writeGrant above.
 */
export async function removeGrant(mainStoreRoot, id) {
  return withStoreLock(path.dirname(mainStoreRoot), 'worktree-admin', () => removeGrantCore(mainStoreRoot, id));
}

/**
 * Returns the MAIN store's grant registry object. A thin named alias over
 * readGrants for the `bee worktree list` CLI surface — deliberately not a
 * second read implementation.
 */
export function listGrants(mainStoreRoot) {
  return readGrants(mainStoreRoot);
}

// ---------------------------------------------------------------------------
// bootstrapWorktreeStore — set up a newly granted worktree's OWN .bee/ so
// bee actually works there (Slice A: the resolver has honored a granted
// worktree's local store since the wire-in slice, but until this function
// existed nothing ever populated that local store, so a granted worktree
// resolved to a store that was simply empty).
// ---------------------------------------------------------------------------

// Mirrors state.mjs's defaultState() schema/gate shape exactly. NOT imported
// from state.mjs: state.mjs imports readGrants FROM this module, so an
// import the other way would be a cycle. Kept as a literal here on purpose.
const FRESH_STATE_SCHEMA_VERSION = '1.0';

// issues-46-53 D4 (#46): the worktree's IMMUTABLE creation identity.
//
// A worktree's directory name, its branch and its `state.feature` all derive
// from ONE slug at creation (createFeatureWorktreeLocked) — the "three
// independent names" the issue reported do not exist. What DOES drift is
// `state.feature`, which `bee state set` / `startFeature` rewrite freely with
// no worktree awareness at all, and the paved road makes that drift
// near-inevitable: the agent is told to create the worktree at session-scout
// time, BEFORE exploring settles the feature's real name. Merge then derived
// its expected branch from that mutated field and refused, blaming the branch
// — the one thing the user must NOT change. So the branch merge expects can
// never be read from a mutable field; it is recorded here once, at bootstrap,
// and preferred forever after (see resolveWorktreeFeature below).
//
// The path is under `.bee/runtime/` deliberately: that prefix is gitignored
// both in this repo and in every host repo onboarding sets up, so this record
// can never make a worktree read "dirty" to the `git status --porcelain`
// pre-check mergeFeatureWorktree runs — the same constraint that forces
// `.bee/companion-session.json` to be torn down before that check.
const WORKTREE_IDENTITY_REL = path.join('runtime', 'worktree-identity.json');

/**
 * Records `feature` as this worktree's creation slug — WRITE-IF-ABSENT, never
 * overwritten: "immutable" is the entire point, so a re-bootstrap (or any
 * later rename) must not be able to restate it.
 *
 * Never throws. This record is an ACCURACY improvement over `state.feature`,
 * not a prerequisite: a worktree without it degrades to exactly the pre-fix
 * behavior (resolveWorktreeFeature falls back to `state.feature`), so a
 * failure here must never take down an otherwise-successful bootstrap.
 */
function writeCreationIdentity(worktreeStoreRoot, feature) {
  if (typeof feature !== 'string' || !feature) {
    return { written: false, reason: 'no feature slug given' };
  }
  const file = path.join(worktreeStoreRoot, WORKTREE_IDENTITY_REL);
  if (fs.existsSync(file)) {
    return { written: false, reason: 'creation identity already recorded — never overwritten' };
  }
  try {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    const tmp = `${file}.tmp`;
    fs.writeFileSync(tmp, `${JSON.stringify({ feature, created_at: new Date().toISOString() }, null, 2)}\n`, 'utf8');
    fs.renameSync(tmp, file);
    return { written: true, feature, file };
  } catch (error) {
    return { written: false, reason: error instanceof Error ? error.message : String(error) };
  }
}

/**
 * Creates <worktreeRoot>/.bee/ if missing, copies onboarding.json and
 * config.json from the MAIN store when present (copy-if-absent, never
 * overwrite — a worktree has no installer of its own to produce them), and
 * writes a FRESH state.json for the worktree: `feature` set, `phase: 'idle'`,
 * every gate false. An independent-feature worktree runs its OWN lifecycle —
 * main's live phase/gates/workers/log are deliberately NOT copied, so a
 * worktree can never inherit a gate approval it never earned locally.
 *
 * Idempotent: if <worktreeRoot>/.bee/state.json already exists, this call
 * does NOT overwrite it — re-running bootstrap must never clobber real
 * in-progress worktree state. onboarding.json/config.json follow the same
 * copy-if-absent rule independently of state.json's presence.
 *
 * issues-46-53 D4 (#46): ALSO records `feature` as the worktree's IMMUTABLE
 * creation slug (`.bee/runtime/worktree-identity.json`, write-if-absent) —
 * deliberately BEFORE the state.json-already-exists early return, so an
 * adopted worktree (`bee worktree register --feature <slug>`) gets one too.
 * Reported as `.identity` on every return shape; never fatal.
 */
export function bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, feature) {
  const worktreeStoreRoot = path.join(worktreeRoot, '.bee');
  fs.mkdirSync(worktreeStoreRoot, { recursive: true });

  const copyIfAbsent = (name) => {
    const dest = path.join(worktreeStoreRoot, name);
    if (fs.existsSync(dest)) return { copied: false, reason: `${name} already exists` };
    const src = path.join(mainStoreRoot, name);
    if (!fs.existsSync(src)) return { copied: false, reason: `main store has no ${name}` };
    fs.copyFileSync(src, dest);
    return { copied: true };
  };

  const onboarding = copyIfAbsent('onboarding.json');
  const config = copyIfAbsent('config.json');
  const identity = writeCreationIdentity(worktreeStoreRoot, feature);

  const stateFile = path.join(worktreeStoreRoot, 'state.json');
  if (fs.existsSync(stateFile)) {
    return { created: false, reason: 'state.json already exists', worktreeStoreRoot, onboarding, config, identity };
  }

  const freshState = {
    schema_version: FRESH_STATE_SCHEMA_VERSION,
    phase: 'idle',
    feature: feature ?? null,
    mode: null,
    approved_gates: { context: false, shape: false, execution: false, review: false },
    workers: [],
    summary: '',
    next_action: 'Invoke bee-hive.',
  };
  const tmp = `${stateFile}.tmp`;
  fs.writeFileSync(tmp, `${JSON.stringify(freshState, null, 2)}\n`, 'utf8');
  fs.renameSync(tmp, stateFile);

  return { created: true, worktreeStoreRoot, onboarding, config, identity, state: freshState };
}

// ---------------------------------------------------------------------------
// createFeatureWorktree — "bee worktree new --feature <slug>" (GH #21,
// decision D7): create AND register a fresh linked git worktree for an
// independent feature in one move, instead of a human running `git worktree
// add` by hand and then `bee worktree register`. Folds the two into a
// single atomic-as-git-allows step: `git worktree add`, then the EXACT
// writeGrant + bootstrapWorktreeStore sequence `worktree register` already
// performs.
//
// `mainRoot` MUST already be a resolved ORDINARY checkout root — the CLI
// caller proves this the same way `handleWorktreeRegister` proves its own
// worktree link is valid: by calling `resolveRoots(process.cwd())` (see
// bee.mjs's `handleWorktreeNew`). This function deliberately does NOT import
// `resolveRoots` itself: state.mjs imports `readGrants` FROM this module (see
// the FRESH_STATE_SCHEMA_VERSION comment above), so importing state.mjs back
// would be a cycle, and it would break this module's "zero deps beyond node
// builtins" contract (module header). Instead it re-derives the same
// ordinary-vs-linked distinction `resolveRoots` uses directly against
// `mainRoot` (a `.git` FILE means "this checkout is itself a linked
// worktree"; a `.git` DIRECTORY means "ordinary") as its own independent
// guard — belt-and-braces, not a substitute for the CLI's check.
// ---------------------------------------------------------------------------

const FEATURE_SLUG_RE = /^[a-z0-9][a-z0-9-]*$/;

/** Typed refusal for createFeatureWorktree: `.code` is a stable string (e.g.
 * "WORKTREE_TARGET_EXISTS"), `.message` already carries the SAME code as a
 * "[CODE] ..." prefix (the convention handleConfigValidate's `[${p.code}] ...`
 * formatting already uses elsewhere in this codebase) because bee.mjs's
 * dispatcher only ever surfaces a caught error's bare `.message` to the CLI
 * caller (emitError), never a separate `.code` field. */
export class WorktreeCreateError extends Error {
  constructor(code, message) {
    super(`[${code}] ${message}`);
    this.name = 'WorktreeCreateError';
    this.code = code;
  }
}

function refuse(code, message) {
  throw new WorktreeCreateError(code, message);
}

/** Same distinction `resolveRoots` draws (a `.git` FILE = linked worktree, a
 * `.git` DIRECTORY = ordinary checkout) — re-derived here, not imported, per
 * the module-cycle note above. */
function isOrdinaryCheckout(root) {
  try {
    return fs.statSync(path.join(root, '.git')).isDirectory();
  } catch {
    return false;
  }
}

function runGit(cwd, args) {
  return spawnSync('git', args, { cwd, encoding: 'utf8' });
}

/**
 * Resolves a `--base-ref` commit-ish to a concrete commit sha via `git
 * rev-parse --verify --end-of-options "<baseRef>^{commit}"` — NOT `git
 * check-ref-format`, which only validates ref-NAME syntax and wrongly
 * rejects/mishandles perfectly valid commit-ish forms `check-ref-format`
 * was never meant to judge: `HEAD~1` (the `~` is a disallowed ref-name
 * character), a short sha (not a ref at all), or `v1.2.0^{commit}` (the `^`
 * and `{`/`}` are disallowed too) — decision D3 / advisor R8. `rev-parse
 * --verify` instead proves the commit-ish actually RESOLVES to a real
 * commit in THIS repo, which is the property `createFeatureWorktree`
 * actually needs before handing it to `git worktree add`.
 *
 * `--end-of-options` (requires git >= 2.24) tells git everything after it
 * is a positional revision argument, never an option — this is what stops a
 * `baseRef` value starting with `-` (e.g. `--upload-pack=evil`) from being
 * parsed as a git flag instead of data, even though it already arrives as
 * its own argv array element (spawnSync, no shell) rather than a
 * shell-interpolated string.
 *
 * Returns the resolved, full commit sha on success, or `null` when the
 * commit-ish doesn't resolve — bad syntax and "doesn't exist" both land
 * here as the SAME `null` (git's own `rev-parse --verify` doesn't
 * distinguish the two either: both fail with "Needed a single revision").
 * The call site collapses both into one typed refusal, `WORKTREE_BASE_NOT_FOUND`
 * — see that refusal's comment for why no separate syntax-only code remains.
 */
function resolveBaseRefCommit(cwd, ref) {
  const result = runGit(cwd, ['rev-parse', '--verify', '--end-of-options', `${ref}^{commit}`]);
  if (result.status !== 0) return null;
  const sha = (result.stdout || '').trim();
  return sha || null;
}

function branchExists(mainRoot, branch) {
  return runGit(mainRoot, ['rev-parse', '--verify', '--quiet', `refs/heads/${branch}`]).status === 0;
}

/**
 * Reads the git-verified worktree id from a freshly-created worktree's own
 * `.git` file — the SAME bidirectional-gitdir mechanism `resolveRoots` (and
 * scripts/test_worktree_cli.mjs's `verifiedId()`) already use. Never assumes
 * id === directory basename: git only defaults a new worktree's id to the
 * sibling directory's own basename when that id is free, and silently
 * suffixes a counter on collision (`<basename>1`, `<basename>2`, ...).
 */
function readWorktreeGitVerifiedId(worktreeRoot) {
  const gitFile = path.join(worktreeRoot, '.git');
  const raw = fs.readFileSync(gitFile, 'utf8').trim();
  const match = raw.match(/^gitdir:\s*(.+)$/);
  if (!match) {
    throw new Error(`worktree .git file at ${gitFile} is not a valid "gitdir: ..." pointer`);
  }
  const gitdir = path.resolve(worktreeRoot, match[1].trim().replace(/\\/g, path.sep));
  return path.basename(gitdir);
}

// ---------------------------------------------------------------------------
// syncWorktreeSkills — best-effort: clone the bee-* skill directories
// (`.claude/skills/bee-*`, `.agents/skills/bee-*`) from the MAIN checkout
// into a freshly created worktree by plain recursive copy.
//
// Those trees are gitignored in a HOST repo (onboard_bee.mjs installs them
// as "installed tooling, regenerable from their sources"), so `git worktree
// add` never brings them into a new worktree, and bootstrapWorktreeStore
// above only ever populated `.bee/` (onboarding.json/config.json/
// state.json). Left alone, every new worktree ends up with a working `bee`
// CLI but ZERO discoverable bee-* skill files for the next session opened
// there — exactly the gap this closes.
//
// Deliberately a plain copy, NOT a call into onboard_bee.mjs: that script's
// source-identity guard refuses to treat a rendered projection as a sync
// source (`blocked_no_source` — "a projection is never an authoritative
// source for any target"), and a HOST repo's own `.claude/skills/bee-hive`
// is exactly such a projection (carries the `.bee-render.json` provenance
// marker). A worktree copy has a fundamentally different safety profile
// than an onboarding sync, though: it's not "upgrade this repo from a
// canonical source," it's "give this sibling worktree the exact same,
// already-working skill files this checkout already has" — no version
// authority question to adjudicate, so the plain copy needs none of that
// script's machinery.
//
// NEVER throws and never blocks/rolls back worktree creation on failure:
// missing/stale skills are regenerable tooling, not a reason to discard an
// otherwise-successful `git worktree add`.
// ---------------------------------------------------------------------------

const SKILL_SYNC_ROOTS = [
  path.join('.claude', 'skills'),
  path.join('.agents', 'skills'),
];

export function syncWorktreeSkills(mainRoot, worktreeRoot) {
  const synced = [];
  const skipped = [];

  for (const rel of SKILL_SYNC_ROOTS) {
    const srcRoot = path.join(mainRoot, rel);
    let entries;
    try {
      entries = fs.readdirSync(srcRoot, { withFileTypes: true });
    } catch {
      continue; // main checkout has no such root — nothing to sync from it.
    }

    for (const entry of entries) {
      if (!entry.isDirectory() || !entry.name.startsWith('bee-')) continue;
      const relPath = path.join(rel, entry.name);
      const dest = path.join(worktreeRoot, relPath);
      if (fs.existsSync(dest)) {
        skipped.push({ path: relPath, reason: 'already exists in worktree' });
        continue;
      }
      try {
        fs.mkdirSync(path.dirname(dest), { recursive: true });
        fs.cpSync(path.join(srcRoot, entry.name), dest, { recursive: true });
        synced.push(relPath);
      } catch (error) {
        skipped.push({ path: relPath, reason: error instanceof Error ? error.message : String(error) });
      }
    }
  }

  if (synced.length === 0 && skipped.length === 0) {
    return {
      attempted: false,
      applied: false,
      reason: `no bee-* skill directories found under ${SKILL_SYNC_ROOTS.join(' or ')} in the main checkout.`,
    };
  }
  if (synced.length === 0) {
    return { attempted: true, applied: false, reason: 'already present in worktree', synced, skipped };
  }
  const reason =
    skipped.length > 0
      ? `synced ${synced.length} bee-* skill dir(s), ${skipped.length} skipped (see .skipped)`
      : `synced ${synced.length} bee-* skill dir(s)`;
  return { attempted: true, applied: true, reason, synced, skipped };
}

/**
 * Creates a NEW linked git worktree for `feature` — `git worktree add
 * <mainRoot's sibling>--wt--<feature> -b wt/<feature> [baseRefSha]` — then
 * grants and bootstraps it exactly as `worktree register` does. Returns
 * `{ id, worktreeRoot, branch, baseRef, baseRefSha, bootstrap }` on success.
 *
 * `options`:
 *   - `feature` (required): slug, validated against `FEATURE_SLUG_RE`.
 *   - `baseRef` (optional): a commit-ish (branch, tag, `HEAD~N`, short sha,
 *     `<tag>^{commit}`, ...), resolved to a concrete commit sha via `git
 *     rev-parse --verify --end-of-options` when given (see
 *     `resolveBaseRefCommit`'s own comment for why not `check-ref-format`;
 *     requires git >= 2.24 for `--end-of-options`).
 *   - `_writeGrant` / `_bootstrapWorktreeStore` / `_syncWorktreeSkills`:
 *     internal test-only injection points (default to the real
 *     `writeGrant` / `bootstrapWorktreeStore` / `syncWorktreeSkills`
 *     exports above) so a test can force the POST-add failure + rollback
 *     path, or stub the skill sync, deterministically without needing a
 *     real bug.
 *
 * Every pre-flight check below is a typed, ZERO-MUTATION refusal (its own
 * stable `WorktreeCreateError.code`, nothing touched on disk or in git)
 * EXCEPT the last line of defense, `git worktree add` itself: the
 * pre-checks are advisory (best-effort races against concurrent/stale
 * state), git's own failure is authoritative and is caught and re-surfaced
 * typed too (`WORKTREE_ADD_FAILED`). A failure AFTER `git worktree add`
 * itself succeeded (deriving the id, writing the grant, or bootstrapping the
 * store throwing) is rolled back best-effort (`git worktree remove --force`
 * + best-effort `removeGrant`); if that rollback itself fails, the error
 * says the tree can be adopted via `bee worktree register`.
 *
 * hardening-4b: the ENTIRE body below (every pre-check through the rollback
 * path) now runs inside withStoreLock(mainRoot, 'worktree-admin') — one
 * critical section, serialized against writeGrant/removeGrant/
 * mergeFeatureWorktree/cleanup. `_writeGrant`'s default is the UNLOCKED
 * `writeGrantCore` (never the exported, lock-acquiring `writeGrant`) and the
 * rollback path below calls `removeGrantCore` directly, for the same
 * non-reentrancy reason: this function is already inside the lock by the
 * time either would run, and withStoreLock is not reentrant. This is why the
 * function is `async` now (previously fully synchronous).
 */
export async function createFeatureWorktree(mainRoot, options = {}) {
  const {
    feature,
    baseRef,
    companionStartCommand,
    companionMountPath,
    _writeGrant = writeGrantCore,
    _bootstrapWorktreeStore = bootstrapWorktreeStore,
    _syncWorktreeSkills = syncWorktreeSkills,
  } = options;
  return withStoreLock(mainRoot, 'worktree-admin', () =>
    createFeatureWorktreeLocked(mainRoot, {
      feature,
      baseRef,
      companionStartCommand,
      companionMountPath,
      _writeGrant,
      _bootstrapWorktreeStore,
      _syncWorktreeSkills,
    }),
  );
}

/**
 * Validates a `commands.worktree_companion_mount` value: a non-empty,
 * relative path with no leading "/" and no ".." segment — it becomes a
 * symlink target INSIDE the new worktree, so an absolute path or a
 * traversal segment would place (or escape) it somewhere the worktree
 * doesn't own. Typed refusal, zero mutation, same posture as every other
 * pre-check in createFeatureWorktreeLocked below.
 */
function validateCompanionMountPath(mountPath) {
  if (typeof mountPath !== 'string' || !mountPath.trim()) {
    refuse('WORKTREE_COMPANION_CONFIG_INVALID', `commands.worktree_companion_mount must be a non-empty relative path string, got ${JSON.stringify(mountPath)}.`);
  }
  const normalized = mountPath.trim();
  if (path.isAbsolute(normalized) || normalized.split(/[\\/]/).includes('..')) {
    refuse(
      'WORKTREE_COMPANION_CONFIG_INVALID',
      `commands.worktree_companion_mount ${JSON.stringify(normalized)} must be a relative path inside the worktree (no leading "/" and no ".." segments).`,
    );
  }
  return normalized;
}

/**
 * Runs the project-configured `commands.worktree_companion_start` (worktree-
 * companion-hook) and wires its result into the freshly created worktree.
 * bee never hardcodes what the companion tool is — a host project's own
 * `.bee/config.json` value is the only place any tool-specific knowledge
 * lives; this function's only contract on stdout is JSON with a non-empty
 * `worktreePath` string (and, optionally, `sessionId` — carried through to
 * the marker file for `worktree merge` to substitute into
 * `worktree_companion_end`, never required: a companion tool need not have a
 * "session id" concept at all).
 *
 * Runs with `mainRoot` as cwd — same root `handleWorktreeNew` resolves the
 * command from — so the configured command is responsible for its own `cd`
 * into whatever nested tree it isolates (mirrors how `commands.verify` is
 * already resolved and run against `mainRoot`, never the worktree).
 *
 * On success: symlinks `worktreePath` at `<worktreeRoot>/<mountPath>` and
 * writes `<worktreeRoot>/.bee/companion-session.json` (`{sessionId,
 * worktreePath, mountPath}`) so `worktree merge` can find and tear it down
 * again without needing any flag of its own — the marker's mere presence is
 * the signal (see `teardownCompanionIfPresent` below).
 *
 * Throws a plain Error on non-zero exit, unparseable stdout, or a missing
 * `worktreePath` — the caller (createFeatureWorktreeLocked's existing
 * try/catch) folds that into the SAME post-add rollback path as any other
 * failure after `git worktree add` itself succeeded: a worktree is never
 * left half-configured (created, but silently missing its companion).
 */
function runCompanionStart(mainRoot, worktreeRoot, companionStartCommand, mountPath) {
  const spawned = spawnSync(companionStartCommand, { cwd: mainRoot, shell: true, encoding: 'utf8' });
  if (spawned.status !== 0) {
    throw new Error(
      `commands.worktree_companion_start failed (exit ${spawned.status}): ${(spawned.stderr || spawned.stdout || '').trim() || '(no output)'}`,
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(spawned.stdout);
  } catch (parseError) {
    throw new Error(
      `commands.worktree_companion_start must print JSON with a "worktreePath" field to stdout — got unparseable output (${parseError instanceof Error ? parseError.message : String(parseError)}). Raw stdout: ${spawned.stdout.slice(0, 500)}`,
    );
  }
  if (!parsed || typeof parsed.worktreePath !== 'string' || !parsed.worktreePath) {
    throw new Error(`commands.worktree_companion_start's JSON output must include a non-empty "worktreePath" string — got ${JSON.stringify(parsed)}.`);
  }
  const sessionId = typeof parsed.sessionId === 'string' && parsed.sessionId ? parsed.sessionId : null;
  const mountFullPath = path.join(worktreeRoot, mountPath);
  fs.mkdirSync(path.dirname(mountFullPath), { recursive: true });
  fs.symlinkSync(parsed.worktreePath, mountFullPath, 'dir');
  const markerPath = path.join(worktreeRoot, '.bee', 'companion-session.json');
  fs.mkdirSync(path.dirname(markerPath), { recursive: true });
  fs.writeFileSync(markerPath, `${JSON.stringify({ sessionId, worktreePath: parsed.worktreePath, mountPath }, null, 2)}\n`);
  return { sessionId, worktreePath: parsed.worktreePath, mountPath };
}

// hardening-4b: the actual body, unchanged apart from `removeGrant` ->
// `removeGrantCore` in the rollback path below — split out so
// createFeatureWorktree's own doc comment above stays attached to the public
// entrypoint while this runs INSIDE its withStoreLock('worktree-admin') hold.
function createFeatureWorktreeLocked(
  mainRoot,
  { feature, baseRef, companionStartCommand, companionMountPath, _writeGrant, _bootstrapWorktreeStore, _syncWorktreeSkills },
) {
  if (typeof feature !== 'string' || !FEATURE_SLUG_RE.test(feature)) {
    refuse(
      'WORKTREE_INVALID_SLUG',
      `feature slug ${JSON.stringify(feature)} must match ${FEATURE_SLUG_RE} (lowercase letters/digits, starting with a letter or digit, hyphens allowed after that).`,
    );
  }

  // worktree-companion-hook: both or neither — a start command with no mount
  // path (or vice versa) is a config error, caught here as a zero-mutation
  // refusal rather than surfacing later as a confusing symlink failure.
  // (The CLI handler already refuses this earlier when --with-companion is
  // passed without commands.worktree_companion_start; this is the defensive
  // invariant for any OTHER caller of createFeatureWorktree directly.)
  let companionMount;
  if (companionStartCommand || companionMountPath) {
    if (!companionStartCommand || !companionMountPath) {
      refuse(
        'WORKTREE_COMPANION_CONFIG_INCOMPLETE',
        'commands.worktree_companion_start and commands.worktree_companion_mount must both be configured to use --with-companion — only one was found.',
      );
    }
    companionMount = validateCompanionMountPath(companionMountPath);
  }

  // Resolved once here (not re-derived later): the SAME sha this refusal
  // check proves resolvable is the sha `git worktree add` below actually
  // receives, so there is no gap between "we checked it exists" and "we
  // used it" for a ref that moves between the two (see the addArgs comment
  // below for why the resolved sha, not the original ref string, is what
  // gets passed to `git worktree add`).
  let baseRefSha;
  if (baseRef !== undefined && baseRef !== null && baseRef !== '') {
    if (typeof baseRef !== 'string') {
      refuse('WORKTREE_BASE_NOT_FOUND', `--base-ref must be a string, got ${JSON.stringify(baseRef)}.`);
    }
    baseRefSha = resolveBaseRefCommit(mainRoot, baseRef);
    if (!baseRefSha) {
      refuse(
        'WORKTREE_BASE_NOT_FOUND',
        `--base-ref ${JSON.stringify(baseRef)} does not resolve to a commit in ${mainRoot} ("git rev-parse --verify" found nothing) — check the ref/sha/tag exists (and isn't just a syntax typo).`,
      );
    }
  }

  if (!isOrdinaryCheckout(mainRoot)) {
    refuse(
      'WORKTREE_CALLER_NOT_ORDINARY',
      `"bee worktree new" must be run from the main checkout, not a linked worktree (${mainRoot} is not an ordinary checkout).`,
    );
  }

  const repoBasename = path.basename(mainRoot);
  const siblingDirName = `${repoBasename}--wt--${feature}`;
  const worktreeRoot = path.resolve(mainRoot, '..', siblingDirName);
  const branch = `wt/${feature}`;
  const mainStoreRoot = path.join(mainRoot, '.bee');

  if (fs.existsSync(worktreeRoot)) {
    refuse('WORKTREE_TARGET_EXISTS', `${worktreeRoot} already exists.`);
  }

  if (branchExists(mainRoot, branch)) {
    refuse('WORKTREE_BRANCH_EXISTS', `branch "${branch}" already exists in ${mainRoot}.`);
  }

  // Advisory only: git assigns a new worktree's id from the sibling
  // directory's own basename whenever that id is free, and only falls back
  // to a suffixed id on collision — so this precheck catches the common
  // "target dir removed by hand, grant registry never cleaned up" case
  // BEFORE any mutating git call runs, at the cost of occasionally missing a
  // collision-suffixed id. `git worktree add` failing at runtime (below) is
  // the real, unconditional guard.
  const likelyId = siblingDirName;
  if (readGrants(mainStoreRoot)[likelyId] === true) {
    refuse(
      'WORKTREE_GRANT_EXISTS',
      `a worktree grant already exists for id "${likelyId}" — run "bee worktree unregister --id ${likelyId}" (or "git worktree prune") before retrying.`,
    );
  }

  // Pass the RESOLVED SHA, not the original `baseRef` string, to `git
  // worktree add` — deterministic even if `baseRef` names something that
  // moves (a branch tip, `HEAD`) in the window between the resolve above
  // and this call, and immune to the same leading-dash/argument-injection
  // concern `resolveBaseRefCommit`'s `--end-of-options` already guards
  // (a raw sha can never be mistaken for a git flag).
  const addArgs = ['worktree', 'add', '-b', branch, '--', worktreeRoot];
  if (baseRefSha) addArgs.push(baseRefSha);
  const addResult = runGit(mainRoot, addArgs);
  if (addResult.status !== 0) {
    refuse(
      'WORKTREE_ADD_FAILED',
      `git worktree add failed: ${(addResult.stderr || addResult.stdout || '').trim() || `exit ${addResult.status}`}`,
    );
  }

  let id;
  try {
    id = readWorktreeGitVerifiedId(worktreeRoot);
    _writeGrant(mainStoreRoot, id);
    const bootstrap = _bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, feature);
    // worktree-companion-hook: deliberately INSIDE this try — a companion
    // start failure folds into the exact same post-add rollback below as any
    // other post-`git worktree add` failure, so a worktree is never left
    // half-configured (created, registered, but silently missing the
    // companion its caller asked for).
    const companion = companionStartCommand ? runCompanionStart(mainRoot, worktreeRoot, companionStartCommand, companionMount) : null;
    // Best-effort: a failed/skipped skill sync never throws (see
    // syncWorktreeSkills's own doc comment) and so never enters the
    // rollback path below — it's reported via `skillsSync`, not fatal.
    const skillsSync = _syncWorktreeSkills(mainRoot, worktreeRoot);
    return {
      id,
      worktreeRoot,
      branch,
      baseRef: baseRef || null,
      baseRefSha: baseRefSha || null,
      bootstrap,
      companion,
      skillsSync,
    };
  } catch (postAddError) {
    // git worktree add itself succeeded, but deriving the id / writing the
    // grant / bootstrapping the store threw. Roll back best-effort so a
    // failed "new" never leaves a half-registered worktree as its only
    // trace — the pre-checks above are what keep the COMMON refusal path
    // zero-mutation; this is the atomic real guard's own failure mode.
    if (id) {
      try {
        removeGrantCore(mainStoreRoot, id);
      } catch {
        // best-effort — the typed error below still fires either way.
      }
    }
    const removeResult = runGit(mainRoot, ['worktree', 'remove', '--force', worktreeRoot]);
    const stillPresent = fs.existsSync(worktreeRoot);
    const postAddMessage = postAddError instanceof Error ? postAddError.message : String(postAddError);
    if (removeResult.status === 0 && !stillPresent) {
      // Worktree gone — also drop the branch `git worktree add -b` created,
      // best-effort, so a rolled-back "new" leaves TRUE zero mutation behind
      // (no dir, no grant, no branch), not just no dir/grant. Only reachable
      // once the worktree itself is confirmed gone: git refuses to delete a
      // branch still checked out by a live worktree, so this is never
      // attempted while stillPresent is true.
      try {
        runGit(mainRoot, ['branch', '-D', branch]);
      } catch {
        // best-effort — the typed error below still fires either way.
      }
      refuse(
        'WORKTREE_POST_ADD_FAILED',
        `${worktreeRoot} was created but could not be registered (${postAddMessage}); it has been rolled back (worktree and branch "${branch}" removed).`,
      );
    }
    refuse(
      'WORKTREE_POST_ADD_ROLLBACK_FAILED',
      `${worktreeRoot} was created but could not be registered (${postAddMessage}), and the rollback itself failed — the tree still exists on disk; run "bee worktree register --feature ${feature}" from inside it to adopt it.`,
    );
  }
}

// ---------------------------------------------------------------------------
// mergeFeatureWorktree — "bee worktree merge --id <id>" (GH #21, decision D8;
// reworked into a STAGED transaction by decision D2-REVISED, user review
// P1-2): merge a granted worktree's branch back into MAIN as a staged,
// uncommitted merge (`git merge --no-ff --no-commit`), run the host
// project's configured verify command against that merged-but-uncommitted
// tree, and only ever `git commit` once verify is green. A textual conflict
// or a red verify both `git merge --abort` and PROVE (mainUntouchedProof
// above) that main was left byte-untouched — no known-red merge commit and
// no stranded conflict state can reach main anymore, superseding the old
// D8 contract where a red verify left a real, never-rolled-back merge
// commit on main. `mainRoot` MUST already be a resolved ORDINARY checkout
// root — see createFeatureWorktree's header comment for why this module
// re-derives that distinction itself instead of importing resolveRoots
// (state.mjs cycle) — and per decision D8/advisor-R5, running merge from
// inside ANY linked worktree (including the very one being merged) already
// fails this SAME check, since a linked worktree's own `.git` is a file,
// never a directory: there is no separate "own worktree" code, the
// not-ordinary refusal IS the own-worktree guard (see mergeFeatureWorktree's
// own doc comment below).
//
// verifyCommand is deliberately a CALLER-PASSED option, not something this
// module reads from .bee/config.json itself: worktree-store.mjs keeps
// createFeatureWorktree's "zero deps beyond node builtins" contract (the
// state.mjs-imports-readGrants-FROM-here cycle note above still applies), so
// the CLI handler in bee.mjs resolves `readConfig(mainRoot).commands.verify`
// and passes the string down as `options.verifyCommand`.
// ---------------------------------------------------------------------------

/** Typed refusal for mergeFeatureWorktree: same `[CODE] message` / `.code`
 * convention as WorktreeCreateError above. */
export class WorktreeMergeError extends Error {
  constructor(code, message) {
    super(`[${code}] ${message}`);
    this.name = 'WorktreeMergeError';
    this.code = code;
  }
}

function refuseMerge(code, message) {
  throw new WorktreeMergeError(code, message);
}

/** `git status --porcelain` — deliberately WITHOUT `--ignored` (decision D8a):
 * a worktree whose only "dirty" content is its fully-bootstrapped gitignored
 * `.bee` store must NOT read as dirty. Throws a plain (untyped) Error only
 * when git itself cannot be asked at all — a git failure here is an
 * environment problem, not a typed merge refusal. */
function gitStatusPorcelain(cwd) {
  const r = runGit(cwd, ['status', '--porcelain']);
  if (r.status !== 0) {
    throw new Error(`"git status --porcelain" failed in ${cwd}: ${(r.stderr || r.stdout || '').trim() || `exit ${r.status}`}`);
  }
  return r.stdout;
}

function isTreeDirty(cwd) {
  return gitStatusPorcelain(cwd).trim().length > 0;
}

/**
 * The three-part "main was left byte-untouched" proof (decision D2-REVISED)
 * required after EVERY `git merge --abort` this module runs — a textual
 * conflict, a red post-merge verify, or the exception-safety net around the
 * verify call. Deliberately checks all three independently rather than
 * trusting `git merge --abort`'s own exit code, because the whole point of
 * this rework is that a claim of "main untouched" is proven, not asserted:
 *   1. `git rev-parse HEAD` is unchanged from the pre-merge value.
 *   2. `.git/MERGE_HEAD` no longer exists (no merge-in-progress state).
 *   3. `git status --porcelain --untracked-files=no` is clean (tracked-file
 *      dirt only — untracked litter from an unrelated source is not this
 *      check's concern, same scope `--untracked-files=no` always implies).
 * Returns `{ ok, reason? }` instead of throwing so the caller can build a
 * SPECIFIC typed refusal when the proof fails, instead of a generic throw.
 */
function mainUntouchedProof(mainRoot, preMergeHead, mergeHeadFile) {
  const headNow = runGit(mainRoot, ['rev-parse', 'HEAD']).stdout.trim();
  if (headNow !== preMergeHead) {
    return { ok: false, reason: `HEAD moved from ${preMergeHead} to ${headNow}` };
  }
  if (fs.existsSync(mergeHeadFile)) {
    return { ok: false, reason: '.git/MERGE_HEAD is still present' };
  }
  const status = runGit(mainRoot, ['status', '--porcelain', '--untracked-files=no']).stdout;
  if (status.trim().length > 0) {
    return { ok: false, reason: `"git status --porcelain --untracked-files=no" is not clean:\n${status}` };
  }
  return { ok: true };
}

/** The worktree's CURRENT checked-out branch, or `null` on detached HEAD (or
 * no HEAD ref at all). */
function currentBranch(cwd) {
  const r = runGit(cwd, ['symbolic-ref', '-q', '--short', 'HEAD']);
  if (r.status !== 0) return null;
  return r.stdout.trim();
}

/** Best-effort read of the worktree's OWN bootstrapped `.bee/state.json`
 * `feature` field. MUTABLE: `bee state set --feature` / `startFeature` rewrite
 * it freely and neither knows worktrees exist — which is exactly why it is the
 * FALLBACK source below and no longer the primary one (issues-46-53 D4).
 * Never throws: a missing/corrupt/foreign state.json just means "unknown". */
function readWorktreeStateFeature(worktreeRoot) {
  try {
    const raw = fs.readFileSync(path.join(worktreeRoot, '.bee', 'state.json'), 'utf8');
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed.feature === 'string' && parsed.feature ? parsed.feature : null;
  } catch {
    return null;
  }
}

/** Best-effort read of the worktree's IMMUTABLE creation slug, recorded once
 * by `bootstrapWorktreeStore` (see WORKTREE_IDENTITY_REL above). Never throws:
 * a worktree created before this record existed simply returns null. */
function readWorktreeCreationFeature(worktreeRoot) {
  try {
    const raw = fs.readFileSync(path.join(worktreeRoot, '.bee', WORKTREE_IDENTITY_REL), 'utf8');
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed.feature === 'string' && parsed.feature ? parsed.feature : null;
  } catch {
    return null;
  }
}

/**
 * issues-46-53 D4 (#46): resolves the feature merge derives its expected
 * `wt/<feature>` branch from, PREFERRING the immutable creation slug over the
 * mutable `state.feature`. A worktree renamed after creation now merges
 * correctly, because the branch is checked against the value it was actually
 * created from.
 *
 * DEGRADES EXACTLY TO THE PRE-FIX BEHAVIOR when no creation record exists (a
 * worktree created before this record shipped, or one whose `.bee/runtime/`
 * was wiped): `created` is null, the resolved feature IS `state.feature`, and
 * every downstream decision is byte-for-byte what it was — never a crash,
 * never a new refusal. The extra fields are returned so the refusal message
 * can name WHICH field drifted instead of blaming the branch.
 */
function resolveWorktreeFeature(worktreeRoot) {
  const created = readWorktreeCreationFeature(worktreeRoot);
  const stateFeature = readWorktreeStateFeature(worktreeRoot);
  return { feature: created ?? stateFeature, created, stateFeature };
}

// Pattern-only fallback when the worktree's own state.json has no readable
// `feature` (e.g. a worktree adopted via `worktree register` whose state.json
// predates this field, or was hand-edited) — still requires SOME "wt/<slug>"
// shaped branch rather than accepting anything checked out.
const WT_BRANCH_RE = /^wt\/[a-z0-9][a-z0-9-]*$/;

/**
 * Resolves a granted worktree id to its worktreeRoot using the SAME
 * bidirectional gitdir validation `resolveRoots` uses (see state.mjs), just
 * keyed by `id` instead of by walking up from a cwd: reads
 * `<mainRoot>/.git/worktrees/<id>/gitdir` (which should point at
 * `<worktreeRoot>/.git`), then reverse-reads `<worktreeRoot>/.git` (which
 * should point back at `<mainRoot>/.git/worktrees/<id>`). Returns `null` on
 * ANY mismatch, missing file, or unreadable content — never throws, so the
 * caller can fold "no such id" and "id's link is broken" into the same typed
 * WORKTREE_MERGE_UNKNOWN_ID refusal.
 */
function resolveWorktreeById(mainRoot, id) {
  const gitWorktreeDir = path.join(mainRoot, '.git', 'worktrees', id);
  let stat;
  try {
    stat = fs.statSync(gitWorktreeDir);
  } catch {
    return null;
  }
  if (!stat.isDirectory()) return null;

  let forwardRaw;
  try {
    forwardRaw = fs.readFileSync(path.join(gitWorktreeDir, 'gitdir'), 'utf8').trim();
  } catch {
    return null;
  }
  if (!forwardRaw) return null;
  const resolvedGitFile = path.resolve(gitWorktreeDir, forwardRaw.replace(/\\/g, path.sep));
  const worktreeRoot = path.dirname(resolvedGitFile);

  let reverseRaw;
  try {
    reverseRaw = fs.readFileSync(path.join(worktreeRoot, '.git'), 'utf8').trim();
  } catch {
    return null;
  }
  const match = reverseRaw.match(/^gitdir:\s*(.+)$/);
  if (!match) return null;
  const reverseResolved = path.resolve(worktreeRoot, match[1].trim().replace(/\\/g, path.sep));
  if (path.resolve(reverseResolved) !== path.resolve(gitWorktreeDir)) return null;

  return { worktreeRoot };
}

/**
 * Re-checks freshness immediately before removal, then `git worktree remove
 * --force` + `git branch -d` (NEVER -D), in that order (decision D8b —
 * advisor R4). `--force` is used ONLY to push past git's own removal-safety
 * check tripping on the disposable gitignored `.bee` store (runtime/cache
 * tier files are untracked-but-ignored, not a real dirty signal) — it is
 * NEVER used to push through a genuinely dirty worktree, because the
 * freshness re-check below (same `git status --porcelain`, no `--ignored`,
 * as the pre-merge dirty check) already refuses, typed, before `remove` is
 * ever invoked, whenever a tracked-modified or untracked file sits at a
 * TRACKED path. Never throws: every outcome is a returned `{ok, code?}`
 * object folded into the merge result's `.cleanup` field, per D8b ("cleanup
 * refuses, typed, merge result still reported ok"). Also drops the id's
 * grant from the MAIN store's registry once the worktree and branch are both
 * gone — the same removeGrant call `createFeatureWorktree`'s own rollback
 * path makes, so a merged-and-cleaned-up id never lingers in `bee worktree
 * list` pointing at a directory that no longer exists (best-effort: a grant
 * removal failure does not turn an otherwise-successful cleanup into a
 * failure, since the worktree and branch are already gone by that point).
 *
 * xwh-2: ALSO releases every mirrored cross-worktree hold for this id
 * (releaseAllForHolder, best-effort, same try/catch posture as the
 * removeGrant call right above it) — a removed worktree must never leave a
 * stale entry in the shared ledger claiming a since-gone checkout still
 * holds a path. `releaseAllForHolder` is async (withStoreLock-backed), which
 * is why this function — and its two callers, `attachCleanupOutcome` and
 * `mergeFeatureWorktree` itself — are async now (xwh-2): a CLI process must
 * not exit before this write actually lands, so it has to be awaited, not
 * fired-and-forgotten.
 */
async function performCleanup(mainRoot, { worktreeRoot, branch, id, verifySkipped = false }) {
  let status;
  try {
    status = gitStatusPorcelain(worktreeRoot);
  } catch (error) {
    return { ok: false, code: 'WORKTREE_MERGE_CLEANUP_CHECK_FAILED', reason: error instanceof Error ? error.message : String(error) };
  }
  if (status.trim().length > 0) {
    return {
      ok: false,
      code: 'WORKTREE_MERGE_CLEANUP_DIRTY',
      reason: `${worktreeRoot} has tracked-modified or untracked files at tracked paths — cleanup refuses. Remove them (a bootstrapped, gitignored .bee store alone does not block cleanup) and retry, or clean up manually.`,
      status,
    };
  }

  const removeResult = runGit(mainRoot, ['worktree', 'remove', '--force', '--', worktreeRoot]);
  if (removeResult.status !== 0) {
    return {
      ok: false,
      code: 'WORKTREE_MERGE_CLEANUP_REMOVE_FAILED',
      reason: (removeResult.stderr || removeResult.stdout || '').trim() || `exit ${removeResult.status}`,
    };
  }

  const branchDeleteResult = runGit(mainRoot, ['branch', '-d', '--', branch]);
  if (branchDeleteResult.status !== 0) {
    return {
      ok: false,
      code: 'WORKTREE_MERGE_CLEANUP_BRANCH_DELETE_FAILED',
      removed: true,
      reason: (branchDeleteResult.stderr || branchDeleteResult.stdout || '').trim() || `exit ${branchDeleteResult.status}`,
    };
  }

  try {
    // hardening-4b: the unlocked core — performCleanup always runs inside
    // mergeFeatureWorktree's own withStoreLock('worktree-admin') hold (it
    // "joins the same lock" rather than acquiring its own), so calling the
    // exported, lock-acquiring `removeGrant` here would deadlock (nested,
    // non-reentrant acquisition of the same named lock).
    removeGrantCore(path.join(mainRoot, '.bee'), id);
  } catch {
    // best-effort — the worktree and branch are already gone either way.
  }

  try {
    await releaseAllForHolder(mainRoot, id);
  } catch {
    // best-effort — same posture as the removeGrant call above: a ledger
    // release failure does not turn an otherwise-successful cleanup into a
    // failure, since the worktree and branch are already gone either way.
  }

  const outcome = { ok: true, removed: true, branch_deleted: true };
  if (verifySkipped) {
    // D8 only names "verify green" for the unconditional --cleanup trigger;
    // it does not address a repo with no commands.verify recorded at all.
    // Extending eligibility to verify:'skipped' (advisor-confirmed gap
    // resolution, not literal D8 text) is conditioned on this NEVER being
    // silent: cleanup that ran with no semantic gate must say so loudly,
    // every time, right here in the result the caller sees.
    outcome.warning = 'verify skipped — no commands.verify recorded; cleaned up unchecked.';
  }
  return outcome;
}

/** With `--cleanup`, runs cleanup on every outcome where the branch's work is
 * settled: a committed merge (`verify: 'green'` or `'skipped'`) AND the
 * `ALREADY_UP_TO_DATE` no-op. `MERGE_CONFLICT`/`MERGE_VERIFY_RED` both return
 * earlier in `mergeFeatureWorktree` and never reach this function — and must
 * not: work is NOT integrated on either path, so removing the worktree there
 * would destroy the only copy of it.
 *
 * issues-46-53 D3 (#47): the no-op path used to return BEFORE this function
 * ran, so an accepted `--cleanup` evaporated silently — exit 0, nothing
 * removed, and not one word about the flag. The old blanket rule "cleanup is
 * strictly post-commit" reads as if the COMMIT were the safety property. It
 * is not: the property is "the worktree holds nothing that would be lost".
 * `ALREADY_UP_TO_DATE` means the branch holds nothing main lacks, and the
 * WORKTREE_MERGE_WORKTREE_DIRTY pre-check upstream has already proved the
 * worktree carries no uncommitted work — so cleanup is exactly as safe there
 * as after a green merge, and dropping the flag was the defect.
 *
 * Without the flag, attaches the suggested command instead of running
 * anything (decision D8b: "never prompt" — the suggestion is informational,
 * not a question). */
async function attachCleanupOutcome(result, { mainRoot, worktreeRoot, branch, id, cleanup, verifySkipped = false }) {
  if (!cleanup) {
    result.cleanup_suggested_command = `bee worktree merge --id ${id} --cleanup --json`;
    return;
  }
  result.cleanup = await performCleanup(mainRoot, { worktreeRoot, branch, id, verifySkipped });
}

/**
 * Merges a granted worktree's branch into `mainRoot` as a STAGED
 * transaction (decision D2-REVISED), split into three phases so the
 * 'worktree-admin' lock is never held across the verify child
 * (hardening-4c, issue #56 3.9 / CONTEXT.md D10b — see docs/history/
 * multisession-native/CONTEXT.md). Sequence, run with `mainRoot` as cwd:
 *
 *   P1 (LOCKED — `mergeFeatureWorktreeStage`): every pre-check, plus
 *   staging the merge itself.
 *     1. Capture pre-merge HEAD (`git rev-parse HEAD`).
 *     2. `git merge --no-ff --no-commit <branch>` — stages the merge WITHOUT
 *        committing it. `--no-ff` also forces a real merge (never a
 *        fast-forward) even when `<branch>` is a fast-forward-eligible
 *        ancestor-of-main relationship, so a staged merge always means a
 *        TRUE merge commit once committed, not a silent ref move.
 *        - Non-zero exit: textual conflict (or another merge-attempt
 *          failure). `git merge --abort`, prove main is untouched
 *          (`mainUntouchedProof`), return typed `MERGE_CONFLICT` — a
 *          TERMINAL P1 outcome (`{ done: true, result }`); P2/P3 never run.
 *        - Zero exit but no `.git/MERGE_HEAD` created: "Already up to date"
 *          — branch has nothing new for main. Returns a typed no-op result;
 *          `git commit` is NEVER attempted here (there is nothing staged —
 *          committing would either error or produce an empty commit). A
 *          `--cleanup` flag DOES still run on this path (issues-46-53 D3 /
 *          #47): the branch holds nothing main lacks and the worktree was
 *          already proven clean, so there is nothing to lose. ALSO
 *          terminal — there is no verify to run for a no-op.
 *     3. A merge is now staged (MERGE_HEAD live, changes staged, nothing
 *        committed). P1 captures `git write-tree`'s hash of the staged
 *        index right now, as `stagedTreeHash` — the identity P3's fence
 *        re-checks once the lock has been up for grabs. Returns
 *        `{ done: false, state }`.
 *
 *   The lock is released between P1 and P2 (and re-acquired for P3) ONLY
 *   when `verifyCommand` was actually given; a merge with nothing to verify
 *   finishes inside the SAME P1 hold, byte-identical to the
 *   pre-hardening-4c single-lock behavior, since there is no long child to
 *   protect against.
 *
 *   P2 (UNLOCKED — inline in `mergeFeatureWorktree`): if `verifyCommand` was
 *   given, it runs NOW, unlocked, against the merged-but-UNCOMMITTED tree
 *   (MERGE_HEAD is live, changes are staged but not committed) — verify is
 *   deliberately run before any commit exists, so a semantic break is
 *   caught before it ever becomes a real commit on main, not after. This is
 *   the whole point of hardening-4c: a verify that runs for minutes no
 *   longer holds a filesystem lock for those minutes. hardening-4b's
 *   grant/create serialization (writeGrant/removeGrant/createFeatureWorktree)
 *   is free to interleave during this window, and a SECOND, unrelated
 *   "worktree merge" attempt is free to even ATTEMPT the lock — it just
 *   self-blocks on `isTreeDirty(mainRoot)` (P1's own pre-check) because
 *   main's tree is genuinely dirty (a real staged merge is sitting there),
 *   the same refusal a concurrent merge would hit today for any other
 *   reason a tree is dirty.
 *
 *   P3 (RE-LOCKED — `mergeFeatureWorktreeFinish`): re-acquires
 *   'worktree-admin', then, BEFORE ever committing, re-checks a FOUR-PART
 *   fence (advisor condition C2) for anything that could have drifted while
 *   the lock was up for grabs:
 *     (a) HEAD unchanged from the P1 capture;
 *     (b) `.git/MERGE_HEAD` still present;
 *     (c) staged-tree identity — `git write-tree` NOW equals P1's
 *         `stagedTreeHash`. HEAD alone is NOT enough: HEAD never moves for
 *         an uncommitted staged merge, so tampering with the staged
 *         index/tree while the lock was free would sail straight past a
 *         HEAD-only re-check — this is the check that actually catches it.
 *     (d) the worktree's grant (`readGrants(...)[id]`) is still `true`.
 *   ANY mismatch: `git merge --abort`, prove main untouched
 *   (`mainUntouchedProof`), throw a typed `WorktreeMergeError`
 *   (`WORKTREE_MERGE_FENCE_DRIFT`) — main is left exactly as untouched as
 *   every other abort path in this module.
 *     - Verify red (checked first, before the fence): `git merge --abort`,
 *       prove main untouched, return typed `MERGE_VERIFY_RED` — the
 *       semantic-conflict alarm, unchanged from before this rework. NO
 *       merge commit exists at this point, so there is nothing to roll
 *       back: main was left byte-untouched.
 *     - Verify green (or no `verifyCommand` given at all) AND the fence is
 *       clean: `git commit` names the id in its message. Post-commit guard:
 *       `git status --porcelain --untracked-files=no` MUST be empty; if
 *       verify itself left tracked files modified (a misbehaving verify
 *       command), the result carries a typed `warning.code:
 *       'verify_mutated_tracked_files'` instead of silently claiming the
 *       tree matches the commit. Recovery if a LATER independent verify
 *       goes red on an already-committed merge: `git revert -m 1
 *       <merge-commit>` (documented, not automated). `attachCleanupOutcome`
 *       (and, through it, `performCleanup`) runs inside THIS hold — same
 *       "joins the lock, does not acquire its own" contract hardening-4b
 *       established, just re-pointed at P3 instead of the old single hold.
 *
 * Returns a plain result object for every outcome that follows a real git
 * mutation (`MERGE_CONFLICT`, `ALREADY_UP_TO_DATE`, `MERGE_VERIFY_RED`, or a
 * green/skipped success) — those are settled facts about repo state the
 * caller needs to inspect, not refusals to retry. Every check BEFORE the
 * merge itself is a typed, ZERO-MUTATION `WorktreeMergeError` throw
 * (unknown/ungranted id, caller checkout not ordinary, dirty main, dirty
 * worktree, detached HEAD, or a branch other than the worktree's expected
 * `wt/<slug>`-style branch) — same error-class style as
 * `createFeatureWorktree`'s `WorktreeCreateError`. A `WorktreeMergeError` is
 * ALSO thrown (not returned) if an abort-then-prove step itself fails
 * (`WORKTREE_MERGE_ABORT_FAILED`), the commit step itself fails
 * (`WORKTREE_MERGE_COMMIT_FAILED`), or the P3 fence catches drift
 * (`WORKTREE_MERGE_FENCE_DRIFT`) — all are environment-level (or, for fence
 * drift, another-actor-level) defects, never a normal outcome to fold into
 * a typed result.
 *
 * `options`:
 *   - `id` (required): the worktree's git-verified id (as granted via
 *     `worktree new`/`worktree register`).
 *   - `cleanup` (optional, default false): see `attachCleanupOutcome` above.
 *     Runs on a committed merge AND on the already-up-to-date no-op
 *     (issues-46-53 D3); NEVER on a textual conflict or a red verify, where
 *     the work is not integrated and removing the worktree would destroy it.
 *   - `verifyCommand` (optional): a shell command string, run via
 *     `spawnSync(verifyCommand, { cwd: mainRoot, shell: true })`. Omit (or
 *     pass a falsy value) when the host project has no `commands.verify`
 *     recorded — the CLI handler is what resolves this from
 *     `readConfig(mainRoot).commands.verify` (see module header note above).
 *   - `companionEndCommand` (optional, worktree-companion-hook): a shell
 *     command string (`commands.worktree_companion_end`), run — with its
 *     literal `<id>` token substituted for the real session id — whenever
 *     the worktree being merged carries a `.bee/companion-session.json`
 *     marker (written by `worktree new --with-companion`). No flag gates
 *     this on the merge side: the marker's presence IS the signal. See
 *     `teardownCompanionIfPresent` below for exactly when this runs and why.
 *
 * "Own worktree" refusal: running merge from inside a linked worktree —
 * including the very worktree named by `id` — is caught by the SAME
 * `WORKTREE_MERGE_CALLER_NOT_ORDINARY` check below (a linked worktree's own
 * `.git` is a file, never a directory, so `isOrdinaryCheckout(mainRoot)` is
 * already false); there is deliberately no second, distinct code for this
 * (decision D8 / advisor R5 belt-and-braces framing, not a separate rule).
 *
 * xwh-2: this function is now `async` (previously fully synchronous) purely
 * because `--cleanup`'s path awaits `attachCleanupOutcome` -> `performCleanup`
 * -> `releaseAllForHolder`, which is itself async (withStoreLock-backed) —
 * every early, zero-mutation `WorktreeMergeError` throw above (unknown id,
 * not-ordinary caller, dirty tree, detached HEAD, branch mismatch) still
 * throws exactly as before, just as a REJECTED PROMISE now instead of a
 * synchronous throw, since the whole function body runs inside the implicit
 * async wrapper — callers must `await`/`.catch()` it, not wrap it in a bare
 * synchronous `try { } catch { }`.
 *
 * hardening-4b (AMENDED by hardening-4c — see the phase description above):
 * every pre-check, the merge/abort sequence, and — via
 * attachCleanupOutcome — performCleanup still run inside a
 * withStoreLock(mainRoot, 'worktree-admin') hold, serialized against
 * writeGrant/removeGrant/createFeatureWorktree exactly as before.
 * performCleanup itself still does NOT acquire the lock (see its own
 * removeGrantCore comment) — it "joins" whichever hold is currently live
 * (P1's, for a no-verify-configured merge; P3's, for a verify-configured
 * one). What hardening-4b no longer claims is that this is ONE unbroken
 * hold end to end: when `verifyCommand` is given, the hold is released
 * between P1 and P3 for exactly the verify child's duration (hardening-4c)
 * — that release is the fix, not a regression of this serialization.
 */
export async function mergeFeatureWorktree(mainRoot, options = {}) {
  const staged = await withStoreLock(mainRoot, 'worktree-admin', () => mergeFeatureWorktreeStage(mainRoot, options));
  if (staged.done) {
    return staged.result;
  }

  const { verifyCommand } = options;
  if (!verifyCommand) {
    // Nothing to release the lock around: no long child ever runs on this
    // path, so stage and finish inside the SAME hold — byte-identical to
    // the pre-hardening-4c single-lock behavior.
    return withStoreLock(mainRoot, 'worktree-admin', () => mergeFeatureWorktreeFinish(mainRoot, options, staged.state, { ran: false }));
  }

  // P2 — UNLOCKED (hardening-4c): the whole point of this rework. No store
  // lock is held while the (possibly multi-minute) verify child runs.
  const verifyResult = spawnSync(verifyCommand, { cwd: mainRoot, shell: true, encoding: 'utf8' });
  const verifyOutcome = { ran: true, status: verifyResult.status, combined: `${verifyResult.stdout || ''}${verifyResult.stderr || ''}` };

  // P3 — re-acquire the lock and re-check the fence before ever committing.
  return withStoreLock(mainRoot, 'worktree-admin', () => mergeFeatureWorktreeFinish(mainRoot, options, staged.state, verifyOutcome));
}

/**
 * P3 fence (advisor condition C2): everything that must still be true right
 * before the commit, re-checked AFTER re-acquiring 'worktree-admin' —
 * because the lock was up for grabs during P2, none of this can be trusted
 * to still hold just because P1 proved it once. Returns a short drift
 * description string on mismatch, or `null` when the fence is clean.
 */
function checkMergeFence(mainRoot, { id, preMergeHead, mergeHeadFile, stagedTreeHash }) {
  const headNow = runGit(mainRoot, ['rev-parse', 'HEAD']).stdout.trim();
  if (headNow !== preMergeHead) {
    return `HEAD moved from ${preMergeHead} to ${headNow} while the lock was released for verify`;
  }
  if (!fs.existsSync(mergeHeadFile)) {
    return '.git/MERGE_HEAD disappeared while the lock was released for verify — the staged merge was cleared out from under this operation';
  }
  const treeNow = runGit(mainRoot, ['write-tree']).stdout.trim();
  if (treeNow !== stagedTreeHash) {
    return `the staged tree changed from ${stagedTreeHash} to ${treeNow} while the lock was released for verify — the index was mutated mid-verify`;
  }
  const grants = readGrants(path.join(mainRoot, '.bee'));
  if (grants[id] !== true) {
    return `the grant for worktree id ${JSON.stringify(id)} was revoked while the lock was released for verify`;
  }
  return null;
}

/**
 * Best-effort companion teardown (worktree-companion-hook), run
 * unconditionally at the very START of a merge attempt — before either
 * dirty-tree pre-check, and regardless of `--cleanup` or of how the merge
 * attempt itself ultimately resolves (conflict, red verify, or success
 * alike). Two independent reasons this can't wait until after a successful
 * merge the way `attachCleanupOutcome`'s bee-worktree removal does:
 *
 *   1. A companion's mounted symlink (`commands.worktree_companion_mount`)
 *      is untracked, and `isTreeDirty(worktreeRoot)` below (`git status
 *      --porcelain`, deliberately without `--ignored`) sees it regardless of
 *      whether a same-named path is gitignored elsewhere in the project — a
 *      directory-only pattern like "repo/" does NOT match a symlink of the
 *      same name (confirmed empirically against a real git worktree, not
 *      assumed). Left in place, every merge of a `--with-companion` worktree
 *      would refuse WORKTREE_MERGE_WORKTREE_DIRTY, cleanup or not.
 *   2. The companion session itself has no reason to keep living past the
 *      moment its worktree is being merged back — `--cleanup` only controls
 *      whether the BEE worktree infrastructure is also removed afterward, a
 *      separate, later concern this function does not touch.
 *
 * No flag gates this — the marker file's mere presence on the worktree being
 * merged is the only signal needed; a worktree created without
 * `--with-companion` has no marker and this is a silent no-op (returns
 * `null`). A missing/failed end command is never fatal to the merge itself:
 * this always returns (never throws), and the symlink + marker are removed
 * best-effort either way so the dirty-check that follows is never blocked by
 * a companion problem — a failure is carried as `.warning` on the returned
 * object for the caller to surface, not swallowed.
 */
function teardownCompanionIfPresent(mainRoot, worktreeRoot, companionEndCommand) {
  const markerPath = path.join(worktreeRoot, '.bee', 'companion-session.json');
  let marker;
  try {
    marker = JSON.parse(fs.readFileSync(markerPath, 'utf8'));
  } catch {
    return null; // no companion on this worktree — nothing to do.
  }

  let warning;
  if (companionEndCommand) {
    const substituted = companionEndCommand.replace('<id>', marker.sessionId || '');
    const spawned = spawnSync(substituted, { cwd: mainRoot, shell: true, encoding: 'utf8' });
    if (spawned.status !== 0) {
      warning =
        `commands.worktree_companion_end failed (exit ${spawned.status}): ${(spawned.stderr || spawned.stdout || '').trim() || '(no output)'}` +
        ' — the mounted symlink was still removed so the merge itself is not blocked; the companion session may need manual teardown.';
    }
  } else {
    warning =
      'a companion marker exists on this worktree but commands.worktree_companion_end is not configured — the mounted symlink was removed so the merge is not blocked, but the companion session (if the tool has one) was never explicitly ended.';
  }

  try {
    fs.unlinkSync(path.join(worktreeRoot, marker.mountPath));
  } catch {
    // best-effort: already gone, or never a real symlink — either way the
    // dirty-check right after this call is the authoritative signal, not
    // this cleanup attempt.
  }
  try {
    fs.unlinkSync(markerPath);
  } catch {
    // best-effort, same reasoning.
  }

  return { ended: !warning, sessionId: marker.sessionId || null, warning };
}

// hardening-4c PHASE 1 (locked): every pre-check plus staging the merge
// itself — split out of the old single-hold `mergeFeatureWorktreeLocked` so
// the public entrypoint's doc comment above stays attached to the exported
// function while this runs inside its own withStoreLock('worktree-admin')
// hold (see mergeFeatureWorktree above). Two possible return shapes:
//   - `{ done: true, result }` — a TERMINAL outcome already fully resolved
//     inside this lock (a textual conflict, already aborted+proven; or an
//     ALREADY_UP_TO_DATE no-op, already cleaned up if requested). The
//     caller returns `result` directly — there is no P2/P3 for these. Every
//     validation refusal above the staged merge itself is a typed
//     WorktreeMergeError THROW, same as before this rework.
//   - `{ done: false, state }` — a merge IS staged (MERGE_HEAD live,
//     nothing yet committed); `state` carries everything P3 needs to
//     re-check the fence and finish.
async function mergeFeatureWorktreeStage(mainRoot, options = {}) {
  const { id, cleanup = false, companionEndCommand } = options;

  if (typeof id !== 'string' || !id) {
    refuseMerge('WORKTREE_MERGE_INVALID_ID', `id ${JSON.stringify(id)} must be a non-empty string.`);
  }

  if (!isOrdinaryCheckout(mainRoot)) {
    refuseMerge(
      'WORKTREE_MERGE_CALLER_NOT_ORDINARY',
      `"bee worktree merge" must be run from the MAIN checkout, not a linked worktree (${mainRoot} is not an ordinary checkout) — a worktree, including the one being merged, cannot merge itself.`,
    );
  }

  const mainStoreRoot = path.join(mainRoot, '.bee');
  const grants = readGrants(mainStoreRoot);
  if (grants[id] !== true) {
    refuseMerge('WORKTREE_MERGE_UNKNOWN_ID', `no granted worktree found for id ${JSON.stringify(id)} — run "bee worktree list" to see granted ids.`);
  }

  const resolved = resolveWorktreeById(mainRoot, id);
  if (!resolved || !fs.existsSync(resolved.worktreeRoot)) {
    refuseMerge(
      'WORKTREE_MERGE_UNKNOWN_ID',
      `id ${JSON.stringify(id)} is granted but no matching, bidirectionally-valid git worktree link was found under ${mainRoot} (or the worktree no longer exists on disk) — run "git worktree prune" and "bee worktree unregister --id ${id}" if it was removed by hand.`,
    );
  }
  const { worktreeRoot } = resolved;

  // worktree-companion-hook: must run BEFORE the worktree-dirty check right
  // below — see teardownCompanionIfPresent's own doc comment for why this
  // can't wait until after a successful merge like bee-worktree cleanup can.
  const companion = teardownCompanionIfPresent(mainRoot, worktreeRoot, companionEndCommand);

  if (isTreeDirty(mainRoot)) {
    refuseMerge('WORKTREE_MERGE_MAIN_DIRTY', `the MAIN checkout at ${mainRoot} has uncommitted changes ("git status --porcelain" is non-empty) — commit or stash before merging.`);
  }
  if (isTreeDirty(worktreeRoot)) {
    refuseMerge(
      'WORKTREE_MERGE_WORKTREE_DIRTY',
      `the worktree at ${worktreeRoot} has uncommitted changes ("git status --porcelain" is non-empty) — commit or stash before merging. (A bootstrapped, gitignored .bee store alone is NOT dirty, per decision D8a.)`,
    );
  }

  const branch = currentBranch(worktreeRoot);
  if (!branch) {
    refuseMerge('WORKTREE_MERGE_DETACHED_HEAD', `the worktree at ${worktreeRoot} is on a detached HEAD — check out its branch before merging.`);
  }

  const { feature, created: createdFeature, stateFeature } = resolveWorktreeFeature(worktreeRoot);
  const expectedBranch = feature ? `wt/${feature}` : null;
  const branchOk = expectedBranch ? branch === expectedBranch : WT_BRANCH_RE.test(branch);
  if (!branchOk) {
    // issues-46-53 D4 (#46): the branch name is FIXED at creation and is
    // never the thing to change — but the old message stopped at "not its
    // expected ... branch", which reads as an accusation against the branch
    // and pointed the user straight at the one field they must not touch.
    // Name the field that actually drifted instead, whenever we can tell.
    let drift = '';
    if (createdFeature && stateFeature && createdFeature !== stateFeature) {
      drift =
        ` This worktree was CREATED as feature ${JSON.stringify(createdFeature)} (its immutable creation slug, which "${expectedBranch}" comes from), while its .bee/state.json now records feature ${JSON.stringify(stateFeature)}` +
        ' — the FEATURE FIELD drifted after creation (a rename, "bee state set --feature", or a new feature started in this worktree); the branch did not. Do NOT rename the branch to match: check "' +
        `${expectedBranch}" back out in the worktree, or merge the branch you actually want by hand.`;
    } else if (!createdFeature && stateFeature) {
      drift =
        ` "${expectedBranch}" is derived from this worktree's MUTABLE .bee/state.json "feature" field (${JSON.stringify(stateFeature)}) because the worktree predates bee's immutable creation-slug record` +
        ' — if the feature was renamed after the worktree was created, that FIELD is what drifted, not the branch. The branch name is fixed at creation; do not rename it to match.';
    }
    refuseMerge(
      'WORKTREE_MERGE_BRANCH_MISMATCH',
      `the worktree at ${worktreeRoot} is checked out to "${branch}", not its expected ${expectedBranch ? `"${expectedBranch}"` : '"wt/<slug>"-style'} branch — merge refuses to guess which branch to consume.${drift}`,
    );
  }

  // ── everything above is zero-mutation; the staged merge below is the
  // first real write. It is deliberately staged with `--no-commit` so verify
  // always runs against a merged-but-UNCOMMITTED tree (or is skipped for a
  // no-op / verify-less repo) and NOTHING is committed to main until a
  // semantic gate says it's safe — decision D2-REVISED. ────────────────────
  const preMergeHead = runGit(mainRoot, ['rev-parse', 'HEAD']).stdout.trim();
  const mergeHeadFile = path.join(mainRoot, '.git', 'MERGE_HEAD');
  const mergeMessage = `Merge worktree ${id} (branch ${branch}) via bee worktree merge`;

  const mergeResult = runGit(mainRoot, ['merge', '--no-ff', '--no-commit', '--', branch]);
  if (mergeResult.status !== 0) {
    // Textual conflict (or another merge-attempt failure) — abort whatever
    // git staged, then PROVE main is back to its pre-merge state before ever
    // telling the caller so; a claim of "main untouched" that isn't checked
    // is exactly the bug this staged-transaction rework replaces.
    runGit(mainRoot, ['merge', '--abort']);
    const proof = mainUntouchedProof(mainRoot, preMergeHead, mergeHeadFile);
    if (!proof.ok) {
      refuseMerge(
        'WORKTREE_MERGE_ABORT_FAILED',
        `"git merge --no-ff --no-commit ${branch}" failed and "git merge --abort" did NOT fully restore ${mainRoot} to its pre-merge state (${proof.reason}) — main may be left mid-merge; inspect it by hand before retrying.`,
      );
    }
    return {
      done: true,
      result: {
        ok: false,
        code: 'MERGE_CONFLICT',
        id,
        branch,
        worktreeRoot,
        message: `"git merge --no-ff ${branch}" hit a textual conflict — the merge was aborted and ${mainRoot} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); bee does not auto-resolve a textual conflict.`,
        output: `${mergeResult.stdout || ''}${mergeResult.stderr || ''}`,
        ...(companion ? { companion } : {}),
      },
    };
  }

  if (!fs.existsSync(mergeHeadFile)) {
    // Zero exit but nothing staged: "Already up to date" — branch has
    // nothing new for main. Never attempt a commit here; there is nothing
    // staged, so "git commit" would either error or produce an empty commit.
    const noopResult = {
      ok: true,
      merged: false,
      id,
      branch,
      worktreeRoot,
      code: 'ALREADY_UP_TO_DATE',
      verify: 'skipped',
      message: `"${branch}" is already up to date with ${mainRoot} — nothing to merge.`,
      ...(companion ? { companion } : {}),
    };
    // issues-46-53 D3 (#47): this early return used to skip attachCleanupOutcome
    // entirely, silently dropping an accepted --cleanup. See that function's
    // own comment for why the no-op is a SAFE cleanup target while
    // MERGE_CONFLICT/MERGE_VERIFY_RED are not.
    //
    // verifySkipped is deliberately FALSE here even though `verify` reads
    // 'skipped': that warning means "no commands.verify was recorded, so this
    // cleanup ran with no semantic gate" — which would be a lie on this path,
    // where a verify may well be configured and was skipped only because
    // nothing was merged for it to check. `code: 'ALREADY_UP_TO_DATE'` plus
    // `merged: false` already say exactly that, without inventing a risk.
    await attachCleanupOutcome(noopResult, { mainRoot, worktreeRoot, branch, id, cleanup, verifySkipped: false });
    return { done: true, result: noopResult };
  }

  // A merge is now staged (MERGE_HEAD live, changes staged, nothing yet
  // committed). Capture the `git write-tree` hash of the staged INDEX right
  // now, before this hold is released — P3's fence (checkMergeFence) proves
  // the staged tree was never tampered with while the lock was up for
  // grabs during P2 (advisor condition C2: HEAD alone cannot detect this,
  // since HEAD never moves for an uncommitted staged merge).
  try {
    const stagedTreeHash = runGit(mainRoot, ['write-tree']).stdout.trim();
    return {
      done: false,
      state: { id, branch, worktreeRoot, mergeMessage, preMergeHead, mergeHeadFile, stagedTreeHash, companion },
    };
  } catch (err) {
    // Exception-safety net: something unexpected blew up capturing the
    // staged-tree identity — abort rather than leave a half-done staged
    // merge sitting on main for a P2/P3 that will now never run.
    runGit(mainRoot, ['merge', '--abort']);
    throw err;
  }
}

// hardening-4c PHASE 3 (re-locked): re-verify the fence, then commit +
// post-commit-guard + cleanup. Called either right after Stage (no verify
// configured — same hold as P1, `verifyOutcome.ran === false`) or after the
// lock was released and re-acquired around the verify child
// (`verifyOutcome.ran === true`).
async function mergeFeatureWorktreeFinish(mainRoot, options, state, verifyOutcome) {
  const { cleanup = false } = options;
  const { id, branch, worktreeRoot, mergeMessage, preMergeHead, mergeHeadFile, stagedTreeHash, companion } = state;

  // Every exit from here must either commit (verify green / no verify
  // configured, fence clean) or abort (red verify below; fence drift below;
  // any unexpected throw) — the `finally` is the safety net for that last
  // case, so an unexpected crash between fence-check and commit can never
  // strand a staged merge on main.
  let committed = false;
  try {
    if (verifyOutcome.ran && verifyOutcome.status !== 0) {
      const tail = verifyOutcome.combined.split('\n').slice(-30).join('\n');
      runGit(mainRoot, ['merge', '--abort']);
      const proof = mainUntouchedProof(mainRoot, preMergeHead, mergeHeadFile);
      if (!proof.ok) {
        refuseMerge(
          'WORKTREE_MERGE_ABORT_FAILED',
          `verify failed and "git merge --abort" did NOT fully restore ${mainRoot} to its pre-merge state (${proof.reason}) — main may be left mid-merge; inspect it by hand before retrying.`,
        );
      }
      return {
        ok: false,
        code: 'MERGE_VERIFY_RED',
        id,
        branch,
        worktreeRoot,
        merged: false,
        verify: 'red',
        message:
          `the merge was textually clean but the post-merge verify failed against the merged-but-uncommitted tree — this is the semantic-conflict alarm: behavior broke even though git found no textual conflict. The merge was aborted and ${mainRoot} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); no merge commit exists. Fix-first before release.`,
        output_tail: tail,
        ...(companion ? { companion } : {}),
      };
    }

    // P3 fence (advisor condition C2): re-check everything that could have
    // drifted while the lock was released for verify, BEFORE ever
    // committing. A no-verify-configured merge reaches this in the SAME
    // hold P1 staged it in, so the fence is trivially clean there — the
    // check still runs (uniform code path, negligible cost) rather than
    // being conditioned on whether verify ran.
    const fenceDrift = checkMergeFence(mainRoot, { id, preMergeHead, mergeHeadFile, stagedTreeHash });
    if (fenceDrift) {
      runGit(mainRoot, ['merge', '--abort']);
      const proof = mainUntouchedProof(mainRoot, preMergeHead, mergeHeadFile);
      if (!proof.ok) {
        refuseMerge(
          'WORKTREE_MERGE_ABORT_FAILED',
          `the P3 fence detected drift (${fenceDrift}) and "git merge --abort" did NOT fully restore ${mainRoot} to its pre-merge state (${proof.reason}) — main may be left mid-merge; inspect it by hand before retrying.`,
        );
      }
      refuseMerge(
        'WORKTREE_MERGE_FENCE_DRIFT',
        `the staged merge was aborted before commit because the P3 re-check (advisor condition C2) detected drift while the 'worktree-admin' lock was released around the verify child: ${fenceDrift}. ${mainRoot} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); no merge commit exists.`,
      );
    }

    const commitResult = runGit(mainRoot, ['commit', '-m', mergeMessage]);
    if (commitResult.status !== 0) {
      // Should not happen — a live MERGE_HEAD always has something staged
      // to commit — but never silently swallow it: abort rather than leave
      // a half-done staged merge sitting on main.
      runGit(mainRoot, ['merge', '--abort']);
      refuseMerge(
        'WORKTREE_MERGE_COMMIT_FAILED',
        `"git commit" failed for the staged merge of ${branch} (${(commitResult.stderr || commitResult.stdout || '').trim() || `exit ${commitResult.status}`}) — the staged merge was aborted; ${mainRoot} was left untouched.`,
      );
    }
    committed = true;

    const result = {
      ok: true,
      merged: true,
      id,
      branch,
      worktreeRoot,
      verify: verifyOutcome.ran ? 'green' : 'skipped',
      ...(companion ? { companion } : {}),
    };

    // Post-commit guard (D2-REVISED): the commit above only ever contains
    // what git staged for the merge itself. If the verify command mutated
    // TRACKED files without bee's knowledge, those changes are now dirty
    // working-tree state sitting on top of an otherwise-clean merge commit
    // — surface that loudly instead of silently treating tree === commit.
    const postCommitStatus = runGit(mainRoot, ['status', '--porcelain', '--untracked-files=no']).stdout;
    if (postCommitStatus.trim().length > 0) {
      result.warning = {
        code: 'verify_mutated_tracked_files',
        message: `the post-merge verify command left tracked files modified after the merge commit landed ("git status --porcelain --untracked-files=no" is non-empty) — the merge commit itself is clean, but verify mutated the working tree afterward; inspect and commit/discard those changes separately. Recovery if a LATER independent verify goes red on this merge: "git revert -m 1 <merge-commit>".`,
        status: postCommitStatus,
      };
    }

    await attachCleanupOutcome(result, { mainRoot, worktreeRoot, branch, id, cleanup, verifySkipped: result.verify === 'skipped' });
    return result;
  } finally {
    if (!committed && fs.existsSync(mergeHeadFile)) {
      // Exception-safety net only: every normal exit above already aborted
      // (and proved it) before returning/throwing, so this fires only when
      // something unexpected blew up mid-commit without going through
      // either handled path yet.
      runGit(mainRoot, ['merge', '--abort']);
    }
  }
}
