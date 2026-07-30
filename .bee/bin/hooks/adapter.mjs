// hooks/adapter.mjs — the one shared runtime adapter used by every bee
// wrapper hook (cell codex-parity-3; CONTEXT.md decision D2: "Codex receives
// full hook parity on every compatible event and tool path ... unsupported
// paths fail open with visible limits and runtime-specific tests";
// approach.md section 2 "Shared hook logic, exact host adapters").
//
// Responsibilities, in order:
//   1. stdin normalization — arbitrary stdin (empty, junk bytes, top-level
//      null, arrays, non-string cwd, missing cwd, multi-MB payloads) is
//      normalized to a plain object BEFORE any property access. No wrapper
//      ever touches `payload.<anything>` on an un-normalized value again.
//   2. root discovery inside the fail-open boundary — findRepoRoot is called
//      inside readHookContext's own try/catch, so no discovery throw can
//      escape into a wrapper crash.
//   3. per-host/event output encoding — Codex requires non-empty stdout on
//      SubagentStop/Stop to be JSON and ignores plain stdout on PreCompact
//      (discovery.md "Current Codex Contracts"); Claude Code accepts the same
//      JSON hook-output shape. Advisories on those events are therefore
//      encoded as a parseable JSON `systemMessage` — NEVER `decision:"block"`,
//      which on SubagentStop continues the child and on Stop loops the main
//      turn (D2: an advisory must stay advisory). SessionStart and
//      UserPromptSubmit keep plain stdout: both hosts consume it as developer
//      context.
//   4. crash + coverage-gap logging that NEVER changes the allow/deny
//      result — every log write is wrapped; a throwing import, a log-write
//      failure, or malformed input can never flip an allow into a deny or a
//      deny into an allow. Fail-open means fail-open, visibly logged in
//      .bee/logs/hooks.jsonl.
//
// Source identity (approach.md section 1): a hook catalog command may pass
// `--source plugin|repo` explicitly. The adapter parses and threads it into
// every crash/coverage-gap log line so source arbitration (a later
// Distribution-slice cell) can audit which installed source produced an
// event. An unknown value is recorded as a visible `invalid-source` coverage
// gap and treated as null — never a behavior change.
//
// Coverage-gap classes (each asserted by a dedicated harness row in
// hooks/test_hook_contracts.mjs):
//   - malformed-payload  : stdin was non-empty but not a JSON object
//   - invalid-cwd        : payload.cwd present but not a usable string
//   - invalid-source     : --source value is not plugin|repo
//   - applypatch-unparsed: (logged by bee-write-guard) an intercepted
//                          apply_patch whose targets could not all be proved
//   - unsupported-subagent-event: (logged by bee-codex-subagent-audit) input
//                          did not name SubagentStart or SubagentStop

import fs from "node:fs";
import nodeModule from "node:module";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

// es-3 (exec-speed D1): guarded on-disk compile cache, enabled at ADAPTER
// module top on purpose — ESM hoisting means a module can never cache its own
// static import graph, but every wrapper hook imports this adapter statically
// FIRST and only then dynamically imports the heavy libs (state.mjs,
// guards.mjs, cells.mjs, ...), so a top-level call here covers exactly those
// loads. Node <22.1 has no enableCompileCache; any error (read-only tmpdir,
// exotic host) is swallowed — a cache is a speedup, never a dependency.
try {
  if (typeof nodeModule.enableCompileCache === "function") {
    nodeModule.enableCompileCache(path.join(os.tmpdir(), "bee-compile-cache"));
  }
} catch {
  // fail-open: run uncached
}

// Events whose non-empty stdout must be a parseable JSON systemMessage.
export const ADVISORY_EVENTS = Object.freeze(["PreCompact", "SubagentStop", "Stop"]);

const SOURCE_IDENTITIES = new Set(["plugin", "repo"]);
const DETAIL_MAX = 300;

function truncateDetail(text) {
  const s = text == null ? "" : String(text);
  return s.length <= DETAIL_MAX ? s : `${s.slice(0, DETAIL_MAX)}...`;
}

// --- source identity -------------------------------------------------------

// Returns { source: "plugin"|"repo"|null, invalid?: string }. Never throws.
export function parseSourceIdentity(argv = process.argv) {
  try {
    for (let i = 0; i < argv.length; i += 1) {
      const arg = argv[i];
      if (typeof arg !== "string") continue;
      let value = null;
      if (arg.startsWith("--source=")) {
        value = arg.slice("--source=".length).trim();
      } else if (arg === "--source") {
        value = typeof argv[i + 1] === "string" ? argv[i + 1].trim() : "";
      } else {
        continue;
      }
      if (SOURCE_IDENTITIES.has(value)) {
        return { source: value };
      }
      return { source: null, invalid: value || "<missing>" };
    }
  } catch {
    // fail-open: source identity is audit metadata, never a decision input
  }
  return { source: null };
}

// --- root discovery --------------------------------------------------------

function realpathOrNull(value) {
  try {
    return fs.realpathSync.native(value);
  } catch {
    return null;
  }
}

function readGitdirFile(file, base) {
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

function locateGitRoot(startDir) {
  let candidate = path.resolve(typeof startDir === "string" && startDir ? startDir : process.cwd());
  while (true) {
    const marker = path.join(candidate, ".git");
    if (fs.existsSync(marker)) return { workRoot: candidate, marker };
    const parent = path.dirname(candidate);
    if (parent === candidate) return null;
    candidate = parent;
  }
}

function locateOnboardedRoot(startDir) {
  let candidate = path.resolve(typeof startDir === "string" && startDir ? startDir : process.cwd());
  while (true) {
    if (fs.existsSync(path.join(candidate, ".bee", "onboarding.json"))) return candidate;
    const parent = path.dirname(candidate);
    if (parent === candidate) return null;
    candidate = parent;
  }
}

// Non-throwing hook-side twin of state.mjs resolveRoots. `root` remains the
// physical checkout for every existing hook consumer; only write-guard uses
// storeRoot for coordination state and authorization. `mainRoot` (present
// only on the "linked-valid" branch below) is what controlRootFor() reads
// to resolve the multisession-native-18c control root — see that function's
// own comment.
export function resolveRoots(startDir) {
  try {
    const onboarded = locateOnboardedRoot(startDir);
    if (onboarded && !fs.existsSync(path.join(onboarded, ".git"))) {
      const root = realpathOrNull(onboarded);
      return { storeRoot: root, workRoot: root, worktreeResolution: "ordinary" };
    }
    const located = locateGitRoot(startDir);
    if (!located) {
      const root = onboarded && realpathOrNull(onboarded);
      return { storeRoot: root, workRoot: root, worktreeResolution: "ordinary" };
    }

    const workRoot = realpathOrNull(located.workRoot);
    if (!workRoot) {
      return { storeRoot: null, workRoot: null, worktreeResolution: "ordinary" };
    }
    let markerStat;
    try {
      markerStat = fs.statSync(located.marker);
    } catch {
      return { storeRoot: null, workRoot, worktreeResolution: "linked-invalid" };
    }
    if (!markerStat.isFile()) {
      return { storeRoot: workRoot, workRoot, worktreeResolution: "ordinary" };
    }

    const gitdir = readGitdirFile(located.marker, located.workRoot);
    if (!gitdir) {
      return { storeRoot: null, workRoot, worktreeResolution: "linked-invalid" };
    }
    const worktreesRoot = path.resolve(gitdir, "..");
    const commonGitDir = path.resolve(worktreesRoot, "..");
    const linkedShape = path.basename(worktreesRoot) === "worktrees" && path.basename(commonGitDir) === ".git";
    // A legitimate `git init --separate-git-dir` checkout also has a .git
    // file, but not the .git/worktrees/<id> relationship. It stays ordinary.
    if (!linkedShape) {
      return { storeRoot: workRoot, workRoot, worktreeResolution: "ordinary" };
    }

    const reverse = readGitdirFile(path.join(gitdir, "gitdir"), gitdir);
    const markerReal = path.resolve(located.marker);
    if (!reverse || path.resolve(reverse) !== markerReal) {
      return { storeRoot: null, workRoot, worktreeResolution: "linked-invalid" };
    }
    const mainRoot = realpathOrNull(path.dirname(commonGitDir));
    if (!mainRoot) {
      return { storeRoot: null, workRoot, worktreeResolution: "linked-invalid" };
    }
    // Opt-in per-worktree store (worktree-feature-parallelism): read the grant
    // INLINE (never via a lib import) to keep this adapter import-light and
    // fail-open. A worktree whose git-verified id is registered in the MAIN
    // store resolves to its own local store; unregistered => main (P40 default,
    // byte-for-byte). Read only from the main store; any error => main default.
    const id = path.basename(gitdir);
    let storeRoot = mainRoot;
    try {
      const grants = JSON.parse(
        fs.readFileSync(path.join(mainRoot, ".bee", "runtime", "worktree-grants.json"), "utf8"),
      );
      if (grants && grants[id] === true) storeRoot = workRoot;
    } catch {
      // no/invalid grants registry => main default (fail-open)
    }
    return { storeRoot, workRoot, worktreeResolution: "linked-valid", id, mainRoot, worktreeRoot: workRoot };
  } catch {
    return { storeRoot: null, workRoot: null, worktreeResolution: "ordinary" };
  }
}

export function findRepoRoot(startDir) {
  return resolveRoots(startDir).workRoot;
}

// controlRootFor(root) — hooks/adapter.mjs's own minimal replica of
// state.mjs's controlRootFor(root) (multisession-native-18a), needed here
// because this adapter is deliberately import-light and fail-open (hook
// posture: no lib/state.mjs import at hook-execution time — see this file's
// header, responsibility 2, and lib/reservations.mjs's own cycle-safe
// replica for the precedent this follows). `root` here is expected to
// already be a resolved, physical checkout path (typically ctx.root from
// readHookContext below) — re-walking it through resolveRoots is the SAME
// cheap, sync, try-once git-worktree walk-up every hook already pays for
// once per invocation, so calling it a second time here stays inside hook
// posture (no unbounded work, no retries).
//
// Sessions/workflows/claims/the handoff mailbox are control-plane
// (multisession-native CONTEXT.md D2): a hook running inside a linked
// worktree must resolve those stores to MAIN's `.bee/`, exactly like
// resolveRoots' own `storeRoot` already does for grant-gated cells/backlog
// state — but controlRoot is grant-INDEPENDENT (always main for a linked
// worktree, regardless of whether that worktree also holds its own granted
// local store for workspace-local state). Fail-open like every other
// resolution in this file: an unresolvable/invalid topology (linked-invalid,
// or `root` not even a git checkout) falls back to `root` itself — never
// null, never a throw — so a caller never has to special-case the result
// before using it as a plain path.
export function controlRootFor(root) {
  if (!root) return root;
  const roots = resolveRoots(root);
  return roots.mainRoot || roots.workRoot || root;
}

export function libModuleUrl(root, name) {
  return pathToFileURL(path.join(root, ".bee", "bin", "lib", name)).href;
}

// --- lite config reads (es-3, exec-speed D3) --------------------------------
// A hook that only needs one scalar from config must not pay the state.mjs
// (195KB) import. These readers mirror state.mjs readConfig's overlay
// precedence for a single `<namespace>.<key>` lookup, byte-for-byte in
// outcome for the scalar namespaces they are used on (hooks.*, guards.*):
//   - .bee/config.local.json overlays .bee/config.json; when the overlay's
//     namespace is a plain object, its own keys win and the tracked value
//     survives for keys the overlay does not define (mergeConfigOverlay's
//     deep-merge on two plain objects);
//   - an overlay namespace that is present but NOT a plain object replaces
//     the tracked namespace wholesale (mergeConfigOverlay's array/scalar
//     branch), so the lookup yields undefined — same as readConfig, where the
//     clobbered namespace no longer carries the key;
//   - a missing or malformed file reads as absent (state.mjs readJson).
// Never throws; any error reads as "value undefined".

function readJsonObjectLite(file) {
  try {
    const parsed = JSON.parse(fs.readFileSync(file, "utf8"));
    return parsed && typeof parsed === "object" && !Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

export function liteConfigValue(root, namespace, key) {
  try {
    const overlay = readJsonObjectLite(path.join(root, ".bee", "config.local.json"));
    const tracked = readJsonObjectLite(path.join(root, ".bee", "config.json"));
    const trackedNs = tracked && isPlainObject(tracked[namespace]) ? tracked[namespace] : null;
    if (overlay && Object.prototype.hasOwnProperty.call(overlay, namespace)) {
      const overlayNs = overlay[namespace];
      if (!isPlainObject(overlayNs)) {
        // Non-object overlay value replaces the namespace wholesale.
        return undefined;
      }
      if (Object.prototype.hasOwnProperty.call(overlayNs, key)) {
        return overlayNs[key];
      }
      return trackedNs ? trackedNs[key] : undefined;
    }
    return trackedNs ? trackedNs[key] : undefined;
  } catch {
    return undefined;
  }
}

// Lite twin of state.mjs hookEnabled(root, name): every DEFAULT_HOOKS value is
// `true` and hookEnabled tests `!== false`, so "no explicit false anywhere
// after overlay precedence" IS the whole contract. ANY error => enabled
// (fail-open: a broken config must never silently disable a hook the full
// reader would run).
export function hookEnabledLite(root, name) {
  try {
    return liteConfigValue(root, "hooks", name) !== false;
  } catch {
    return true;
  }
}

// --- fail-open logging -----------------------------------------------------

// Low-level append to .bee/logs/hooks.jsonl. A log failure is swallowed:
// logging never changes a hook's decision or exit code.
export function appendHookLog(root, entry) {
  try {
    const logsDir = path.join(root, ".bee", "logs");
    fs.mkdirSync(logsDir, { recursive: true });
    fs.appendFileSync(path.join(logsDir, "hooks.jsonl"), `${JSON.stringify(entry)}\n`);
  } catch {
    // fail-open: never break (or flip) a hook over logging
  }
}

export function logCrash(root, hookName, error, source = null) {
  if (!root) return;
  appendHookLog(root, {
    ts: new Date().toISOString(),
    hook: hookName,
    ...(source ? { source } : {}),
    error: String((error && error.stack) || error),
  });
}

// A coverage gap is a host path the hook could not fully cover (malformed
// payload, unprovable patch target, ...). It is logged VISIBLY and the hook
// proceeds fail-open — the gap line never alters the allow/deny result (D2).
export function logCoverageGap(root, hookName, gap, detail, source = null) {
  if (!root) return;
  appendHookLog(root, {
    ts: new Date().toISOString(),
    hook: hookName,
    event: "coverage-gap",
    gap,
    detail: truncateDetail(detail),
    ...(source ? { source } : {}),
  });
}

// --- stdin normalization + context -----------------------------------------

async function readRawStdin() {
  const chunks = [];
  try {
    for await (const chunk of process.stdin) {
      chunks.push(chunk);
    }
  } catch {
    return "";
  }
  return Buffer.concat(chunks).toString("utf8");
}

// The one entry point every wrapper calls first. NEVER throws. Returns:
//   payload : plain object (malformed stdin normalized to {})
//   cwd     : usable string cwd (payload.cwd when valid, else process.cwd())
//   root    : bee repo root or null
//   source  : "plugin" | "repo" | null (explicit --source identity)
//   event   : payload.hook_event_name when a string, else ""
//   gaps    : coverage gaps found during normalization (already logged when
//             a root was found)
export async function readHookContext(hookName, { argv = process.argv } = {}) {
  // Mark hook context ONCE, before any lib (state.mjs et al) is dynamically
  // imported by the caller. Libs (e.g. normalizeDogfoodRepos) read this flag
  // to suppress console.warn spam that would otherwise prefix every hook's
  // stdout/stderr with unrelated dogfood_repos noise (ag-2); the underlying
  // behavior — a dead/unreadable entry is skipped — is unchanged. Plain CLI
  // runs (status, onboarding — no adapter) never set this and keep the
  // warning verbatim.
  process.env.BEE_HOOK_CONTEXT = "1";

  const gaps = [];

  const raw = await readRawStdin();
  let payload = {};
  if (raw.trim()) {
    try {
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        payload = parsed;
      } else {
        gaps.push({
          gap: "malformed-payload",
          detail: `top-level ${parsed === null ? "null" : Array.isArray(parsed) ? "array" : typeof parsed} payload — normalized to {}`,
        });
      }
    } catch {
      gaps.push({ gap: "malformed-payload", detail: "stdin is not parseable JSON — normalized to {}" });
    }
  }

  let cwd = process.cwd();
  if (typeof payload.cwd === "string" && payload.cwd.trim()) {
    cwd = payload.cwd;
  } else if (payload.cwd !== undefined) {
    gaps.push({
      gap: "invalid-cwd",
      detail: `payload.cwd is ${Array.isArray(payload.cwd) ? "an array" : typeof payload.cwd}, not a usable string — fell back to process.cwd()`,
    });
  }

  const parsedSource = parseSourceIdentity(argv);
  if (parsedSource.invalid) {
    gaps.push({
      gap: "invalid-source",
      detail: `--source "${parsedSource.invalid}" is not plugin|repo — recorded as unknown`,
    });
  }

  const roots = resolveRoots(cwd);
  const root = roots.workRoot;
  // msn-18c: reuse THIS call's own resolveRoots walk (roots.mainRoot is
  // already set on the "linked-valid" branch) instead of paying for a
  // second one via controlRootFor(root) — same result, half the fs work per
  // hook invocation. Fail-open like `root` itself: null when unresolvable.
  const controlRoot = roots.mainRoot || root || null;

  if (root) {
    for (const g of gaps) {
      logCoverageGap(root, hookName, g.gap, g.detail, parsedSource.source);
    }
  }

  const event = typeof payload.hook_event_name === "string" ? payload.hook_event_name : "";
  return {
    payload,
    cwd,
    root,
    storeRoot: roots.storeRoot,
    // msn-18c: session/workflow/claims/handoff-mailbox stores are
    // control-plane — a hook consumer that touches those (heartbeat touch,
    // session reads) should resolve against `ctx.controlRoot`, never
    // `ctx.root`/`ctx.storeRoot` (storeRoot is grant-topology, orthogonal to
    // the control plane). Wiring the individual hook consumers
    // (bee-session-init.mjs, bee-prompt-context.mjs, bee-state-sync.mjs) is
    // out of this cell's file scope — this field/helper is the primitive a
    // later cell wires them to.
    controlRoot,
    worktreeResolution: roots.worktreeResolution,
    source: parsedSource.source,
    event,
    gaps,
  };
}

// --- output encoding -------------------------------------------------------

export function isAdvisoryEvent(event) {
  return ADVISORY_EVENTS.includes(event);
}

// Encode one advisory as the parseable JSON shape both hosts accept. NEVER
// emits decision:"block" — on Codex that would continue a child
// (SubagentStop) or loop the main turn (Stop) instead of advising (D2,
// discovery.md "Current Codex Contracts").
export function encodeAdvisory(text) {
  return JSON.stringify({ systemMessage: String(text) });
}

// Encode a Stop-event BLOCK. This is the deliberate inverse of encodeAdvisory:
// `decision:"block"` on a Stop event CONTINUES the turn (Claude blocks the stop
// and feeds `reason` back to the model; Codex loops the main turn) — exactly the
// behavior the gate-bypass net wants (GitHub #18). Callers must restrict this to
// ctx.event==="Stop" ONLY: on SubagentStop it would continue a child, and on
// PreCompact it is meaningless. Never use it for an advisory.
export function encodeBlock(reason) {
  return JSON.stringify({ decision: "block", reason: String(reason) });
}

// Write hook output encoded for the event: advisory events (PreCompact,
// SubagentStop, Stop) get JSON systemMessage; context events (SessionStart,
// UserPromptSubmit) stay plain stdout. `defaultEvent` covers hosts/payloads
// that omit hook_event_name: pass the event the wrapper is wired to.
export function emitHookOutput(ctx, text, { defaultEvent = "" } = {}) {
  const message = String(text);
  if (!message.trim()) return;
  const event = (ctx && ctx.event) || defaultEvent;
  if (isAdvisoryEvent(event)) {
    process.stdout.write(encodeAdvisory(message));
  } else {
    process.stdout.write(message);
  }
}
