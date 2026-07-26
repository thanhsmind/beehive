#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { createHash, randomUUID } from "node:crypto";
import { fileURLToPath } from "node:url";

const PROJECT_SKILL_ROOTS = [".claude/skills", ".agents/skills", ".codex/skills"];
const PROJECT_HOOK_FILES = [".claude/settings.json", ".codex/hooks.json"];
// GH #22 P0-1 / advisor R4 (self-erasure fix): .codex/hooks.json's bee
// entries as written by onboard_bee.mjs's codex-hybrid path (--plugin-source
// --runtime codex|both) are BYTE-IDENTICAL to the entries this cleanup
// pass's cleanHookConfig() strips — same "/.bee/bin/hooks/" command shape,
// same recognizedBeeCommand() match. Without a scoping flag, a plugin-first
// install that hybrid-wrote the ONLY mechanical enforcement Codex sessions
// get would have this very cleanup pass immediately delete it again, right
// after onboarding reported the apply successful.
const CODEX_HYBRID_EXEMPT_FILE = ".codex/hooks.json";
// D9/cnr2-12: plugin_skill_*_render cover the committed per-runtime rendered
// skill trees (.claude-plugin/skills/, .codex-plugin/skills/) that the plugin
// manifests now route to; they must be counted as expected package content
// (proveInstalledPackage) alongside the unchanged canonical plugin_skill tree.
const PACKAGE_ROLES = new Set([
  "plugin_skill",
  "plugin_skill_claude_render",
  "plugin_skill_codex_render",
  "plugin_hook",
  "plugin_manifest",
  "plugin_marketplace",
  // D5 (packages-restructure): the packages/bee/ vendor payload role,
  // enumerated by release_manifest.mjs's dedicated packages/bee walk.
  "package_payload",
]);
const BEE_HOOK_HANDLERS = new Set([
  "bee-session-init.mjs", "bee-prompt-context.mjs", "bee-write-guard.mjs",
  "bee-model-guard.mjs", "bee-state-sync.mjs", "bee-chain-nudge.mjs",
  "bee-session-close.mjs", "bee-codex-subagent-audit.mjs", "bee-tools-logger.mjs",
]);

function sha256(data) {
  return createHash("sha256").update(data).digest("hex");
}

function fail(message) {
  const error = new Error(message);
  error.code = "DISTRIBUTION_REFUSED";
  throw error;
}

function isInside(root, candidate) {
  const relative = path.relative(root, candidate);
  return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== "..");
}

function lstatOrNull(target) {
  try { return fs.lstatSync(target); } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
}

function assertPlainDirectory(target, label, { allowMissing = false } = {}) {
  const stat = lstatOrNull(target);
  if (!stat && allowMissing) return null;
  if (!stat) fail(`${label} is missing: ${target}`);
  if (stat.isSymbolicLink() || !stat.isDirectory()) fail(`${label} must be a plain directory: ${target}`);
  const resolved = fs.realpathSync.native(target);
  if (resolved !== path.resolve(target)) fail(`${label} aliases another path: ${target}`);
  return stat;
}

function assertPlainFile(target, label) {
  const stat = lstatOrNull(target);
  if (!stat || stat.isSymbolicLink() || !stat.isFile()) fail(`${label} must be a plain file: ${target}`);
  return stat;
}

function walkPlainFiles(root, relative = "") {
  const current = path.join(root, relative);
  const stat = lstatOrNull(current);
  if (!stat) return [];
  if (stat.isSymbolicLink()) fail(`symlink is forbidden in inventory: ${current}`);
  if (stat.isFile()) return [{ path: relative.split(path.sep).join("/"), abs: current, stat }];
  if (!stat.isDirectory()) fail(`unsupported inventory entry: ${current}`);
  const result = [];
  for (const entry of fs.readdirSync(current, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const childRel = path.join(relative, entry.name);
    if (entry.isSymbolicLink()) fail(`symlink is forbidden in inventory: ${path.join(root, childRel)}`);
    result.push(...walkPlainFiles(root, childRel));
  }
  return result;
}

function fileRecord(abs, rel) {
  const stat = assertPlainFile(abs, "inventory entry");
  return {
    path: rel.split(path.sep).join("/"),
    sha256: sha256(fs.readFileSync(abs)),
    mode: (stat.mode & 0o777).toString(8).padStart(3, "0"),
  };
}

export function loadPackageInventory(manifestPath) {
  assertPlainFile(manifestPath, "release manifest");
  let manifest;
  try { manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")); } catch (error) {
    fail(`release manifest is not valid JSON: ${error.message}`);
  }
  if (!manifest || !Array.isArray(manifest.files)) fail("release manifest has no files array");
  const inventory = manifest.files
    .filter((record) => PACKAGE_ROLES.has(record.role))
    .map((record) => ({
      path: record.packagePath ?? record.path,
      sha256: record.sha256,
      mode: record.mode,
      role: record.role,
    }))
    .sort((a, b) => a.path.localeCompare(b.path));
  if (inventory.length === 0) fail("release manifest has no package inventory records");
  const seen = new Set();
  for (const record of inventory) {
    if (!record.path || path.isAbsolute(record.path) || record.path.split(/[\\/]/).includes("..")) fail(`unsafe package path: ${record.path}`);
    if (!/^[a-f0-9]{64}$/.test(record.sha256) || !/^[0-7]{3}$/.test(record.mode)) fail(`malformed package record: ${record.path}`);
    if (seen.has(record.path)) fail(`duplicate package record: ${record.path}`);
    seen.add(record.path);
  }
  return inventory;
}

export function managedSkillNames(inventory) {
  if (!Array.isArray(inventory)) fail("release inventory is not an array");
  const names = new Set();
  for (const record of inventory) {
    if (record?.role !== "plugin_skill") continue;
    if (typeof record.path !== "string") fail(`release inventory has a malformed plugin skill record`);
    const segments = record.path.split("/");
    if (segments[0] !== "skills" || !segments[1]) fail(`release inventory has an unexpected plugin skill path: ${record.path}`);
    if (!/^bee-[a-z0-9-]+$/.test(segments[1])) fail(`release inventory names an unsafe managed skill: ${segments[1]}`);
    names.add(segments[1]);
  }
  if (names.size === 0) fail("release inventory names no managed plugin skills");
  return names;
}

function normalizePluginList(payload) {
  if (Array.isArray(payload)) return payload;
  for (const key of ["plugins", "items", "data"]) if (Array.isArray(payload?.[key])) return payload[key];
  return payload && typeof payload === "object" ? [payload] : [];
}

export function discoverBeePlugin(payload, runtime) {
  const candidates = normalizePluginList(payload);
  const plugin = candidates.find((item) => {
    const name = item?.name ?? item?.id ?? item?.plugin?.name;
    return name === "bee" || String(name ?? "").startsWith("bee@");
  });
  if (!plugin) return { runtime, installed: false, enabled: false, root: null, version: null };
  const state = String(plugin.status ?? plugin.state ?? "").toLowerCase();
  const installed = plugin.installed === true || !["removed", "not_installed"].includes(state);
  const enabled = installed && (plugin.enabled === true || ["enabled", "active"].includes(state));
  return {
    runtime,
    installed,
    enabled,
    root: plugin.root ?? plugin.path ?? plugin.installPath ?? plugin.install_path ?? plugin.sourcePath ?? plugin.source_path ?? null,
    version: plugin.version ?? plugin.plugin?.version ?? null,
    sourceKind: plugin.sourceKind ?? plugin.source_kind ?? plugin.provenance ?? null,
  };
}

export function proveInstalledPackage(state, expectedInventory) {
  if (!state?.installed || !state?.enabled) fail(`${state?.runtime ?? "runtime"} bee plugin is not installed and enabled`);
  if (["source_checkout", "checkout", "repository"].includes(String(state.sourceKind ?? "").toLowerCase())) fail("source checkout cannot substitute for an installed plugin package");
  if (typeof state.root !== "string" || !path.isAbsolute(state.root)) fail("enabled plugin did not report an absolute installed package root");
  const packageRoot = path.resolve(state.root);
  assertPlainDirectory(packageRoot, "installed package root");
  const expected = new Map(expectedInventory.map((record) => [record.path, record]));
  const prefixes = [...new Set(expectedInventory.map((record) => record.path.split("/")[0]))];
  const actual = new Map();
  for (const prefix of prefixes) {
    const prefixPath = path.join(packageRoot, prefix);
    const stat = lstatOrNull(prefixPath);
    if (!stat) continue;
    if (stat.isSymbolicLink()) fail(`installed package prefix is a symlink: ${prefix}`);
    if (stat.isFile()) actual.set(prefix, fileRecord(prefixPath, prefix));
    else for (const item of walkPlainFiles(prefixPath)) {
      const rel = [prefix, item.path].filter(Boolean).join("/");
      actual.set(rel, fileRecord(item.abs, rel));
    }
  }
  const missing = [...expected.keys()].filter((key) => !actual.has(key));
  const unexpected = [...actual.keys()].filter((key) => !expected.has(key));
  const changed = [...expected.entries()].filter(([key, record]) => {
    const value = actual.get(key);
    return value && (value.sha256 !== record.sha256 || value.mode !== record.mode);
  }).map(([key]) => key);
  if (missing.length || unexpected.length || changed.length) {
    fail(`installed package inventory mismatch (missing=${missing.join(",") || "none"}; unexpected=${unexpected.join(",") || "none"}; changed=${changed.join(",") || "none"})`);
  }
  return { root: packageRoot, files: actual.size, version: state.version };
}

export function provePluginInactive(states) {
  const active = states.filter((state) => state.installed || state.enabled);
  if (active.length) fail(`bee plugin remains active for: ${active.map((state) => state.runtime).join(", ")}`);
  return { inactive: states.map((state) => state.runtime) };
}

function recognizedBeeCommand(command) {
  if (typeof command !== "string") return false;
  const normalized = command.replaceAll("\\", "/");
  const handler = [...BEE_HOOK_HANDLERS].find((name) => normalized.includes(`/hooks/${name}`));
  if (!handler) return false;
  return normalized.includes("CLAUDE_PLUGIN_ROOT") || normalized.includes("CLAUDE_PROJECT_DIR") || normalized.includes("/.bee/bin/hooks/") || normalized.includes("/hooks/bee-");
}

function cleanHookConfig(absPath) {
  const stat = lstatOrNull(absPath);
  if (!stat) return null;
  assertPlainFile(absPath, "hook configuration");
  let json;
  try { json = JSON.parse(fs.readFileSync(absPath, "utf8")); } catch (error) { fail(`malformed hook configuration ${absPath}: ${error.message}`); }
  if (!json || typeof json !== "object" || Array.isArray(json)) fail(`hook configuration must be an object: ${absPath}`);
  if (json.hooks === undefined) return null;
  if (!json.hooks || typeof json.hooks !== "object" || Array.isArray(json.hooks)) fail(`hooks must be an object: ${absPath}`);
  let removed = 0;
  const next = structuredClone(json);
  for (const [event, groups] of Object.entries(next.hooks)) {
    if (!Array.isArray(groups)) fail(`hook event ${event} must be an array: ${absPath}`);
    next.hooks[event] = groups.filter((group) => {
      if (!group || typeof group !== "object" || Array.isArray(group) || !Array.isArray(group.hooks)) fail(`hook group ${event} is malformed: ${absPath}`);
      const kept = group.hooks.filter((hook) => {
        if (!hook || typeof hook !== "object" || Array.isArray(hook)) fail(`hook entry ${event} is malformed: ${absPath}`);
        const recognized = hook.type === "command" && recognizedBeeCommand(hook.command);
        if (recognized) removed += 1;
        return !recognized;
      });
      group.hooks = kept;
      return kept.length > 0;
    });
    if (next.hooks[event].length === 0) delete next.hooks[event];
  }
  if (Object.keys(next.hooks).length === 0) delete next.hooks;
  return removed ? { path: absPath, removed, before: fs.readFileSync(absPath), after: Buffer.from(`${JSON.stringify(next, null, 2)}\n`) } : null;
}

// codexHybrid (default false, GH #22 P0-1 / advisor R4): when true,
// .codex/hooks.json is EXCLUDED from this pass entirely — its bee entries
// are the codex-hybrid projection onboard_bee.mjs just wrote (the only
// mechanical enforcement Codex sessions get; codex-cli has no plugin-hook
// mechanism), and must survive this cleanup rather than be stripped again.
// .claude/settings.json cleanup is UNCHANGED either way: Claude's own plugin
// hooks work under plugin-first, so its repo-local entries (if any) are
// always stale and always stripped.
//
// CONTRACT (advisor R4's "pick the cleanest contract"): the default is
// false, so every EXISTING caller of collectProjectCleanup/buildDistributionPlan
// that never passes codexHybrid keeps today's behavior byte-for-byte — both
// hook files get their bee entries stripped, exactly as before this flag
// existed. codexHybrid is never inferred from `runtimes` (buildDistributionPlan's
// existing parameter): `runtimes` names which CLIENT plugin(s) this cleanup
// is proving installed/inactive for — a different axis from which hook
// projection onboarding wrote. The caller (install.sh, the next cell) owns
// deciding when a run is a codex-hybrid install and must pass codexHybrid:true
// on exactly those runs — this file only guarantees that, once told, it
// keeps its hands off .codex/hooks.json.
function collectProjectCleanup(repoRoot, managedSkills, { codexHybrid = false } = {}) {
  assertPlainDirectory(repoRoot, "repository root");
  if (!(managedSkills instanceof Set) || managedSkills.size === 0) fail("project cleanup requires the managed release skill set");
  const dirs = [];
  const seen = new Set();
  for (const relativeRoot of PROJECT_SKILL_ROOTS) {
    const root = path.join(repoRoot, relativeRoot);
    const stat = lstatOrNull(root);
    if (!stat) continue;
    assertPlainDirectory(root, "project skill root");
    if (!isInside(repoRoot, root)) fail(`project skill root escapes repository: ${root}`);
    for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
      if (!managedSkills.has(entry.name)) continue;
      const target = path.join(root, entry.name);
      if (entry.isSymbolicLink() || !entry.isDirectory()) fail(`managed cleanup target must be a direct plain directory: ${target}`);
      assertPlainDirectory(target, "managed cleanup target");
      const real = fs.realpathSync.native(target);
      if (seen.has(real)) fail(`duplicate cleanup target alias: ${target}`);
      seen.add(real);
      dirs.push(target);
    }
  }
  const hookFiles = codexHybrid
    ? PROJECT_HOOK_FILES.filter((relative) => relative !== CODEX_HYBRID_EXEMPT_FILE)
    : PROJECT_HOOK_FILES;
  const configs = hookFiles.map((relative) => cleanHookConfig(path.join(repoRoot, relative))).filter(Boolean);
  return { dirs: dirs.sort(), configs };
}

function readOwnershipLedger(ledgerPath, requestedRoots) {
  if (!requestedRoots.length) return { dirs: [], update: null };
  if (!ledgerPath) fail("user-root cleanup requires an ownership ledger");
  assertPlainFile(ledgerPath, "ownership ledger");
  let ledger;
  try { ledger = JSON.parse(fs.readFileSync(ledgerPath, "utf8")); } catch (error) { fail(`ownership ledger is not valid JSON: ${error.message}`); }
  if (ledger?.schemaVersion !== 1 || !Array.isArray(ledger.roots)) fail("ownership ledger has an unsupported shape");
  const requested = requestedRoots.map((root) => path.resolve(root));
  if (new Set(requested).size !== requested.length) fail("user skill roots contain a duplicate or alias");
  const dirs = [];
  const seen = new Set();
  for (const root of requested) {
    assertPlainDirectory(root, "user skill root");
    const matches = ledger.roots.filter((entry) => path.resolve(entry?.path ?? "") === root);
    if (matches.length !== 1 || !Array.isArray(matches[0].skills)) fail(`ownership ledger does not exactly name user root: ${root}`);
    for (const name of matches[0].skills) {
      if (typeof name !== "string" || !/^bee-[a-z0-9-]+$/.test(name)) fail(`ownership ledger contains unsafe skill name: ${name}`);
      const target = path.join(root, name);
      if (!isInside(root, target)) fail(`ledger target escapes root: ${target}`);
      const stat = lstatOrNull(target);
      if (!stat) continue;
      assertPlainDirectory(target, "ledger-owned skill");
      const real = fs.realpathSync.native(target);
      if (seen.has(real)) fail(`duplicate ledger target alias: ${target}`);
      seen.add(real);
      dirs.push(target);
    }
  }
  const next = structuredClone(ledger);
  for (const entry of next.roots) if (requested.includes(path.resolve(entry.path))) entry.skills = [];
  return { dirs, update: { path: ledgerPath, before: fs.readFileSync(ledgerPath), after: Buffer.from(`${JSON.stringify(next, null, 2)}\n`) } };
}

function snapshotTree(root) {
  const rows = [];
  const walk = (current, relative = "") => {
    const stat = lstatOrNull(current);
    if (!stat || stat.isSymbolicLink()) fail(`snapshot target is missing or symlinked: ${current}`);
    const mode = (stat.mode & 0o777).toString(8).padStart(3, "0");
    if (stat.isFile()) { rows.push([relative, "file", mode, sha256(fs.readFileSync(current))]); return; }
    if (!stat.isDirectory()) fail(`snapshot target has an unsupported entry: ${current}`);
    if (relative) rows.push([relative, "dir", mode]);
    for (const entry of fs.readdirSync(current).sort()) walk(path.join(current, entry), path.join(relative, entry.name ?? entry));
  };
  walk(root);
  return sha256(Buffer.from(JSON.stringify(rows)));
}

function snapshotTargets(dirs, writes) {
  return {
    dirs: dirs.map((target) => ({ path: target, digest: snapshotTree(target) })),
    writes: writes.map((item) => ({ path: item.path, digest: sha256(item.before) })),
  };
}

function revalidateSnapshot(snapshot) {
  for (const item of snapshot.dirs) {
    assertPlainDirectory(item.path, "cleanup target");
    const digest = snapshotTree(item.path);
    if (digest !== item.digest) fail(`cleanup target changed after preflight: ${item.path}`);
  }
  for (const item of snapshot.writes) {
    assertPlainFile(item.path, "planned configuration write");
    if (sha256(fs.readFileSync(item.path)) !== item.digest) fail(`configuration changed after preflight: ${item.path}`);
  }
}

export function buildDistributionPlan({ mode, runtimes, repoRoot, pluginStates, inventory, ledgerPath = null, userSkillRoots = [], codexHybrid = false }) {
  if (!['plugin-first', 'repo-copy'].includes(mode)) fail(`unknown distribution mode: ${mode}`);
  if (!Array.isArray(runtimes) || runtimes.length === 0) fail("at least one runtime is required");
  const selectedStates = runtimes.map((runtime) => pluginStates.find((state) => state.runtime === runtime) ?? { runtime, installed: false, enabled: false });
  if (mode === "repo-copy") {
    provePluginInactive(selectedStates);
    return { mode, runtimes, status: "ready_for_onboarding", dirs: [], writes: [], snapshot: { dirs: [], writes: [] } };
  }
  const managedSkills = managedSkillNames(inventory);
  const proofs = selectedStates.map((state) => proveInstalledPackage(state, inventory));
  const project = collectProjectCleanup(path.resolve(repoRoot), managedSkills, { codexHybrid });
  const user = readOwnershipLedger(ledgerPath, userSkillRoots);
  const dirs = [...project.dirs, ...user.dirs];
  const cleanupRealpaths = dirs.map((target) => fs.realpathSync.native(target));
  if (new Set(cleanupRealpaths).size !== cleanupRealpaths.length) fail("cleanup plan contains duplicate or aliased targets");
  const writes = [...project.configs, ...(user.update ? [user.update] : [])];
  const snapshot = snapshotTargets(dirs, writes);
  return { mode, runtimes, status: dirs.length || writes.length ? "changes_needed" : "up_to_date", proofs, dirs, writes, snapshot };
}

export function applyDistributionPlan(plan) {
  revalidateSnapshot(plan.snapshot);
  if (plan.mode === "repo-copy" || (!plan.dirs.length && !plan.writes.length)) return { status: plan.mode === "repo-copy" ? "ready_for_onboarding" : "up_to_date", removed: 0, updated: 0 };
  const token = randomUUID();
  const moved = [];
  const configMoves = [];
  try {
    for (const target of plan.dirs) {
      const quarantine = `${target}.bee-cleanup-${token}`;
      if (lstatOrNull(quarantine)) fail(`quarantine path already exists: ${quarantine}`);
      fs.renameSync(target, quarantine);
      moved.push({ target, quarantine });
    }
    for (const item of plan.writes) {
      const temp = `${item.path}.bee-write-${token}`;
      const backup = `${item.path}.bee-cleanup-${token}`;
      if (lstatOrNull(temp) || lstatOrNull(backup)) fail(`configuration transaction path already exists: ${item.path}`);
      const originalMode = fs.statSync(item.path).mode & 0o777;
      fs.renameSync(item.path, backup);
      configMoves.push({ item, temp, backup });
      fs.writeFileSync(temp, item.after, { mode: originalMode });
      fs.renameSync(temp, item.path);
    }
  } catch (error) {
    for (const config of configMoves.reverse()) {
      fs.rmSync(config.temp, { force: true });
      fs.rmSync(config.item.path, { force: true });
      if (lstatOrNull(config.backup)) fs.renameSync(config.backup, config.item.path);
    }
    for (const item of moved.reverse()) if (lstatOrNull(item.quarantine) && !lstatOrNull(item.target)) fs.renameSync(item.quarantine, item.target);
    throw error;
  }
  for (const item of moved) fs.rmSync(item.quarantine, { recursive: true, force: true });
  for (const config of configMoves) fs.rmSync(config.backup, { force: true });
  return { status: "applied", removed: moved.length, updated: configMoves.length };
}

function parseArgs(argv) {
  const options = { apply: false, userSkillRoots: [], codexHybrid: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--apply") options.apply = true;
    // GH #22 P0-1: boolean, like --apply — no value token follows it. Kept
    // OUT of the generic "--"-prefixed fallback below (which always consumes
    // a value arg) so `--codex-hybrid` never eats the next positional/flag.
    else if (arg === "--codex-hybrid") options.codexHybrid = true;
    else if (arg === "--user-skill-root") options.userSkillRoots.push(argv[++i]);
    else if (arg.startsWith("--")) options[arg.slice(2).replaceAll("-", "_")] = argv[++i];
    else fail(`unexpected argument: ${arg}`);
  }
  return options;
}

function parseRuntime(value) {
  if (value === "both") return ["claude", "codex"];
  if (["claude", "codex"].includes(value)) return [value];
  fail("--runtime must be claude, codex, or both");
}

function runCli() {
  const args = parseArgs(process.argv.slice(2));
  const runtimes = parseRuntime(args.runtime);
  if (!args.mode || !args.repo_root || !args.release_manifest || !args.plugin_state_file) fail("--mode, --runtime, --repo-root, --release-manifest, and --plugin-state-file are required");
  // Strip a leading UTF-8 BOM (U+FEFF) before parsing: install.ps1 writes this
  // state file via PowerShell `Set-Content -Encoding UTF8`, which on PS 5.1
  // prepends a BOM, and a bare JSON.parse then throws "Unexpected token" on it —
  // surfaced as "Distribution preflight refused" and a broken Windows install (#9).
  const payload = JSON.parse(fs.readFileSync(args.plugin_state_file, "utf8").replace(/^\uFEFF/, ""));
  const pluginStates = runtimes.map((runtime) => discoverBeePlugin(payload?.[runtime] ?? payload, runtime));
  const inventory = loadPackageInventory(args.release_manifest);
  const plan = buildDistributionPlan({ mode: args.mode, runtimes, repoRoot: args.repo_root, pluginStates, inventory, ledgerPath: args.ledger, userSkillRoots: args.userSkillRoots, codexHybrid: args.codexHybrid });
  const result = args.apply ? applyDistributionPlan(plan) : { status: plan.status, removed: plan.dirs.length, updated: plan.writes.length };
  process.stdout.write(`${JSON.stringify({ ok: true, mode: args.mode, runtimes, dryRun: !args.apply, ...result })}\n`);
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  try { runCli(); } catch (error) {
    process.stdout.write(`${JSON.stringify({ ok: false, status: "blocked", error: error.message })}\n`);
    process.exitCode = 1;
  }
}
