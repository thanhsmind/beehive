#!/usr/bin/env node
// onboard_bee.mjs - install/update bee in a target repo.
//
//   node onboard_bee.mjs --repo-root <path> [--apply] [--json] [--repo-hooks]
//                        [--no-claude-md] [--claude-md] [--global-skills]
//                        [--force-downgrade]
//
// Plan mode (default) reports {status: 'up_to_date'|'changes_needed'|
// 'blocked_downgrade'|'blocked_no_source', plan:[...]}.
// --apply applies the plan and writes .bee/onboarding.json with managed versions.
// CLAUDE.md is a default onboarding artifact (D1): every apply writes/extends
// CLAUDE.md with the @AGENTS.md import unless --no-claude-md is passed.
// --claude-md remains accepted as a no-op alias of the default.
// Every apply also mirrors the bee-* skill set into the HOST REPO's own skill
// roots (installer-hardening D2/D6): <repo>/.claude/skills (Claude Code) and
// <repo>/.agents/skills (Codex), committed to the host repo (D4 - never
// gitignored). --global-skills additionally targets the legacy global
// ~/.claude/skills root (D3) as a fully managed target (creation + deletion).
// WITHOUT the flag the global root is never created into or deleted from, but a
// best-effort version-parity pass (installer-version-parity-1-3-1) refreshes IN
// PLACE every managed skill whose directory ALREADY EXISTS there to current
// source content (action refresh_legacy_global_skill) - so a pre-1.0 global
// install can no longer keep loading a stale bee version alongside the
// per-project copy. That pass never creates an absent skill, never deletes, and
// never blocks the primary repo sync (see computeLegacyGlobalRefresh). Per
// target (D1-D5): drift shows up as
// sync_skill/remove_skill plan items, an older source refuses with zero
// mutations (--force-downgrade overrides only a fully-resolved version
// refusal), and non-bee skills are structurally untouchable. When the repo
// being onboarded contains the running script's own skill tree, its discoverable
// per-project projections are refreshed like every other managed target.
// --repo-hooks additionally vendors the plugin hooks into <repo>/.bee/bin/hooks/
// and merges the hook entries into <repo>/.claude/settings.json (with a .bak
// backup) for environments that do not load plugin hooks.
// The opt-in is STICKY: once a repo records repo_hooks in its onboarding marker,
// every later run vendors hooks whether or not the flag is passed. The flag opts
// a repo in; it is not a re-consent owed on each upgrade. (Before this, a bare
// --apply refreshed doctrine, helpers, and the version stamp while leaving
// first-onboard guards in place — and still reported up_to_date.)
//
// --runtime claude|codex|both (default both) names which runtime(s) this
// onboarding invocation covers. Combined with --plugin-source it drives the
// codex-hybrid path (GH #22 P0-1): codex-cli's plugin manifest packages
// skills only — there is no codex plugin-hook mechanism (capability matrix
// row B1) — so `--plugin-source --runtime codex` (or `both`) ALWAYS also
// vendors .bee/bin/hooks/ and merges .codex/hooks.json, exactly like
// --repo-hooks's codex projection, so a plugin-first-onboarded repo never
// reports itself onboarded while carrying zero mechanical enforcement for
// Codex sessions. This gate reads the PASSED --runtime only, never
// hasRepoHooksRecorded or any other recorded state (an old plugin-source
// install carries no runtime record to infer coverage from). --runtime
// claude leaves --plugin-source's existing behavior byte-identical (no
// repo-local Claude hooks, no Codex files either) — plugin-first for Claude
// relies on the plugin's own hooks, which codex-cli does not have.
//
// Never overwrites existing .bee/state.json, .bee/decisions.jsonl, or .bee/cells/.

import { execFileSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { detectCommands } from "../../../packages/bee/lib/commands_detect.mjs";
import { hashFile } from "../../../packages/bee/lib/fsutil.mjs";
import { classifySource } from "../../../packages/bee/lib/source-identity.mjs";
// msn-18d deliberately does NOT `import { resolveContext } from
// "../../../packages/bee/lib/state.mjs"` here, even though that is the real,
// canonical resolver every leaf coordination module uses (advisor-digest-
// slice4 binding condition 5). This file's OWN test suite (test_onboard_bee.
// mjs's makeFakeSkillsRoot) ships a deliberately minimal fake state.mjs
// (BEE_VERSION + COMMAND_KEYS only, to pin a controlled version for
// skill-sync tests) at the fixture's packages/bee/lib/state.mjs — a static
// `import { resolveContext } from ...` against a module that does not
// export that name fails the WHOLE ESM module load, uncatchable by any
// try/catch in this file, before a single line of onboard_bee.mjs runs.
// reservations.mjs carries the exact same kind of minimal, cycle-safe
// replica for its own reasons (see its own module header) — the same
// precedent applies here. See resolveWorktreeContextForMigration below.

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const SCRIPTS_DIR = path.dirname(SCRIPT_PATH);
const HIVE_DIR = path.dirname(SCRIPTS_DIR);
const PLUGIN_ROOT = path.dirname(path.dirname(HIVE_DIR));
const PLUGIN_HOOKS_DIR = path.join(PLUGIN_ROOT, "packages", "bee", "hooks");
const TEMPLATES_DIR = path.join(PLUGIN_ROOT, "packages", "bee");
const TEMPLATES_LIB_DIR = path.join(TEMPLATES_DIR, "lib");
const TEMPLATES_STATUSLINE_DIR = path.join(TEMPLATES_DIR, "statusline");
const TEMPLATES_AGENTS_DIR = path.join(TEMPLATES_DIR, "agents");
const AGENTS_BLOCK_TEMPLATE = path.join(TEMPLATES_DIR, "AGENTS.block.md");

const ONBOARDING_SCHEMA_VERSION = "1.0";
const MIN_NODE_MAJOR = 18;
const MARKER_START = "<!-- BEE:START -->";
const MARKER_END = "<!-- BEE:END -->";
// D1 (decision 26203bd3): the .gitignore managed block uses '#'-comment
// markers (gitignore syntax) - an HTML comment would be parsed as a literal
// ignore pattern, not a comment, and would never match anything.
const GITIGNORE_MARKER_START = "# BEE:START";
const GITIGNORE_MARKER_END = "# BEE:END";
// Review P2 (test-coverage) / P3 (security anchor): whole-line anchored, not
// bare substring - a user comment like "# BEE:START custom notes" must never
// be adopted as the managed block. `[ \t]*\r?$` allows only trailing
// horizontal whitespace and an optional CRLF `\r` before end-of-line; it
// deliberately does NOT use `\s*$` (which is greedy across newlines too and
// would swallow the user's blank lines/footer after the marker).
const GITIGNORE_START_RE = /^# BEE:START[ \t]*\r?$/m;
const GITIGNORE_END_RE = /^# BEE:END[ \t]*\r?$/m;
// Machine-local .bee runtime churn only (D1) - team-durable paths (bin/,
// config.json, config-sample.json, onboarding.json, decisions.jsonl,
// backlog.jsonl, cells/) are NEVER listed here; the block anticipates D2's
// spikes home (.bee/spikes/) as a plain gitignore pattern regardless of
// whether that cell has landed yet.
const GITIGNORE_BLOCK_PATTERNS = [
  ".bee/state.json",
  ".bee/reservations.json",
  ".bee/workers/",
  ".bee/logs/",
  ".bee/capture-queue.jsonl",
  ".bee/feedback-digest.json",
  ".bee/.inject-cache.json",
  ".bee/HANDOFF.json",
  ".bee/spikes/",
  ".bee/manifest-hash.json",
  // Runtime + cache tiers (worktree-feature-parallelism D3): live coordination
  // and derived caches — never tracked, never merged. .bee/runtime/ holds the
  // worktree grant registry; .bee/sessions/ and .bee/claims/ are live session
  // state; .bee/cache/ is replay-derived.
  ".bee/sessions/",
  ".bee/claims/",
  ".bee/runtime/",
  ".bee/cache/",
  // Intent anchors (intent-anchor D1/D2): the verbatim objective of whatever
  // task is live in THIS checkout, keyed by feature or session id. Live
  // per-session task state, exactly like .bee/sessions/ above — never tracked,
  // never merged, and never a diff on someone else's branch.
  ".bee/intent/",
  // Cross-process store-mutex lockfiles (hardening-4b): withStoreLock
  // (lock.mjs) writes a short-lived .lock file per named critical section —
  // machine/process-local coordination state, never tracked. Left unignored,
  // a "worktree new"/"worktree merge" run holds this lock for the ENTIRE
  // operation (worktree-admin mutex, hardening-4b), so its own lock file is
  // untracked dirt sitting in the checkout for that whole window — `git
  // status --porcelain` (the D8a dirty-check every merge/create runs FIRST)
  // would see it and refuse WORKTREE_MERGE_MAIN_DIRTY/WORKTREE_CALLER_NOT_
  // ORDINARY-adjacent checks against a tree that is only "dirty" because bee
  // itself is holding a lock on it.
  ".bee/locks/",
  // Static "doctor attest" record (g22-3, D5-REVISED): hash/version/identity
  // pairing a human vouched for via `bee doctor attest --runtime codex` —
  // machine-local runtime state, never tracked (a different checkout or a
  // re-clone must re-attest, never inherit someone else's attestation).
  ".bee/doctor-attest.json",
  // Native transport capability probe (codex-native-transport D3/D4, advisor
  // Δ2): a SEPARATE version+config-scoped machine-observed capability record
  // — never doctor-attest.json, never tracked (see bee.mjs's
  // readNativeTransportClassification/writeNativeTransportProbe).
  ".bee/native-transport-probe.json",
  // Machine-local config overlay (hardening-8, config-overlay D-local-config):
  // deep-merged OVER the tracked .bee/config.json by readConfig (overlay
  // wins, arrays replace) so a single contributor's machine-local values
  // (today: dogfood_repos absolute paths) never land in the shared, tracked
  // config file. Never tracked — every checkout/clone starts without one.
  ".bee/config.local.json",
  // Crash-leak scratch (tree-hygiene D1-D3): the ONE canonical scratch home
  // and its swept backups directory. Cleanup that only runs on the happy
  // path is not cleanup, but ignoring is still the belt to self-cleaning's
  // braces — a leaked scratch dir must never show up as git-status noise
  // either.
  ".bee/tmp/",
  ".bee/backups/",
  // Atomic-tmp SHAPE only — `<file>.<pid>-<counter>-<random>.tmp`, the exact
  // name writeJsonAtomic/writeJsonlAtomic/writeTextAtomic use for their
  // rename-target — never a bare `*.tmp`. A human's own hand-made notes.tmp
  // must stay visible to git status; only the machine-generated shape (a
  // literal dot, then a digit-led pid, then two more dash-separated
  // segments, then `.tmp`) is ever matched.
  "*.[0-9]*-*-*.tmp",
  // Plugin render swap dirs (tree-hygiene D3): render_plugin_skill_trees.mjs
  // writes into a tmp sibling and swaps via rename, keeping the previous
  // tree in an '.old-*' sibling until its own cleanup removes it — both
  // prefixes are crash-leak-prone and never deliberate deliverables.
  ".claude-plugin/skills.tmp-*/",
  ".claude-plugin/skills.old-*/",
  ".codex-plugin/skills.tmp-*/",
  ".codex-plugin/skills.old-*/",
  // Legacy-flow debris (bee-gitignore-debris gi-1): pre-`--evidence-stdin`
  // verify/evidence/deviations scratch a worker's flow used to write straight
  // into the .bee ROOT (e.g. `.bee/tmp_verify_1710-4.txt`,
  // `.bee/tmp_evidence_1710-4.json`) before being read and discarded, plus
  // one-off `.bee/patch-cz*.json` hand patches applied outside the normal
  // cell-cap path. Both shapes are machine/session-scratch, never a
  // deliverable, and never team-durable (contrast .bee/cells/, .bee/bin/,
  // .bee/config.json above) — six such files were found still tracked at
  // the .bee root and removed by this same cell.
  ".bee/tmp_*",
  ".bee/patch-*.json",
];

const HOOK_FILENAMES = [
  // adapter.mjs is the shared runtime adapter every wrapper hook imports
  // (cell codex-parity-3) — vendoring the wrappers without it would break
  // their import and crash every repo-fallback hook in the host repo.
  "adapter.mjs",
  // Codex repo projections wire this handler on both SubagentStart and
  // SubagentStop. It must travel with the generated projection on a fresh
  // host; plugin-first loads the same handler from the package instead.
  "bee-codex-subagent-audit.mjs",
  "bee-session-init.mjs",
  "bee-prompt-context.mjs",
  "bee-write-guard.mjs",
  // tokenize-command.mjs is imported by bee-write-guard.mjs's CLI-shape check
  // (decision D2, cell codex-parity-3) — vendoring the guard without it
  // crashes ERR_MODULE_NOT_FOUND on every fresh repo-hooks install, making
  // the guard fail-open instead of blocking pre-Gate-3 writes (canary P5,
  // cell i54-closeout-9).
  "tokenize-command.mjs",
  "bee-state-sync.mjs",
  "bee-chain-nudge.mjs",
  "bee-session-close.mjs",
  "bee-model-guard.mjs",
  "bee-tools-logger.mjs",
];

const DEFAULT_STATE = {
  schema_version: "1.0",
  phase: "idle",
  feature: null,
  mode: null,
  approved_gates: { context: false, shape: false, execution: false, review: false },
  workers: [],
  summary: "",
  next_action: "Invoke bee-hive.",
};

const DEFAULT_CONFIG = {
  hooks: {
    "session-init": true,
    "prompt-context": true,
    "write-guard": true,
    "state-sync": true,
    "chain-nudge": true,
    "session-close": true,
  },
  lanes: {},
  capabilities: {},
  // Opt-in autopilot (decision 0010): when true, the agent auto-approves
  // Gates 1-3 for tiny/small/standard non-hard-gate work instead of stopping
  // for the human. High-risk/hard-gate work, secret reads, and Gate 4 UAT are
  // never bypassed. Toggle with the bee-bypass-gate skill. Default off.
  gate_bypass: false,
  // Model tiers, runtime-keyed (decision 0012). swarming resolves tier → model
  // per dispatch so the strongest model stays scarce (ceiling) and cheap models
  // run the loops (extraction/generation). Edit per repo. null = the runtime
  // cannot switch per-agent model → tier enforced via read budget + output cap.
  // Only the cheaper tiers are configured; the ceiling is always the session
  // model (decision 0015), so it has no entry here.
  models: {
    claude: { extraction: "haiku", generation: "sonnet" },
    codex: { extraction: null, generation: null },
  },
};

const CRITICAL_PATTERNS_STUB = `# Critical Patterns

Mandatory pre-planning / pre-execution context for this repository.
bee-compounding appends hard-won patterns here; keep it short and current.

(none captured yet)
`;

// State-layer skeletons (create-only, never overwritten): bee-scribing owns
// the content; onboarding only guarantees the files exist so "read the spec
// before the code" and "where does X live" have a landing page from day one.
const READING_MAP_STUB = `# Reading Map

Where each area of this project lives. bee-scribing owns this file: it is
updated whenever an area spec is created or moved. Read this before any broad
search — it answers "where does X live" without a grep.

| Area | Spec | Code entry points |
|---|---|---|
| (none mapped yet — run a bee-scribing bootstrap pass) | | |
`;

const SYSTEM_OVERVIEW_STUB = `# System Overview

One-page, technology-agnostic description of what this system does and how its
areas fit together. bee-scribing owns this file; it is the first read for any
human or agent new to the repository.

(not written yet — run a bee-scribing bootstrap pass to fill this in)
`;

// CLAUDE.md @import fallback: Claude Code auto-loads CLAUDE.md but not
// AGENTS.md; a bare @AGENTS.md line imports the BEE block at context-load
// time (repository-harness pattern). Third belt when plugin hooks are absent.
const CLAUDE_MD_IMPORT_SECTION = `## bee

This repo uses bee. The bare import below loads the BEE operating block from
AGENTS.md at context-load time. Never wrap it in backticks; that disables it.

@AGENTS.md
`;

const CLAUDE_MD_TEMPLATE = `# Project Rules

${CLAUDE_MD_IMPORT_SECTION}`;

// ---------- small utilities ----------

function utcNow() {
  return new Date().toISOString();
}

function sha256(text) {
  return crypto.createHash("sha256").update(text).digest("hex");
}

function readTextIfExists(filePath) {
  return fs.existsSync(filePath) ? fs.readFileSync(filePath, "utf8") : "";
}

function writeFileAtomic(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const tmp = `${filePath}.tmp`;
  fs.writeFileSync(tmp, content, "utf8");
  fs.renameSync(tmp, filePath);
}

function readJsonIfExists(filePath) {
  const text = readTextIfExists(filePath);
  if (!text.trim()) {
    return null;
  }
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function nodeRuntimeStatus() {
  const major = Number.parseInt(String(process.versions.node).split(".")[0] || "0", 10);
  return {
    version: process.versions.node,
    minimum_major: MIN_NODE_MAJOR,
    supported: Number.isFinite(major) && major >= MIN_NODE_MAJOR,
  };
}

// ---------- skill sync (D1-D5, per-target since installer-hardening) ----------
//
// Source = the skill tree the RUNNING script belongs to (D2), proven by a
// realpath identity with its own bee-hive dir (F2: a misplaced launcher never
// adopts a sibling tree). Targets = the host repo's two managed in-repo skill
// roots by default, plus the user's global skills dir only under
// --global-skills - there is deliberately NO free-form override of any kind,
// env or CLI (D1/F5: an override would widen the deletion root to arbitrary
// paths). Tests isolate by redirecting HOME/USERPROFILE for the spawned
// process, which os.homedir() honors.

const SKILL_DIR_RE = /^bee-/;

// The exactly-two managed in-repo roots (installer-hardening D2/D6): Claude
// Code discovers <repo>/.claude/skills, Codex discovers <repo>/.agents/skills.
// Committed to the host repo (D4) - onboarding never gitignores them.
const REPO_SKILL_TARGETS = [
  { kind: "repo-claude", segments: [".claude", "skills"] },
  { kind: "repo-agents", segments: [".agents", "skills"] },
];

function skillsTargetRoot() {
  return path.join(os.homedir(), ".claude", "skills");
}

// Target order is stable (repo-claude, repo-agents, then global): blocked-first
// aggregates below surface the FIRST blocked target's status/versions.
function skillSyncTargets(repoRoot, { globalSkills = false } = {}) {
  const targets = REPO_SKILL_TARGETS.map(({ kind, segments }) => ({
    kind,
    target_root: path.join(repoRoot, ...segments),
  }));
  if (globalSkills) {
    targets.push({ kind: "global", target_root: skillsTargetRoot() });
  }
  return targets;
}

function lstatIfExists(p) {
  try {
    return fs.lstatSync(p);
  } catch {
    return null;
  }
}

// Review P1-8: every blocked_no_source return happens BEFORE the three-version
// preflight ever runs (identity/overlap are structural checks, independent of
// file content) - so none of the three versions were, or could be, resolved.
// D3's letter requires all three reported on every blocked return; "unknown"
// is the honest label for "resolution was impossible", distinct from the
// version-preflight's own "absent" state (a tree that provably does not exist).
function unknownVersionsTriple() {
  return { source: "unknown", host_helpers: "unknown", installed_skills: "unknown" };
}

// Fallback-free version reader (D3, hardened per review P1-1/P1-2). A fallback
// version on a missing/unparsable state.mjs would let a resolution failure
// masquerade as an old version and become force-able. Here: treeExists=false
// means "absent" (fresh install /
// first onboard, proceed); an EXISTING tree whose version cannot be read ->
// "unknown" (refuse, never forceable). "Read" is strict: the marker must be a
// REGULAR, non-symlinked file - when componentRoot is given, every path
// component from that root down to the marker is lstat'ed (a symlinked
// directory on the way is as untrusted as a symlinked marker); without it the
// marker file itself is lstat'ed - and the content must carry exactly ONE
// line-anchored `export const BEE_VERSION = 'x.y.z'` declaration. Substring
// matches (comment decoys) never resolve; multiple declarations are unknown.
const BEE_VERSION_LINE_RE = /^export const BEE_VERSION = ['"]([^'"]*)['"];?[ \t]*\r?$/gm;
const NUMERIC_RELEASE_VERSION_RE = /^\d+\.\d+\.\d+$/;

function readVersionStrict(stateFile, treeExists, { componentRoot = null } = {}) {
  if (!treeExists) {
    return { state: "absent", value: null };
  }
  const unknown = { state: "unknown", value: null };
  const components = [];
  if (componentRoot) {
    const rel = path.relative(componentRoot, stateFile);
    if (!rel || rel.startsWith("..") || path.isAbsolute(rel)) {
      return unknown; // marker escaped the managed root: never trusted
    }
    let current = componentRoot;
    for (const part of rel.split(path.sep)) {
      current = path.join(current, part);
      components.push(current);
    }
  } else {
    components.push(stateFile);
  }
  for (let i = 0; i < components.length; i += 1) {
    const st = lstatIfExists(components[i]);
    const isMarker = i === components.length - 1;
    if (!st || st.isSymbolicLink() || (isMarker ? !st.isFile() : !st.isDirectory())) {
      return unknown;
    }
  }
  let text = null;
  try {
    text = fs.readFileSync(stateFile, "utf8");
  } catch {
    return unknown;
  }
  const matches = [...text.matchAll(BEE_VERSION_LINE_RE)];
  if (matches.length !== 1 || !NUMERIC_RELEASE_VERSION_RE.test(matches[0][1])) {
    return unknown;
  }
  return { state: "resolved", value: matches[0][1] };
}

function readManifestVersionStrict(manifestPath) {
  const unknown = { state: "unknown", value: null };
  const stat = lstatIfExists(manifestPath);
  if (!stat || stat.isSymbolicLink() || !stat.isFile()) return unknown;
  let parsed;
  try {
    parsed = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  } catch {
    return unknown;
  }
  if (
    !parsed ||
    Array.isArray(parsed) ||
    typeof parsed !== "object" ||
    typeof parsed.version !== "string" ||
    !NUMERIC_RELEASE_VERSION_RE.test(parsed.version)
  ) {
    return unknown;
  }
  return { state: "resolved", value: parsed.version };
}

// Installer-version-parity D1: the canonical runtime marker and both package
// manifests are one authoritative source identity. This runs before any plan
// that can be applied, including plugin-first runs that intentionally skip
// project skill projection, so a malformed or mixed package can never mutate a
// target and only discover the split afterwards.
function readSourceReleaseIdentity() {
  const runtimeComponent = {
    name: "packages/bee/lib/state.mjs",
    version: readVersionStrict(path.join(TEMPLATES_LIB_DIR, "state.mjs"), true),
  };
  let sourceKind = "unknown";
  try {
    sourceKind = classifySource({ hiveDir: HIVE_DIR, homeDir: os.homedir() }).kind;
  } catch {}

  // D9 provenance: a rendered per-runtime projection (carries the render
  // sidecar) is NEVER an authoritative source, for ANY target — its own runtime
  // included. Refuse fail-closed with zero mutations; the canonical checkout or
  // a plugin package is required. No target-filter semantics.
  if (sourceKind === "rendered_projection") {
    return {
      version: null,
      components: [runtimeComponent],
      blocked: {
        status: "blocked_no_source",
        reason:
          "onboarding source is a rendered per-runtime projection (carries the render provenance marker) - a projection is never an authoritative source for any target; use the canonical checkout or plugin package",
        forceable: false,
      },
    };
  }

  // Project/global projections intentionally do not carry package manifests.
  // They remain valid downgrade sentinels, but never become package release
  // authorities: validate only their runtime marker and let the existing
  // three-version guard compare it against the target runtime/skills.
  if (sourceKind === "project_projection" || sourceKind === "legacy_global") {
    if (runtimeComponent.version.state !== "resolved") {
      return {
        version: null,
        components: [runtimeComponent],
        blocked: {
          status: "blocked_no_source",
          reason: "projection runtime version is missing, unreadable, or non-numeric",
          forceable: false,
        },
      };
    }
    return { version: runtimeComponent.version.value, components: [runtimeComponent], blocked: null };
  }

  const components = [
    runtimeComponent,
    {
      name: ".claude-plugin/plugin.json",
      version: readManifestVersionStrict(path.join(PLUGIN_ROOT, ".claude-plugin", "plugin.json")),
    },
    {
      name: ".codex-plugin/plugin.json",
      version: readManifestVersionStrict(path.join(PLUGIN_ROOT, ".codex-plugin", "plugin.json")),
    },
  ];
  const unreadable = components.filter(function ({ version }) {
    return version.state !== "resolved";
  });
  if (unreadable.length !== 0) {
    return {
      version: null,
      components,
      blocked: {
        status: "blocked_no_source",
        reason:
          "authoritative source release tuple is invalid: missing, unreadable, or non-numeric " +
          unreadable.map(function ({ name }) { return name; }).join(", "),
        forceable: false,
      },
    };
  }
  const values = new Set(components.map(function ({ version }) { return version.value; }));
  if (values.size !== 1) {
    return {
      version: null,
      components,
      blocked: {
        status: "blocked_no_source",
        reason:
          "authoritative source release tuple is invalid: tuple members disagree (" +
          components.map(function ({ name, version }) {
            return `${name}=${version.value}`;
          }).join(", ") +
          ")",
        forceable: false,
      },
    };
  }
  return { version: components[0].version.value, components, blocked: null };
}

function compareVersions(a, b) {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i += 1) {
    if (pa[i] !== pb[i]) {
      return pa[i] - pb[i];
    }
  }
  return 0;
}

function versionLabel(v) {
  return v.state === "resolved" ? v.value : v.state;
}

// lstat-only walk of one skill dir: symlinks are never followed. The first
// symlink (or other non-file/dir entry) found blocks the WHOLE skill (F6 - a
// symlinked skill dir is plausibly a developer's live checkout; writing
// through or unlinking it would destroy real work).
// `transform`, when given, maps a file's raw bytes to the bytes that would be
// materialized (the per-runtime render, D9). Source walks pass it so the drift
// fingerprint compares render(source, runtime) against the installed bytes;
// target walks omit it (the installed tree already holds rendered bytes). With
// no markers in any file, render is byte-identity, so every fingerprint is
// exactly the pre-render one and the whole sync path is unchanged.
export function walkSkillTree(rootDir, transform) {
  const files = new Map(); // rel path ("/"-joined) -> sha256
  const dirs = [];
  let blocked = null;
  const walk = (dir, relPrefix) => {
    const entries = fs
      .readdirSync(dir, { withFileTypes: true })
      .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
    for (const entry of entries) {
      if (blocked) {
        return;
      }
      const rel = relPrefix ? `${relPrefix}/${entry.name}` : entry.name;
      const abs = path.join(dir, entry.name);
      if (entry.isSymbolicLink()) {
        blocked = { path: rel, reason: "symlink" };
        return;
      }
      if (entry.isDirectory()) {
        dirs.push(rel);
        walk(abs, rel);
      } else if (entry.isFile()) {
        const raw = fs.readFileSync(abs);
        files.set(rel, sha256(transform ? transform(raw, rel) : raw));
      } else {
        blocked = { path: rel, reason: "unsupported entry type" };
        return;
      }
    }
  };
  walk(rootDir, "");
  return { files, dirs, blocked };
}

export function manifestFingerprint(files) {
  return JSON.stringify([...files.entries()].sort((a, b) => (a[0] < b[0] ? -1 : 1)));
}

// ---------- skill runtime rendering (D9) ----------
//
// A skill source file may carry runtime-scoped blocks fenced by strict,
// full-line HTML-comment markers:
//
//   <!-- bee:only claude -->   ...claude-only prose...   <!-- bee:end -->
//   <!-- bee:only codex  -->   ...codex-only prose...     <!-- bee:end -->
//
// render(bytes, runtime) drops the blocks not meant for `runtime`, strips every
// marker line, and passes unmarked content through. TODAY NO SKILL FILE CARRIES
// A MARKER, so render is byte-for-byte identity on every real file and the whole
// sync/drift path is unchanged until content is tagged in a later cell. Byte
// identity for the no-marker case is exact — a file whose bytes contain no
// marker at all is returned unchanged (BOM, CRLF, final-newline state, and
// arbitrary bytes preserved), never decoded-and-re-encoded.
export const RENDER_RUNTIMES = ["claude", "codex"];
// bee-render/2 (g22-4, decision D7 / advisor R5): the sidecar is now a full
// inventory, not just a provenance stamp — {schema, target_runtime,
// skills:[{name, sha256}]} where sha256 is a deterministic digest of that
// skill's RENDERED file set (see skillDigest below). Single-sourced: this
// constant plus buildRenderSidecar/skillDigest are the ONLY place the shape
// or the hash algorithm is defined; every writer (render_plugin_skill_trees.mjs,
// the onboarding managed-target sync below) imports them rather than
// hand-building the object. doctor (bee.mjs, which cannot import this file -
// see its own mirror-discipline note) duplicates the read-side algorithm by
// hand and must be kept in lockstep with skillDigest/walkSkillTree here.
export const RENDER_SCHEMA = "bee-render/2";
// Provenance sidecar written at each rendered target's skills ROOT (a sibling
// of the bee-* dirs, never inside one). source-identity classifies any skills
// root carrying it as a rendered projection and refuses it as an onboarding
// source for ANY target (D9 provenance).
export const RENDER_SIDECAR = ".bee-render.json";

// D1/D3 (packages-restructure deviation, logged): skills are instruction-only
// now, so the vendored payload's own version marker (packages/bee/lib/
// state.mjs) no longer travels inside a synced skill dir - a fresh sync's
// source walk never produces bee-hive/templates/**, so nothing would ever
// repopulate the three-version preflight's old installed_skills marker
// (<target>/bee-hive/templates/lib/state.mjs) post-sync, permanently
// bricking re-onboarding (unknown, never forceable) the moment a synced
// target's stale templates/ gets pruned by the next whole-skill resync.
// SKILLS_VERSION_STAMP is a sync-owned replacement: written at the skills
// ROOT (sibling of the RENDER_SIDECAR above, never inside a bee-* dir so the
// per-skill mirror never touches or deletes it) on every successful apply,
// for every target kind. computeSkillSyncTarget reads it first and treats it
// as authoritative; only when it is genuinely ABSENT (never when malformed or
// symlinked) does the preflight fall back to the legacy nested marker, so an
// already-onboarded host self-migrates on its very next onboard with zero
// flags and zero manual steps.
export const SKILLS_VERSION_STAMP = ".bee-skills-version.json";

// One skill's content digest: sha256 over manifestFingerprint's sorted
// [relPath, sha256(fileBytes)] pairs, so it folds the same fingerprint
// already used for drift detection (walkSkillTree + manifestFingerprint) into
// one fixed-length hash per skill. `files` is a Map<relPath, sha256hex> (or
// an iterable of entries) of ONE skill dir's RENDERED file tree - exactly
// what walkSkillTree(dir, renderTransform).files already produces.
export function skillDigest(files) {
  const map = files instanceof Map ? files : new Map(files);
  return sha256(manifestFingerprint(map));
}

// Builds the full bee-render/2 sidecar object (D7). `skillsDirEntries` is
// [{name, files}] - one entry per bee-* skill dir at the rendered
// target/tree, `files` being that skill's rendered file-hash Map (see
// skillDigest above). Deterministic and target-independent given the same
// (source content, runtime): callers stringify + write the result, never
// hand-build the shape.
export function buildRenderSidecar(targetRuntime, skillsDirEntries) {
  const skills = skillsDirEntries
    .map(({ name, files }) => ({ name, sha256: skillDigest(files) }))
    .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
  return { schema: RENDER_SCHEMA, target_runtime: targetRuntime, skills };
}

// A line that begins (indentation allowed, so a mis-indented marker is caught,
// not silently ignored) with an HTML-comment bee marker. Any line matching this
// participates in rendering/validation; a file with none is passed through
// untouched.
const NEAR_MARKER_RE = /^[ \t]*<!--[ \t]*bee:(only|end)\b/;
// The exact, canonical, column-0 markers (only trailing horizontal whitespace
// tolerated). A NEAR line that is not one of these is malformed.
const MARKER_ONLY_RE = /^<!-- bee:only (\S+) -->[ \t]*$/;
const MARKER_END_RE = /^<!-- bee:end -->[ \t]*$/;
const FRONTMATTER_DELIM_RE = /^---[ \t]*$/;

function fenceChar(line) {
  const m = line.match(/^[ \t]*(`{3,}|~{3,})/);
  return m ? m[1][0] : null;
}

// True when the raw bytes could contain a marker at all — the cheap gate that
// keeps the no-marker path from ever decoding (byte-identity guarantee).
function bufHasMarkerBytes(buf) {
  return buf.includes("bee:only") || buf.includes("bee:end");
}

// Split into [content, terminator] pairs preserving exact line endings (the
// final line's terminator is "" when the file has no trailing newline), so
// concatenating every pair rebuilds the input byte-for-byte.
function splitLinesPreserving(text) {
  const out = [];
  const re = /\r\n|\n/g;
  let last = 0;
  let m;
  while ((m = re.exec(text)) !== null) {
    out.push([text.slice(last, m.index), m[0]]);
    last = m.index + m[0].length;
  }
  out.push([text.slice(last), ""]);
  return out;
}

// Classify one NEAR line: {kind:"only",runtime} | {kind:"end"} | {error}.
function classifyMarkerLine(line) {
  const only = line.match(MARKER_ONLY_RE);
  if (only) {
    const label = only[1];
    if (!RENDER_RUNTIMES.includes(label)) {
      return { error: `unknown runtime label "${label}" (expected ${RENDER_RUNTIMES.join(" or ")})` };
    }
    return { kind: "only", runtime: label };
  }
  if (MARKER_END_RE.test(line)) {
    return { kind: "end" };
  }
  return { error: `ambiguous near-marker "${line.trim()}" (not an exact full-line bee marker)` };
}

// Whole-file grammar check. Returns a list of human-readable error strings;
// empty means the file is well-formed (or carries no markers). Enforced BEFORE
// any mutation, per file, across the whole tree.
export function validateSkillMarkers(text) {
  const errors = [];
  const lines = text.split(/\r\n|\n/);
  // Frontmatter span: only when the very first line is a `---` delimiter.
  let frontmatterEnd = -1;
  if (lines.length > 0 && FRONTMATTER_DELIM_RE.test(lines[0])) {
    for (let i = 1; i < lines.length; i += 1) {
      if (FRONTMATTER_DELIM_RE.test(lines[i])) {
        frontmatterEnd = i;
        break;
      }
    }
  }
  let fence = null;
  let openRuntime = null;
  let openLine = -1;
  let firstFrontmatterOpener = -1;
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    // Track code fences (outside the frontmatter span).
    if (i > frontmatterEnd) {
      const fc = fenceChar(line);
      if (fence === null) {
        if (fc) fence = fc;
      } else if (fc === fence) {
        fence = null;
      }
    }
    if (!NEAR_MARKER_RE.test(line)) {
      // A `---` opener appearing AFTER a marker but with the marker above the
      // top of the file signals a marker placed before frontmatter.
      if (firstFrontmatterOpener === -1 && FRONTMATTER_DELIM_RE.test(line) && i > 0 && frontmatterEnd === -1) {
        firstFrontmatterOpener = i;
      }
      continue;
    }
    // Inside a fenced code block: forbidden, and never parsed as a marker.
    if (fence !== null) {
      errors.push(`marker inside a fenced code block at line ${i + 1}: "${line.trim()}"`);
      continue;
    }
    // Inside the YAML frontmatter span: forbidden.
    if (frontmatterEnd !== -1 && i <= frontmatterEnd) {
      errors.push(`marker inside YAML frontmatter at line ${i + 1}: "${line.trim()}"`);
      continue;
    }
    const cls = classifyMarkerLine(line);
    if (cls.error) {
      errors.push(`${cls.error} at line ${i + 1}`);
      continue;
    }
    if (cls.kind === "only") {
      if (openRuntime !== null) {
        errors.push(`nested bee:only block at line ${i + 1} (block opened at line ${openLine + 1} not closed)`);
        continue;
      }
      openRuntime = cls.runtime;
      openLine = i;
      // A well-formed marker sitting above a later frontmatter opener is a
      // marker placed before frontmatter.
      if (firstFrontmatterOpener !== -1) {
        errors.push(`marker before YAML frontmatter at line ${i + 1}`);
      }
    } else {
      // end
      if (openRuntime === null) {
        errors.push(`stray bee:end with no open block at line ${i + 1}`);
        continue;
      }
      openRuntime = null;
    }
  }
  if (openRuntime !== null) {
    errors.push(`unclosed bee:only block opened at line ${openLine + 1}`);
  }
  return errors;
}

// Filter one file's bytes for `runtime`. On any parse ambiguity the caller has
// already refused via validateSkillMarkers; here a file with no marker line is
// returned byte-identical (never decoded), and a marked file is rebuilt with
// exact line endings, its marker lines stripped and its off-runtime blocks
// dropped.
export function renderSkillBytes(buf, runtime) {
  if (!bufHasMarkerBytes(buf)) {
    return buf;
  }
  const text = buf.toString("utf8");
  const lines = text.split(/\r\n|\n/);
  if (!lines.some((line) => NEAR_MARKER_RE.test(line))) {
    return buf; // literal "bee:only"/"bee:end" in prose, no actual marker line
  }
  const pairs = splitLinesPreserving(text);
  let out = "";
  let openRuntime = null;
  for (const [content, term] of pairs) {
    if (NEAR_MARKER_RE.test(content)) {
      const cls = classifyMarkerLine(content);
      // Validation guarantees well-formedness before render is ever reached;
      // treat any recognized marker as a control line and strip it.
      if (!cls.error) {
        if (cls.kind === "only") openRuntime = cls.runtime;
        else openRuntime = null;
        continue;
      }
    }
    if (openRuntime === null || openRuntime === runtime) {
      out += content + term;
    }
  }
  return Buffer.from(out, "utf8");
}

// Which runtime a target root renders for: repo-agents is Codex, every other
// managed root (repo-claude, global, legacy-global) is Claude.
function runtimeForTargetKind(kind) {
  return kind === "repo-agents" ? "codex" : "claude";
}

// Whole-tree grammar gate (D9): scan every bee-* skill FILE in the source tree
// and collect marker-grammar errors. Read-only; symlinked/unsupported entries
// are left to the sync path's own block handling. A non-empty result refuses
// the WHOLE apply with zero writes.
function validateSkillTreeMarkers(sourceRoot) {
  const errors = [];
  for (const entry of listBeeSkillEntries(sourceRoot)) {
    if (entry.isSymbolicLink() || !entry.isDirectory()) {
      continue;
    }
    const skillDir = path.join(sourceRoot, entry.name);
    const walk = walkSkillTree(skillDir);
    if (walk.blocked) {
      continue;
    }
    for (const rel of walk.files.keys()) {
      const abs = path.join(skillDir, ...rel.split("/"));
      let buf;
      try {
        buf = fs.readFileSync(abs);
      } catch {
        continue;
      }
      if (!bufHasMarkerBytes(buf)) {
        continue;
      }
      for (const e of validateSkillMarkers(buf.toString("utf8"))) {
        errors.push(`${entry.name}/${rel}: ${e}`);
      }
    }
  }
  return errors;
}

// The deletion domain is constructed here: only /^bee-/ entries are ever
// enumerated, so non-bee skills are structurally unreachable - the fence is
// the iteration domain, not a guard clause (D4).
function listBeeSkillEntries(root) {
  if (!fs.existsSync(root)) {
    return [];
  }
  return fs
    .readdirSync(root, { withFileTypes: true })
    .filter((entry) => SKILL_DIR_RE.test(entry.name))
    .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
}

// Every plain bee-* skill dir directly under sourceRoot, rendered for
// `runtime`, as [{name, files}] ready for buildRenderSidecar (D7). This is
// the "should be installed" inventory: symlinked or otherwise-blocked source
// skill dirs are excluded, because they are never synced to any target
// either (computeSkillItems/applySyncSkill both refuse them the same way) -
// a sidecar must never promise a skill that onboarding itself cannot write.
// Target-independent for a given runtime (render(source, runtime) does not
// vary per target root), so callers compute this once per runtime, not once
// per target.
export function sourceSkillDigestEntries(sourceRoot, runtime) {
  const renderSource = (buf) => renderSkillBytes(buf, runtime);
  const out = [];
  for (const entry of listBeeSkillEntries(sourceRoot)) {
    if (entry.isSymbolicLink() || !entry.isDirectory()) {
      continue;
    }
    const walk = walkSkillTree(path.join(sourceRoot, entry.name), renderSource);
    if (walk.blocked) {
      continue;
    }
    out.push({ name: entry.name, files: walk.files });
  }
  return out;
}

// Canonical filesystem identity for case-alias detection (review P1-5): on a
// case-insensitive filesystem, `bee-hive` and `bee-Hive` are two NAMES for one
// physical entry - exact-case string comparison would let the sync pass write
// it and the removal pass then delete it. Identity is dev:ino via lstat (never
// follows links); any two bee-* names resolving to one identity block those
// skills loudly - never sync-then-delete.
function entryIdentity(p) {
  const st = lstatIfExists(p);
  return st ? `${st.dev}:${st.ino}` : null;
}

// Probe every candidate name (source names + installed entries) under the
// target root; two DIFFERENT names on one physical identity collide - all
// names involved are returned as blocked.
function detectAliasCollisions(sourceNames, targetRoot) {
  const names = new Set(sourceNames);
  for (const entry of listBeeSkillEntries(targetRoot)) {
    names.add(entry.name);
  }
  const byIdentity = new Map();
  for (const name of names) {
    const id = entryIdentity(path.join(targetRoot, name));
    if (!id) {
      continue;
    }
    if (!byIdentity.has(id)) {
      byIdentity.set(id, []);
    }
    byIdentity.get(id).push(name);
  }
  const collided = new Set();
  for (const aliasNames of byIdentity.values()) {
    if (aliasNames.length > 1) {
      for (const n of aliasNames) {
        collided.add(n);
      }
    }
  }
  return collided;
}

// Nested variant of the same check inside ONE skill: every source and target
// rel path is probed under the installed skill dir; two different rels
// resolving to one physical entry (e.g. references/ vs References/ on a
// case-insensitive fs) block the whole skill.
function detectNestedAlias(targetDir, sourceWalk, targetWalk) {
  const rels = new Set([
    ...sourceWalk.files.keys(),
    ...sourceWalk.dirs,
    ...targetWalk.files.keys(),
    ...targetWalk.dirs,
  ]);
  const byIdentity = new Map();
  for (const rel of rels) {
    const id = entryIdentity(path.join(targetDir, ...rel.split("/")));
    if (!id) {
      continue;
    }
    if (byIdentity.has(id) && byIdentity.get(id) !== rel) {
      return { a: byIdentity.get(id), b: rel };
    }
    byIdentity.set(id, rel);
  }
  return null;
}

// Review P1-9: every plan item's `path` is root-relative, but legacy items
// (repo files) and skill-stage items (global ~/.claude/skills entries) are
// relative to TWO DIFFERENT roots - an approval surface reading `path` alone
// could render a global deletion against repoRoot. `scope` disambiguates:
// "installed" = target_root-relative (skillsTargetRoot()), "source" =
// source_root-relative (the running script's own tree). Legacy items carry no
// `scope` at all (unchanged) - their root is always repoRoot, documented in
// SKILL.md alongside this field.
function aliasBlockedItem(name, detail) {
  return {
    action: "blocked_alias",
    skill: name,
    path: name,
    scope: "installed", // alias identity is always probed under targetRoot
    reason: `installed ${name} ${detail} - blocked, never sync-then-delete`,
  };
}

// D4/D5 drift plan items. Content difference IS drift, at any version (D5);
// a bee-* skill absent from the anchored source IS an intentional removal (D2).
function computeSkillItems(sourceRoot, targetRoot, runtime) {
  const items = [];
  const renderSource = (buf) => renderSkillBytes(buf, runtime);
  const sourceEntries = listBeeSkillEntries(sourceRoot);
  const sourceNames = new Set(sourceEntries.map((entry) => entry.name));
  const aliasCollisions = detectAliasCollisions(sourceNames, targetRoot);

  for (const entry of sourceEntries) {
    const name = entry.name;
    if (aliasCollisions.has(name)) {
      items.push(aliasBlockedItem(name,
        "shares one physical entry with a differently-named bee-* entry (case-insensitive alias)"));
      continue;
    }
    if (entry.isSymbolicLink()) {
      items.push({
        action: "blocked_symlink",
        skill: name,
        path: name,
        scope: "source",
        reason: `source ${name} is a symlink - skipped, never followed`,
      });
      continue;
    }
    if (!entry.isDirectory()) {
      continue; // stray bee-* file in source: not a skill dir
    }
    const sourceWalk = walkSkillTree(path.join(sourceRoot, name), renderSource);
    if (sourceWalk.blocked) {
      items.push({
        action: "blocked_symlink",
        skill: name,
        path: `${name}/${sourceWalk.blocked.path}`,
        scope: "source",
        reason: `source ${name} contains a ${sourceWalk.blocked.reason} at ${sourceWalk.blocked.path} - skipped`,
      });
      continue;
    }
    const targetDir = path.join(targetRoot, name);
    const targetStat = lstatIfExists(targetDir);
    if (targetStat && targetStat.isSymbolicLink()) {
      items.push({
        action: "blocked_symlink",
        skill: name,
        path: name,
        scope: "installed",
        reason: `installed ${name} is a symlink (plausibly a live checkout) - skipped, never written through or unlinked`,
      });
      continue;
    }
    if (!targetStat || !targetStat.isDirectory()) {
      // absent, or a non-link type collision (remove entry, write source shape)
      items.push({ action: "sync_skill", skill: name, path: name, scope: "installed" });
      continue;
    }
    const targetWalk = walkSkillTree(targetDir);
    if (targetWalk.blocked) {
      items.push({
        action: "blocked_symlink",
        skill: name,
        path: `${name}/${targetWalk.blocked.path}`,
        scope: "installed",
        reason: `installed ${name} contains a ${targetWalk.blocked.reason} at ${targetWalk.blocked.path} - skipped, nothing inside it written or deleted`,
      });
      continue;
    }
    const nestedAlias = detectNestedAlias(targetDir, sourceWalk, targetWalk);
    if (nestedAlias) {
      items.push(aliasBlockedItem(name,
        `has nested entries ${nestedAlias.a} and ${nestedAlias.b} resolving to one physical entry (case-insensitive alias)`));
      continue;
    }
    if (manifestFingerprint(sourceWalk.files) !== manifestFingerprint(targetWalk.files)) {
      items.push({ action: "sync_skill", skill: name, path: name, scope: "installed" });
    }
  }

  for (const entry of listBeeSkillEntries(targetRoot)) {
    const name = entry.name;
    if (sourceNames.has(name)) {
      continue;
    }
    if (aliasCollisions.has(name)) {
      items.push(aliasBlockedItem(name,
        "shares one physical entry with a differently-named bee-* entry (case-insensitive alias)"));
      continue;
    }
    if (entry.isSymbolicLink()) {
      items.push({
        action: "blocked_symlink",
        skill: name,
        path: name,
        scope: "installed",
        reason: `installed ${name} is a symlink (plausibly a live checkout) - skipped, never unlinked`,
      });
      continue;
    }
    if (!entry.isDirectory()) {
      continue; // deletion domain is /^bee-/ DIRECTORY entries only (D4)
    }
    const targetWalk = walkSkillTree(path.join(targetRoot, name));
    if (targetWalk.blocked) {
      items.push({
        action: "blocked_symlink",
        skill: name,
        path: `${name}/${targetWalk.blocked.path}`,
        scope: "installed",
        reason: `installed ${name} contains a ${targetWalk.blocked.reason} at ${targetWalk.blocked.path} - skipped, nothing deleted`,
      });
      continue;
    }
    items.push({ action: "remove_skill", skill: name, path: name, scope: "installed" });
  }

  return items;
}

// One sync target's resolution + D3 three-version preflight. Fully read-only.
// Semantics are the pre-per-target ones, applied per target root: identity is
// checked once by the caller; overlap guard, three-version preflight
// (unknown-version refusal never forceable), and item computation all run
// here against THIS target.
function computeSkillSyncTarget({
  realRepo,
  sourceRoot,
  realSource,
  sourceVersion,
  hostVersion,
  kind,
  targetRoot,
}) {
  const target = {
    kind,
    target_root: targetRoot,
    mode: null, // "sync" | "fresh" | "noop" | null (blocked before resolution)
    versions: null,
    blocked: null, // { status, reason, forceable }
    items: [],
  };
  const refuse = (reason) => {
    target.versions = unknownVersionsTriple();
    target.blocked = { status: "blocked_no_source", reason, forceable: false };
    return target;
  };

  // Never realpath a nonexistent target (absent target = fresh install);
  // ancestor overlap fails closed (F6).
  const targetExists = fs.existsSync(targetRoot);
  const realTarget = targetExists ? fs.realpathSync(targetRoot) : path.resolve(targetRoot);

  if (kind === "global") {
    // Repo<->global-target overlap (review P1-4): a repo living under the
    // global skills root (or containing it) must never be mutable or deletable
    // by its own onboard - the remove_skill pass could erase the live
    // checkout, git history included. Refused at preflight, never forceable,
    // zero mutations. The two managed in-repo roots are exempt from the
    // repo-contains-target direction BY DESIGN (D2) - see the else branch.
    if (
      realRepo === realTarget ||
      realRepo.startsWith(realTarget + path.sep) ||
      realTarget.startsWith(realRepo + path.sep)
    ) {
      return refuse(
        "repo root and the global skills root overlap (one contains the other) - a repo inside the managed skill target must never be touched by its own onboard, refusing fail-closed",
      );
    }
  } else if (targetExists && !realTarget.startsWith(realRepo + path.sep)) {
    // A managed in-repo root lives inside the repo BY DESIGN (D2) - the
    // repo-contains-target refusal is exempt for exactly these two roots. But
    // a root that RESOLVES outside the repo (or onto the repo root itself,
    // e.g. via a symlink) could silently write a tree - the global
    // ~/.claude/skills included - that this run was never authorized to touch.
    // Fail closed.
    return refuse(
      `managed in-repo skills root ${path.join(...REPO_SKILL_TARGETS.find((t) => t.kind === kind).segments)} resolves outside the repo root - refusing fail-closed`,
    );
  }

  if (targetExists && realSource === realTarget) {
    target.mode = "noop"; // running the installed copy itself (D2)
  } else if (
    realTarget.startsWith(realSource + path.sep) ||
    realSource.startsWith(realTarget + path.sep)
  ) {
    return refuse(
      "source and target skill roots overlap (one contains the other) - refusing fail-closed",
    );
  } else {
    target.mode = targetExists ? "sync" : "fresh";
  }

  // Three-version preflight (D3), per target. Review P1-1: "absent" is earned
  // only by a target with NO lstat-visible bee-* entry at all (a true fresh
  // install). ANY bee-* presence without a readable bee-hive version marker is
  // "unknown" - refuse, never forceable: a target holding newer bee-* skills
  // but no readable bee-hive must never read as fresh and get
  // overwritten/deleted by an older source.
  const installedHive = path.join(targetRoot, "bee-hive");
  let installedTreeExists = false;
  if (targetExists) {
    try {
      installedTreeExists = fs
        .readdirSync(targetRoot, { withFileTypes: true })
        .some((entry) => SKILL_DIR_RE.test(entry.name));
    } catch {
      installedTreeExists = true; // unreadable target: fail closed -> unknown
    }
  }
  // packages-restructure deviation (see SKILLS_VERSION_STAMP doc comment):
  // skills are instruction-only now, so the vendored payload's own version
  // marker no longer travels inside a synced skill dir - read the sync-owned
  // stamp first (authoritative: present-but-malformed/symlinked stays
  // "unknown", never falls through), and only when the stamp is genuinely
  // ABSENT fall back to the legacy nested marker so an already-onboarded host
  // (synced by a pre-restructure bee) still resolves a real version, applies,
  // and gets the stamp written for every onboard after that.
  const stampPath = path.join(targetRoot, SKILLS_VERSION_STAMP);
  const stampPresent = Boolean(lstatIfExists(stampPath));
  const installedVersion =
    target.mode === "noop"
      ? sourceVersion
      : stampPresent
        ? readManifestVersionStrict(stampPath)
        : readVersionStrict(
            path.join(installedHive, "templates", "lib", "state.mjs"),
            installedTreeExists,
            { componentRoot: targetRoot }, // lstat every component inside the managed target (review P1-2)
          );
  target.versions = {
    source: versionLabel(sourceVersion),
    host_helpers: versionLabel(hostVersion),
    installed_skills: versionLabel(installedVersion),
  };

  const unknowns = [
    ["source", sourceVersion],
    ["host_helpers", hostVersion],
    ["installed_skills", installedVersion],
  ]
    .filter(([, v]) => v.state === "unknown")
    .map(([name]) => name);
  if (unknowns.length > 0) {
    target.blocked = {
      status: "blocked_downgrade",
      reason: `version unresolvable for ${unknowns.join(", ")}: tree exists but its version cannot be read - refusing (never forceable)`,
      forceable: false,
    };
    return target;
  }
  const older = [];
  if (hostVersion.state === "resolved" && compareVersions(sourceVersion.value, hostVersion.value) < 0) {
    older.push(`host_helpers ${hostVersion.value}`);
  }
  if (
    installedVersion.state === "resolved" &&
    compareVersions(sourceVersion.value, installedVersion.value) < 0
  ) {
    older.push(`installed_skills ${installedVersion.value}`);
  }
  if (older.length > 0) {
    // --force-downgrade may override ONLY when all three versions resolved
    // numeric (D3): absent/unknown trees are resolution states, not versions.
    const allNumeric = [sourceVersion, hostVersion, installedVersion].every(
      (v) => v.state === "resolved",
    );
    target.blocked = {
      status: "blocked_downgrade",
      reason: `source ${sourceVersion.value} is older than ${older.join(" and ")}${
        allNumeric ? " - refusing (--force-downgrade overrides after review)" : " - refusing (not forceable: not all versions resolved numeric)"
      }`,
      forceable: allNumeric,
    };
  }

  if (target.mode === "sync" || target.mode === "fresh") {
    if (!target.blocked || target.blocked.forceable) {
      // D2 forced-apply transparency, per target: a forceable blocked target
      // still carries its computed items BEFORE any --force-downgrade.
      // `target` on every item names the root it belongs to; `path` stays
      // target_root-relative (scope semantics unchanged).
      target.items = computeSkillItems(sourceRoot, targetRoot, runtimeForTargetKind(kind)).map((item) => ({
        ...item,
        target: kind,
      }));
    }
  }
  return target;
}

// Blocked-first aggregation across targets (D5): ANY blocked target blocks the
// whole stage; the aggregate is forceable only when EVERY blocked target is
// forceable (a refused apply stays all-or-nothing, zero mutations anywhere).
// status/versions surface the first blocked target in stable target order;
// reason names every blocked target.
function aggregateSkillBlocked(targets) {
  const blockedTargets = targets.filter((t) => t.blocked);
  if (blockedTargets.length === 0) {
    return null;
  }
  const reasons = blockedTargets.map((t) =>
    blockedTargets.length > 1 || targets.length > 1
      ? `[${t.kind}] ${t.blocked.reason}`
      : t.blocked.reason,
  );
  return {
    status: blockedTargets[0].blocked.status,
    reason: reasons.join("; "),
    forceable: blockedTargets.every((t) => t.blocked.forceable),
    versions: blockedTargets[0].versions,
  };
}

// Target-independent runtime-lib downgrade guard (VER-02..06). computePlan
// step 3 vendors the running script's lib (copy_lib -> .bee/bin/lib) and
// helpers (copy_helper -> .bee/bin, bee.mjs itself included) into the host by
// byte-diff. This guard stays target-independent: runtime safety must not
// depend on which projection targets exist or how an individual target's
// resolution path evolves. It returns a blocked-first downgrade block that the
// whole-apply abort honors with zero mutation (fe6593c0; SPEC VER-02..06).
// runs per non-self_skip target; when every target self_skips it never fires.
// This guard is target-independent (it compares the running lib source against
// the installed .bee/bin/lib) so it fires under self_skip too, returning a
// blocked-first downgrade block that the whole-apply abort honors with zero
// mutation (fe6593c0; SPEC VER-02..06).
function hostLibDowngradeBlock(sourceVersion, hostVersion) {
  // VER-04: a fully absent runtime lib is a fresh install, never a block.
  if (hostVersion.state === "absent") return null;
  const versions = {
    source: versionLabel(sourceVersion),
    host_helpers: versionLabel(hostVersion),
    installed_skills: versionLabel(hostVersion),
  };
  // VER-03: the runtime lib exists but its version cannot be read (or the
  // running source's own version is unreadable) - refuse, never forceable: an
  // unresolved runtime must never be overwritten by an older/unknown source.
  if (sourceVersion.state !== "resolved" || hostVersion.state !== "resolved") {
    return {
      status: "blocked_downgrade",
      reason: `runtime lib .bee/bin/lib version unresolvable (source ${versionLabel(
        sourceVersion,
      )}, runtime ${versionLabel(hostVersion)}) - refusing (never forceable)`,
      forceable: false,
      versions,
    };
  }
  // VER-02/05: both resolved - block only a true downgrade; forceable because
  // both identities are trusted numeric (--force-downgrade overrides after review).
  if (compareVersions(sourceVersion.value, hostVersion.value) < 0) {
    return {
      status: "blocked_downgrade",
      reason: `source ${sourceVersion.value} is older than the installed runtime lib .bee/bin/lib ${hostVersion.value} - refusing (--force-downgrade overrides after review)`,
      forceable: true,
      versions,
    };
  }
  return null;
}

// Legacy-global version-parity refresh (installer-version-parity-1-3-1).
// Field report: a repo on the current bee release still loads a pre-1.0
// ~/.claude/skills/bee-* global install alongside its per-project copy, so the
// user sees two conflicting bee versions. Since per-project sync became the
// default the legacy global root is only touched under --global-skills, so its
// stale copies never update. WITHOUT the flag this best-effort pass refreshes,
// IN PLACE, every MANAGED skill (the exact source name set the sync manages)
// whose directory ALREADY EXISTS under the legacy global root to current source
// content. It NEVER creates a global copy that is absent, NEVER deletes anything
// (no remove pass), and never touches non-managed dirs (bee-custom, foreign -
// they surface only as computeSkillItems remove_skill items, which are dropped).
// It is strictly additive: it never participates in blocked-first aggregation,
// so an unrefreshable global never refuses the primary repo sync; and it is
// skipped entirely when the running source IS the legacy global root (a global
// install / legacy_global source, or a self-onboard from there) so it can never
// self-copy. Under --global-skills it does not run at all - the global root is a
// fully managed target there, semantics unchanged. Fully read-only.
function computeLegacyGlobalRefresh({ sourceRoot, realSource, realRepo, sourceVersion }) {
  const globalRoot = skillsTargetRoot();
  const out = { target_root: globalRoot, items: [] };
  if (!fs.existsSync(globalRoot)) {
    return out; // nothing installed there -> never create
  }
  let realGlobal;
  try {
    realGlobal = fs.realpathSync(globalRoot);
  } catch {
    return out;
  }
  // Never self-copy: the running source tree IS the legacy global root.
  if (realSource === realGlobal) {
    return out;
  }
  // Fail-closed overlaps (same spirit as the global target's own guards): a
  // repo inside/containing the global root, or a source overlapping it, is
  // never refreshed by this pass.
  if (
    realRepo === realGlobal ||
    realRepo.startsWith(realGlobal + path.sep) ||
    realGlobal.startsWith(realRepo + path.sep) ||
    realGlobal.startsWith(realSource + path.sep) ||
    realSource.startsWith(realGlobal + path.sep)
  ) {
    return out;
  }
  // Downgrade guard: never overwrite a RESOLVED-newer global copy with older
  // source. An absent/unknown installed version proceeds (parity intent: bring
  // pre-1.0 copies, whose marker may be unreadable, up to current).
  // packages-restructure deviation (SKILLS_VERSION_STAMP doc comment): same
  // stamp-first, legacy-marker-fallback read as computeSkillSyncTarget above
  // - a refreshed legacy global root gets the stamp written on its own next
  // ordinary sync (once --global-skills is used) or keeps resolving via the
  // legacy marker here in the meantime.
  const globalStampPath = path.join(globalRoot, SKILLS_VERSION_STAMP);
  const installedVersion = lstatIfExists(globalStampPath)
    ? readManifestVersionStrict(globalStampPath)
    : readVersionStrict(
        path.join(globalRoot, "bee-hive", "templates", "lib", "state.mjs"),
        true,
        { componentRoot: globalRoot },
      );
  if (
    sourceVersion.state === "resolved" &&
    installedVersion.state === "resolved" &&
    compareVersions(sourceVersion.value, installedVersion.value) < 0
  ) {
    return out; // source older than the installed global -> skip, never downgrade
  }
  // Scope the whole pass to bee-* dirs that ALREADY EXIST under the global root.
  // computeSkillItems() carries every symlink/alias safety check; we drop its
  // remove pass (never delete) and its create case (never create an absent
  // dir), relabel a drifted managed dir as refresh_legacy_global_skill, and keep
  // any per-skill block as a loud skip.
  for (const item of computeSkillItems(sourceRoot, globalRoot, "claude")) {
    if (item.action === "remove_skill") {
      continue; // never delete from the legacy global root
    }
    const st = lstatIfExists(path.join(globalRoot, item.skill));
    if (!st) {
      continue; // skill not present in the legacy global -> never create, never report
    }
    if (item.action === "sync_skill") {
      if (st.isSymbolicLink() || !st.isDirectory()) {
        continue; // present but not a plain dir -> never create/replace
      }
      out.items.push({
        ...item,
        action: "refresh_legacy_global_skill",
        target: "legacy-global",
        scope: "installed",
      });
      continue;
    }
    // blocked_symlink / blocked_alias on a present entry: keep, tagged, so it is
    // reported and skipped loudly at apply time.
    out.items.push({ ...item, target: "legacy-global" });
  }
  return out;
}

// D2 resolution over ALL sync targets. Fully read-only.
function computeSkillSync(repoRoot, { globalSkills = false } = {}) {
  const sourceRoot = path.dirname(HIVE_DIR);
  const targetSpecs = skillSyncTargets(repoRoot, { globalSkills });
  const result = {
    source_root: sourceRoot,
    targets: [],
    blocked: null, // blocked-first aggregate: { status, reason, forceable, versions }
  };

  const blockAll = (reason, status = "blocked_no_source") => {
    const blocked = { status, reason, forceable: false };
    result.targets = targetSpecs.map(({ kind, target_root }) => ({
      kind,
      target_root,
      mode: null,
      versions: unknownVersionsTriple(),
      blocked: { ...blocked },
      items: [],
    }));
    result.blocked = { ...blocked, versions: unknownVersionsTriple() };
    return result;
  };

  // Identity anchor (F2): the source is authoritative only if the running
  // script's own skill dir IS <sourceRoot>/bee-hive by realpath. Structural,
  // target-independent: a failure blocks every target before resolution.
  let identityOk = false;
  try {
    identityOk =
      fs.realpathSync(HIVE_DIR) === fs.realpathSync(path.join(sourceRoot, "bee-hive"));
  } catch {
    identityOk = false;
  }
  if (!identityOk) {
    return blockAll(
      "no authoritative skill source: the running script's tree failed the bee-hive realpath identity check",
    );
  }

  // Whole-tree marker-grammar gate (D9): a malformed marker anywhere refuses the
  // ENTIRE apply with zero writes, BEFORE any per-target resolution or mutation.
  const markerErrors = validateSkillTreeMarkers(sourceRoot);
  if (markerErrors.length > 0) {
    return blockAll(
      `skill source markers are malformed - refusing to render, zero writes: ${markerErrors.join("; ")}`,
      "blocked_render",
    );
  }

  const realSource = fs.realpathSync(sourceRoot);
  let realRepo;
  try {
    realRepo = fs.realpathSync(repoRoot);
  } catch {
    realRepo = path.resolve(repoRoot);
  }

  // Shared version resolutions (D3): source and host helpers are per-run, the
  // installed tree is per target (resolved inside computeSkillSyncTarget).
  const sourceVersion = readVersionStrict(
    path.join(TEMPLATES_LIB_DIR, "state.mjs"),
    true, // the running script's tree exists by definition
  );
  const hostStateFile = path.join(repoRoot, ".bee", "bin", "lib", "state.mjs");
  const hostVersion = readVersionStrict(hostStateFile, fs.existsSync(hostStateFile));

  for (const { kind, target_root } of targetSpecs) {
    result.targets.push(
      computeSkillSyncTarget({
        realRepo,
        sourceRoot,
        realSource,
        sourceVersion,
        hostVersion,
        kind,
        targetRoot: target_root,
      }),
    );
  }
  result.blocked = aggregateSkillBlocked(result.targets);
  // Fill the self-onboard gap (VER-02..06): when every target self_skipped,
  // aggregation finds no block, yet copy_lib/copy_helper (computePlan step 3)
  // would still downgrade .bee/bin. A target-independent runtime-lib downgrade
  // blocks the WHOLE apply here so the existing applyPlan abort refuses with
  // zero mutation. Blocked-first: only ever FILLS a genuine gap, never
  // overrides a block already found by the per-target host_helpers check - so
  // ordinary hosts are unchanged and no forceable-combination ambiguity arises.
  if (!result.blocked) {
    const libBlocked = hostLibDowngradeBlock(sourceVersion, hostVersion);
    if (libBlocked) result.blocked = libBlocked;
  }
  // Legacy-global version-parity refresh (installer-version-parity-1-3-1): only
  // WITHOUT --global-skills (with the flag the global root is already a fully
  // managed target). Strictly additive and never part of blocked-first
  // aggregation - see computeLegacyGlobalRefresh.
  result.legacyRefresh = globalSkills
    ? null
    : computeLegacyGlobalRefresh({ sourceRoot, realSource, realRepo, sourceVersion });
  return result;
}

// Unpredictable temp names inside the managed namespace (F6): a predictable
// <file>.tmp under ~/.claude/skills would be a symlink-swap target.
function writeFileAtomicRandom(filePath, buffer) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const tmp = `${filePath}.${crypto.randomBytes(8).toString("hex")}.tmp`;
  fs.writeFileSync(tmp, buffer);
  fs.renameSync(tmp, filePath);
}

// Mirror one bee-* skill dir into the target (D4/D5). Re-verifies the symlink
// policy at apply time so plan-to-apply races fail closed.
function applySyncSkill(sourceRoot, targetRoot, name, runtime) {
  const renderSource = (buf) => renderSkillBytes(buf, runtime);
  const sourceDir = path.join(sourceRoot, name);
  const sourceStat = lstatIfExists(sourceDir);
  if (!sourceStat || sourceStat.isSymbolicLink() || !sourceStat.isDirectory()) {
    return { blocked: `source ${name} is not a plain directory - skipped` };
  }
  const sourceWalk = walkSkillTree(sourceDir, renderSource);
  if (sourceWalk.blocked) {
    return {
      blocked: `source ${name} contains a ${sourceWalk.blocked.reason} at ${sourceWalk.blocked.path} - skipped`,
    };
  }
  // Apply-time alias recheck (review P1-5): plan-to-apply races fail closed.
  if (detectAliasCollisions(new Set([name]), targetRoot).has(name)) {
    return {
      blocked: `installed ${name} shares one physical entry with a differently-named bee-* entry (case-insensitive alias) - skipped, never sync-then-delete`,
    };
  }
  const targetDir = path.join(targetRoot, name);
  let targetStat = lstatIfExists(targetDir);
  if (targetStat && targetStat.isSymbolicLink()) {
    return {
      blocked: `installed ${name} is a symlink (plausibly a live checkout) - skipped, never written through or unlinked`,
    };
  }
  let targetWalk = { files: new Map(), dirs: [] };
  if (targetStat && targetStat.isDirectory()) {
    const walked = walkSkillTree(targetDir);
    if (walked.blocked) {
      return {
        blocked: `installed ${name} contains a ${walked.blocked.reason} at ${walked.blocked.path} - skipped, nothing inside it written or deleted`,
      };
    }
    targetWalk = walked;
  } else if (targetStat) {
    // non-link type collision: remove the entry, write the source shape
    fs.rmSync(targetDir, { force: true });
    targetStat = null;
  }
  const nestedAlias = detectNestedAlias(targetDir, sourceWalk, targetWalk);
  if (nestedAlias) {
    return {
      blocked: `installed ${name} has nested entries ${nestedAlias.a} and ${nestedAlias.b} resolving to one physical entry (case-insensitive alias) - skipped, never sync-then-delete`,
    };
  }
  fs.mkdirSync(targetDir, { recursive: true });
  const sourceDirSet = new Set(sourceWalk.dirs);
  // Phase 1 - cleanup, deepest-first, BEFORE materializing anything (review
  // P1-3: the old order ran cleanup from the stale target snapshot AFTER
  // writing the source shape, deleting freshly written content on dir<->file
  // transitions). Every removal below targets a pre-write snapshot entry that
  // is stale or of the opposite type; source-shaped paths are never touched
  // (a source file's ancestors are all source dirs, so no stale dir can
  // contain a kept file).
  const staleEntries = [
    ...[...targetWalk.files.keys()]
      .filter((rel) => !sourceWalk.files.has(rel))
      .map((rel) => ({ rel, recursive: false })),
    ...targetWalk.dirs
      .filter((rel) => !sourceDirSet.has(rel))
      .map((rel) => ({ rel, recursive: true })),
  ].sort((a, b) => b.rel.split("/").length - a.rel.split("/").length);
  for (const { rel, recursive } of staleEntries) {
    fs.rmSync(path.join(targetDir, ...rel.split("/")), { recursive, force: true });
  }
  // Phase 2 - materialize the source shape onto the cleaned target: every
  // remaining target entry now matches its source type, so a plain mkdir +
  // atomic write per path suffices.
  for (const rel of sourceWalk.dirs) {
    fs.mkdirSync(path.join(targetDir, ...rel.split("/")), { recursive: true });
  }
  for (const [rel, hash] of sourceWalk.files) {
    if (targetWalk.files.get(rel) === hash) {
      continue; // already byte-identical; cleanup above never touches source-shaped rels
    }
    writeFileAtomicRandom(
      path.join(targetDir, ...rel.split("/")),
      renderSource(fs.readFileSync(path.join(sourceDir, ...rel.split("/")))),
    );
  }
  return { blocked: null };
}

// Remove one bee-* skill dir from the target (D4). The /^bee-/ recheck is a
// structural backstop; the iteration domain already guarantees it.
function applyRemoveSkill(targetRoot, name) {
  if (!SKILL_DIR_RE.test(name)) {
    return { blocked: `refusing to remove ${name}: outside the bee-* namespace` };
  }
  const targetDir = path.join(targetRoot, name);
  const st = lstatIfExists(targetDir);
  if (!st) {
    return { blocked: null }; // already gone
  }
  if (st.isSymbolicLink()) {
    return {
      blocked: `installed ${name} is a symlink (plausibly a live checkout) - skipped, never unlinked`,
    };
  }
  if (!st.isDirectory()) {
    return { blocked: `installed ${name} is not a directory - outside the deletion domain, skipped` };
  }
  // Apply-time alias recheck (review P1-5): never delete a physical entry that
  // another bee-* name (e.g. the sync pass's fresh output) also resolves to.
  if (detectAliasCollisions(new Set([name]), targetRoot).has(name)) {
    return {
      blocked: `installed ${name} shares one physical entry with a differently-named bee-* entry (case-insensitive alias) - skipped, never sync-then-delete`,
    };
  }
  const walked = walkSkillTree(targetDir);
  if (walked.blocked) {
    return {
      blocked: `installed ${name} contains a ${walked.blocked.reason} at ${walked.blocked.path} - skipped, nothing deleted`,
    };
  }
  fs.rmSync(targetDir, { recursive: true, force: true });
  return { blocked: null };
}

// ---------- template sources ----------

function listTemplateHelpers() {
  if (!fs.existsSync(TEMPLATES_DIR)) {
    return [];
  }
  return fs
    .readdirSync(TEMPLATES_DIR, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".mjs"))
    .map((entry) => entry.name)
    .sort();
}

// ---------- retired helper shims (D2, shim-retire) --------------------------
// bee.mjs <group> <verb> is the sole shipped CLI (decision bbc6bcea, D1); the
// 9 per-group shims below were deleted from packages/bee/ in
// shim-retire-1. listTemplateHelpers() is name-agnostic (readdir), so it
// naturally stops copying them - but nothing ever deletes a copy a host
// already has vendored into its own .bee/bin/, so a host upgrading through
// this version would keep the dead shims forever without an explicit removal
// pass. RETIRED_HELPERS is that removal list; only removal (never copy) uses
// it going forward.
const RETIRED_HELPERS = [
  "bee_status.mjs",
  "bee_cells.mjs",
  "bee_reservations.mjs",
  "bee_decisions.mjs",
  "bee_state.mjs",
  "bee_backlog.mjs",
  "bee_capture.mjs",
  "bee_reviews.mjs",
  "bee_feedback.mjs",
];

function listTemplateLibModules() {
  if (!fs.existsSync(TEMPLATES_LIB_DIR)) {
    return [];
  }
  return fs
    .readdirSync(TEMPLATES_LIB_DIR, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".mjs"))
    .map((entry) => entry.name)
    .sort();
}

function listTemplateStatusline() {
  if (!fs.existsSync(TEMPLATES_STATUSLINE_DIR)) {
    return [];
  }
  return fs
    .readdirSync(TEMPLATES_STATUSLINE_DIR, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort();
}

// A repo opts in to the vendored statusline by already pointing its
// .claude/settings.json statusLine at the project-level script. Onboarding
// then keeps the pair current; it never creates the opt-in, never touches
// settings.json in this stage, and any unparseable/unexpected settings shape
// simply means "not opted in" (fail-safe, never a throw).
function statuslineOptIn(repoRoot) {
  const settings = readJsonIfExists(path.join(repoRoot, ".claude", "settings.json"));
  const command =
    settings && settings.statusLine && typeof settings.statusLine === "object"
      ? settings.statusLine.command
      : null;
  if (typeof command !== "string" || !command.includes(".claude/statusline-command.sh")) {
    return false;
  }
  // Project-level references only: a $CLAUDE_PROJECT_DIR-anchored path
  // (the variable must anchor the script path itself, not merely appear
  // anywhere in the command — review P2-1), or a bare repo-relative
  // ".claude/…" (nothing before the dot). A user-level "~/.claude/…" or
  // "/home/x/.claude/…" contains the same substring but is NOT this repo's
  // script — vendoring there would shadow the user's own copy.
  return (
    /\$\{?CLAUDE_PROJECT_DIR[^"'\s{}]*\}?\/\.claude\/statusline-command\.sh/.test(command) ||
    /(^|[\s"'=(])\.claude\/statusline-command\.sh/.test(command)
  );
}

// ---------- bee agent files (config-rendered, AO10-safe flat sync) ----------
// Pinned agent types (advisor-and-orchestration, Slice 3B, AO5/AO10/AO11):
// each type's frontmatter `model:` is rendered from the HOST repo's own
// models.claude tier config at sync time, never hand-pinned - a static
// `model: sonnet` in the template output would drift the moment an owner
// reconfigures a tier (AO5: config is the authority). Sync target is
// <repo>/.claude/agents/bee-*.md, a flat managed-file step of the SAME CLASS
// as the AGENTS.md block / settings.json hook merge above - deliberately NOT
// added to REPO_SKILL_TARGETS (AO10): an agents root has no bee-hive
// directory for the three-version onboarding preflight to resolve a version
// from, so joining it there would resolve "unknown" and brick onboarding
// non-forceably on every host. Codex gets no agent files at all (AO11): Codex
// has no per-agent model selection (DEFAULT_MODELS.codex is all-null by
// design, packages/bee/lib/state.mjs), so a `model:` pin would be a no-op file
// implying an enforcement that does not exist for that runtime.
function listTemplateAgents() {
  if (!fs.existsSync(TEMPLATES_AGENTS_DIR)) {
    return [];
  }
  return fs
    .readdirSync(TEMPLATES_AGENTS_DIR, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".md.tmpl"))
    .map((entry) => entry.name)
    .sort();
}

// gather <- generation, extract <- extraction, review <- review (plan.md
// Slice 3B item 1). Every template basename here must have an entry; a
// template with no mapping is a planning defect, not a silent skip.
const AGENT_TIER_BY_NAME = {
  "bee-gather": "generation",
  "bee-extract": "extraction",
  "bee-review": "review",
};

// Deliberately duplicated, not imported: this script never import-depends on
// packages/bee/lib/state.mjs's exports (see the STALE_ADVISOR_KEY_WARNING
// comment below, same discipline) - the skill-sync test fixture's fake
// state.mjs is minimal by design, and importing modelForTier/resolveTier here
// would break against it. AGENT_TIER_DEFAULTS_CLAUDE is text-pinned against
// state.mjs's DEFAULT_MODELS.claude by test_onboard_bee.mjs's no-drift check
// (same pattern as the COMMAND_KEYS / STALE_ADVISOR_KEY_WARNING checks).
const AGENT_TIER_DEFAULTS_CLAUDE = { extraction: "haiku", generation: "sonnet", review: "opus" };

// Mirrors packages/bee/lib/state.mjs normalizeTierValue (state.mjs:159-175),
// narrowed to what agent-file rendering needs: a resolved model NAME string,
// `undefined` for "no override" (default stands, same as normalizeTierValue),
// `null` for an EXPLICIT null override, or the CLI_TIER_SENTINEL for a
// cli-shaped override. The three non-string outcomes are kept distinct
// (rather than all collapsing to null) because resolveAgentTierModel's
// review->generation fallback fires ONLY on an explicit null, exactly like
// resolveTier (state.mjs:1146-1148) - a cli-shaped review must resolve to
// "no model" WITHOUT falling back to generation's model.
const CLI_TIER_SENTINEL = Symbol("cli-tier");
function normalizeAgentTierValueLocal(value) {
  if (typeof value === "string" && value.trim()) return value.trim();
  if (value === null) return null;
  if (value && typeof value === "object" && !Array.isArray(value)) {
    if (value.kind === "cli") return CLI_TIER_SENTINEL; // no model name to pin
    if (value.kind === undefined && typeof value.model === "string" && value.model.trim()) {
      return value.model.trim();
    }
  }
  return undefined; // invalid shape - default for that slot stands
}

// Resolve one tier -> model name for a TARGET repo's own .bee/config.json,
// claude runtime only (Codex gets no agent files - AO11). Mirrors
// resolveTier's tier-only contract (state.mjs:1140-1173): a string or
// {model,...} override resolves to that name; an explicit null or cli-shaped
// value resolves to null (no agent file - AO5: a slot naming no real model
// has nothing honest to pin); an unset slot falls back to
// AGENT_TIER_DEFAULTS_CLAUDE, and an unset `review` additionally falls back
// to the resolved `generation` value (decision 0021) exactly as resolveTier
// does - but only when config is silent; an EXPLICIT `review: null` is
// honored as "no agent file", never coerced back to generation.
function resolveAgentTierModel(repoRoot, tier) {
  const config = readJsonIfExists(path.join(repoRoot, ".bee", "config.json"));
  const rawClaude =
    config &&
    typeof config === "object" &&
    !Array.isArray(config) &&
    config.models &&
    typeof config.models === "object" &&
    !Array.isArray(config.models)
      ? config.models.claude
      : null;
  const resolved = { ...AGENT_TIER_DEFAULTS_CLAUDE };
  if (rawClaude && typeof rawClaude === "object" && !Array.isArray(rawClaude)) {
    for (const slot of Object.keys(AGENT_TIER_DEFAULTS_CLAUDE)) {
      const value = normalizeAgentTierValueLocal(rawClaude[slot]);
      if (value !== undefined) {
        resolved[slot] = value;
      }
    }
  }
  let value = resolved[tier];
  if (value === null && tier === "review") {
    value = resolved.generation; // explicit null only, exactly like resolveTier
  }
  return typeof value === "string" ? value : null;
}

function renderAgentTemplate(agentName, model) {
  const source = fs.readFileSync(path.join(TEMPLATES_AGENTS_DIR, `${agentName}.md.tmpl`), "utf8");
  return source.split("{{TIER_MODEL}}").join(model);
}

// Plan the flat .claude/agents/bee-*.md sync for one repo: byte-compare each
// rendered template against the target, same discipline as copy_helper /
// copy_lib above. A tier that resolves to null (cli-shaped or explicitly
// unset) skips the render and, if a stale copy exists from a prior config,
// removes it - an agent type must name a real model (must_haves: "a
// cli-shaped or null tier slot skips (and removes) its agent file").
function computeAgentFilePlan(repoRoot) {
  const items = [];
  for (const tmplName of listTemplateAgents()) {
    const agentName = tmplName.replace(/\.md\.tmpl$/, "");
    const tier = AGENT_TIER_BY_NAME[agentName];
    const relPath = `.claude/agents/${agentName}.md`;
    const target = path.join(repoRoot, ".claude", "agents", `${agentName}.md`);
    const model = tier ? resolveAgentTierModel(repoRoot, tier) : null;
    if (model) {
      const rendered = renderAgentTemplate(agentName, model);
      if (readTextIfExists(target) !== rendered) {
        items.push({ action: "sync_agent_file", path: relPath, agent: agentName });
      }
    } else if (fs.existsSync(target)) {
      items.push({ action: "remove_agent_file", path: relPath, agent: agentName });
    }
  }
  return items;
}

// The sync's own version marker (same class as the settings.json hook merge
// marker), recorded in .bee/onboarding.json as a sibling of `managed`:
// {bee_version, files, rendered_from: {tier: model}}. Computed post-apply
// (called after the apply loop, once .claude/agents/ reflects the new
// state) so `files` names what is ACTUALLY present, not merely planned.
// Codex asymmetry (AO11) is recorded inline, never a separate file.
function computeAgentsSyncRecord(repoRoot, beeVersion) {
  const files = [];
  const renderedFrom = {};
  for (const tmplName of listTemplateAgents()) {
    const agentName = tmplName.replace(/\.md\.tmpl$/, "");
    const tier = AGENT_TIER_BY_NAME[agentName];
    const model = tier ? resolveAgentTierModel(repoRoot, tier) : null;
    if (model) {
      files.push(`.claude/agents/${agentName}.md`);
      renderedFrom[tier] = model;
    }
  }
  return {
    bee_version: beeVersion,
    files,
    rendered_from: renderedFrom,
    codex: {
      agents: [],
      note:
        "Codex has no per-agent model selection (DEFAULT_MODELS.codex is all-null by design, " +
        "packages/bee/lib/state.mjs) - tiers are enforced as a read budget + output cap in the " +
        "worker prompt instead. No agent files are rendered under .agents/ (AO11).",
    },
  };
}

function listPluginHooks() {
  if (!fs.existsSync(PLUGIN_HOOKS_DIR)) {
    return [];
  }
  return HOOK_FILENAMES.filter((name) => fs.existsSync(path.join(PLUGIN_HOOKS_DIR, name)));
}

function renderAgentsBlock() {
  const body = readTextIfExists(AGENTS_BLOCK_TEMPLATE).replace(/\s*$/, "");
  return `${MARKER_START}\n${body}\n${MARKER_END}\n`;
}

function renderGitignoreBlock() {
  return `${GITIGNORE_MARKER_START}\n${GITIGNORE_BLOCK_PATTERNS.join("\n")}\n${GITIGNORE_MARKER_END}\n`;
}

// ---------- AGENTS.md merging ----------

function agentsBlockPresent(text) {
  return text.includes(MARKER_START) && text.includes(MARKER_END);
}

function extractAgentsBlock(text) {
  const start = text.indexOf(MARKER_START);
  const end = text.indexOf(MARKER_END);
  if (start === -1 || end === -1 || end < start) {
    return null;
  }
  return `${text.slice(start, end + MARKER_END.length)}\n`;
}

function mergeAgentsContent(existing, renderedBlock) {
  if (!existing.trim()) {
    return { text: renderedBlock, status: "created" };
  }
  if (agentsBlockPresent(existing)) {
    const start = existing.indexOf(MARKER_START);
    let end = existing.indexOf(MARKER_END) + MARKER_END.length;
    if (existing[end] === "\n") {
      end += 1;
    }
    const updated = existing.slice(0, start) + renderedBlock + existing.slice(end);
    return { text: `${updated.replace(/\s*$/, "")}\n`, status: "updated" };
  }
  return {
    text: `${existing.replace(/\s*$/, "")}\n\n${renderedBlock}`,
    status: "appended",
  };
}

// ---------- .gitignore merging (decision D1) ----------
//
// Same marker-splice pattern as mergeAgentsContent, with '#'-comment markers
// instead of HTML comments. The append path fixes the exact bug class this
// feature was filed over: `${existing.replace(/\s*$/, "")}\n\n${block}` always
// inserts a real blank-line separator, even when `existing` has no trailing
// newline at all - so two gitignore patterns can never be silently merged
// onto one line the way the corrupt `.bee/feedback-digest.json.spikes/` entry
// was.
//
// Review P2 (test-coverage) hardening: marker detection is whole-line
// anchored (GITIGNORE_START_RE / GITIGNORE_END_RE above), and the update path
// only ever touches the bytes between the two marker lines - everything
// before GITIGNORE_START_RE's match and everything after GITIGNORE_END_RE's
// match (including its own trailing newline) is copied through byte-for-byte,
// never re-normalized.

function gitignoreBlockPresent(text) {
  return GITIGNORE_START_RE.test(text) && GITIGNORE_END_RE.test(text);
}

function findGitignoreMarkers(text) {
  const startMatch = GITIGNORE_START_RE.exec(text);
  const endMatch = GITIGNORE_END_RE.exec(text);
  if (!startMatch || !endMatch || endMatch.index < startMatch.index) {
    return null;
  }
  return { start: startMatch.index, end: endMatch.index + endMatch[0].length };
}

function extractGitignoreBlock(text) {
  const markers = findGitignoreMarkers(text);
  if (!markers) {
    return null;
  }
  return `${text.slice(markers.start, markers.end)}\n`;
}

// Drift comparison only (review P3 / CRLF): a CRLF-saving editor must not
// cause a perpetual update_gitignore_block loop, so \r\n is normalized to \n
// ONLY for this equality check. Writes below always stay LF - normalizing the
// comparison never changes what gets written to disk.
function normalizeGitignoreForCompare(text) {
  return (text || "").replace(/\r\n/g, "\n");
}

function mergeGitignoreContent(existing, renderedBlock) {
  if (!existing.trim()) {
    return { text: renderedBlock, status: "created" };
  }
  const markers = findGitignoreMarkers(existing);
  if (markers) {
    let end = markers.end;
    if (existing[end] === "\n") {
      end += 1;
    }
    const updated = existing.slice(0, markers.start) + renderedBlock + existing.slice(end);
    return { text: updated, status: "updated" };
  }
  return {
    text: `${existing.replace(/\s*$/, "")}\n\n${renderedBlock}`,
    status: "appended",
  };
}

// ---------- AGENTS.md minimal header (decision D4) ----------
//
// Propose-only Q1 upgrade: when the region outside the BEE markers carries no
// prose, onboarding proposes a minimal header. The any-prose test is the
// mechanical stand-in for the semantic "does this answer what is this
// project?" check - conservative, it never fires on existing prose, and
// whitespace-only or comment-only lines (including lines inside a multi-line
// HTML comment) never count as prose. Existing user content is never touched.

const HEADER_POINTER_CANDIDATES = [
  "README.md",
  "docs/specs/system-overview.md",
  "docs/specs/reading-map.md",
];

function hasProseOutsideBlock(text) {
  let outside = text;
  const start = outside.indexOf(MARKER_START);
  const end = outside.indexOf(MARKER_END);
  if (start !== -1 && end !== -1 && end >= start) {
    outside = outside.slice(0, start) + outside.slice(end + MARKER_END.length);
  }
  // Strip closed HTML comments (multi-line aware). An unclosed comment stays
  // in place and counts as prose - conservative: never propose over content.
  const stripped = outside.replace(/<!--[\s\S]*?-->/g, "");
  return stripped.split("\n").some((line) => line.trim() !== "");
}

function composeAgentsHeader(repoRoot) {
  // Mechanically provable parts only (never-invent): the repo folder name as
  // title, one loud fill-me gap for the project one-liner, and pointer lines
  // only to files that actually exist at plan time.
  const lines = [
    `# ${path.basename(repoRoot)}`,
    "",
    "<!-- [unknown] one-line project description - replace me -->",
  ];
  const pointers = HEADER_POINTER_CANDIDATES.filter((rel) =>
    fs.existsSync(path.join(repoRoot, ...rel.split("/"))),
  );
  if (pointers.length > 0) {
    lines.push("");
    for (const rel of pointers) {
      lines.push(`- ${rel}`);
    }
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

// ---------- repo hooks (.claude/settings.json) ----------

function repoHookCommand(fileName) {
  return `node "$CLAUDE_PROJECT_DIR"/.bee/bin/hooks/${fileName}`;
}

function renderRepoHookEntries() {
  const entry = (fileName) => ({ type: "command", command: repoHookCommand(fileName) });
  return {
    SessionStart: [
      { matcher: "startup|resume|clear|compact", hooks: [entry("bee-session-init.mjs")] },
    ],
    UserPromptSubmit: [{ hooks: [entry("bee-prompt-context.mjs")] }],
    PreToolUse: [
      { matcher: "Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion", hooks: [entry("bee-write-guard.mjs")] },
      { matcher: "Agent|Task", hooks: [entry("bee-model-guard.mjs")] },
    ],
    PostToolUse: [
      { matcher: "update_plan|TaskCreate|TaskUpdate|TodoWrite", hooks: [entry("bee-state-sync.mjs")] },
      { hooks: [entry("bee-tools-logger.mjs")] },
    ],
    SubagentStop: [{ hooks: [entry("bee-state-sync.mjs"), entry("bee-chain-nudge.mjs")] }],
    // PreCompact mirrors the plugin hooks.json (decision 0017): an unflushed
    // capture queue must warn LOUDLY before compaction buries its context.
    PreCompact: [{ hooks: [entry("bee-session-close.mjs")] }],
    Stop: [{ hooks: [entry("bee-state-sync.mjs"), entry("bee-session-close.mjs")] }],
  };
}

function isBeeHookEntry(entry) {
  for (const hook of entry?.hooks || []) {
    if (String(hook?.command || "").includes(".bee/bin/hooks/bee-")) {
      return true;
    }
  }
  return false;
}

function mergeRepoSettings(settingsPath) {
  const existing = readJsonIfExists(settingsPath) || {};
  const hooks = existing.hooks && typeof existing.hooks === "object" ? existing.hooks : {};
  const merged = { ...hooks };
  let changed = false;

  for (const [eventName, entries] of Object.entries(renderRepoHookEntries())) {
    const current = Array.isArray(merged[eventName]) ? merged[eventName] : [];
    const next = [...current.filter((e) => !isBeeHookEntry(e)), ...entries];
    if (JSON.stringify(current) !== JSON.stringify(next)) {
      changed = true;
    }
    merged[eventName] = next;
  }

  return {
    text: `${JSON.stringify({ ...existing, hooks: merged }, null, 2)}\n`,
    changed,
  };
}

// ---------- codex hooks (.codex/hooks.json) ----------
// The Codex projection of the same repo-hook set, mirroring hooks/catalog.mjs
// TARGETS.REPO but with host-repo paths: Codex never sets $CLAUDE_PROJECT_DIR
// (the Claude-only variable above), so every command resolves the git root
// from the session cwd and fails open VISIBLY when there is none. Two pinned
// differences from renderRepoHookEntries(), both from hooks/catalog.mjs:
//   - bee-model-guard.mjs is wired on a DIFFERENT matcher per runtime: Claude
//     guards Agent|Task, Codex guards spawn_agent (the collaboration-spawn tool
//     name Codex exposes through PreToolUse — capability-matrix row D1). Same
//     handler, only the matcher differs (ALLOWED_DIFFERENCES).
//   - each entry carries a statusMessage (Codex TUI shows it while running).

const CODEX_TRANSPORT_DIAGNOSTIC = "bee: hook transport unavailable (no git root)";

// A repo that ships hooks/catalog.mjs IS bee, and that catalog — not this file —
// is the authority for its own .codex/hooks.json (TARGETS.REPO renders
// `"$r"/hooks/<script>`, because bee's hooks live in hooks/). renderCodexHookEntries()
// below is the HOST projection: it points at .bee/bin/hooks/, correct for a host repo
// (which has no hooks/ dir, only the vendored copy) and WRONG for bee itself. Writing
// it into bee's own repo silently clobbers the catalog rendering and turns
// hooks/test_hook_contracts.mjs red on the NEXT run — which is exactly how a release
// broke: the drift check fired, the file was "repaired" by hand, and the next
// self-onboard undid the repair. Two renderers, one generated file; this is the
// self-skip the skill sync already does (mode "self_skip"), keyed on the file that
// makes a repo the catalog's owner rather than on where the script happens to run from.
function repoOwnsHookCatalog(repoRoot) {
  // packages-restructure D2/cell 2: bee's own hooks/ tree moved to
  // packages/bee/hooks/ (mirrors the earlier templates/ -> packages/bee/
  // move). The legacy repo-root hooks/catalog.mjs check stays as an OR
  // fallback so a checkout still on an older bee version (pre-move) is still
  // correctly self-identified — never forceable, just backward-compatible
  // detection.
  return (
    fs.existsSync(path.join(repoRoot, "packages", "bee", "hooks", "catalog.mjs")) ||
    fs.existsSync(path.join(repoRoot, "hooks", "catalog.mjs"))
  );
}

// GH #22 P0-1: does the PASSED --runtime cover Codex? "both" (the default)
// and "codex" do; "claude" does not. Never reads any recorded/sticky state -
// see the --runtime block comment at the top of this file for why.
function runtimeCoversCodex(runtime) {
  return runtime === "codex" || runtime === "both";
}

function codexHookCommand(fileName) {
  return [
    'r="$(git rev-parse --show-toplevel 2>/dev/null)"',
    `[ -n "$r" ] || { echo "${CODEX_TRANSPORT_DIAGNOSTIC}" >&2; exit 0; }`,
    `exec node "$r"/.bee/bin/hooks/${fileName} --source=repo`,
  ].join("\n");
}

// Windows override for this HOST projection (Codex hook schema optional
// `commandWindows` field, run with the session cwd as working dir — no
// `$SHELL -lc`, so the POSIX command above is Windows-broken). A bare
// cwd-relative `node .bee/bin/hooks/<f> --source=repo` (the pre-cell form)
// breaks the instant Codex's session cwd is a NESTED subdirectory of the
// repo (MODULE_NOT_FOUND) — cwd-relative resolution assumes the session cwd
// IS the repo root, which is not guaranteed. codexWindowsBootstrap below
// resolves the real git root first, mirroring the POSIX command's own `git
// rev-parse --show-toplevel` step, so both transports share the same
// cwd-independence guarantee. Mirrors hooks/catalog.mjs's windowsBootstrap
// for TARGETS.REPO (duplicated, not imported — these are two independently
// rendered projections, same as codexHookCommand/repoCommand above); both
// point at the host's vendored .bee/bin/hooks/.
//
// The bootstrap is `node -e SCRIPT <fileName>`: SCRIPT is plain JS using ONLY
// single quotes internally and NO `$`, `%`, or backtick character anywhere —
// cmd.exe and powershell.exe both parse the outer double-quoted `-e`
// argument identically as long as none of those three characters appear
// inside it. `node -e` is cross-platform, so this same string is also
// directly executable on POSIX for the contract test.
//
// SCRIPT resolves the git root via `execSync('git rev-parse
// --show-toplevel')`; on failure (no git root) it exits 0 SILENTLY (POSIX
// parity: no git root = transport unavailable), else spawnSync's the real
// hook (process.execPath, so it runs under the SAME node the bootstrap is
// running under) with stdio 'inherit' (hooks read their JSON payload from
// stdin — this must be inherited, never piped), then process.exit's with
// the child's status (1 if the spawn itself errored).
function codexWindowsBootstrap(fileName) {
  const body = [
    "var cp=require('child_process');",
    "var path=require('path');",
    "var hook=process.argv[1];",
    "var root='';",
    "try{root=cp.execSync('git rev-parse --show-toplevel',{stdio:['ignore','pipe','ignore']}).toString().trim();}catch(e){root='';}",
    "if(!root){process.exit(0);}",
    "var target=path.join(root,'.bee','bin','hooks',hook);",
    "var r=cp.spawnSync(process.execPath,[target,'--source=repo'],{stdio:'inherit'});",
    "if(r.error){process.exit(1);}",
    "process.exit(r.status===null?1:r.status);",
  ].join("");
  return 'node -e "' + body + '" ' + fileName;
}

function codexHookCommandWindows(fileName) {
  return codexWindowsBootstrap(fileName);
}

function renderCodexHookEntries() {
  const entry = (fileName, statusMessage) => ({
    type: "command",
    command: codexHookCommand(fileName),
    commandWindows: codexHookCommandWindows(fileName),
    statusMessage,
  });
  return {
    SessionStart: [
      {
        matcher: "startup|resume|clear|compact",
        hooks: [entry("bee-session-init.mjs", "bee: session bootstrap")],
      },
    ],
    UserPromptSubmit: [{ hooks: [entry("bee-prompt-context.mjs", "bee: phase reminder")] }],
    PreToolUse: [
      {
        matcher: "Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion",
        hooks: [entry("bee-write-guard.mjs", "bee: write guard")],
      },
      {
        // Codex-native spawn guard (codex-native-runtime-v2 D4): Codex exposes
        // agent spawns as tool_name "spawn_agent"; bee-model-guard.mjs runs an
        // isolated Codex branch on the observed envelope. Claude wires the same
        // handler on Agent|Task (renderRepoHookEntries) — matcher differs only.
        matcher: "spawn_agent",
        hooks: [entry("bee-model-guard.mjs", "bee: model-tier guard")],
      },
    ],
    PostToolUse: [
      {
        matcher: "update_plan|TaskCreate|TaskUpdate|TodoWrite",
        hooks: [entry("bee-state-sync.mjs", "bee: state sync")],
      },
      {
        hooks: [entry("bee-tools-logger.mjs", "bee: tools logger")],
      },
    ],
    SubagentStart: [
      {
        hooks: [entry("bee-codex-subagent-audit.mjs", "bee: subagent start audit")],
      },
    ],
    SubagentStop: [
      {
        hooks: [
          entry("bee-state-sync.mjs", "bee: state sync"),
          entry("bee-chain-nudge.mjs", "bee: chain nudge"),
        ],
      },
      {
        hooks: [entry("bee-codex-subagent-audit.mjs", "bee: subagent stop audit")],
      },
    ],
    PreCompact: [{ hooks: [entry("bee-session-close.mjs", "bee: pre-compact flush check")] }],
    Stop: [
      {
        hooks: [
          entry("bee-state-sync.mjs", "bee: state sync"),
          entry("bee-session-close.mjs", "bee: session close check"),
        ],
      },
    ],
  };
}

// A bee entry in a Codex hooks file, in ANY historical transport shape:
// ".bee/bin/hooks/bee-*" (this projection), "$r"/hooks/bee-* (the bee source
// repo's own file), or an old hand-authored "$CLAUDE_PROJECT_DIR" form (dead
// on Codex — the exact MODULE_NOT_FOUND migration case in hooks/catalog.mjs).
// All of them are bee-shipped wiring and must be REPLACED by the canonical
// render, never preserved beside it (a preserved stale twin double-fires
// every event).
function isBeeCodexHookEntry(entry) {
  for (const hook of entry?.hooks || []) {
    if (/hooks\/bee-[a-z-]+\.mjs/.test(String(hook?.command || ""))) {
      return true;
    }
  }
  return false;
}

// Same merge discipline as mergeRepoSettings: non-bee entries are preserved
// verbatim, stale bee entries are replaced, a second apply is a no-op.
function mergeCodexHooks(hooksPath) {
  const existing = readJsonIfExists(hooksPath) || {};
  const hooks = existing.hooks && typeof existing.hooks === "object" ? existing.hooks : {};
  const merged = { ...hooks };
  let changed = false;

  for (const [eventName, entries] of Object.entries(renderCodexHookEntries())) {
    const current = Array.isArray(merged[eventName]) ? merged[eventName] : [];
    const next = [...current.filter((e) => !isBeeCodexHookEntry(e)), ...entries];
    if (JSON.stringify(current) !== JSON.stringify(next)) {
      changed = true;
    }
    merged[eventName] = next;
  }

  return {
    text: `${JSON.stringify({ ...existing, hooks: merged }, null, 2)}\n`,
    changed,
  };
}

// GH #22 P0-1 / advisor R3: a typed preflight for the codex-hybrid write
// path, checked BEFORE applyPlan's main loop ever runs (see the codexHybrid
// block there). Without it, a `.codex` path occupied by a plain file (or a
// `.bee/bin/hooks` path occupied by a plain file) would throw a raw
// ENOTDIR/EEXIST out of writeFileAtomic's mkdirSync deep inside the apply
// loop, AFTER skills may already have been synced - an untyped {error:...}
// escape hatch that reports a broken partial apply as a generic crash rather
// than a named refusal, and (fail-closed requirement) must never let skills
// be reported "applied" without the hooks that make them mechanically
// enforced for Codex. Returns null when both target paths are writable
// (absent or already a plain directory); otherwise returns a blocked
// descriptor shaped exactly like skillSync.blocked ({status, reason,
// forceable}) so main()'s existing blocked-result rendering handles it with
// zero new payload shapes. Never forceable: there is no downgrade to force
// through a filesystem collision.
function codexHookWriteBlocker(repoRoot) {
  const checks = [
    { target: path.join(repoRoot, ".bee", "bin", "hooks"), label: ".bee/bin/hooks" },
    { target: path.join(repoRoot, ".codex"), label: ".codex" },
  ];
  for (const { target, label } of checks) {
    const stat = lstatIfExists(target);
    if (stat && !stat.isDirectory()) {
      return {
        status: "blocked",
        reason:
          `codex hook apply refused: "${label}" exists and is not a directory. FIX: remove or rename it, ` +
          "then retry the hybrid apply, or pass --distribution repo-copy for a plain repo-local install instead.",
        forceable: false,
      };
    }
  }
  return null;
}

// ---------- codex user config status line (machine-level) ----------
// Codex has no per-repo status line and no custom-script support: the only
// surface is [tui] status_line in the USER config (~/.codex/config.toml),
// built from Codex's fixed segment ids. Onboarding guarantees the key EXISTS
// (mirroring the Claude statusline pair's intent: cwd | branch | model
// [effort] | ctx | 5h | 7d + tokens) and never touches a present one — the
// user's own segment choice is preference, not drift. When Codex is not
// installed (no user config file), this stays out entirely: onboarding never
// creates the file for a tool that is not there.

const CODEX_STATUS_LINE_BLOCK = `status_line = ["current-dir", "git-branch", "model-with-reasoning", "context-remaining", "five-hour-limit", "weekly-limit", "used-tokens"]
status_line_use_colors = true
`;

function codexUserConfigPath() {
  return path.join(os.homedir(), ".codex", "config.toml");
}

function codexStatuslineMissing() {
  try {
    const configPath = codexUserConfigPath();
    if (!fs.existsSync(configPath)) {
      return false; // Codex absent — stay out
    }
    return !/^[ \t]*status_line[ \t]*=/m.test(fs.readFileSync(configPath, "utf8"));
  } catch {
    return false; // unreadable — fail open, never block onboarding on it
  }
}

// ---------- standard commands notice (docs/09 item 1, decision D4) ----------

const COMMAND_KEYS = ["setup", "start", "test", "verify"];

function commandsNotices(repoRoot, { firstOnboard = false } = {}) {
  const config = readJsonIfExists(path.join(repoRoot, ".bee", "config.json")) || {};
  const raw = config.commands && typeof config.commands === "object" ? config.commands : {};
  const recorded = COMMAND_KEYS.filter(
    (key) => typeof raw[key] === "string" && raw[key].trim(),
  );
  if (recorded.length > 0) {
    return [];
  }
  // Detection is propose-only (decision D3): candidates ride the notice for
  // the agent to present as one confirmation question. This script never
  // writes detected values to .bee/config.json — only user-confirmed values
  // are written, by the agent.
  let candidates = [];
  try {
    candidates = detectCommands(repoRoot);
  } catch {
    candidates = [];
  }
  if (candidates.length > 0) {
    const proposals = candidates.map((c) => `${c.key}: ${c.value} — ${c.source}`).join("; ");
    return [
      `No standard commands recorded. Detected candidates: ${proposals}. Present them to the user as one pre-filled confirmation question (skippable) and write only confirmed values to .bee/config.json \`commands\` — never write unconfirmed values (D3). They power the session CI status gate.`,
    ];
  }
  const notices = [
    "No standard commands recorded. Ask the user for the host project's setup/start/test/verify commands and write them to .bee/config.json `commands` (skippable — never invent values). They power the session CI status gate.",
  ];
  // P1 / docs/09 item 6: first onboard of a repo without any detectable build →
  // offer the init lane. Planning convention, not a new skill: the first slice
  // is one init cell whose must_haves are the initialization checklist.
  if (firstOnboard) {
    notices.push(
      "Greenfield init lane (docs/09 item 6): this is the first onboard and no build was detected. Offer the init lane before any feature work — the first planning slice is one init cell whose must_haves are exactly: setup succeeds from scratch, one passing test exists, standard commands are recorded in .bee/config.json, and the repo has a clean first commit.",
    );
  }
  return notices;
}

// ---------- stale advisor key notice (D1: advisor mode removed in full) -----
// Warn, never error, when a repo's raw .bee/config.json still carries the
// removed `advisor` key — packages/bee/lib/state.mjs readConfig() tolerates and
// strips it, but the human should still be told to delete it. Same warning
// text as packages/bee/lib/state.mjs STALE_ADVISOR_KEY_WARNING / bee_status.mjs
// so it reads identically wherever it is noticed. Deliberately NOT imported
// from packages/bee/lib/state.mjs (this script only ever text-scans that tree
// for BEE_VERSION — see readBeeVersion — and never import-depends on its
// exports; the skill-sync test fixture's fake state.mjs is minimal by design).
const STALE_ADVISOR_KEY_WARNING =
  "advisor mode was removed in 0.1.23; the top-level advisor key in .bee/config.json is ignored — delete it. (This does not affect the models.<runtime>.advisor slot, which is separate and still valid.)";

function staleAdvisorNotices(repoRoot) {
  const config = readJsonIfExists(path.join(repoRoot, ".bee", "config.json"));
  const hasStaleKey = Boolean(
    config && typeof config === "object" && !Array.isArray(config) && "advisor" in config,
  );
  return hasStaleKey ? [STALE_ADVISOR_KEY_WARNING] : [];
}

// ---------- tracked-paths advisory (review P2, D1) --------------------------
// `.gitignore` is inert for paths that are already git-tracked: if a
// previously-onboarded host still has any GITIGNORE_BLOCK_PATTERNS path
// staged or committed, the managed block goes silent for it and the exact
// git-status churn this feature exists to kill keeps showing up. Advisory
// only - this script NEVER runs `git rm` itself (that rewrites the host's
// index); it only names the count and the exact command for a human to run.
// `execFileSync` with an argv array (never a shell string, never string
// interpolation) so nothing in GITIGNORE_BLOCK_PATTERNS can be read as shell
// syntax. Degrades to silence - no notice, never a crash - when git is
// missing, the directory is not a repo, or git exits nonzero for any other
// reason: the advisory is a nice-to-have, never a blocker.
function trackedGitignorePaths(repoRoot) {
  try {
    const output = execFileSync(
      "git",
      ["ls-files", "-z", "--", ...GITIGNORE_BLOCK_PATTERNS],
      { cwd: repoRoot, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    );
    return output.split("\0").filter(Boolean);
  } catch {
    return [];
  }
}

function trackedPathsNotices(repoRoot) {
  const tracked = trackedGitignorePaths(repoRoot);
  if (tracked.length === 0) {
    return [];
  }
  return [
    `${tracked.length} managed path(s) are still git-tracked; the ignore block cannot ` +
      `silence them — run: git rm -r --cached ${tracked.join(" ")}`,
  ];
}

// ---------- worktree-local coordination migration (msn-18d) ----------
//
// advisor-digest-slice4 binding condition F4: a linked worktree that
// accumulated its OWN .bee/sessions, .bee/claims, .bee/runtime/workflows,
// .bee/runtime/leases, and .bee/runtime/handoffs records (a granted
// worktree's storeRoot, pre-msn-18a-c) now has those stores stranded —
// msn-18a-c re-rooted every coordination read/write onto controlRootFor(root)
// (always mainRoot, D2), so bee will never look at the worktree's own copies
// again. onboarding a checkout that resolveContext identifies as a linked
// worktree MUST either MIGRATE those stranded records into main's control
// store or FAIL LOUD — never leave them silently orphaned.
//
// Migration semantics (documented choice, msn-18d): ALL-OR-NOTHING ACROSS
// THE WHOLE MIGRATION SET, not per-store and not per-record. Every local
// coordination file across every store is classified first (pure scan, zero
// writes); if ANY record conflicts (main's store already holds a DIFFERENT
// record under the same id), the ENTIRE apply is refused before a single
// byte moves anywhere — not just the migration, the whole onboarding run —
// matching the existing preflight-abort shape this file already uses for
// skillSync.blocked and the codex-hybrid hook-write gate below. This is the
// simplest rule that can never partially migrate: a human resolving one
// stranded record never has to wonder whether nine others already moved out
// from under them. A record that is unreadable or not valid JSON is treated
// as a conflict too (never silently dropped, never blindly copied unread).

// kind/relDir/match/id — one entry per coordination store this cell owns.
// `relDir` is POSIX-joined (this repo's convention throughout this file: see
// GITIGNORE_BLOCK_PATTERNS above), `match` filters a walked relative file
// path (also POSIX, forward-slash-joined) to the shape that store's own
// module gives its records, and `id` derives the logical record id from
// that relative path.
const WORKTREE_COORD_STORES = [
  {
    kind: "session",
    relDir: ".bee/sessions",
    match: (relFile) => /^[^/]+\.json$/.test(relFile),
    id: (relFile) => relFile.slice(0, -".json".length),
  },
  {
    kind: "claim",
    relDir: ".bee/claims",
    // claims.mjs also writes transient <cellId>.adopting gate files
    // alongside <cellId>.json records — the .json filter already excludes
    // them (they are locks, not persisted records worth migrating).
    match: (relFile) => /^[^/]+\.json$/.test(relFile),
    id: (relFile) => relFile.slice(0, -".json".length),
  },
  {
    kind: "workflow",
    relDir: ".bee/runtime/workflows",
    match: (relFile) => /^[^/]+\/state\.json$/.test(relFile),
    id: (relFile) => relFile.split("/")[0],
  },
  {
    kind: "lease-cell",
    relDir: ".bee/runtime/leases/cells",
    match: (relFile) => /^[^/]+\.json$/.test(relFile),
    id: (relFile) => relFile.slice(0, -".json".length),
  },
  {
    kind: "lease-path",
    relDir: ".bee/runtime/leases/paths",
    // lease-store.mjs shards these under <prefix>/<hash>.json — the full
    // relative path IS the natural composite id (no shorter id exists).
    match: () => true,
    id: (relFile) => relFile,
  },
  {
    kind: "handoff",
    relDir: ".bee/runtime/handoffs",
    match: (relFile) => /^[^/]+\/\d+\.json$/.test(relFile),
    id: (relFile) => relFile,
  },
];

// walkJsonFiles — every *.json file under rootDir, returned as POSIX
// (forward-slash) relative paths regardless of platform. Fail-open: a
// missing/unreadable directory anywhere in the walk contributes zero files
// rather than throwing (mirrors claims.mjs listSessionRecords/
// workflow-store.mjs listWorkflows's own "absent store = no records" stance).
function walkJsonFiles(rootDir) {
  const results = [];
  const stack = [""];
  while (stack.length) {
    const relDir = stack.pop();
    let entries;
    try {
      entries = fs.readdirSync(path.join(rootDir, relDir), { withFileTypes: true });
    } catch {
      continue;
    }
    for (const entry of entries) {
      const relPath = relDir ? `${relDir}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        stack.push(relPath);
      } else if (entry.isFile() && entry.name.endsWith(".json")) {
        results.push(relPath);
      }
    }
  }
  return results;
}

// canonicalJson — deterministic (sorted-key) JSON serialization used ONLY to
// compare two records for "identical content"; never used for anything that
// gets written to disk (a migrated record is always written with its
// original, unmodified bytes — see applyWorktreeMigration below).
function canonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map(canonicalJson).join(",")}]`;
  }
  if (value && typeof value === "object") {
    const keys = Object.keys(value).sort();
    return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalJson(value[k])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

// classifyMigrationRecord — pure classification of ONE local coordination
// file against its would-be home in main's control store. Never writes.
//   status: "migrate"  — main has no record at this path; safe to move.
//   status: "duplicate" — main already has a BYTE-for-structurally-identical
//                          record; the local copy is stale leftover, safe to
//                          delete without touching main.
//   status: "conflict"  — main has a DIFFERENT record, or either copy is
//                          unreadable/not valid JSON; never auto-resolved.
function classifyMigrationRecord({ workspaceRoot, mainRoot, kind, relPath, id }) {
  const localAbs = path.join(workspaceRoot, ...relPath.split("/"));
  const mainAbs = path.join(mainRoot, ...relPath.split("/"));
  let localRaw;
  try {
    localRaw = fs.readFileSync(localAbs, "utf8");
  } catch (error) {
    return { kind, path: relPath, id, status: "conflict", reason: `local record unreadable: ${String(error?.message || error)}` };
  }
  let localParsed;
  try {
    localParsed = JSON.parse(localRaw);
  } catch {
    return { kind, path: relPath, id, status: "conflict", reason: "local record is not valid JSON — cannot verify it is safe to migrate" };
  }
  if (!fs.existsSync(mainAbs)) {
    return { kind, path: relPath, id, status: "migrate", localAbs, mainAbs, content: localRaw };
  }
  let mainParsed;
  try {
    mainParsed = JSON.parse(fs.readFileSync(mainAbs, "utf8"));
  } catch (error) {
    return { kind, path: relPath, id, status: "conflict", reason: `main store already has a record at this id but it is not valid JSON: ${String(error?.message || error)}` };
  }
  if (canonicalJson(localParsed) === canonicalJson(mainParsed)) {
    return { kind, path: relPath, id, status: "duplicate", localAbs, mainAbs };
  }
  return {
    kind,
    path: relPath,
    id,
    status: "conflict",
    reason: "main's control store already has a DIFFERENT record under this id",
  };
}

// resolveWorktreeContextForMigration — a minimal, SELF-CONTAINED replica of
// state.mjs's resolveRootsCore git-worktree walk-up (see the module-header
// note next to this file's imports for why it is never imported). Computes
// ONLY what migration detection needs: the physical workspace root, whether
// it is a linked worktree, and the shared main root — not the full
// resolveContext shape. Grants/workspaceId are irrelevant here: D2 made
// controlRoot unconditionally mainRoot for every linked worktree regardless
// of grant status (msn-18a), so this never reads worktree-grants.json.
// Fails open to "ordinary, not a linked worktree" (worktreeId null) on
// anything unresolvable or malformed — a broken worktree link is
// resolveContext's own concern to fail loud about when something actually
// tries to read/write coordination state there, not this scan's job to
// diagnose (mirrors resolveContext's own "give up gracefully" posture for a
// no-git-root repo).
function locateGitRootForMigration(start) {
  let dir = path.resolve(start || process.cwd());
  while (true) {
    const marker = path.join(dir, ".git");
    if (fs.existsSync(marker)) return { workRoot: dir, marker };
    const parent = path.dirname(dir);
    if (parent === dir) return null;
    dir = parent;
  }
}

function readGitdirFileForMigration(file, base) {
  try {
    let raw = fs.readFileSync(file, "utf8").trim();
    if (!raw) return null;
    if (raw.startsWith("gitdir:")) raw = raw.slice("gitdir:".length).trim();
    return path.resolve(base, raw.replace(/\\/g, path.sep));
  } catch {
    return null;
  }
}

function resolveWorktreeContextForMigration(startDir) {
  const located = locateGitRootForMigration(startDir);
  if (!located) {
    return { workspaceRoot: null, mainRoot: null, worktreeId: null };
  }
  const { workRoot, marker } = located;
  const ordinary = { workspaceRoot: workRoot, mainRoot: workRoot, worktreeId: null };
  let isFile = false;
  try {
    isFile = fs.statSync(marker).isFile();
  } catch {
    return ordinary;
  }
  if (!isFile) {
    return ordinary; // a real .git DIRECTORY: ordinary checkout.
  }
  const gitdir = readGitdirFileForMigration(marker, workRoot);
  if (!gitdir) return ordinary;
  const worktreesRoot = path.resolve(gitdir, "..");
  const commonGitDir = path.resolve(worktreesRoot, "..");
  if (path.basename(commonGitDir) !== ".git" || path.basename(worktreesRoot) !== "worktrees") {
    return ordinary;
  }
  const id = path.basename(gitdir);
  if (!id || id === "." || id === "..") return ordinary;
  const reverse = readGitdirFileForMigration(path.join(gitdir, "gitdir"), gitdir);
  if (!reverse || path.resolve(reverse) !== path.resolve(marker)) return ordinary;
  return { workspaceRoot: workRoot, mainRoot: path.dirname(commonGitDir), worktreeId: id };
}

// detectWorktreeMigration — the read-only scan. `applicable` is false for
// every ordinary/main/solo checkout (byte-identical, zero behavior change —
// must_have truth) and true only when this checkout is a linked worktree
// whose main root differs from its own workspace root.
function detectWorktreeMigration(repoRoot) {
  const context = resolveWorktreeContextForMigration(repoRoot);
  if (!context.workspaceRoot || !context.worktreeId || context.mainRoot === context.workspaceRoot) {
    return {
      applicable: false,
      workspaceRoot: context.workspaceRoot || null,
      mainRoot: context.mainRoot || null,
      records: [],
      conflicts: [],
    };
  }
  const { workspaceRoot, mainRoot } = context;
  const records = [];
  for (const store of WORKTREE_COORD_STORES) {
    const baseDir = path.join(workspaceRoot, ...store.relDir.split("/"));
    for (const relFile of walkJsonFiles(baseDir)) {
      if (!store.match(relFile)) continue;
      const relPath = `${store.relDir}/${relFile}`;
      const id = store.id(relFile);
      records.push(classifyMigrationRecord({ workspaceRoot, mainRoot, kind: store.kind, relPath, id }));
    }
  }
  const conflicts = records.filter((r) => r.status === "conflict");
  return { applicable: true, workspaceRoot, mainRoot, records, conflicts };
}

// buildMigrationConflictReason — the loud, exact-list failure message (never
// silent orphaning): every stranded record's path/id/kind/reason, plus the
// fix.
function buildMigrationConflictReason(conflicts) {
  const lines = conflicts.map((c) => `  - [${c.kind}] ${c.path} (id: ${c.id}) — ${c.reason}`);
  return (
    `${conflicts.length} worktree-local coordination record(s) could not be migrated into main's ` +
    `control store — onboarding refused UNTOUCHED, no partial migration:\n${lines.join("\n")}\n` +
    "FIX: compare each local copy against main's, keep the correct one, delete the stale one by " +
    "hand, then re-run onboarding."
  );
}

// applyWorktreeMigration — the only function in this section that writes.
// Only ever called after detectWorktreeMigration reported zero conflicts
// (both call sites below check that first). "migrate" records are copied
// verbatim (original bytes, not the canonicalized form) then removed
// locally; "duplicate" records are simply removed locally (main already has
// the same content) — idempotent by construction: a second run finds
// nothing left to walk.
function applyWorktreeMigration(migration) {
  for (const record of migration.records) {
    if (record.status === "migrate") {
      writeFileAtomic(record.mainAbs, record.content);
      fs.rmSync(record.localAbs, { force: true });
    } else if (record.status === "duplicate") {
      fs.rmSync(record.localAbs, { force: true });
    }
  }
}

// ---------- plan computation ----------

function blockedSourceIdentitySkillSync(repoRoot, options, identity) {
  const targetSpecs = options.syncSkills
    ? skillSyncTargets(repoRoot, { globalSkills: options.globalSkills })
    : [];
  const versions = {
    source: versionLabel(identity.components[0].version),
    host_helpers: "unknown",
    installed_skills: "unknown",
  };
  const blocked = { ...identity.blocked, versions };
  return {
    source_root: path.dirname(HIVE_DIR),
    targets: targetSpecs.map(function ({ kind, target_root }) {
      return {
        kind,
        target_root,
        mode: null,
        versions,
        blocked: { ...identity.blocked },
        items: [],
      };
    }),
    blocked,
  };
}

function computePlan(
  repoRoot,
  {
    repoHooks = false,
    claudeMd = true,
    globalSkills = false,
    syncSkills = true,
    pluginSource = false,
    runtime = "both",
  } = {},
) {
  const plan = [];
  // GH #22 P0-1: whether THIS run's --plugin-source + --runtime combination
  // covers the codex-hybrid hook path. Computed once, read by both the plan
  // block below and the managed-set gate (buildManagedVersions/subsetManaged)
  // so the two can never drift from each other.
  const codexHybrid = pluginSource && runtimeCoversCodex(runtime);

  // 0. worktree-local coordination migration (msn-18d): computed before the
  // release-identity gate below since it never depends on it, so even a
  // blocked-source-identity plan/apply still reports pending/stranded
  // migration state honestly.
  const worktreeMigration = detectWorktreeMigration(repoRoot);
  if (worktreeMigration.applicable && !worktreeMigration.conflicts.length && worktreeMigration.records.length) {
    plan.push({
      action: "migrate_worktree_records",
      path: ".bee/ (worktree-local coordination stores)",
      count: worktreeMigration.records.length,
    });
  }

  const releaseIdentity = readSourceReleaseIdentity();
  if (releaseIdentity.blocked) {
    return {
      plan,
      beeVersion: null,
      renderedBlock: "",
      renderedGitignoreBlock: "",
      desiredManaged: {},
      skillSync: blockedSourceIdentitySkillSync(
        repoRoot,
        { globalSkills, syncSkills },
        releaseIdentity,
      ),
      codexHybrid,
      worktreeMigration,
    };
  }
  const beeVersion = releaseIdentity.version;
  const renderedBlock = renderAgentsBlock();
  const renderedGitignoreBlock = renderGitignoreBlock();

  // 1. AGENTS.md BEE block
  const agentsPath = path.join(repoRoot, "AGENTS.md");
  const agentsText = readTextIfExists(agentsPath);
  if (!agentsText.trim()) {
    plan.push({ action: "create_agents_block", path: "AGENTS.md" });
  } else if (!agentsBlockPresent(agentsText)) {
    plan.push({ action: "append_agents_block", path: "AGENTS.md" });
  } else if (extractAgentsBlock(agentsText) !== renderedBlock) {
    plan.push({ action: "update_agents_block", path: "AGENTS.md" });
  }

  // 1b. minimal header proposal (decision D4, propose-only): fires only when
  // no prose line exists outside the BEE markers - so fresh repos get the
  // header alongside create_agents_block (ordered after it), block-only
  // AGENTS.md files flip up_to_date -> changes_needed (intended upgrade),
  // and any existing prose suppresses the item entirely.
  if (!hasProseOutsideBlock(agentsText)) {
    plan.push({ action: "propose_agents_header", path: "AGENTS.md" });
  }

  // 2. runtime files (create-if-missing only; never overwrite state/decisions/cells)
  const runtimeFiles = [
    [".bee/state.json", () => `${JSON.stringify(DEFAULT_STATE, null, 2)}\n`],
    [".bee/config.json", () => `${JSON.stringify(DEFAULT_CONFIG, null, 2)}\n`],
    [".bee/reservations.json", () => `${JSON.stringify({ reservations: [] }, null, 2)}\n`],
    [".bee/decisions.jsonl", () => ""],
    [".bee/backlog.jsonl", () => ""],
  ];
  for (const [rel] of runtimeFiles) {
    if (!fs.existsSync(path.join(repoRoot, rel))) {
      plan.push({ action: "create_runtime_file", path: rel });
    }
  }
  for (const relDir of [".bee/cells", ".bee/logs"]) {
    if (!fs.existsSync(path.join(repoRoot, relDir))) {
      plan.push({ action: "create_dir", path: relDir });
    }
  }

  // 3. vendored helpers + lib (copy when missing or drifted)
  const onboarding = readJsonIfExists(path.join(repoRoot, ".bee", "onboarding.json"));
  for (const name of listTemplateHelpers()) {
    const source = fs.readFileSync(path.join(TEMPLATES_DIR, name), "utf8");
    const target = path.join(repoRoot, ".bee", "bin", name);
    if (readTextIfExists(target) !== source) {
      plan.push({ action: "copy_helper", path: `.bee/bin/${name}` });
    }
  }
  // 3a. retired helper shims (D2): a host with a leftover bee_*.mjs shim in
  // its own .bee/bin/ gets a removal item on the next apply. Idempotent - once
  // the file is gone (this run, or already), no item is produced.
  for (const name of RETIRED_HELPERS) {
    if (fs.existsSync(path.join(repoRoot, ".bee", "bin", name))) {
      plan.push({ action: "remove_helper", path: `.bee/bin/${name}` });
    }
  }
  const currentLibNames = new Set(listTemplateLibModules());
  for (const name of currentLibNames) {
    const source = fs.readFileSync(path.join(TEMPLATES_LIB_DIR, name), "utf8");
    const target = path.join(repoRoot, ".bee", "bin", "lib", name);
    if (readTextIfExists(target) !== source) {
      plan.push({ action: "copy_lib", path: `.bee/bin/lib/${name}` });
    }
  }
  // 3c. retired lib modules (mirrors 3a, but self-derived instead of a hand
  // list): a lib module the recorded ledger's managed.lib still names, but the
  // current source packages/bee/lib no longer has, is an orphan on every host
  // that installed it before the removal. computeRuntimeDrift (bee.mjs)
  // already flags such a file as "(extra)" forever, because nothing before
  // this removed it - copy_lib only ever adds/updates names the source still
  // has. Deriving from the ledger diff (not a RETIRED_HELPERS-style allowlist)
  // means a future lib retirement never needs its own entry here.
  const previousLibNames =
    onboarding && onboarding.managed && onboarding.managed.lib
      ? Object.keys(onboarding.managed.lib)
      : [];
  for (const name of previousLibNames) {
    if (!currentLibNames.has(name) && fs.existsSync(path.join(repoRoot, ".bee", "bin", "lib", name))) {
      plan.push({ action: "remove_lib", path: `.bee/bin/lib/${name}` });
    }
  }

  // 3b. statusline pair (opt-in sync): only for repos whose settings.json
  // already points statusLine at .claude/statusline-command.sh. Byte-compare
  // like the vendored helpers; never creates the opt-in on other repos.
  if (statuslineOptIn(repoRoot)) {
    for (const name of listTemplateStatusline()) {
      const source = fs.readFileSync(path.join(TEMPLATES_STATUSLINE_DIR, name), "utf8");
      const target = path.join(repoRoot, ".claude", name);
      if (readTextIfExists(target) !== source) {
        plan.push({ action: "copy_statusline", path: `.claude/${name}` });
      }
    }
  }

  // 4. learnings stub
  if (!fs.existsSync(path.join(repoRoot, "docs", "history", "learnings", "critical-patterns.md"))) {
    plan.push({ action: "create_stub", path: "docs/history/learnings/critical-patterns.md" });
  }

  // 4a. state-layer skeletons: reading-map + system-overview must exist after
  // onboarding (create-only — bee-scribing owns the content and an existing
  // file is NEVER touched, drifted or not).
  for (const name of ["reading-map.md", "system-overview.md"]) {
    if (!fs.existsSync(path.join(repoRoot, "docs", "specs", name))) {
      plan.push({ action: "create_specs_stub", path: `docs/specs/${name}` });
    }
  }

  // 4b. .gitignore managed block (D1): marker-splice pattern identical to the
  // AGENTS.md block above, but with '#'-comment markers (gitignore syntax -
  // never HTML comments, which gitignore would read as a literal pattern).
  // no .gitignore -> create; .gitignore without markers -> append (preserving
  // existing content, even without a trailing newline); markers present but
  // body drifted -> update, splicing ONLY between the markers.
  const gitignorePath = path.join(repoRoot, ".gitignore");
  const gitignoreText = readTextIfExists(gitignorePath);
  if (!gitignoreText.trim()) {
    plan.push({ action: "create_gitignore_block", path: ".gitignore" });
  } else if (!gitignoreBlockPresent(gitignoreText)) {
    plan.push({ action: "append_gitignore_block", path: ".gitignore" });
  } else if (
    normalizeGitignoreForCompare(extractGitignoreBlock(gitignoreText)) !== renderedGitignoreBlock
  ) {
    plan.push({ action: "update_gitignore_block", path: ".gitignore" });
  }

  // 5. repo hooks fallback (--repo-hooks only)
  if (repoHooks) {
    for (const name of listPluginHooks()) {
      const source = fs.readFileSync(path.join(PLUGIN_HOOKS_DIR, name), "utf8");
      const target = path.join(repoRoot, ".bee", "bin", "hooks", name);
      if (readTextIfExists(target) !== source) {
        plan.push({ action: "copy_repo_hook", path: `.bee/bin/hooks/${name}` });
      }
    }
    const settingsPath = path.join(repoRoot, ".claude", "settings.json");
    try {
      if (mergeRepoSettings(settingsPath).changed) {
        plan.push({ action: "merge_repo_hook_settings", path: ".claude/settings.json" });
      }
    } catch {
      plan.push({ action: "merge_repo_hook_settings", path: ".claude/settings.json" });
    }
    // Codex projection of the same hook set (see renderCodexHookEntries):
    // without it a Codex session in the host repo runs with NO bee guards.
    // Skipped when the repo owns hooks/catalog.mjs — see repoOwnsHookCatalog:
    // there the catalog is the authority and this projection would clobber it.
    if (!repoOwnsHookCatalog(repoRoot)) {
      const codexHooksPath = path.join(repoRoot, ".codex", "hooks.json");
      try {
        if (mergeCodexHooks(codexHooksPath).changed) {
          plan.push({ action: "merge_codex_hooks", path: ".codex/hooks.json" });
        }
      } catch {
        plan.push({ action: "merge_codex_hooks", path: ".codex/hooks.json" });
      }
    }
  }

  // 5a. codex-hybrid hooks (plugin-first + --runtime covers codex, GH #22
  // P0-1): repoHooks is ALWAYS false under --plugin-source (see options
  // below / main()), so this is a separately-gated path, never folded into
  // the `if (repoHooks)` block above — it fires purely off codexHybrid
  // (computed from the PASSED --runtime, above). Reuses the exact same
  // projection (listPluginHooks/mergeCodexHooks/renderCodexHookEntries) the
  // --repo-hooks path uses, so the two mechanisms can never drift from each
  // other, and the same repoOwnsHookCatalog self-skip applies (bee's own
  // repo is the catalog's authority, never a projection target). Never
  // touches .claude/settings.json: Claude's own plugin hooks already work
  // under plugin-first (only Codex lacks a plugin-hook mechanism), so no
  // repo-local Claude entries are written here.
  if (codexHybrid && !repoOwnsHookCatalog(repoRoot)) {
    for (const name of listPluginHooks()) {
      const source = fs.readFileSync(path.join(PLUGIN_HOOKS_DIR, name), "utf8");
      const target = path.join(repoRoot, ".bee", "bin", "hooks", name);
      if (readTextIfExists(target) !== source) {
        plan.push({ action: "copy_repo_hook", path: `.bee/bin/hooks/${name}` });
      }
    }
    const codexHooksPath = path.join(repoRoot, ".codex", "hooks.json");
    try {
      if (mergeCodexHooks(codexHooksPath).changed) {
        plan.push({ action: "merge_codex_hooks", path: ".codex/hooks.json" });
      }
    } catch {
      plan.push({ action: "merge_codex_hooks", path: ".codex/hooks.json" });
    }
  }

  // 5c. Codex user-config status line (machine-level, add-only): the item's
  // path is display-only — the apply case resolves the real user-config path
  // itself and never joins it under repoRoot.
  if (codexStatuslineMissing()) {
    plan.push({ action: "ensure_codex_statusline", path: "~/.codex/config.toml" });
  }

  // 5b. CLAUDE.md @import fallback (D1, default; --no-claude-md opts out):
  // auto-load the BEE block on Claude Code even when plugin hooks are
  // unavailable. Never touches an existing CLAUDE.md that already imports
  // AGENTS.md.
  if (claudeMd) {
    const claudeMdPath = path.join(repoRoot, "CLAUDE.md");
    if (!fs.existsSync(claudeMdPath)) {
      plan.push({ action: "create_claude_md", path: "CLAUDE.md" });
    } else if (!/^@AGENTS\.md\s*$/m.test(readTextIfExists(claudeMdPath))) {
      plan.push({ action: "append_claude_md_import", path: "CLAUDE.md" });
    }
  }

  // 5d. bee agent files (config-rendered, AO10-safe flat sync - see the
  // block comment above computeAgentFilePlan): NOT part of REPO_SKILL_TARGETS
  // and not gated by any opt-in - every repo with a configured tier gets its
  // agent files, exactly like the AGENTS.md block above.
  plan.push(...computeAgentFilePlan(repoRoot));

  // 6. onboarding.json drift (managed versions)
  const statusline = statuslineOptIn(repoRoot);
  const desiredManaged = buildManagedVersions(
    renderedBlock, renderedGitignoreBlock, repoHooks, statusline, codexHybrid);
  // onboarding already read in section 3, above, and reused here.
  const onboardingCurrent =
    onboarding &&
    onboarding.schema_version === ONBOARDING_SCHEMA_VERSION &&
    onboarding.bee_version === beeVersion &&
    JSON.stringify(subsetManaged(onboarding.managed, repoHooks, statusline, codexHybrid)) ===
      JSON.stringify(subsetManaged(desiredManaged, repoHooks, statusline, codexHybrid));
  if (!onboardingCurrent) {
    plan.push({ action: "write_onboarding", path: ".bee/onboarding.json" });
  }

  // 7. skill sync (D1-D5, per target): drift between the running tree and
  // each target root's bee-* set appears as plan items, every item tagged
  // with its target kind. Read-only. A blocked stage (any target) withholds
  // ALL skill items from the flat plan; per-target items stay visible in
  // skills.targets for forced-apply transparency (D2).
  const skillSync = syncSkills
    ? computeSkillSync(repoRoot, { globalSkills })
    : { blocked: null, source_root: HIVE_DIR, targets: [] };
  if (!skillSync.blocked) {
    for (const target of skillSync.targets) {
      plan.push(...target.items);
    }
    // Legacy-global version-parity refresh items (installer-version-parity):
    // additive, never blocked; only present without --global-skills.
    if (skillSync.legacyRefresh) {
      plan.push(...skillSync.legacyRefresh.items);
    }
  }

  return { plan, beeVersion, renderedBlock, renderedGitignoreBlock, desiredManaged, skillSync, codexHybrid, worktreeMigration };
}

// Legacy-global version-parity refresh items are a best-effort side pass over
// ~/.claude/skills (installer-version-parity-1-3-1): they are listed in the plan
// for transparency and applied on --apply, but they NEVER drive the
// up_to_date/changes_needed status. A fully-onboarded repo must not read
// "changes_needed" forever merely because the user's legacy global install is
// stale, and requirement (5): a refreshed global never flips drift or breaks
// recheck. Status counts only the repo/target work; refresh_legacy_global_skill
// is excluded here (and only here - apply still applies these items).
function coreChangesNeeded(plan) {
  return plan.some((item) => item.action !== "refresh_legacy_global_skill");
}

// Shared by the --repo-hooks managed.repo_hooks map and the codex-hybrid
// managed.codex_hooks map below (GH #22 P0-1): both track the identical
// vendored-script + .codex/hooks.json projection, so one render function
// keeps them from ever drifting apart.
function buildHookVersions() {
  const hooks = {};
  for (const name of listPluginHooks()) {
    hooks[name] = sha256(fs.readFileSync(path.join(PLUGIN_HOOKS_DIR, name), "utf8"));
  }
  // Pseudo-entry: the desired Codex projection rides the same managed map,
  // so a render change here surfaces as onboarding drift like any hook edit.
  hooks[".codex/hooks.json"] = sha256(JSON.stringify(renderCodexHookEntries()));
  return hooks;
}

function buildManagedVersions(
  renderedBlock, renderedGitignoreBlock, repoHooks, statusline = false, codexHybrid = false,
) {
  const helpers = {};
  for (const name of listTemplateHelpers()) {
    helpers[name] = hashFile(path.join(TEMPLATES_DIR, name));
  }
  const lib = {};
  for (const name of listTemplateLibModules()) {
    lib[name] = hashFile(path.join(TEMPLATES_LIB_DIR, name));
  }
  const managed = {
    agents_block: sha256(renderedBlock),
    gitignore_block: sha256(renderedGitignoreBlock),
    helpers,
    lib,
  };
  if (repoHooks) {
    managed.repo_hooks = buildHookVersions();
  }
  if (codexHybrid) {
    // Advisor R5: a DISTINCT key from repo_hooks — repo_hooks means "this
    // repo opted into the full --repo-hooks install" (Claude + Codex, sticky
    // via hasRepoHooksRecorded); codex_hooks means only the codex-hybrid
    // projection is active (no Claude repo-local entries). Conflating the
    // two would make hasRepoHooksRecorded misfire on a plugin-first repo
    // that only ever asked for Codex coverage.
    managed.codex_hooks = buildHookVersions();
  }
  if (statusline) {
    const pair = {};
    for (const name of listTemplateStatusline()) {
      pair[name] = sha256(fs.readFileSync(path.join(TEMPLATES_STATUSLINE_DIR, name), "utf8"));
    }
    managed.statusline = pair;
  }
  return managed;
}

// Has this repo already opted into repo-local hook wiring? The opt-in is sticky:
// the record of a prior --repo-hooks install is what keeps later upgrades honest,
// so the owner never has to re-supply the flag to stay current.
function hasRepoHooksRecorded(repoRoot) {
  try {
    const raw = fs.readFileSync(path.join(repoRoot, ".bee", "onboarding.json"), "utf8");
    const recorded = JSON.parse(raw)?.managed?.repo_hooks;
    return !!recorded && typeof recorded === "object" && Object.keys(recorded).length > 0;
  } catch {
    return false; // no marker, unreadable, or malformed — treat as never opted in
  }
}

// Compare only the parts we manage in this run: without --repo-hooks, ignore
// any repo_hooks entry recorded by a previous --repo-hooks run; without the
// statusline opt-in, ignore any statusline entry the same way. Without
// codexHybrid, ignore any codex_hooks entry a previous hybrid run recorded
// (advisor R5) — a claude-only run must never report codex_hooks drift, nor
// treat one as present to preserve.
function subsetManaged(managed, repoHooks, statusline = false, codexHybrid = false) {
  const src = managed && typeof managed === "object" ? managed : {};
  const out = {
    agents_block: src.agents_block || null,
    gitignore_block: src.gitignore_block || null,
    helpers: src.helpers || {},
    lib: src.lib || {},
  };
  if (repoHooks) {
    out.repo_hooks = src.repo_hooks || {};
  }
  if (codexHybrid) {
    out.codex_hooks = src.codex_hooks || {};
  }
  if (statusline) {
    out.statusline = src.statusline || {};
  }
  return out;
}

// GH #22 P0-1 / advisor R6 (point 6): a repo that once recorded a full
// --repo-hooks install (Claude + Codex repo-local wiring, sticky via
// hasRepoHooksRecorded) and is now re-onboarded as --plugin-source no longer
// gets that record silently carried forward (see applyPlan's onboarding.json
// write) — repoHooks is unconditionally false under --plugin-source (main()
// options), so the block that WRITES .claude/settings.json entries never
// runs this pass, and plugin-first distribution cleanup (plugin_distribution.mjs)
// is expected to strip the stale Claude entries it left behind (Claude's own
// plugin hooks take over). Silently dropping the record would be honest
// about the mechanism but dishonest about the SURPRISE: the human opted into
// full repo-local coverage once and it just changed shape underneath them.
// This notice makes that transition visible exactly once, naming which
// coverage survives (codex-hybrid, when --runtime covers codex) and which
// does not (Claude repo-local entries, always; Codex too when --runtime
// claude was passed without codex/both).
function repoHooksTransitionNotices(repoRoot, { pluginSource, codexHybrid }) {
  if (!pluginSource || !hasRepoHooksRecorded(repoRoot)) {
    return [];
  }
  if (codexHybrid) {
    return [
      "This repo previously opted into --repo-hooks (full repo-local Claude + Codex hook wiring). " +
        "Onboarding as --plugin-source retires the repo-local Claude entries in .claude/settings.json " +
        "(Claude's own plugin hooks take over) and keeps Codex mechanically enforced through the " +
        "codex-hybrid .codex/hooks.json projection instead — no action needed.",
    ];
  }
  return [
    "This repo previously opted into --repo-hooks (full repo-local Claude + Codex hook wiring). " +
      "Onboarding as --plugin-source with --runtime claude retires ALL repo-local hook entries, " +
      "including Codex's — pass --runtime codex or --runtime both to keep Codex mechanically " +
      "enforced via the codex-hybrid path, or use --distribution repo-copy to keep the full " +
      "repo-local install as-is.",
  ];
}

// ---------- apply ----------

function applyPlan(
  repoRoot,
  {
    repoHooks = false,
    claudeMd = true,
    globalSkills = false,
    syncSkills = true,
    forceDowngrade = false,
    pluginSource = false,
    runtime = "both",
  } = {},
) {
  const { plan, beeVersion, renderedBlock, renderedGitignoreBlock, desiredManaged, skillSync, codexHybrid, worktreeMigration } =
    computePlan(repoRoot, {
      repoHooks,
      claudeMd,
      globalSkills,
      syncSkills,
      pluginSource,
      runtime,
    });

  // msn-18d / advisor condition F4: the worktree-migration preflight. Checked
  // FIRST, before every other write gate below (including codexHybrid and
  // skillSync.blocked) — an untrustworthy/stranded coordination store must
  // never let the rest of onboarding proceed. ALL-OR-NOTHING: a single
  // conflicting record anywhere refuses the ENTIRE apply, zero mutations,
  // every stranded record named (see buildMigrationConflictReason).
  if (worktreeMigration.conflicts.length) {
    return {
      blocked: {
        status: "blocked_worktree_migration_conflict",
        reason: buildMigrationConflictReason(worktreeMigration.conflicts),
        stranded: worktreeMigration.conflicts.map((c) => ({ path: c.path, id: c.id, kind: c.kind, reason: c.reason })),
      },
      beeVersion,
    };
  }

  // GH #22 P0-1 / advisor R3: the codex-hybrid write preflight. Checked
  // BEFORE the skillSync.blocked branch below and before any write anywhere
  // — fail-closed (point 3): skills must never be reported applied without
  // the hooks that make them mechanically enforced for Codex, so a blocked
  // hook write refuses the WHOLE apply, typed exactly like skillSync.blocked.
  if (codexHybrid && !repoOwnsHookCatalog(repoRoot)) {
    const hookBlock = codexHookWriteBlocker(repoRoot);
    if (hookBlock) {
      return { blocked: hookBlock, beeVersion };
    }
  }

  // D3 preflight: refusal aborts the ENTIRE apply BEFORE any write - the item
  // loop below and the unconditional onboarding.json rewrite after it are
  // unreachable on refusal, so a refused apply mutates nothing anywhere (repo,
  // in-repo skill roots, or global). Blocked-first across targets: ANY blocked
  // target refuses the whole apply. --force-downgrade overrides only when
  // EVERY blocked target is a version refusal with all three versions resolved
  // numeric; unknown and blocked_no_source are resolution failures and are
  // never forceable.
  let forcedDowngrade = false;
  if (skillSync.blocked) {
    if (forceDowngrade && skillSync.blocked.forceable) {
      forcedDowngrade = true;
      // computePlan withholds ALL targets' items from the flat plan while the
      // stage is blocked - restore every target's computed items for the
      // forced apply (unblocked targets included).
      for (const target of skillSync.targets) {
        plan.push(...target.items);
      }
      if (skillSync.legacyRefresh) {
        plan.push(...skillSync.legacyRefresh.items);
      }
    } else {
      // Review P1-6 / D2: computeSkillSyncTarget() already computed each
      // target's items whenever its refusal is forceable (empty [] otherwise)
      // - a human deciding whether to pass --force-downgrade must see exactly
      // what it will overwrite/delete PER TARGET before authorizing it, not
      // only after the fact in a forced apply's own report. Surfaced here so
      // the refused-apply response (the response most users actually see
      // first) carries it.
      const blockedResult = {
        blocked: {
          status: skillSync.blocked.status,
          reason: skillSync.blocked.reason,
          forceable: skillSync.blocked.forceable,
        },
        versions: skillSync.blocked.versions,
        skills: { source_root: skillSync.source_root, targets: skillSync.targets },
        beeVersion,
      };
      // P49: a forceable refusal names its blast radius beyond skills - the
      // copy_lib/copy_helper paths a --force-downgrade would also overwrite
      // under .bee/bin. Filtered from the already-computed `plan` verbatim,
      // order preserved, never recomputed. Non-forceable refusals (unknown
      // version, blocked_no_source) omit the field entirely - it never
      // invites a force that can't happen.
      if (skillSync.blocked.forceable) {
        blockedResult.host_items = plan.filter(
          ({ action }) => action === "copy_lib" || action === "copy_helper",
        );
      }
      return blockedResult;
    }
  }
  const skillTargetRootByKind = new Map(
    skillSync.targets.map((t) => [t.kind, t.target_root]),
  );
  // The legacy-global refresh target is not one of skillSync.targets (it never
  // participates in aggregation); register its root so refresh items resolve
  // the same way sync_skill items do.
  if (skillSync.legacyRefresh) {
    skillTargetRootByKind.set("legacy-global", skillSync.legacyRefresh.target_root);
  }

  const applied = [];
  const skippedSkills = [];

  // Compose the header BEFORE any mergeAgentsContent call (decision D4): it
  // rides the existing-content input of the same merge - one write mechanism,
  // no new merge helper parameter.
  const proposeHeader = plan.some((item) => item.action === "propose_agents_header");
  const headerText = proposeHeader ? composeAgentsHeader(repoRoot) : "";
  let headerApplied = false;

  for (const item of plan) {
    const target = path.join(repoRoot, ...item.path.split("/"));
    switch (item.action) {
      case "create_agents_block":
      case "append_agents_block":
      case "update_agents_block": {
        const merged = mergeAgentsContent(headerText + readTextIfExists(target), renderedBlock);
        writeFileAtomic(target, merged.text);
        headerApplied = proposeHeader;
        break;
      }
      case "propose_agents_header": {
        if (headerApplied) {
          break; // header already rode the block write above
        }
        // Block-only file (already onboarded, block current): prepend the
        // header through the same merge path - the in-place block replace
        // keeps everything outside the markers untouched.
        const merged = mergeAgentsContent(headerText + readTextIfExists(target), renderedBlock);
        writeFileAtomic(target, merged.text);
        headerApplied = true;
        break;
      }
      case "create_gitignore_block":
      case "append_gitignore_block":
      case "update_gitignore_block": {
        const merged = mergeGitignoreContent(readTextIfExists(target), renderedGitignoreBlock);
        writeFileAtomic(target, merged.text);
        break;
      }
      case "create_runtime_file": {
        if (!fs.existsSync(target)) {
          const rel = item.path;
          let content = "";
          if (rel.endsWith("state.json")) {
            content = `${JSON.stringify(DEFAULT_STATE, null, 2)}\n`;
          } else if (rel.endsWith("config.json")) {
            content = `${JSON.stringify(DEFAULT_CONFIG, null, 2)}\n`;
          } else if (rel.endsWith("reservations.json")) {
            content = `${JSON.stringify({ reservations: [] }, null, 2)}\n`;
          }
          writeFileAtomic(target, content);
        }
        break;
      }
      case "create_dir": {
        fs.mkdirSync(target, { recursive: true });
        break;
      }
      case "copy_helper": {
        const name = path.basename(item.path);
        writeFileAtomic(target, fs.readFileSync(path.join(TEMPLATES_DIR, name), "utf8"));
        break;
      }
      case "remove_helper": {
        // Never a generic rm: only ever the exact retired-shim basename, and
        // only ever under .bee/bin/ (item.path is always .bee/bin/<name>,
        // constructed by this script - never host/user-supplied).
        const name = path.basename(item.path);
        if (RETIRED_HELPERS.includes(name) && path.dirname(item.path) === ".bee/bin") {
          fs.rmSync(target, { force: true });
        }
        break;
      }
      case "copy_lib": {
        const name = path.basename(item.path);
        writeFileAtomic(target, fs.readFileSync(path.join(TEMPLATES_LIB_DIR, name), "utf8"));
        break;
      }
      case "remove_lib": {
        // Same exact-path safety as remove_helper: item.path is always
        // .bee/bin/lib/<name>, derived from the ledger diff in section 3c
        // above, never host/user-supplied.
        if (path.dirname(item.path) === ".bee/bin/lib") {
          fs.rmSync(target, { force: true });
        }
        break;
      }
      case "copy_repo_hook": {
        const name = path.basename(item.path);
        writeFileAtomic(target, fs.readFileSync(path.join(PLUGIN_HOOKS_DIR, name), "utf8"));
        break;
      }
      case "copy_statusline": {
        const name = path.basename(item.path);
        writeFileAtomic(target, fs.readFileSync(path.join(TEMPLATES_STATUSLINE_DIR, name), "utf8"));
        break;
      }
      case "create_stub": {
        writeFileAtomic(target, CRITICAL_PATTERNS_STUB);
        break;
      }
      case "create_specs_stub": {
        // create-only: scribing owns these files; an existing one is never
        // rewritten even when its content drifted from the stub.
        if (!fs.existsSync(target)) {
          writeFileAtomic(
            target,
            item.path.endsWith("reading-map.md") ? READING_MAP_STUB : SYSTEM_OVERVIEW_STUB,
          );
        }
        break;
      }
      case "create_claude_md": {
        writeFileAtomic(target, CLAUDE_MD_TEMPLATE);
        break;
      }
      case "append_claude_md_import": {
        const existing = readTextIfExists(target) || "";
        const separator = existing.endsWith("\n") ? "\n" : "\n\n";
        writeFileAtomic(target, `${existing}${separator}${CLAUDE_MD_IMPORT_SECTION}`);
        break;
      }
      case "merge_repo_hook_settings": {
        const merged = mergeRepoSettings(target);
        if (fs.existsSync(target)) {
          fs.copyFileSync(target, `${target}.bak`);
        }
        writeFileAtomic(target, merged.text);
        break;
      }
      case "merge_codex_hooks": {
        const merged = mergeCodexHooks(target);
        if (fs.existsSync(target)) {
          fs.copyFileSync(target, `${target}.bak`);
        }
        writeFileAtomic(target, merged.text);
        break;
      }
      case "ensure_codex_statusline": {
        // Machine-level target: NEVER the repoRoot-joined `target` above.
        const configPath = codexUserConfigPath();
        if (!codexStatuslineMissing()) {
          break; // plan-to-apply race: someone added it meanwhile — stay out
        }
        const text = readTextIfExists(configPath);
        fs.copyFileSync(configPath, `${configPath}.bak`);
        const tuiHeader = /^\[tui\][ \t]*\r?$/m;
        let next;
        if (tuiHeader.test(text)) {
          next = text.replace(tuiHeader, (header) => `${header}\n${CODEX_STATUS_LINE_BLOCK.trimEnd()}`);
        } else {
          const sep = !text || text.endsWith("\n") ? "" : "\n";
          next = `${text}${sep}\n[tui]\n${CODEX_STATUS_LINE_BLOCK}`;
        }
        writeFileAtomic(configPath, next);
        break;
      }
      case "sync_agent_file": {
        const model = resolveAgentTierModel(repoRoot, AGENT_TIER_BY_NAME[item.agent]);
        if (model) {
          writeFileAtomic(target, renderAgentTemplate(item.agent, model));
        }
        break;
      }
      case "remove_agent_file": {
        fs.rmSync(target, { force: true });
        break;
      }
      case "write_onboarding": {
        // handled after the loop so managed versions reflect the final state
        break;
      }
      case "migrate_worktree_records": {
        // Safe by construction: applyPlan already refused above (zero
        // mutations) when worktreeMigration.conflicts was non-empty, so
        // every record here is "migrate" or "duplicate".
        applyWorktreeMigration(worktreeMigration);
        break;
      }
      case "sync_skill": {
        const result = applySyncSkill(
          skillSync.source_root,
          skillTargetRootByKind.get(item.target),
          item.skill,
          runtimeForTargetKind(item.target),
        );
        if (result.blocked) {
          skippedSkills.push({ skill: item.skill, target: item.target, reason: result.blocked });
          continue; // skipped loudly, not applied
        }
        break;
      }
      case "refresh_legacy_global_skill": {
        // Version-parity in-place refresh of a managed skill that ALREADY
        // EXISTS under the legacy global root. Honor "already exists" at apply
        // time too (plan-to-apply race): never create a copy that vanished, and
        // never replace a non-plain entry.
        const root = skillTargetRootByKind.get(item.target);
        const st = lstatIfExists(path.join(root, item.skill));
        if (!st || st.isSymbolicLink() || !st.isDirectory()) {
          skippedSkills.push({
            skill: item.skill,
            target: item.target,
            reason: "legacy global skill is absent or not a plain directory - skipped, never created",
          });
          continue;
        }
        const result = applySyncSkill(
          skillSync.source_root,
          root,
          item.skill,
          runtimeForTargetKind(item.target),
        );
        if (result.blocked) {
          skippedSkills.push({ skill: item.skill, target: item.target, reason: result.blocked });
          continue; // skipped loudly, not applied
        }
        break;
      }
      case "remove_skill": {
        const result = applyRemoveSkill(skillTargetRootByKind.get(item.target), item.skill);
        if (result.blocked) {
          skippedSkills.push({ skill: item.skill, target: item.target, reason: result.blocked });
          continue; // skipped loudly, not applied
        }
        break;
      }
      case "blocked_symlink":
      case "blocked_alias": {
        // Loud per-skill report (F6 / review P1-5): never written through,
        // unlinked, deleted, or sync-then-deleted.
        skippedSkills.push({ skill: item.skill, target: item.target, reason: item.reason });
        continue;
      }
      default:
        break;
    }
    applied.push(item);
  }

  // D9/D7 provenance: stamp each rendered target's skills ROOT with the
  // bee-render/2 inventory sidecar (schema + target_runtime + per-skill
  // sha256) so source-identity refuses it as an onboarding source for any
  // target, AND doctor can deep-audit the installed skill set against it.
  // Deterministic content (no timestamp) keeps re-applies idempotent.
  // `noop` targets are the running source itself (source === target) and are
  // NEVER stamped — that would poison the canonical source into a projection.
  // Sidecar content is target-independent for a given runtime, so it is
  // built once per runtime (not once per target) and reused across every
  // target root that renders for that runtime.
  if (syncSkills) {
    const sidecarByRuntime = new Map();
    for (const t of skillSync.targets) {
      // packages-restructure deviation (SKILLS_VERSION_STAMP doc comment):
      // the stamp is written on its own condition, NOT gated on `t.blocked`
      // like the sidecar below - by the time this loop runs, the apply
      // already proved either `t` was never blocked, or every blocked target
      // was forceable and the human forced it through (blocked-first
      // aggregation above), so `t.blocked` alone is not "this target was
      // skipped", it can also mean "this target was forced". A forced target
      // DID actually receive its sync_skill items via the main item loop
      // above (unconditional on t.blocked) and must get its stamp updated
      // too, or the very next onboard falls back to a legacy marker that no
      // longer exists post-sync and reads permanently unknown.
      // Mirrors the OLD marker's exact fate, not "every skill in parity": the
      // legacy marker lived INSIDE bee-hive's own skill dir, so it only ever
      // failed to update when bee-hive ITSELF was skipped/blocked - an
      // unrelated skill (bee-alpha, say) sitting out never stopped bee-hive's
      // own sync from landing the current version. Gating on "any skill
      // skipped" is stricter than that and wrongly forces a resolvable
      // version into "unknown" (recheck 10f: unaffected skills still land,
      // only the affected ones stay out of parity - that must read as
      // ordinary changes_needed, never blocked_downgrade).
      const targetSynced = t.mode === "sync" || t.mode === "fresh";
      if (targetSynced && !skippedSkills.some((s) => s.target === t.kind && s.skill === "bee-hive")) {
        writeFileAtomic(
          path.join(t.target_root, SKILLS_VERSION_STAMP),
          `${JSON.stringify({ version: beeVersion }, null, 2)}\n`,
        );
      }
      if (t.blocked || !targetSynced) {
        continue;
      }
      const runtime = runtimeForTargetKind(t.kind);
      if (!sidecarByRuntime.has(runtime)) {
        sidecarByRuntime.set(
          runtime,
          buildRenderSidecar(runtime, sourceSkillDigestEntries(skillSync.source_root, runtime)),
        );
      }
      writeFileAtomic(
        path.join(t.target_root, RENDER_SIDECAR),
        `${JSON.stringify(sidecarByRuntime.get(runtime), null, 2)}\n`,
      );
    }
  }

  // Always (re)write onboarding.json on apply so managed versions are current.
  const onboardingPath = path.join(repoRoot, ".bee", "onboarding.json");
  const previous = readJsonIfExists(onboardingPath) || {};
  const managed = { ...desiredManaged };
  // Advisor R6 / point 6: a --plugin-source apply lets a prior --repo-hooks
  // record LAPSE rather than silently carrying it forward — repoHooksTransitionNotices
  // (above) surfaces this transition to the human in the same run, so it is
  // documented, not silent. Every OTHER path is unaffected: hasRepoHooksRecorded
  // already forces repoHooks true again on a normal (non-plugin-source)
  // re-run, so `!repoHooks` below is otherwise unreachable while a repo_hooks
  // record exists — this line only ever changed behavior for --plugin-source.
  if (!repoHooks && !pluginSource && previous.managed && previous.managed.repo_hooks) {
    // preserve the record of a prior --repo-hooks install
    managed.repo_hooks = previous.managed.repo_hooks;
  }
  const onboardingPayload = {
    schema_version: ONBOARDING_SCHEMA_VERSION,
    bee_version: beeVersion,
    managed,
    agents_sync: computeAgentsSyncRecord(repoRoot, beeVersion),
    created_at: previous.created_at || utcNow(),
    updated_at: utcNow(),
  };
  writeFileAtomic(onboardingPath, `${JSON.stringify(onboardingPayload, null, 2)}\n`);

  return {
    applied,
    onboarding: onboardingPayload,
    beeVersion,
    forcedDowngrade,
    // F9: a forced apply must still report which versions it overrode -
    // blocked-first, the first blocked target's triple (pre-force state).
    forcedVersions: skillSync.blocked ? skillSync.blocked.versions : null,
    skills: {
      source_root: skillSync.source_root,
      targets: skillSync.targets,
      skipped: skippedSkills,
    },
  };
}

// ---------- CLI ----------

function parseArgs(argv) {
  const args = {
    repoRoot: null,
    apply: false,
    json: false,
    repoHooks: false,
    // D1: CLAUDE.md is a default onboarding artifact; --no-claude-md opts out.
    // --claude-md is still accepted, now a no-op alias of the default.
    claudeMd: true,
    // D3 (installer-hardening): the legacy global ~/.claude/skills target is
    // opt-in; without the flag it is never read as a sync target, written, or
    // deleted.
    globalSkills: false,
    pluginSource: false,
    forceDowngrade: false,
    // GH #22 P0-1: which runtime(s) this invocation covers. Default "both"
    // matches install.sh's own default and keeps a bare invocation's
    // behavior unchanged (codexHybrid still requires --plugin-source too).
    runtime: "both",
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--repo-root") {
      args.repoRoot = argv[i + 1];
      i += 1;
    } else if (arg.startsWith("--repo-root=")) {
      args.repoRoot = arg.slice("--repo-root=".length);
    } else if (arg === "--apply") {
      args.apply = true;
    } else if (arg === "--json") {
      args.json = true;
    } else if (arg === "--repo-hooks") {
      args.repoHooks = true;
    } else if (arg === "--claude-md") {
      args.claudeMd = true;
    } else if (arg === "--no-claude-md") {
      args.claudeMd = false;
    } else if (arg === "--global-skills") {
      args.globalSkills = true;
    } else if (arg === "--plugin-source") {
      args.pluginSource = true;
    } else if (arg === "--force-downgrade") {
      args.forceDowngrade = true;
    } else if (arg === "--runtime") {
      args.runtime = argv[i + 1];
      i += 1;
    } else if (arg.startsWith("--runtime=")) {
      args.runtime = arg.slice("--runtime=".length);
    } else if (arg === "--help" || arg === "-h") {
      process.stdout.write(
        "Usage: onboard_bee.mjs --repo-root <path> [--apply] [--json] [--repo-hooks] [--plugin-source] " +
          "[--runtime claude|codex|both] [--no-claude-md] [--claude-md] [--global-skills] [--force-downgrade]\n",
      );
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  if (!["claude", "codex", "both"].includes(args.runtime)) {
    throw new Error(`--runtime must be claude, codex, or both (got: ${args.runtime})`);
  }
  return args;
}

function emit(payload, asJson) {
  if (asJson) {
    process.stdout.write(`${JSON.stringify(payload, null, 2)}\n`);
    return;
  }
  process.stdout.write(`bee onboarding - repo: ${payload.repo_root}\n`);
  process.stdout.write(`status: ${payload.status}\n`);
  const items = payload.plan || payload.applied || [];
  for (const item of items) {
    process.stdout.write(`  ${item.action}  ${item.path}\n`);
  }
  if (items.length === 0) {
    process.stdout.write("  (nothing to do)\n");
  }
  if (payload.reason) {
    process.stdout.write(`reason: ${payload.reason}\n`);
  }
  if (payload.versions) {
    process.stdout.write(
      `versions: source=${payload.versions.source} host_helpers=${payload.versions.host_helpers} installed_skills=${payload.versions.installed_skills}\n`,
    );
  }
  for (const skipped of payload.skills?.skipped || []) {
    process.stdout.write(
      `skipped skill: ${skipped.skill}${skipped.target ? ` [${skipped.target}]` : ""} - ${skipped.reason}\n`,
    );
  }
  for (const notice of payload.notices || []) {
    process.stdout.write(`notice: ${notice}\n`);
  }
}

function sourceKindForReport() {
  try {
    return classifySource({ hiveDir: HIVE_DIR, homeDir: os.homedir() }).kind;
  } catch {
    return "unknown";
  }
}

export function main(argv = process.argv.slice(2)) {
  let args;
  try {
    args = parseArgs(argv);
  } catch (error) {
    process.stdout.write(`${JSON.stringify({ error: String(error.message || error) })}\n`);
    return 1;
  }

  const runtime = nodeRuntimeStatus();
  if (!runtime.supported) {
    emit(
      {
        repo_root: args.repoRoot || process.cwd(),
        status: "missing_runtime",
        error: `bee requires Node.js ${MIN_NODE_MAJOR}+ (found ${runtime.version}).`,
      },
      args.json,
    );
    return 1;
  }

  const repoRoot = path.resolve(args.repoRoot || process.cwd());
  // Captured before any apply: "first onboard" means no onboarding marker yet.
  const firstOnboard = !fs.existsSync(path.join(repoRoot, ".bee", "onboarding.json"));

  try {
    const options = {
      // --repo-hooks opts a repo IN; it is not a re-consent required on every upgrade.
      // Once a repo carries vendored hooks, an upgrade that skipped them would leave
      // first-onboard guards running against current doctrine — silently, and while
      // still reporting up_to_date, because subsetManaged() ignores repo_hooks when
      // the flag is absent. Every prior upgrade did exactly that.
      repoHooks: args.pluginSource ? false : args.repoHooks || hasRepoHooksRecorded(repoRoot),
      claudeMd: args.claudeMd,
      globalSkills: args.globalSkills,
      syncSkills: !args.pluginSource,
      forceDowngrade: args.forceDowngrade,
      // GH #22 P0-1: threaded through so computePlan/applyPlan can gate the
      // codex-hybrid path (pluginSource && runtimeCoversCodex(runtime)) —
      // see the --runtime block comment at the top of this file.
      pluginSource: args.pluginSource,
      runtime: args.runtime,
    };
    // Advisor R6 (point 6): surfaced in BOTH plan and apply notices, so a
    // dry-run already shows the transition before anything is written.
    const hooksTransitionNotices = repoHooksTransitionNotices(repoRoot, {
      pluginSource: args.pluginSource,
      codexHybrid: args.pluginSource && runtimeCoversCodex(args.runtime),
    });
    if (!args.apply) {
      const { plan, beeVersion, skillSync, worktreeMigration } = computePlan(repoRoot, options);
      // msn-18d: the migration conflict gate outranks EVERY other status,
      // including skillSync.blocked — an untrustworthy coordination store is
      // more fundamental than a stale skill tree, and plan mode must report
      // it honestly even though (being read-only) it never refuses to run.
      const migrationBlocked = worktreeMigration.conflicts.length > 0;
      const payload = {
        repo_root: repoRoot,
        // Blocked-first across targets (D5): any blocked target's status wins.
        status: migrationBlocked
          ? "blocked_worktree_migration_conflict"
          : skillSync.blocked
            ? skillSync.blocked.status
            : coreChangesNeeded(plan)
              ? "changes_needed"
              : "up_to_date",
        // Source identity of THIS launcher (DIST-04, SRC-01): the same detector
        // status uses. Report-only — the authoritative-source decision stays
        // with identityOk/computeSkillSync; this only names what ran.
        source: sourceKindForReport(),
        bee_version: beeVersion,
        plan,
        skills: {
          source_root: skillSync.source_root,
          // Per-target collection: [{kind, target_root, mode, blocked,
          // versions, items}]. Review P1-6 / D2: each target's items are
          // computed whenever its refusal is forceable (empty [] otherwise) -
          // a blocked dry-run must still show exactly which skills a
          // --force-downgrade would overwrite/delete per target, not just the
          // general-item plan.
          targets: skillSync.targets,
        },
        notices: [
          ...commandsNotices(repoRoot, { firstOnboard }),
          ...staleAdvisorNotices(repoRoot),
          ...trackedPathsNotices(repoRoot),
          ...hooksTransitionNotices,
        ],
      };
      // Only surfaced for a linked worktree (applicable) — an ordinary/main/
      // solo checkout's payload carries no worktree_migration key at all
      // (must_have truth: zero behavior change there).
      if (worktreeMigration.applicable) {
        payload.worktree_migration = {
          blocked: migrationBlocked,
          pending: worktreeMigration.records.filter((r) => r.status !== "conflict").length,
          stranded: worktreeMigration.conflicts.map((c) => ({ path: c.path, id: c.id, kind: c.kind, reason: c.reason })),
        };
      }
      if (migrationBlocked) {
        payload.reason = buildMigrationConflictReason(worktreeMigration.conflicts);
      } else if (skillSync.blocked) {
        // Reporting is not failing: plan mode exits 0 with the blocked status.
        // Top-level reason/versions are blocked-first aggregates (first
        // blocked target's versions; every blocked target named in reason).
        payload.reason = skillSync.blocked.reason;
        payload.versions = skillSync.blocked.versions;
      }
      emit(payload, args.json);
      return 0;
    }

    const result = applyPlan(repoRoot, options);
    if (result.blocked) {
      // Refused apply: zero mutations happened; exit nonzero (D3).
      const refusalPayload = {
        repo_root: repoRoot,
        status: result.blocked.status,
        bee_version: result.beeVersion,
        reason: result.blocked.reason,
        versions: result.versions,
        // Review P1-6 / D2: same forced-apply-transparency payload as plan
        // mode - this refused response is what most users see BEFORE
        // deciding whether to pass --force-downgrade, so it must carry every
        // target's computed items too.
        skills: result.skills,
      };
      // P49: thread the host-lib blast radius through to the emitted refusal
      // payload, top-level sibling beside `skills`. Present (possibly empty)
      // only when applyPlan() computed it for a forceable refusal; absent
      // otherwise.
      if (result.host_items !== undefined) {
        refusalPayload.host_items = result.host_items;
      }
      // msn-18d: a worktree-migration-conflict refusal names every stranded
      // record machine-readably, same shape as plan mode's worktree_migration.
      if (result.blocked.stranded !== undefined) {
        refusalPayload.worktree_migration = { blocked: true, stranded: result.blocked.stranded };
      }
      emit(refusalPayload, args.json);
      return 1;
    }
    const recheck = computePlan(repoRoot, options);
    // Review P1-7: computePlan() withholds skill items from `plan` while its
    // skillSync stage is blocked (see step 7 above), so `recheck.plan.length`
    // alone can go to zero - and falsely report up_to_date - while the skill
    // stage itself is still genuinely blocked (reachable after a forced
    // downgrade that left one skill mid-refusal, e.g. a residual per-skill
    // symlink/alias block that keeps its version marker un-synced). Blocked-
    // first precedence, aggregated across ALL targets (D5): recheck can NEVER
    // read "up_to_date" while ANY target is still blocked.
    const recheckBlocked = recheck.skillSync.blocked;
    const payload = {
      repo_root: repoRoot,
      status: "applied",
      bee_version: result.beeVersion,
      applied: result.applied,
      recheck: recheckBlocked
        ? recheckBlocked.status
        : coreChangesNeeded(recheck.plan)
          ? "changes_needed"
          : "up_to_date",
      recheck_plan: recheck.plan,
      recheck_skills: recheckBlocked
        ? {
            blocked: true,
            reason: recheckBlocked.reason,
            // Top-level versions obey blocked-first aggregation (the first
            // blocked target's triple); targets carries the per-target state.
            versions: recheckBlocked.versions,
            targets: recheck.skillSync.targets.map((t) => ({
              kind: t.kind,
              target_root: t.target_root,
              blocked: t.blocked,
              versions: t.versions,
            })),
          }
        : null,
      skills: result.skills,
      onboarding: result.onboarding,
      notices: [
        ...commandsNotices(repoRoot, { firstOnboard }),
        ...staleAdvisorNotices(repoRoot),
        ...trackedPathsNotices(repoRoot),
        ...hooksTransitionNotices,
      ],
    };
    if (result.forcedDowngrade) {
      // F9: a forced apply reports the fact machine-readably, with the
      // overridden versions (blocked-first: first blocked target, pre-force).
      payload.forced_downgrade = true;
      payload.versions = result.forcedVersions;
    }
    emit(payload, args.json);
    return 0;
  } catch (error) {
    process.stdout.write(
      `${JSON.stringify({ error: String((error && error.message) || error) })}\n`,
    );
    return 1;
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === SCRIPT_PATH) {
  process.exitCode = main();
}
