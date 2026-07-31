#!/usr/bin/env node
// bee-write-guard: PreToolUse (Edit|Write|MultiEdit|Bash|Read|Glob|Grep) plus
// the Codex apply_patch tool path (cell codex-parity-3, decision D2).
// Four checks in one guard, first hit wins:
//   (a) gate guard   - no source writes before Gate 3 (execution approval)
//   (b) reservation  - during swarming, writes to unreserved paths are denied
//   (c) privacy/scout- secret-file reads emit the @@BEE_PRIVACY@@ marker;
//                      scout dirs (node_modules/, dist/, ...) are denied
//   (d) CLI-shape    - a Bash call shaped like a bee.mjs/bee_*.mjs invocation
//                      is validated against the shared command registry
//                      (harness-integration D4); malformed args are denied
//                      before the shell executes them. Strictly additive:
//                      runs only when checks (a)-(c) found no denial, and its
//                      own parsing failures are contained to itself (never
//                      allowed to reach the shared catch below, which would
//                      fail-open for ALL FOUR checks instead of just this one).
// Codex apply_patch: the canonical patch envelope's Add/Update/Delete/Move
// target lines are parsed and every proved target runs the SAME
// gate/direct-edit/reservation decisions as Edit/Write/Bash (cell
// codex-parity-3). P1 repair (cell codex-parity-4, plan-review third bullet):
// once an apply_patch event is INTERCEPTED (a canonical "*** Begin Patch"
// envelope was found), a target set that cannot be fully proved — zero
// Add/Update/Delete/Move/"Move to" lines parsed, or any parsed target that
// does not resolve to an in-repo relative path — DENIES (exit 2) with a
// corrective message, never allows. A visible "applypatch-unparsed" coverage
// gap is still logged either way. Malformed OUTER hook payloads (apply_patch
// called but no canonical patch envelope is present in tool_input at all)
// and genuinely unsupported host paths keep D2's visible fail-open.
// Input/root/logging go through the shared runtime adapter (hooks/adapter.mjs):
// stdin is normalized before any property access and root discovery lives
// inside the fail-open boundary.
// Deny = exit 2 with the reason (and marker, for privacy) on stderr.
// Everything else is fail-open: exit 0 (crashes logged to .bee/logs/hooks.jsonl).

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { readHookContext, logCrash, logCoverageGap, libModuleUrl } from "./adapter.mjs";
import { tokenizeCommand } from "./tokenize-command.mjs";

const HOOK_NAME = "write-guard";
const READ_TOOLS = new Set(["Read", "Glob", "Grep"]);
const WRITE_TOOLS = new Set(["Edit", "Write", "MultiEdit"]);
const APPLY_PATCH_TOOLS = new Set(["apply_patch", "ApplyPatch"]);

// Convert a tool-supplied path (absolute or relative) to a forward-slash
// path relative to the repo root. Returns null when the path escapes the repo.
function lexicalRelPath(root, cwd, rawPath) {
  if (!rawPath || typeof rawPath !== "string") {
    return null;
  }
  const abs = path.isAbsolute(rawPath) ? rawPath : path.resolve(cwd || root, rawPath);
  const rel = path.relative(root, abs);
  if (!rel || rel === "." || rel.startsWith("..") || path.isAbsolute(rel)) {
    return null;
  }
  return rel.split(path.sep).join("/");
}

function normalizeToolPath(rawPath) {
  // Preserve shell's `\ ` escaped-space spelling, but treat every other
  // backslash as a Windows separator so traversal cannot hide behind it.
  return String(rawPath).replace(/\\(?!\s)/g, path.sep);
}

// ─── home-prefixed target refusal (cell gmr-1, GH #71, CONTEXT
// guard-memory-roots D8) ──────────────────────────────────────────────────
// A raw target whose FIRST path segment is a shell home reference — `~/…`,
// `~someuser/…`, `$HOME/…`, `${HOME}/…` — is refused outright, BEFORE any
// containment work, and therefore takes the same deny path (and the same
// deny string) as the absolute spelling of the same destination.
//
// THE HOLE THIS CLOSES: a leading `~/` is neither absolute nor does it
// contain `..`, so canonicalRelPath used to resolve it against cwd as a
// literal directory named `~` (or `$HOME`), producing an in-repo relative
// path that PASSED containment and flowed into checkWrite under a
// repo-relative identity with no relation to where the shell would actually
// put the bytes. `echo hi > /home/u/.claude/x` denied while
// `echo hi > ~/.claude/x` — the same file — allowed.
//
// DENY-OUTRIGHT, NOT EXPAND-THEN-CONTAIN — and cell gmr-2's declared
// `guards.memory_root` builds on this choice, so it is stated here:
//   1. Expanding would make the wall's decision depend on an ENVIRONMENT
//      VARIABLE. CONTEXT D1 puts the security boundary at declaration
//      precisely because "a root the guard infers is a root an attacker can
//      arrange"; reading $HOME to decide containment is inference.
//   2. We cannot faithfully model the shell anyway. The tokenizer discards
//      quoting, and bash expands `~/x` and "$HOME/x" but NOT "~/x" and
//      '$HOME/x'. The destination is genuinely ambiguous, and D4 says an
//      ambiguity resolves closed.
//   3. Deny needs no resolution step, so it has no error path that could
//      fail open, and it is byte-identical across runtimes and machines
//      without plumbing an environment through the differential rig.
// CONSEQUENCE FOR gmr-2: a declared memory root is honored on the ABSOLUTE
// spelling of a target only. If a tilde spelling is ever to be honored, it
// must be expanded from the declared config value, never from the process
// environment.
//
// A BARE `~` (the whole token, no separator) is deliberately NOT matched:
// BROAD_TARGETS (lib/guards.mjs) already owns it and its behavior is
// unchanged. A tilde-prefix bash itself would not expand — anything but a
// login-name-shaped word before the separator, e.g. `~$Report.docx/x` — is
// left alone too, so an in-repo file with a tilde-ish name is never
// false-denied.
const HOME_PREFIXED_TARGET_RE = /^(?:~[A-Za-z0-9._+-]*|\$HOME|\$\{HOME\})[/\\]/;

function isHomePrefixedTarget(rawTarget) {
  if (!rawTarget || typeof rawTarget !== "string") return false;
  return HOME_PREFIXED_TARGET_RE.test(rawTarget);
}

function canonicalRelPath(workRoot, cwd, rawPath) {
  if (!rawPath || typeof rawPath !== "string") return null;
  // gmr-1: refuse before containment — never resolve a home reference.
  if (isHomePrefixedTarget(rawPath)) return null;
  const rootReal = (() => {
    try {
      return fs.realpathSync.native(workRoot);
    } catch {
      return null;
    }
  })();
  if (!rootReal) return null;

  const normalized = normalizeToolPath(rawPath);
  // A foreign Windows absolute/UNC spelling cannot be safely mapped by a
  // POSIX host. Windows itself handles these through path.isAbsolute and its
  // case-insensitive path.relative implementation below.
  if (path.sep !== "\\" && (/^[A-Za-z]:[\\/]/.test(rawPath) || /^\\\\/.test(rawPath))) {
    return null;
  }
  if (!path.isAbsolute(normalized) && normalized.split(path.sep).includes("..")) return null;

  const cwdBase = path.isAbsolute(cwd || "") ? cwd : rootReal;
  const lexicalTarget = path.isAbsolute(normalized) ? path.resolve(normalized) : path.resolve(cwdBase, normalized);
  let cursor = lexicalTarget;
  const unresolved = [];
  while (true) {
    try {
      fs.lstatSync(cursor);
      break;
    } catch (error) {
      if (!error || error.code !== "ENOENT") return null;
      const parent = path.dirname(cursor);
      if (parent === cursor) return null;
      unresolved.unshift(path.basename(cursor));
      cursor = parent;
    }
  }

  let ancestorReal;
  try {
    ancestorReal = fs.realpathSync.native(cursor);
  } catch {
    return null;
  }
  const canonicalTarget = path.resolve(ancestorReal, ...unresolved);
  const rel = path.relative(rootReal, canonicalTarget);
  if (!rel || rel === "." || rel === ".." || rel.startsWith(`..${path.sep}`) || path.isAbsolute(rel)) {
    return null;
  }
  return rel.split(path.sep).join("/");
}

// ─── sibling-worktree-aware denial enrichment (GH #31, message-only) ──────
// When canonicalRelPath has ALREADY failed containment for a raw target
// against the physical worktree `root`, this cheaply checks whether the
// target actually lives inside a KNOWN sibling checkout — a granted worktree
// registered in the MAIN store's runtime/worktree-grants.json (mirrors the
// inline, dependency-light read adapter.mjs's own resolveRoots does for the
// CURRENT worktree's storeRoot decision — never imported here, kept
// dependency-light on purpose), or, when THIS session is itself rooted in a
// linked worktree, the MAIN checkout it was cut from. This NEVER changes the
// deny decision — it only replaces the generic containment text with a more
// specific one when it can prove where the target belongs. Any failure at
// any step (unreadable/malformed grants file, a broken worktree link, or the
// target simply being elsewhere) returns null and the caller keeps the
// existing generic message untouched — fail-open to generic, by design.

const GENERIC_CONTAINMENT_MESSAGE =
  "bee write guard denied this target: it could not be canonically contained inside the physical worktree. " +
  "FIX: use a plain in-worktree path without traversal, outside absolute paths, or symlink escapes.";
const GENERIC_BASH_CONTAINMENT_MESSAGE =
  "bee write guard denied Bash: one or more extracted targets could not be canonically contained inside the physical worktree. " +
  "FIX: use plain in-worktree paths without traversal, outside absolute paths, or symlink escapes.";

function readGitdirPointer(file, base) {
  try {
    let raw = fs.readFileSync(file, "utf8").trim();
    if (!raw) return null;
    if (raw.startsWith("gitdir:")) raw = raw.slice("gitdir:".length).trim();
    if (!raw) return null;
    return path.resolve(base, raw.replace(/\\/g, path.sep));
  } catch {
    return null;
  }
}

function realpathOrNull(value) {
  try {
    return fs.realpathSync.native(value);
  } catch {
    return null;
  }
}

// Mirrors adapter.mjs resolveRoots' linked-valid branch, without importing
// it: if `workRoot`'s own ".git" is a FILE pointing at
// "<mainRoot>/.git/worktrees/<id>", returns { mainRoot, id }; else null
// (ordinary checkout, or a broken/foreign link — never guessed).
function deriveCurrentWorktree(workRoot) {
  try {
    const marker = path.join(workRoot, ".git");
    const stat = fs.statSync(marker);
    if (!stat.isFile()) return null;
    const gitdir = readGitdirPointer(marker, workRoot);
    if (!gitdir) return null;
    const worktreesRoot = path.resolve(gitdir, "..");
    const commonGitDir = path.resolve(worktreesRoot, "..");
    if (path.basename(worktreesRoot) !== "worktrees" || path.basename(commonGitDir) !== ".git") {
      return null;
    }
    const mainRoot = realpathOrNull(path.dirname(commonGitDir));
    if (!mainRoot) return null;
    return { mainRoot, id: path.basename(gitdir) };
  } catch {
    return null;
  }
}

// Resolves a granted worktree `id` (from the MAIN store's grants registry)
// to its worktreeRoot, with the SAME bidirectional gitdir check
// worktree-store.mjs's resolveWorktreeById uses (forward:
// <mainRoot>/.git/worktrees/<id>/gitdir -> <worktreeRoot>/.git; reverse:
// <worktreeRoot>/.git -> back to that same <mainRoot>/.git/worktrees/<id>) —
// never trusts a one-directional pointer alone. Null on any mismatch,
// missing file, or unreadable content.
function resolveGrantedWorktreeRoot(mainRoot, id) {
  try {
    const gitWorktreeDir = path.join(mainRoot, ".git", "worktrees", id);
    if (!fs.statSync(gitWorktreeDir).isDirectory()) return null;
    const forward = readGitdirPointer(path.join(gitWorktreeDir, "gitdir"), gitWorktreeDir);
    if (!forward) return null;
    const worktreeRoot = path.dirname(forward);
    const reverse = readGitdirPointer(path.join(worktreeRoot, ".git"), worktreeRoot);
    if (!reverse || path.resolve(reverse) !== path.resolve(gitWorktreeDir)) return null;
    return realpathOrNull(worktreeRoot);
  } catch {
    return null;
  }
}

// Reads <mainRoot>/.bee/runtime/worktree-grants.json and returns the ids
// granted `true`, else [] — fail-open on ANY error (missing file, unparseable
// JSON, non-object payload): never throws, never allows, just yields no known
// siblings so the caller falls back to the generic message.
function readGrantedWorktreeIds(mainRoot) {
  try {
    const raw = fs.readFileSync(path.join(mainRoot, ".bee", "runtime", "worktree-grants.json"), "utf8");
    const grants = JSON.parse(raw);
    if (!grants || typeof grants !== "object" || Array.isArray(grants)) return [];
    return Object.keys(grants).filter((id) => grants[id] === true);
  } catch {
    return [];
  }
}

// Resolves `rawTarget` the same lenient way canonicalRelPath does (walk up
// through ENOENT segments, realpath the first existing ancestor) but returns
// the resolved ABSOLUTE path instead of a root-relative one — needed here
// because a sibling/main root can live entirely OUTSIDE `root`, which
// canonicalRelPath's root-relative contract can't express. Null on any
// failure (unresolvable path, Windows-foreign spelling on a POSIX host, ...).
function resolveTargetRealpath(cwd, root, rawTarget) {
  if (!rawTarget || typeof rawTarget !== "string") return null;
  // gmr-1: a home-prefixed spelling resolves to nothing here either, so the
  // companion-mount escape hatch cannot rescue it and the cross-worktree
  // message enrichment cannot claim to know where it points. Both callers
  // treat null as "no match", which keeps the generic containment denial —
  // byte-identical to the absolute spelling of the same destination.
  if (isHomePrefixedTarget(rawTarget)) return null;
  const normalized = normalizeToolPath(rawTarget);
  if (path.sep !== "\\" && (/^[A-Za-z]:[\\/]/.test(rawTarget) || /^\\\\/.test(rawTarget))) {
    return null;
  }
  const cwdBase = path.isAbsolute(cwd || "") ? cwd : root;
  const lexicalTarget = path.isAbsolute(normalized) ? path.resolve(normalized) : path.resolve(cwdBase, normalized);
  let cursor = lexicalTarget;
  const unresolved = [];
  while (true) {
    try {
      fs.lstatSync(cursor);
      break;
    } catch (error) {
      if (!error || error.code !== "ENOENT") return null;
      const parent = path.dirname(cursor);
      if (parent === cursor) return null;
      unresolved.unshift(path.basename(cursor));
      cursor = parent;
    }
  }
  const ancestorReal = realpathOrNull(cursor);
  if (!ancestorReal) return null;
  return path.resolve(ancestorReal, ...unresolved);
}

// True when real path `childReal` is real root `parentReal` itself or
// strictly nested under it.
function isUnderRoot(parentReal, childReal) {
  if (!parentReal || !childReal) return false;
  const rel = path.relative(parentReal, childReal);
  return rel === "" || (rel !== ".." && !rel.startsWith(`..${path.sep}`) && !path.isAbsolute(rel));
}

// Returns a replacement denial reason naming a known sibling/main checkout,
// or null to keep the existing generic containment message (unknown outside
// path, or ANY failure reading/deriving worktree state).
function describeCrossWorktreeTarget(root, cwd, rawTarget) {
  try {
    const targetReal = resolveTargetRealpath(cwd, root, rawTarget);
    if (!targetReal) return null;

    const current = deriveCurrentWorktree(root);
    const mainRoot = current ? current.mainRoot : realpathOrNull(root);
    if (!mainRoot) return null;

    // Session rooted in a worktree, target inside the MAIN checkout instead.
    if (current && isUnderRoot(mainRoot, targetReal)) {
      return (
        "bee write guard denied this target: it could not be canonically contained inside the physical worktree — " +
        "this path belongs to the main checkout, not this worktree. FIX: run this from a session rooted there."
      );
    }

    // Target inside a KNOWN GRANTED sibling worktree.
    for (const id of readGrantedWorktreeIds(mainRoot)) {
      if (current && id === current.id) continue; // this session's own root, not a sibling
      const worktreeRoot = resolveGrantedWorktreeRoot(mainRoot, id);
      if (worktreeRoot && isUnderRoot(worktreeRoot, targetReal)) {
        return (
          "bee write guard denied this target: it could not be canonically contained inside the physical worktree — " +
          `it resolves inside worktree "${id}". FIX: open a session with cwd=${worktreeRoot} to work there, or merge it ` +
          `back from main via \`bee worktree merge --id ${id}\`.`
        );
      }
    }

    return null;
  } catch {
    return null;
  }
}

// ─── large-read guard (router-cost D1/D2/D3/D4) ────────────────────────────
// Denies an unbounded Read of a file at/above a configurable line threshold.
// Lives directly in this branch (not guards.mjs — checkRead is pattern-only,
// no I/O) so the fail-open posture is local and easy to audit: any stat/read
// error, a directory, a nonexistent path, an oversized file, or binary
// content all return null (allow) — a guard that denies because it could not
// measure is worse than no guard. The threshold comes from
// `.bee/config.json`'s `guards.max_read_lines` (a LOCAL_ONLY namespace, set
// via `bee config set --key guards.max_read_lines --value <n>`), defaulting
// to 800 (D2's measured value) when the key is absent — the same
// absent-key-means-default reading `hookEnabled`'s `!== false` uses in
// .bee/bin/lib/state.mjs, adapted for a numeric default instead of a boolean
// one.
const DEFAULT_MAX_READ_LINES = 800;
// Files larger than this are never measured — counting lines would mean
// reading the whole thing into memory just to decide whether to deny reading
// the whole thing into context. Fail-open (allow) instead.
const READ_SIZE_GUARD_CAP_BYTES = 25 * 1024 * 1024;

function resolveMaxReadLines(config) {
  const raw = config && config.guards && config.guards.max_read_lines;
  return typeof raw === "number" && Number.isFinite(raw) && raw > 0 ? raw : DEFAULT_MAX_READ_LINES;
}

// Null-byte sniff over a bounded prefix: cheap and matches the common
// heuristic other tools use to distinguish text from binary content.
function looksBinary(buffer) {
  const scanLen = Math.min(buffer.length, 8000);
  for (let i = 0; i < scanLen; i += 1) {
    if (buffer[i] === 0) return true;
  }
  return false;
}

// Counts newline-delimited lines, counting a non-empty trailing partial line
// (no final "\n") as one more line — matches what a human reading the file
// would call "how many lines", including the last one.
function countLines(buffer) {
  let count = 0;
  for (let i = 0; i < buffer.length; i += 1) {
    if (buffer[i] === 10) count += 1;
  }
  if (buffer.length > 0 && buffer[buffer.length - 1] !== 10) count += 1;
  return count;
}

// Returns a denial reason string, or null to allow. `absPath` must already be
// proven inside the repo by the caller; `label` is the repo-relative path
// used in the message. Every error path (ENOENT, EACCES, a directory, a
// symlink loop, ...) is caught and returns null: fail-open, never deny
// because measurement failed.
function checkReadSizeDenial(absPath, label, threshold) {
  try {
    const stat = fs.statSync(absPath);
    if (!stat.isFile()) return null;
    if (stat.size > READ_SIZE_GUARD_CAP_BYTES) return null;
    const buffer = fs.readFileSync(absPath);
    if (looksBinary(buffer)) return null;
    const lineCount = countLines(buffer);
    if (lineCount < threshold) return null;
    return (
      `bee read-size guard: "${label}" is ${lineCount} lines (threshold: ${threshold}) and this Read ` +
      "has neither `offset` nor `limit` — reading it unbounded would load the whole file into context. " +
      "FIX: pass `limit` (and optionally `offset`) to read a slice, or dispatch a `bee-extract` worker to read the whole file."
    );
  } catch {
    return null;
  }
}

// ─── worktree-companion-hook mount recognition (fix-write-guard-symlink) ──
// `bee worktree new --with-companion` symlinks a nested repo's own worktree
// into this one at `commands.worktree_companion_mount` and records the
// mapping in `<root>/.bee/companion-session.json` (worktree-store.mjs's
// runCompanionStart: `{sessionId, worktreePath, mountPath}`). A target that
// lexically resolves outside the physical worktree PURELY because it crosses
// that specific, marker-declared symlink is not an escape — it is the
// companion's own working tree, one hop away. Returns a root-relative path
// (rooted at `mountPath`, matching canonicalRelPath's contract) when the
// marker is present, parseable, names an existing `worktreePath`/`mountPath`
// pair, the marker's declared `worktreePath` realpath matches the MOUNT
// SYMLINK'S live realpath (a stale/tampered marker must not grant access to
// wherever the symlink happens to point today), and `rawTarget` resolves
// inside that mount. Null on any mismatch or failure — the caller falls back
// to the existing generic containment denial, unchanged.
function resolveCompanionMountedRelPath(root, cwd, rawTarget) {
  try {
    const raw = fs.readFileSync(path.join(root, ".bee", "companion-session.json"), "utf8");
    const marker = JSON.parse(raw);
    const declaredWorktreePath = marker && typeof marker === "object" ? marker.worktreePath : undefined;
    const mountPath = marker && typeof marker === "object" ? marker.mountPath : undefined;
    if (
      typeof declaredWorktreePath !== "string" || !declaredWorktreePath ||
      typeof mountPath !== "string" || !mountPath
    ) {
      return null;
    }

    const declaredReal = realpathOrNull(declaredWorktreePath);
    const liveMountReal = realpathOrNull(path.join(root, mountPath));
    if (!declaredReal || !liveMountReal || declaredReal !== liveMountReal) {
      return null;
    }

    const targetReal = resolveTargetRealpath(cwd, root, rawTarget);
    if (!targetReal || !isUnderRoot(liveMountReal, targetReal)) {
      return null;
    }

    const offset = path.relative(liveMountReal, targetReal);
    if (!offset) return mountPath;
    return `${mountPath}/${offset.split(path.sep).join("/")}`;
  } catch {
    return null;
  }
}

// ─── declared memory root (cell gmr-3, GH #71, CONTEXT guard-memory-roots
// D1/D3/D4/D5/D6/D9/D10/D11) ──────────────────────────────────────────────
// The write guard contains every write to the physical worktree, which makes
// the agent's own persistent memory (conventionally
// `~/.claude/projects/<slug>/memory/`) unreachable and silently loses durable
// learnings. This is the ONE declared escape hatch for it.
//
// THE SECURITY ARGUMENT, in two parts — neither is sufficient alone:
//   1. DECLARATION (D1). The path comes from `guards.memory_root` in config,
//      read through the same `readConfig(storeRoot)` this hook already calls,
//      which overlays the gitignored `.bee/config.local.json`. Never
//      auto-discovered, never read from the environment: "a root the guard
//      infers is a root an attacker can arrange".
//   2. A LIVE MARKER (D9). Config alone is self-service — an agent CAN write
//      `.bee/config.local.json` at phase `idle`, because `.bee/` sits in
//      GATE_ALLOWED_PREFIXES. So the declared root is honored only while a
//      file named `.bee-write-root` exists directly inside its REALPATHED
//      directory, re-checked on every hit. An agent cannot forge that marker,
//      because every out-of-worktree write is denied until the root is
//      honored, and the root is not honored until the marker exists. This is
//      the same live-marker discipline resolveCompanionMountedRelPath uses.
//
// A HIT IS A REAL ALLOW THAT SHORT-CIRCUITS (D6): the target never reaches
// guards.checkWrite, so the intake gate, Gate 3, reservations and holds do not
// apply to it. That is deliberate and is the whole point — a learning must be
// recordable at phase `idle`, which is exactly when the intake gate is shut.
// Those semantics are repo-scoped and have no meaning for a path with no
// repo-relative form.
//
// SCOPE LIMITS, stated so they are not mistaken for oversights:
//   - Absolute target spellings only. `isHomePrefixedTarget` (cell gmr-1, D8)
//     refuses `~/…`/`$HOME/…` BEFORE any containment work and this branch does
//     not weaken that: a tilde-spelled target stays denied even when it names
//     the declared root. Expanding a target would put the wall's decision in
//     an environment variable. Expanding the CONFIG VALUE's leading `~` is a
//     different thing and is safe — that is a value we read ourselves, not a
//     shell token we are guessing the expansion of.
//   - Not honored on the apply_patch leg (D11): that leg denies any unproved
//     target and does not consult the companion mount either.
//   - Everything here fails CLOSED (D4/D10). Every path returns "no match",
//     and the whole evaluation traps its own exceptions so a throw can never
//     reach this hook's outer catch, which returns exit 0 and would fail the
//     ENTIRE hook open rather than just this check.
const MEMORY_ROOT_MARKER = ".bee-write-root";

// Expands a leading `~` in a CONFIG VALUE only (see the scope note above).
// A `~user` spelling is NOT expanded — we do not resolve other users' homes.
function expandConfigHomePrefix(value) {
  if (value === "~") return os.homedir();
  if (value.startsWith("~/") || value.startsWith("~\\")) {
    return path.join(os.homedir(), value.slice(2));
  }
  return value;
}

// True when the realpathed root is, or directly contains, a `.git` or `.bee`
// directory. "Is" is checked over every path SEGMENT (so a root anywhere
// inside a `.git`/`.bee` tree is refused, not just one whose basename
// matches); "contains" is a bounded depth-1 child check — a deep recursive
// scan on every hook invocation would be a real cost for a sanity refusal, and
// the marker file still has to be placed by a human at the root itself.
function rootTouchesRepoControlDir(rootReal) {
  const banned = new Set([".git", ".bee"]);
  for (const segment of rootReal.split(path.sep)) {
    if (banned.has(segment)) return true;
  }
  for (const name of banned) {
    try {
      if (fs.statSync(path.join(rootReal, name)).isDirectory()) return true;
    } catch {
      // ENOENT (the common case) and any other stat error both mean "not a
      // directory we can see here" — keep checking the remaining names.
    }
  }
  return false;
}

// Resolves `guards.memory_root` to a REALPATHED, marker-verified, sanity-
// checked absolute directory, or null. Every refusal in D5/D10 is evaluated on
// the realpathed root, never the raw spelling: a refusal list checked on the
// raw spelling is trivially evaded by pointing a symlink at `/`.
function resolveDeclaredMemoryRoot(workRoot, config) {
  const raw = config && config.guards ? config.guards.memory_root : undefined;
  if (typeof raw !== "string" || !raw.trim()) return null;
  const expanded = expandConfigHomePrefix(raw.trim());
  if (!path.isAbsolute(expanded)) return null;

  // Must already exist as a directory (D10) — realpath proves both.
  const rootReal = realpathOrNull(expanded);
  if (!rootReal) return null;
  let rootStat;
  try {
    rootStat = fs.statSync(rootReal);
  } catch {
    return null;
  }
  if (!rootStat.isDirectory()) return null;

  // D5 refusals, on the realpathed root.
  if (path.dirname(rootReal) === rootReal) return null; // the filesystem root
  const homeReal = realpathOrNull(os.homedir());
  if (homeReal && rootReal === homeReal) return null; // a bare home directory
  const worktreeReal = realpathOrNull(workRoot);
  if (!worktreeReal) return null; // cannot prove non-containment -> refuse
  if (isUnderRoot(rootReal, worktreeReal)) return null; // a root containing the worktree
  if (rootTouchesRepoControlDir(rootReal)) return null; // is/contains .git or .bee

  // D9: the live marker, re-read on every hit. A directory named
  // `.bee-write-root` is not a marker.
  try {
    if (!fs.statSync(path.join(rootReal, MEMORY_ROOT_MARKER)).isFile()) return null;
  } catch {
    return null;
  }

  return rootReal;
}

// True when `rawTarget` is a write into the declared, marker-verified memory
// root. `config` is the already-read config object (or null). Containment
// reuses exactly the discipline the worktree wall itself uses —
// resolveTargetRealpath (realpath-walk through not-yet-existing segments) plus
// isUnderRoot — so traversal out of the root, and a symlink INSIDE the root
// that resolves outside it, both come back false and are denied as today.
// Returns false on absolutely every failure, and traps its own throws.
function isDeclaredMemoryRootTarget(workRoot, cwd, config, rawTarget) {
  try {
    if (!rawTarget || typeof rawTarget !== "string") return false;
    // gmr-1/D8: a home-prefixed spelling is refused here too. A declared root
    // is honored on the ABSOLUTE spelling of a target only.
    if (isHomePrefixedTarget(rawTarget)) return false;
    const normalized = normalizeToolPath(rawTarget);
    if (!path.isAbsolute(normalized)) return false;
    // D10: `..` is rejected on this branch regardless of absoluteness. The
    // realpath containment below would catch a genuine escape anyway; this
    // refuses the spelling outright so there is nothing subtle to reason about.
    if (normalized.split(path.sep).includes("..")) return false;

    const rootReal = resolveDeclaredMemoryRoot(workRoot, config);
    if (!rootReal) return false;

    const targetReal = resolveTargetRealpath(cwd, workRoot, rawTarget);
    if (!targetReal) return false;
    if (targetReal === rootReal) return false; // the directory itself is not a write target
    return isUnderRoot(rootReal, targetReal);
  } catch {
    return false;
  }
}

// The subset of guards.mjs `isBroad`'s shape test an ABSOLUTE target can
// match (`<dir>/*`, `<dir>/.`). Used only to tell a broad-write signal that
// came from a memory-root target's own spelling apart from one that came from
// a pathless trigger like `git add --all` — see the Bash branch below.
function isBroadTargetSpelling(rawTarget) {
  if (!rawTarget || typeof rawTarget !== "string") return false;
  const normalized = rawTarget.replace(/\\/g, "/");
  return normalized.endsWith("/*") || normalized.endsWith("/.");
}

function getNestedString(obj, keys) {
  for (const key of keys) {
    const value = obj && typeof obj === "object" ? obj[key] : undefined;
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return "";
}

// wcg-2 (worktree-concurrency-guard, D1b): the lexical absolute spelling of a
// raw tool target, derived exactly as canonicalRelPath/resolveTargetRealpath
// derive theirs (cwd-relative unless already absolute). guards.
// isSharedNestedCheckoutTarget realpaths it itself, so a lexical spelling is
// all it needs.
function lexicalAbsTarget(root, cwd, rawTarget) {
  const normalized = normalizeToolPath(rawTarget);
  const cwdBase = path.isAbsolute(cwd || "") ? cwd : root;
  return path.isAbsolute(normalized) ? path.resolve(normalized) : path.resolve(cwdBase, normalized);
}

// wcg-2 (D3/D4): the typed, actionable refusal when a write targets a shared
// nested checkout that no verified companion marker covers while another
// session is live. Per D4 it directs to opening a FRESH companion worktree —
// never an in-place conversion of the current one, which `bee worktree new`
// cannot do.
function sharedNestedCheckoutRefusal(rel) {
  return (
    `bee shared-checkout guard: "${rel}" is inside a nested checkout that another ` +
    "live session can also reach, and no verified companion mount covers it. " +
    "Writing here can silently overwrite the other session's work — the exact " +
    "failure this guard exists to prevent. " +
    "FIX: open a FRESH companion worktree — run `bee worktree new --with-companion` " +
    "to create a new worktree that mounts this shared checkout under a verified " +
    "marker, then do this work there. The current worktree cannot be converted " +
    "into a companion mount; you must create a new one."
  );
}

// A DISTINCT, typed refusal for when the shared-checkout detection primitive
// itself throws (an unreadable nested `.git`, a broken symlink, an EACCES
// realpath). plan.md's Test Matrix requires this exact surface to fail CLOSED
// — an error is not a "nothing shared here" answer, and it is most likely to
// happen during the very concurrent race this guard exists to catch, so we
// deny rather than let the hook's outer catch-all fail open.
function sharedNestedDetectionErrorRefusal(rel) {
  return (
    `bee shared-checkout guard: could not determine whether "${rel}" is inside a ` +
    "nested checkout another live session can reach — the detection check itself " +
    "errored. This guard fails CLOSED on a detection error and never silently " +
    "allows the write, because an error here most likely means exactly the " +
    "concurrent race this guard exists to prevent. " +
    "FIX: resolve the underlying filesystem error (a broken symlink, an unreadable " +
    "nested `.git`, or a permission problem), then retry — or open a FRESH " +
    "companion worktree with `bee worktree new --with-companion` and do this work " +
    "there under a verified marker."
  );
}

function inferAgentName(payload, toolInput) {
  const fromPayload = getNestedString(payload, [
    "agent_name",
    "agentName",
    "agent_nickname",
    "subagent_type",
  ]);
  if (fromPayload) {
    return fromPayload;
  }
  const command = typeof toolInput.command === "string" ? toolInput.command : "";
  const match = command.match(/\bBEE_AGENT_NAME=(["']?)([^"'\s]+)\1/);
  if (match) {
    return match[2];
  }
  return process.env.BEE_AGENT_NAME || null;
}

// --- Codex apply_patch target extraction (canonical envelope) ---------------
// One target line per file operation:
//   *** Add File: <path> | *** Update File: <path> | *** Delete File: <path>
//   *** Move to: <path>   (destination of an Update File move)
const PATCH_TARGET_RE = /^\*\*\*\s+(?:Add File|Update File|Delete File|Move to):\s*(.+?)\s*$/;

function applyPatchText(toolInput) {
  // Canonical Codex shape is tool_input.input; tolerate the patch envelope
  // arriving under patch/command without forking per runtime.
  for (const key of ["input", "patch", "command"]) {
    const value = toolInput[key];
    if (typeof value === "string" && value.includes("*** Begin Patch")) {
      return value;
    }
  }
  return null;
}

function extractApplyPatchTargets(patchText) {
  const targets = [];
  for (const line of String(patchText).split(/\r?\n/)) {
    const match = PATCH_TARGET_RE.exec(line);
    if (match) {
      // Trim: the lazy `(.+?)\s*$` can otherwise capture a lone leftover
      // whitespace character for a verb line whose path is pure whitespace
      // (e.g. "*** Add File:    "). Trimming turns that into "", which
      // toRelPath's `!rawPath` check correctly treats as unprovable below —
      // a bug found while building this cell's matrix (auto-fixed per the
      // worker's rule-1 deviation policy: a bug in touched code).
      targets.push(match[1].trim());
    }
  }
  return targets;
}

// ─── check (d): CLI-shape validation (harness-integration D4, additive) ────
// Recognizes a Bash command shaped like `node .../bee.mjs cells show --id X`
// (the sole shipped CLI, decision bbc6bcea D1) and resolves it to a
// command-registry entry, validating its parsed flags against that entry's
// JSON-Schema `parameters` via validate-args.mjs. Unknown/unrecognized shapes
// are left alone (fail open) — that classification (nearest-match
// suggestions for a typo'd command) is the dispatcher's own job, not this
// guard's.
//
// LEGACY_HELPER_RE below (`bee_cells.mjs`-shaped names) is a TRANSITION
// GUARD, not a supported surface: shim-retire (decision bbc6bcea D1) deleted
// the 9 bee_*.mjs shims from templates and onboarding, but a host mid-upgrade
// can still have old vendored bins under .bee/bin/, and a session's shell
// history may still invoke shim names against them. This regex keeps those
// legacy command SHAPES resolving to the same registry entries so the guard
// doesn't silently stop validating them. Removal is future grooming debt
// (decision bbc6bcea D3) — once hosts have re-onboarded past this release,
// drop LEGACY_HELPER_RE and this comment along with it.

const LEGACY_HELPER_RE = /^bee_([a-z]+)\.mjs$/i;
const DISPATCHER_RE = /^bee\.mjs$/i;
const CLI_SEGMENT_SEPARATORS = new Set(["&&", "||", ";", "|", "&"]);

function splitCliSegments(tokens) {
  const segments = [];
  let current = [];
  for (const token of tokens) {
    if (CLI_SEGMENT_SEPARATORS.has(token)) {
      if (current.length > 0) segments.push(current);
      current = [];
    } else {
      current.push(token);
    }
  }
  if (current.length > 0) segments.push(current);
  return segments;
}

// Resolve (scriptBasename, positional-tokens-after-script) to a registry
// command name plus how many positional tokens it consumed. Longest-prefix
// match over `registry`'s own names — the SAME rule du-1 added to
// resolveCommand() in skills/bee-hive/templates/bee.mjs (that function is the
// source of truth; it is duplicated rather than imported here because this
// hook only ever dynamically imports repo-root `lib/*.mjs` modules via
// libModuleUrl, never bee.mjs itself). Without this, a 3-segment command
// (state.worker.add, reviews.candidate.add) collapsed onto the old hardcoded
// 2-token shape (e.g. "state.worker.add" -> guessed as "state.worker"),
// matched no registry entry, and silently skipped schema validation — a
// documented fail-open gap this closes (plan.md "Write-guard hook gap").
// Returns null when the shape is ambiguous (no verb token at all) or no
// prefix length matches any registry name — left to fail open, never guessed.
function resolveCliCommandName(scriptBasename, positionalTokens, registry) {
  const legacyMatch = scriptBasename.match(LEGACY_HELPER_RE);
  const isDispatcher = !legacyMatch && DISPATCHER_RE.test(scriptBasename);
  if (!legacyMatch && !isDispatcher) return null;

  const group = legacyMatch ? legacyMatch[1] : positionalTokens[0];
  if (legacyMatch && group === "status") {
    return { commandName: "status", consumed: 0 };
  }
  if (isDispatcher) {
    if (!group || group.startsWith("-")) return null;
    if (group === "status") {
      return { commandName: "status", consumed: 1 };
    }
  }

  // Collect the run of non-flag tokens after the group — the same "leading
  // tokens" shape bee.mjs's own splitCommandTokens/resolveCommand match
  // against, so a 3-segment name resolves identically here.
  const scanFrom = isDispatcher ? positionalTokens.slice(1) : positionalTokens;
  const verbTokens = [];
  for (const token of scanFrom) {
    if (token.startsWith("-")) break;
    verbTokens.push(token);
  }
  if (verbTokens.length === 0) return null; // no verb token at all: ambiguous, fail open

  const names = registry && Array.isArray(registry) ? new Set(registry.map((e) => e.name)) : null;
  if (!names) return null;

  const nameSegments = [group, ...verbTokens];
  for (let n = nameSegments.length; n >= 2; n -= 1) {
    const candidate = nameSegments.slice(0, n).join(".");
    if (names.has(candidate)) {
      // Legacy shape: positionalTokens holds ONLY verb tokens (the group came
      // from the script name), so consumed = n - 1 (excludes the group).
      // Dispatcher shape: positionalTokens[0] IS the group, so consumed = n.
      return { commandName: candidate, consumed: isDispatcher ? n : n - 1 };
    }
  }
  return null;
}

// Parse the remaining flag tokens into a { flagName: value } object, using
// the resolved registry entry's own parameter schema to decide whether a
// `--flag` is boolean (no value consumed) or value-taking (next token
// consumed) — the schema is the parsing contract, not a hardcoded flag list.
function parseCliFlags(flagTokens, propertiesSchema) {
  const parsed = {};
  for (let i = 0; i < flagTokens.length; i += 1) {
    const token = flagTokens[i];
    if (!token.startsWith("--")) continue;
    const eq = token.indexOf("=");
    if (eq !== -1) {
      parsed[token.slice(2, eq)] = token.slice(eq + 1);
      continue;
    }
    const name = token.slice(2);
    const propSchema = propertiesSchema && propertiesSchema[name];
    const next = flagTokens[i + 1];
    if (propSchema && propSchema.type === "boolean") {
      parsed[name] = true;
    } else if (next !== undefined) {
      // Consume the next token as the value unconditionally, even if it
      // starts with "--" — matching bee.mjs's parseFlags exactly (a value
      // legitimately starting with "--" must not be misread as a new flag).
      parsed[name] = next;
      i += 1;
    } else {
      parsed[name] = true;
    }
  }
  return parsed;
}

// Scan every shell segment of `command` for a recognizable bee-cli
// invocation and validate it against `registry` via `validateFn`. Returns
// `{ reason }` on the first structural mismatch found, else null. Never
// throws by construction (empty/malformed inputs just fail to match); the
// caller still wraps this in its own try/catch as a second line of defense.
function checkCliShape(command, registry, validateFn) {
  if (!command || !Array.isArray(registry)) return null;
  const segments = splitCliSegments(tokenizeCommand(command));
  for (const segment of segments) {
    for (let i = 0; i < segment.length; i += 1) {
      const base = segment[i].replace(/\\/g, "/").split("/").pop();
      if (!LEGACY_HELPER_RE.test(base) && !DISPATCHER_RE.test(base)) continue;
      const positional = segment.slice(i + 1);
      const resolved = resolveCliCommandName(base, positional, registry);
      if (!resolved) break; // ambiguous shape for this segment: fail open
      const entry = registry.find((candidate) => candidate.name === resolved.commandName);
      if (!entry) break; // unknown command name: dispatcher's concern, not this guard's
      const flagTokens = positional.slice(resolved.consumed);
      const parsedArgs = parseCliFlags(flagTokens, entry.parameters && entry.parameters.properties);
      const result = validateFn(entry, parsedArgs);
      if (result && result.ok === false) {
        const field = result.error && result.error.field;
        const reason = (result.error && result.error.reason) || "does not match the command's schema";
        // ce-1 (cli-ergonomics D1): when validateFn returns the batched
        // `problems` array (validate-args.mjs), render every problem joined
        // instead of just the first — the pinned substrings stay exactly
        // where they were: "bee CLI-shape guard" and entry.name in the
        // opening clause, "field: <first>" still naming result.error.field
        // (the FIRST problem) at the end, unchanged position.
        const problems = Array.isArray(result.problems) && result.problems.length > 0 ? result.problems : [{ field, reason }];
        const detail = problems.map((p) => `${p.reason}${p.field ? ` (--${p.field})` : ""}`).join("; ");
        return {
          reason:
            `bee CLI-shape guard: "${String(command).trim()}" ` +
            `does not match ${entry.name}'s schema — ${detail}${field ? ` (field: ${field})` : ""}. ` +
            `Correction: run \`${entry.invoke}\` with the required parameters (see \`${entry.invoke} --help --json\`).`,
        };
      }
      break; // this segment resolved to one bee-cli call; move to the next segment
    }
  }
  return null;
}

async function main() {
  const ctx = await readHookContext(HOOK_NAME);
  const root = ctx.root;
  if (!root) {
    return 0;
  }

  const payload = ctx.payload;
  const toolName = payload.tool_name || payload.toolName || "";
  const writeCapable =
    WRITE_TOOLS.has(toolName) || toolName === "Bash" || APPLY_PATCH_TOOLS.has(toolName);
  if (writeCapable && ctx.worktreeResolution === "linked-invalid") {
    process.stderr.write(
      "bee worktree guard denied this write: WORKTREE_LINK_INVALID — linked worktree metadata could not be validated. " +
        "FIX: repair or recreate the Git worktree before retrying; no worktree-local .bee store is trusted.",
    );
    return 2;
  }
  const storeRoot = ctx.storeRoot || root;
  if (!fs.existsSync(path.join(storeRoot, ".bee", "bin", "lib", "state.mjs"))) return 0;

  let denial = null; // { reason }
  let fixedAskVerdict = null; // { fixed, notes } — ask-guard-autofix D1/D2
  const reservationWarnings = []; // multisession-native-13 (D4): advisory-only intent-overlap notices
  try {
    const stateLib = await import(libModuleUrl(storeRoot, "state.mjs"));
    if (!stateLib.hookEnabled(storeRoot, HOOK_NAME)) {
      return 0;
    }
    const guards = await import(libModuleUrl(storeRoot, "guards.mjs"));

    const toolInput =
      payload.tool_input && typeof payload.tool_input === "object" ? payload.tool_input : {};
    const cwd = ctx.cwd;

    if (toolName === "AskUserQuestion") {
      // Pre-validate the AskUserQuestion schema so a violation surfaces as a
      // clear, specific message instead of the harness's opaque "Invalid tool
      // parameters" (which names neither the tool nor the bad field).
      const verdict = guards.checkAskUserQuestion
        ? guards.checkAskUserQuestion(toolInput)
        : { allow: true };
      if (verdict && verdict.allow === false) {
        denial = { reason: verdict.reason };
      } else if (verdict && verdict.fixed) {
        fixedAskVerdict = verdict;
      }
    } else if (READ_TOOLS.has(toolName)) {
      const rel = lexicalRelPath(root, cwd, toolInput.file_path || toolInput.path || "");
      if (rel) {
        const verdict = guards.checkRead(rel);
        if (verdict && verdict.allow === false) {
          const parts = [verdict.reason || `bee ${verdict.kind || "read"} guard denied: ${rel}`];
          if (verdict.marker) {
            parts.push(verdict.marker);
          }
          denial = { reason: parts.join("\n") };
        } else if (
          toolName === "Read" &&
          toolInput.offset === undefined &&
          toolInput.limit === undefined
        ) {
          // router-cost rc-1 (D1/D2/D3/D4): a Read with no offset/limit of a
          // big file is denied, naming both escapes. Never fires for Glob/
          // Grep (no whole-file content to load), and never fires when the
          // call already carries offset or limit (D4: a slice read is always
          // the correct, frictionless path).
          const config = stateLib.readConfig(storeRoot);
          const threshold = resolveMaxReadLines(config);
          const sizeReason = checkReadSizeDenial(path.join(root, rel), rel, threshold);
          if (sizeReason) {
            denial = { reason: sizeReason };
          }
        }
      }
    } else if (
      WRITE_TOOLS.has(toolName) ||
      toolName === "Bash" ||
      APPLY_PATCH_TOOLS.has(toolName)
    ) {
      const state = stateLib.readState(storeRoot);
      const agentName = inferAgentName(payload, toolInput);
      // fresh-session-handoff fsh-8 (D3/D4): thread the acting session into
      // guards.checkWrite so a cross-session hold (fsh-7) and lane-bound
      // gating (fsh-5) are enforced through the real production hook, not
      // just the lib. Absent/empty session_id is null here, which is
      // byte-identical to today's 4-arg checkWrite call (runtimes that never
      // send session_id see zero behavior difference).
      const sessionId =
        typeof payload.session_id === "string" && payload.session_id.trim()
          ? payload.session_id.trim()
          : null;
      let relPaths = [];
      // wcg-2 (D1b): raw absolute targets that resolved as physically inside
      // this checkout (canonicalRelPath) — the only ones that can be an
      // unverified shared nested checkout. A companion-marker-resolved target
      // is verified-covered and never a candidate.
      const sharedNestedCandidates = [];

      // gmr-3: the config behind `guards.memory_root` is read lazily and at
      // most ONCE per hook invocation — a repo that never declares a root
      // pays nothing here, and nothing at all until a target has already
      // failed the worktree wall. The ROOT itself (marker included) is
      // re-resolved per target inside isDeclaredMemoryRootTarget, which is
      // what D9's "re-checked on every hit" requires.
      let memoryRootConfig; // undefined = not read yet
      const isMemoryRootHit = (rawTarget) => {
        if (memoryRootConfig === undefined) {
          try {
            memoryRootConfig = stateLib.readConfig(storeRoot);
          } catch {
            memoryRootConfig = null;
          }
        }
        return isDeclaredMemoryRootTarget(root, cwd, memoryRootConfig, rawTarget);
      };

      if (APPLY_PATCH_TOOLS.has(toolName)) {
        // D2 / approach.md §2: an intercepted apply_patch runs the existing
        // gate/direct-edit/reservation decisions on every proved target.
        const patchText = applyPatchText(toolInput);
        if (patchText === null) {
          // Malformed OUTER payload: apply_patch fired but tool_input carries
          // no recognizable "*** Begin Patch" envelope at all — nothing was
          // genuinely intercepted, so this stays D2's visible fail-open.
          logCoverageGap(
            root,
            HOOK_NAME,
            "applypatch-unparsed",
            "apply_patch intercepted but no canonical patch envelope found in tool_input",
            ctx.source,
          );
        } else {
          const targets = extractApplyPatchTargets(patchText);
          relPaths = targets.map((p) => canonicalRelPath(root, cwd, p)).filter(Boolean);
          if (targets.length === 0 || relPaths.length < targets.length) {
            // P1 repair (codex-parity-4): the envelope WAS intercepted, but
            // the target set cannot be fully proved (no Add/Update/Delete/
            // Move line parsed at all, or a parsed target escapes the repo /
            // fails to resolve) — deny rather than risk an unchecked write.
            // Still logged as a visible coverage gap for audit (D2).
            logCoverageGap(
              root,
              HOOK_NAME,
              "applypatch-unparsed",
              targets.length === 0
                ? "apply_patch intercepted but no Add/Update/Delete/Move/\"Move to\" target line could be parsed from the patch body"
                : `apply_patch intercepted but ${targets.length - relPaths.length} of ${targets.length} target(s) could not be proved inside the repo`,
              ctx.source,
            );
            denial = {
              reason:
                "bee apply_patch guard: this patch's target set could not be fully proved inside the repo — " +
                "denying rather than risking an unchecked write. " +
                "FIX: use canonical \"*** Add File:\", \"*** Update File:\", \"*** Delete File:\", and \"*** Move to:\" " +
                "lines naming plain in-repo relative paths (no path traversal, no unresolvable escapes), then resubmit.",
            };
          }
        }
      } else if (toolName === "Bash") {
        const command = typeof toolInput.command === "string" ? toolInput.command : "";
        if (command) {
          const targets = guards.extractBashTargets(command);
          const paths = (targets && targets.paths) || [];
          const canonicalized = paths.map((p) => {
            const canonical = canonicalRelPath(root, cwd, p);
            const rel = canonical || resolveCompanionMountedRelPath(root, cwd, p);
            return {
              raw: p,
              canonical,
              rel,
              // gmr-3 (D6): consulted ONLY for a target the worktree wall has
              // already refused, so an in-worktree write can never be
              // re-routed through this branch and lose its gate/reservation
              // checks. A declared root can never contain the worktree (D5),
              // so the two sets are disjoint by construction anyway.
              memoryRoot: rel ? false : isMemoryRootHit(p),
            };
          });
          relPaths = canonicalized.filter((c) => c.rel).map((c) => c.rel);
          const memoryRootTargets = canonicalized.filter((c) => c.memoryRoot);
          for (const c of canonicalized) {
            if (c.canonical) {
              sharedNestedCandidates.push({ rel: c.canonical, abs: lexicalAbsTarget(root, cwd, c.raw) });
            }
          }
          // gmr-3 (D6): a memory-root target is PRE-APPROVED — it is kept out
          // of relPaths so it never reaches checkWrite, and it must equally
          // not trip the containment denial below, which counts targets.
          if (relPaths.length + memoryRootTargets.length !== paths.length) {
            const firstFailing = canonicalized.find((c) => !c.rel && !c.memoryRoot);
            const enriched = firstFailing ? describeCrossWorktreeTarget(root, cwd, firstFailing.raw) : null;
            denial = { reason: enriched || GENERIC_BASH_CONTAINMENT_MESSAGE };
          } else if (
            relPaths.length === 0 &&
            targets &&
            targets.broadWrite &&
            // gmr-3: do NOT fall through to the blanket "**" when the broad
            // signal is a memory-root target's own broad SPELLING (`rm -rf
            // <root>/*`) and every extracted target is a memory-root hit —
            // that command writes only inside the declared root, and "**"
            // would route it back into checkWrite, undoing the short-circuit.
            // When broadWrite instead came from a PATHLESS trigger (`git add
            // --all`, `git commit -a`, a bare `rm`) no memory target is
            // broad-shaped, so "**" still applies and that blanket write is
            // still fully checked — this narrows nothing but the case D6
            // exists for.
            !(
              memoryRootTargets.length === paths.length &&
              memoryRootTargets.some((c) => isBroadTargetSpelling(c.raw))
            )
          ) {
            relPaths = ["**"];
          }
        }
      } else {
        const rawTarget = toolInput.file_path || "";
        const canonicalRel = canonicalRelPath(root, cwd, rawTarget);
        const rel = canonicalRel || resolveCompanionMountedRelPath(root, cwd, rawTarget);
        if (rel) {
          relPaths = [rel];
          if (canonicalRel) {
            sharedNestedCandidates.push({ rel: canonicalRel, abs: lexicalAbsTarget(root, cwd, rawTarget) });
          }
        } else if (isMemoryRootHit(rawTarget)) {
          // gmr-3 (D6): a declared-memory-root hit is a real ALLOW that
          // short-circuits. relPaths stays empty, so this target never enters
          // checkWrite — no intake gate, no Gate 3, no reservations, no
          // holds. Deliberate: a learning must be recordable at phase `idle`.
        } else {
          const enriched = describeCrossWorktreeTarget(root, cwd, rawTarget);
          denial = { reason: enriched || GENERIC_CONTAINMENT_MESSAGE };
        }
      }

      // wcg-2 (worktree-concurrency-guard, D1b/D3/D5): a hard fail-closed
      // refusal, BEFORE checkWrite, of a write into a genuinely shared nested
      // checkout another live session can also reach and no verified companion
      // marker covers. Never consults gate_bypass (D5) — it lives in the hook
      // like every other guard here. isSharedNestedCheckoutTarget is a pure
      // no-op unless a second session is concurrently live (D6), and the
      // acting session is excluded so a solo session never trips it. Its own
      // typed, paved-road refusal (D4) is the message the human sees, so it
      // short-circuits checkWrite when it fires.
      let sharedNestedDenied = false;
      if (!denial) {
        for (const cand of sharedNestedCandidates) {
          // wcg-fix-2 (P1 review finding #2): scope a try/catch to THIS call
          // only. If the detection primitive throws, the write is DENIED with a
          // typed message — a detection error is not a "nothing shared here"
          // answer and must fail closed (plan.md Test Matrix). Without this the
          // throw would propagate to the hook's outer catch-all and return 0
          // (fail open). The catch is deliberately NOT the outer general one:
          // unrelated checks keep their established fail-open philosophy.
          let isShared;
          try {
            // Port-D5: reuse ctx.controlRoot exactly as the checkWrite call
            // below does — no new topology resolution.
            isShared = guards.isSharedNestedCheckoutTarget(root, cand.abs, {
              excludeSessionId: sessionId,
              controlRoot: ctx.controlRoot,
            });
          } catch (sharedNestedError) {
            logCrash(root, HOOK_NAME, sharedNestedError, ctx.source);
            denial = { reason: sharedNestedDetectionErrorRefusal(cand.rel) };
            sharedNestedDenied = true;
            break;
          }
          if (isShared) {
            denial = { reason: sharedNestedCheckoutRefusal(cand.rel) };
            sharedNestedDenied = true;
            break;
          }
        }
      }

      // Preserve the established diagnostic precedence when a mixed request
      // contains both an unprovable target and a proved policy-denied target:
      // the whole request is denied either way, and the concrete policy
      // reason (for example direct-edit) remains the user-facing correction.
      if (!sharedNestedDenied) {
        for (const rel of relPaths) {
          const verdict = guards.checkWrite(storeRoot, state, rel, agentName, {
            sessionId,
            // msn-21: the adapter already resolved topology once
            // (readHookContext's own resolveRoots walk, exposed as
            // ctx.controlRoot since d69d81e) — pass it through so
            // guards.checkWrite's own resolveWriteTopology reuses it instead
            // of re-deriving controlRoot from scratch a second time.
            controlRoot: ctx.controlRoot,
          });
          if (verdict && verdict.allow === false) {
            denial = {
              reason:
                verdict.reason || `bee ${verdict.kind || "write"} guard denied write to: ${rel}`,
            };
            break;
          }
          // multisession-native-13 (D4): an ALLOWED write can still carry a
          // non-blocking `warning` (a declared 'intent' reservation whose
          // broad/glob scope covers this path — advisory only, never a deny).
          // Collected across every relPath, never breaks the loop — a warning
          // is not a denial and must never suppress a LATER path's real deny.
          if (verdict && verdict.allow === true && verdict.warning) {
            reservationWarnings.push(verdict.warning);
          }
        }
      }

      // Intake-gate git exemption (D1/D3/D4, cell ige-2, closes P46 / GH #1)
      // AND the concurrent-worker whole-tree denial (gc-2): additive and
      // scoped to Bash only. `guards.checkGitBashCommand` returns null unless
      // the command contains a recognizable `git` invocation AND either the
      // phase is terminal (the intake-gate branch) or more than one worker is
      // live in this checkout (the gc-2 branch, which is phase-independent
      // BECAUSE the phase it exists for — swarming — is never terminal).
      // Everywhere else this is a no-op, so it can never override or discard
      // a denial checks (a)-(c) above (or the reservation/gate loop just run)
      // already computed.
      if (!denial && toolName === "Bash" && typeof guards.checkGitBashCommand === "function") {
        const bashCommand = typeof toolInput.command === "string" ? toolInput.command : "";
        if (bashCommand) {
          const gitVerdict = guards.checkGitBashCommand(storeRoot, state, bashCommand, {
            cwd,
            sessionId,
            controlRoot: ctx.controlRoot, // msn-21: reuse the adapter's own topology resolution
          });
          if (gitVerdict && gitVerdict.allow === false) {
            denial = {
              reason: gitVerdict.reason || `bee ${gitVerdict.kind || "git"} guard denied: ${bashCommand}`,
            };
          }
        }
      }

      // Internals-reach guard (state-query-surface, cell sqs-a, D 3fbe2f79):
      // deny an inline-eval `node -e`/`--eval`/`-p` Bash command whose script
      // text imports/requires a `bin/lib/` or `templates/lib/` module — never
      // a file-based `node <path>.mjs` run (tests import lib legitimately
      // that way). Additive, scoped to Bash, and can only ever ASSIGN a
      // denial when none exists yet, same first-hit-wins precedence as every
      // other check above.
      if (!denial && toolName === "Bash" && typeof guards.checkBinLibImportBashCommand === "function") {
        const bashCommand = typeof toolInput.command === "string" ? toolInput.command : "";
        if (bashCommand) {
          const internalsVerdict = guards.checkBinLibImportBashCommand(bashCommand);
          if (internalsVerdict && internalsVerdict.allow === false) {
            denial = {
              reason: internalsVerdict.reason || `bee internals-reach guard denied: ${bashCommand}`,
            };
          }
        }
      }
    }

    // Check (d) — CLI-shape validation (additive, D4). Runs unconditionally
    // for Bash calls (appended after checks (a)-(c), never gating on them),
    // but can only ever ASSIGN a denial when none exists yet (`!denial` right
    // before the write — first hit wins, matching this file's documented
    // semantics) — so it can never overwrite or discard a denial checks
    // (a)-(c) already computed. Its try/catch is intentionally separate from
    // the outer one below: a bug in the Bash-parsing logic here must fail
    // open for THIS check only, never propagate to the shared catch (which
    // would discard any denial already set by checks (a)-(c) and fail open
    // for all four checks at once).
    if (toolName === "Bash") {
      const command = typeof toolInput.command === "string" ? toolInput.command : "";
      if (command) {
        try {
          const validateLib = await import(libModuleUrl(storeRoot, "validate-args.mjs"));
          const registryLib = await import(libModuleUrl(storeRoot, "command-registry.mjs"));
          const cliDenial = checkCliShape(command, registryLib.COMMAND_REGISTRY, validateLib.validate);
          if (cliDenial && !denial) {
            denial = cliDenial;
          }
        } catch (cliError) {
          logCrash(root, HOOK_NAME, cliError, ctx.source);
        }
      }
    }
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    return 0;
  }

  if (fixedAskVerdict) {
    // ask-guard-autofix D1/D2: an AskUserQuestion call with ONLY fixable
    // violations (an over-long header) is allowed to proceed with the
    // rewritten input. Emit the PreToolUse updatedInput contract on stdout
    // (D2, confirmed against the Claude Code hooks doc) and exit 0 — this
    // never runs when `denial` is also set (checkAskUserQuestion returns
    // either a deny OR a fix, never both; deny wins).
    const notes = Array.isArray(fixedAskVerdict.notes) ? fixedAskVerdict.notes : [];
    const notesJoined = notes.join("; ");
    const output = {
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "allow",
        permissionDecisionReason: notesJoined,
        updatedInput: fixedAskVerdict.fixed,
        additionalContext: `bee AskUserQuestion guard auto-fixed: ${notesJoined}`,
      },
      systemMessage: `bee AskUserQuestion guard: ${notesJoined}`,
    };
    process.stdout.write(JSON.stringify(output));
    return 0;
  }

  if (!denial && reservationWarnings.length > 0) {
    // multisession-native-13 (D4): a declared-'intent' reservation covered
    // this write's path but never blocked it (prohibition: "no hard deny
    // from an intent record") — surface the advisory as a non-blocking
    // systemMessage, same allow+notice shape as the ask-guard-autofix path
    // above, so the agent sees the overlap without the write being denied.
    const joined = reservationWarnings.join("\n");
    const output = {
      hookSpecificOutput: {
        hookEventName: "PreToolUse",
        permissionDecision: "allow",
        permissionDecisionReason: joined,
      },
      systemMessage: joined,
    };
    process.stdout.write(JSON.stringify(output));
    return 0;
  }

  if (denial) {
    // Deliberate deny: exit 2 with the reason on stderr (Claude Code feeds
    // stderr back to the model on PreToolUse exit 2; Codex blocks supported
    // PreToolUse paths the same way). A log-write failure can never cancel
    // this deny — logging is fail-open inside the adapter.
    process.stderr.write(denial.reason);
    return 2;
  }
  return 0;
}

process.exitCode = await main();
