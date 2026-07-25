#!/usr/bin/env node
// End-to-end proof for the top-level installers (cell installer-version-parity-1-3-1-3,
// decisions D2 + D8). This runs scripts/install.sh as a real child process inside a
// fully isolated fixture and asserts the WRAPPER contract — not merely its helpers:
//
//   * pre-confirmation / dry-run paths perform ZERO mutating plugin calls and ZERO
//     target/home writes (only read-only `plugin list` status probes are allowed);
//   * the plugin transition happens ONLY after confirmation, proven by PATH-isolated
//     fake Codex/Claude CLIs that log every call and classify read-only vs mutating;
//   * greenfield (missing + empty) and brownfield runs finish on ONE exact release
//     version with complete onboarding, no drift, and an immediate up_to_date recheck;
//   * refusals (noninteractive, missing CLI for plugin-first, unavailable source,
//     package-list shape drift, mixed source tuple) fail loudly with no unauthorized
//     mutation;
//   * an injected post-transition/pre-onboarding failure restores the exact pre-run
//     plugin state, leaves the target unchanged, and reports both the primary failure
//     and any rollback failure without ever reporting success;
//   * a repeat install reports "current" without timestamp-only managed rewrites.
//
// Isolation guarantees (deny sentinels): HOME / USERPROFILE / CLAUDE_HOME / CODEX_HOME
// point inside the temp sandbox, PATH excludes the real ~/.local/bin so the real codex
// / claude binaries can never be reached, and a canary tree outside the sandbox is
// asserted untouched. Nothing in this test writes to a real user home or real plugin.
//
// Usage: node scripts/tests/test_installers_e2e.mjs --installer bash
// Only the Bash installer is proven by this cell; PowerShell is a later release slice.

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.resolve(__dirname, "..", "..");
const INSTALL_SH = path.join(REPO_ROOT, "scripts", "install.sh");
const INSTALL_PS1 = path.join(REPO_ROOT, "scripts", "install.ps1");
const SOURCE_VERSION = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, ".claude-plugin/plugin.json"), "utf8")).version;

// A clean PATH: node's own dir plus the standard bins. It deliberately OMITS the
// real ~/.local/bin, so the real codex/claude CLIs are unreachable. Fake CLIs are
// prepended per-run when a fixture wants them present.
const BASE_PATH = [path.dirname(process.execPath), "/usr/bin", "/bin", "/usr/local/bin"]
  .filter((dir, i, all) => all.indexOf(dir) === i)
  .join(path.delimiter);

let passed = 0;
let failed = 0;
const cleanups = [];

function check(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`PASS ${name}`);
  } catch (error) {
    failed += 1;
    console.error(`FAIL ${name}: ${error.stack ?? error.message}`);
  }
}

function sha256(buf) {
  return createHash("sha256").update(buf).digest("hex");
}

// Byte-and-mode digest of a whole tree (missing path -> stable sentinel). Used to
// prove "zero writes" / "byte-idempotent" postconditions.
// Volatile bee RUNTIME caches (not managed onboarding artifacts). `bee.mjs status`
// stamps .bee/cache/manifest-hash.json ({ hash, checked_at }) on every read, and
// logs churn; excluding these lets an idempotency check target the MANAGED files
// only. `.bee/cache` is bee's derived/scratch dir (GitHub #11) — skip it wholesale.
// `.bee/logs` is excluded wholesale too (rb-3): every file under it is a
// fail-open, append-only runtime log — never installer-managed content, and
// already git-ignored in full (.gitignore: `.bee/logs/`). It used to list only
// `.bee/logs/hooks.jsonl`, which missed `.bee/logs/timings.jsonl` (the
// per-invocation direct-run timing log bee.mjs itself appends to on every
// call, decision 4439bd7e/work-visibility D3) — install.sh's own final
// verification step shells out to `node .bee/bin/bee.mjs status --json`
// (scripts/install.sh:487), so a repeat install appends a second timings.jsonl
// line and the tree digest churns even though every actually-managed file is
// byte-identical. Excluding the whole directory (matching lib/compaction.mjs's
// own "every other .bee/logs/*.jsonl" convention) closes this for any future
// sibling log too, not just this one file.
const RUNTIME_CACHE = [".bee/cache", ".bee/logs", ".git"];

function treeDigest(root, ignore = []) {
  if (!fs.existsSync(root)) return "ABSENT";
  if (fs.statSync(root).isFile()) return `FILE:${sha256(fs.readFileSync(root))}`;
  const skip = new Set(ignore);
  const rows = [];
  const walk = (current) => {
    for (const entry of fs.readdirSync(current, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const target = path.join(current, entry.name);
      const rel = path.relative(root, target).split(path.sep).join("/");
      if (skip.has(rel)) continue;
      if (entry.isSymbolicLink()) rows.push([rel, "link", fs.readlinkSync(target)]);
      else if (entry.isDirectory()) { rows.push([rel, "dir"]); walk(target); }
      else rows.push([rel, "file", sha256(fs.readFileSync(target)), (fs.statSync(target).mode & 0o777).toString(8)]);
    }
  };
  walk(root);
  return sha256(Buffer.from(JSON.stringify(rows)));
}

function readLog(logPath) {
  if (!fs.existsSync(logPath)) return [];
  return fs.readFileSync(logPath, "utf8").split("\n").filter(Boolean).map((line) => JSON.parse(line));
}

// ─── PATH-isolated fake runtime CLI ──────────────────────────────────────────
// One shared implementation, invoked as `codex <args>` / `claude <args>` through
// tiny shims. It records every call to BEE_FAKE_CALLLOG (with a mutating flag),
// tracks plugin install state in BEE_FAKE_STATE, and never writes anywhere else.
const FAKE_CLI = `#!/usr/bin/env node
const fs = require("node:fs");
const runtime = process.argv[2];
const rest = process.argv.slice(3);
const statePath = process.env.BEE_FAKE_STATE;
const logPath = process.env.BEE_FAKE_CALLLOG;
const pkgRoot = process.env.BEE_FAKE_PKG_ROOT || null;
const version = process.env.BEE_FAKE_VERSION || "0.0.0";
const malformed = process.env.BEE_FAKE_MALFORMED === "1";
const fails = (process.env.BEE_FAKE_FAIL || "").split(",").filter(Boolean);

const loadState = () => { try { return JSON.parse(fs.readFileSync(statePath, "utf8")); } catch { return {}; } };
const saveState = (s) => fs.writeFileSync(statePath, JSON.stringify(s));

const [group, sub, ...args] = rest;
let verb = "unknown";
let mutating = false;
if (group === "plugin") {
  if (sub === "list") { verb = "list"; mutating = false; }
  else if (sub === "marketplace" && args[0] === "add") { verb = "marketplace-add"; mutating = true; }
  else if (sub === "add" || sub === "install") { verb = "install"; mutating = true; }
  else if (sub === "remove" || sub === "uninstall") { verb = "remove"; mutating = true; }
}
if (logPath) fs.appendFileSync(logPath, JSON.stringify({ runtime, verb, sub: sub ?? null, mutating, argv: rest }) + "\\n");

// Real-CLI fidelity: \`plugin list\` accepts --json, but the MUTATION verbs
// (marketplace add, add/install, remove/uninstall) reject it with an
// unknown-option error and a nonzero exit. This makes the suite fail if the
// installer ever passes --json to a mutation again (the field regression).
if (mutating && rest.includes("--json")) {
  process.stderr.write("error: unknown option '--json'\\n");
  process.exit(2);
}

if (fails.includes(runtime + ":" + verb) || fails.includes(runtime + ":" + (sub ?? ""))) {
  process.stderr.write("fake " + runtime + " " + verb + " forced failure\\n");
  process.exit(7);
}

if (verb === "list") {
  if (malformed) { process.stdout.write("<<not json at all>>\\n"); process.exit(0); }
  const rec = loadState()[runtime] || { installed: false };
  const plugins = rec.installed
    ? [{ name: "bee", installed: true, enabled: true, version: rec.version || version, install_path: rec.root || pkgRoot, sourceKind: rec.sourceKind || "marketplace" }]
    : [];
  process.stdout.write(JSON.stringify({ plugins }) + "\\n");
  process.exit(0);
}
if (verb === "marketplace-add") { process.exit(0); }
if (verb === "install") { const s = loadState(); s[runtime] = { installed: true, root: pkgRoot, version, sourceKind: "marketplace" }; saveState(s); process.exit(0); }
if (verb === "remove") {
  // Real-CLI fidelity: removing a plugin that is NOT installed is an error with
  // a nonzero exit. An honest rollback must therefore never call remove on a
  // never-installed plugin — this is exactly the field regression it guards.
  const s = loadState();
  if (!(s[runtime] && s[runtime].installed)) { process.stderr.write("error: plugin bee@bee is not installed\\n"); process.exit(1); }
  s[runtime] = { installed: false, root: null, version: null, sourceKind: null }; saveState(s); process.exit(0);
}
process.exit(0);
`;

// ─── self-consistent staged source (for plugin-first package proof) ──────────
// The real working-tree manifest does not match the working tree (later cells edit
// package files without re-manifesting). For plugin-first the installer proves the
// installed package against the manifest byte-for-byte, so we stage a copy of the
// tracked tree, regenerate ITS manifest, and build a fake package that matches it.
let stagedCache = null;
function stagedSource() {
  if (stagedCache) return stagedCache;
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "ivp-src-"));
  cleanups.push(root);
  const src = path.join(root, "src");
  fs.mkdirSync(src, { recursive: true });
  // Copy tracked working-tree files (respects .gitignore, excludes .git).
  const files = execFileSync("git", ["-C", REPO_ROOT, "ls-files", "-z"], { encoding: "buffer" })
    .toString("utf8").split("\0").filter(Boolean);
  for (const rel of files) {
    const from = path.join(REPO_ROOT, rel);
    if (!fs.existsSync(from)) continue;
    const to = path.join(src, rel);
    fs.mkdirSync(path.dirname(to), { recursive: true });
    fs.copyFileSync(from, to);
  }
  // Regenerate the staged manifest so it matches the staged package files exactly.
  execFileSync("node", [path.join(src, "scripts/release_manifest.mjs"), "--write"], { cwd: src, stdio: "ignore" });
  // Build a fake installed package = exactly the package files named by the manifest.
  const pkg = path.join(root, "pkg");
  // Kept in lockstep with plugin_distribution.mjs's PACKAGE_ROLES (D9/cnr2-12):
  // the committed per-runtime rendered skill trees are now expected package
  // content too, alongside the unchanged canonical skills/ tree.
  const roles = new Set([
    "plugin_skill",
    "plugin_skill_claude_render",
    "plugin_skill_codex_render",
    "plugin_hook",
    "plugin_manifest",
    "plugin_marketplace",
    // D5 (packages-restructure): packages/bee/ vendor payload role.
    "package_payload",
  ]);
  const manifest = JSON.parse(fs.readFileSync(path.join(src, "docs/history/codex-harness-hardening/release-manifest.json"), "utf8"));
  for (const record of manifest.files) {
    if (!roles.has(record.role)) continue;
    const rel = record.packagePath ?? record.path;
    const to = path.join(pkg, rel);
    fs.mkdirSync(path.dirname(to), { recursive: true });
    fs.copyFileSync(path.join(src, rel), to);
  }
  const stagedVersion = JSON.parse(fs.readFileSync(path.join(src, ".claude-plugin/plugin.json"), "utf8")).version;
  stagedCache = { src, pkg, version: stagedVersion };
  return stagedCache;
}

// ─── sandbox + runner ────────────────────────────────────────────────────────
function sandbox({ targetName = "target", preinstalled = false, version = SOURCE_VERSION, pkgRoot = null } = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "ivp-e2e-"));
  cleanups.push(root);
  const home = path.join(root, "home");
  const claudeHome = path.join(root, "claude-home");
  const codexHome = path.join(root, "codex-home");
  const bin = path.join(root, "bin");
  for (const dir of [home, claudeHome, codexHome, bin]) fs.mkdirSync(dir, { recursive: true });
  // Canary OUTSIDE every managed home — a deny sentinel: it must never be touched.
  const canary = path.join(root, "OUTSIDE-CANARY");
  fs.writeFileSync(canary, "do-not-touch\n");

  const fakeMjs = path.join(bin, "_fake_cli.cjs");
  fs.writeFileSync(fakeMjs, FAKE_CLI);
  for (const rt of ["codex", "claude"]) {
    const shim = path.join(bin, rt);
    fs.writeFileSync(shim, `#!/usr/bin/env bash\nexec node "${fakeMjs}" ${rt} "$@"\n`);
    fs.chmodSync(shim, 0o755);
  }

  const statePath = path.join(root, "plugins-state.json");
  const logPath = path.join(root, "calls.log");
  fs.writeFileSync(statePath, JSON.stringify(preinstalled
    ? { codex: { installed: true, root: pkgRoot, version, sourceKind: "marketplace" }, claude: { installed: true, root: pkgRoot, version, sourceKind: "marketplace" } }
    : { codex: { installed: false }, claude: { installed: false } }));
  fs.writeFileSync(logPath, "");

  const target = path.join(root, targetName);
  return { root, home, claudeHome, codexHome, bin, canary, statePath, logPath, target, version, pkgRoot };
}

// sandboxEnv — the ONE place that builds the isolated child env (deny
// sentinel: HOME/USERPROFILE/CLAUDE_HOME/CODEX_HOME point inside the sandbox,
// PATH excludes the real ~/.local/bin). Passing `env` to spawnSync/execFileSync
// REPLACES the entire child environment (node child_process semantics) rather
// than merging with process.env, so every caller that builds its env through
// this helper is fully sealed off from whatever HOME/CODEX_HOME/PATH the
// outer test process (or a developer's shell) happens to have — not just the
// install run itself, but every POST-install verification call too (E2E env
// seal, hardening-8: before this, assertVersionParity's status/onboard-recheck
// calls and the codex-plugin-first doctor check carried no `env` option at
// all, so they silently inherited the real environment).
function sandboxEnv(sb, { withFakes = true, extraEnv = {} } = {}) {
  return {
    PATH: withFakes ? `${sb.bin}${path.delimiter}${BASE_PATH}` : BASE_PATH,
    HOME: sb.home,
    USERPROFILE: sb.home,
    CLAUDE_HOME: sb.claudeHome,
    CODEX_HOME: sb.codexHome,
    LANG: "C.UTF-8",
    LC_ALL: "C.UTF-8",
    BEE_FAKE_STATE: sb.statePath,
    BEE_FAKE_CALLLOG: sb.logPath,
    BEE_FAKE_PKG_ROOT: sb.pkgRoot ?? "",
    BEE_FAKE_VERSION: sb.version,
    ...extraEnv,
  };
}

function run(sb, { args = [], withFakes = true, extraEnv = {}, input = "" } = {}) {
  const env = sandboxEnv(sb, { withFakes, extraEnv });
  const result = spawnSync("bash", [INSTALL_SH, ...args], { env, input, encoding: "utf8", timeout: 240000 });
  return { code: result.status, signal: result.signal, stdout: result.stdout ?? "", stderr: result.stderr ?? "", out: `${result.stdout ?? ""}${result.stderr ?? ""}` };
}

// Everything a run may legitimately write lives under one of the managed homes or
// the target. This digests all of them plus the canary — for zero-write assertions.
function managedDigest(sb) {
  return [sb.target, sb.home, sb.claudeHome, sb.codexHome, sb.canary].map((p) => `${path.basename(p)}:${treeDigest(p)}`).join("|");
}

function mutatingCalls(sb) {
  return readLog(sb.logPath).filter((c) => c.mutating);
}

function assertVersionParity(sb, expected = SOURCE_VERSION, { sourceRoot = REPO_ROOT, onboardFlags = [] } = {}) {
  // Both post-install verification calls below run AFTER install.sh's own
  // (already-sealed) run, so they must carry the exact same sandbox env — an
  // execFileSync/spawnSync call with no explicit `env` inherits the outer
  // test process's real environment instead (node child_process semantics),
  // silently reopening the isolation this suite's header promises (E2E env
  // seal, hardening-8).
  const statusRaw = execFileSync("node", [".bee/bin/bee.mjs", "status", "--json"], { cwd: sb.target, encoding: "utf8", env: sandboxEnv(sb) });
  const status = JSON.parse(statusRaw);
  assert.equal(status.onboarding?.installed, true, "onboarding.installed must be true");
  assert.equal(status.onboarding?.bee_version, expected, "bee_version must equal source");
  assert.equal(status.onboarding?.plugin_version, expected, "plugin_version must equal source");
  assert.equal(status.onboarding?.drift, false, "status must report no drift");
  // Independent immediate up_to_date recheck (mirrors the flags the installer used).
  const planRaw = execFileSync("node", [path.join(sourceRoot, "skills/bee-hive/scripts/onboard_bee.mjs"), "--repo-root", sb.target, "--json", ...onboardFlags], { encoding: "utf8", env: sandboxEnv(sb) });
  assert.equal(JSON.parse(planRaw).status, "up_to_date", "onboarding must be up_to_date immediately after apply");
}

// ─── ARGUMENT PARSING ────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
let installer = "bash";
for (let i = 0; i < argv.length; i += 1) {
  if (argv[i] === "--installer") installer = argv[i + 1];
}
if (installer !== "bash") {
  console.error(`test_installers_e2e: only --installer bash is implemented in this slice (got ${installer}).`);
  console.error("PowerShell entrypoint execution is a later release slice (D5) and must run on real Windows.");
  process.exit(2);
}

// Quick environment sanity: without bash and git there is nothing to prove.
if (spawnSync("bash", ["-n", INSTALL_SH], { encoding: "utf8" }).status !== 0) {
  console.error("test_installers_e2e: install.sh failed to parse");
  process.exit(1);
}

console.log(`test_installers_e2e --installer bash (source version ${SOURCE_VERSION})\n`);

// Regression guard (install-ps1-hooks-1): the Windows installer's sparse
// checkout MUST fetch every top-level tree onboard_bee.mjs reads from
// PLUGIN_ROOT. `hooks/` was omitted, so listPluginHooks() saw an absent dir,
// vendored zero hooks into .bee/bin/hooks/, yet .codex/hooks.json was still
// written full (hardcoded render) -> every Codex hook MODULE_NOT_FOUND.
check("install.ps1 sparse-checkout set fetches every tree onboarding reads (incl. hooks)", () => {
  const ps1 = fs.readFileSync(INSTALL_PS1, "utf8");
  const m = ps1.match(/sparse-checkout set ([^\r\n]+)/);
  assert.ok(m, "install.ps1 must contain a `sparse-checkout set` line");
  const roots = m[1].trim().split(/\s+/);
  for (const required of ["skills", "packages", ".claude-plugin", ".codex-plugin", "docs/history/codex-harness-hardening"]) {
    assert.ok(roots.includes(required), `sparse-checkout set must include ${required} (got: ${roots.join(" ")})`);
  }
});

// Regression guard (hardening-8, E2E env seal): every POST-install
// verification call this suite makes (assertVersionParity's status +
// onboard-recheck, the codex-plugin-first doctor check) must pass an
// explicit `env: sandboxEnv(...)` — passing `env` REPLACES the entire child
// environment (node child_process semantics), so this is what seals a
// post-install check off from whatever HOME/CODEX_HOME/PATH the outer test
// process (or a developer's shell) happens to have, exactly like the actual
// install run already is. Before this cell these three call sites carried no
// `env` option at all and silently inherited the real environment — a leak
// this static source check pins so it can never quietly regress.
check("every post-install verification call in this suite carries the sandbox env (no real HOME/CODEX_HOME leak)", () => {
  const selfSource = fs.readFileSync(__filename, "utf8");
  const postInstallCallSites = [
    /execFileSync\("node", \[".bee\/bin\/bee\.mjs", "status", "--json"\][^;]*?\);/s,
    /execFileSync\("node", \[path\.join\(sourceRoot, "skills\/bee-hive\/scripts\/onboard_bee\.mjs"\)[^;]*?\);/s,
    /spawnSync\("node", \[".bee\/bin\/bee\.mjs", "doctor", "--runtime", "codex", "--json"\][^;]*?\);/s,
  ];
  for (const re of postInstallCallSites) {
    const m = re.exec(selfSource);
    assert.ok(m, `expected post-install call site not found for pattern: ${re}`);
    assert.match(m[0], /env:\s*sandboxEnv\(/, `call site must pass env: sandboxEnv(sb) — no post-install call may inherit the real process env: ${m[0]}`);
  }
});

// ── 1. greenfield MISSING target: dir created, one exact version, up_to_date ──
check("greenfield missing target: creates dir, one exact version, complete onboarding, no drift", () => {
  const sb = sandbox({ targetName: "does-not-exist-yet", preinstalled: true, pkgRoot: null });
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(r.code, 0, `install must succeed:\n${r.out}`);
  assert.ok(fs.existsSync(sb.target), "greenfield target directory must be created");
  assertVersionParity(sb);
  // advisor-and-orchestration Slice 3B: the real install.sh path (not just
  // onboard_bee.mjs's own unit tests) must render the three config-tiered
  // agent files (AO5) and must never create an .agents/ agents root (AO11).
  for (const name of ["bee-gather", "bee-extract", "bee-review"]) {
    const agentFile = path.join(sb.target, ".claude", "agents", `${name}.md`);
    assert.ok(fs.existsSync(agentFile), `installer must render ${name}.md under .claude/agents/`);
    const text = fs.readFileSync(agentFile, "utf8");
    assert.ok(!text.includes("{{TIER_MODEL}}"), `${name}.md must have its {{TIER_MODEL}} placeholder rendered`);
  }
  assert.ok(!fs.existsSync(path.join(sb.target, ".agents", "agents")),
    "installer must never create an .agents/agents root (AO11 - Codex gets no agent files)");
});

// The 10 hooks repo-copy vendors into <target>/.bee/bin/hooks (the manual
// skills-copy route does not load plugin hooks, so onboarding must ship them).
const VENDORED_HOOKS = [
  "adapter.mjs", "bee-chain-nudge.mjs", "bee-codex-subagent-audit.mjs",
  "bee-model-guard.mjs", "bee-prompt-context.mjs", "bee-session-close.mjs",
  "bee-session-init.mjs", "bee-state-sync.mjs", "bee-tools-logger.mjs",
  "bee-write-guard.mjs",
];

// ── 2. greenfield EMPTY target (repo-copy) ────────────────────────────────────
check("greenfield empty target: finishes on one exact version with no drift, vendors all 10 hooks", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(r.code, 0, `install must succeed:\n${r.out}`);
  assertVersionParity(sb);
  // repo-copy vendors the hook runtime: .bee/bin/hooks/ must exist with all 10 files.
  const hooksDir = path.join(sb.target, ".bee/bin/hooks");
  assert.ok(fs.existsSync(hooksDir) && fs.statSync(hooksDir).isDirectory(), ".bee/bin/hooks/ must be vendored by repo-copy onboarding");
  for (const hook of VENDORED_HOOKS) {
    assert.ok(fs.existsSync(path.join(hooksDir, hook)), `vendored hook ${hook} must exist in .bee/bin/hooks/`);
  }
});

// ── 3. brownfield: existing repo, owner content outside markers preserved ─────
check("brownfield: existing repo onboarded, owner content outside BEE markers preserved byte-for-byte", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  execFileSync("git", ["init", "-q"], { cwd: sb.target });
  const ownerAgents = "# My Project\n\nHand-written notes that must survive.\n";
  fs.writeFileSync(path.join(sb.target, "AGENTS.md"), ownerAgents);
  fs.writeFileSync(path.join(sb.target, "README.md"), "owner readme\n");
  const readmeBefore = sha256(fs.readFileSync(path.join(sb.target, "README.md")));
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(r.code, 0, `install must succeed:\n${r.out}`);
  assertVersionParity(sb);
  assert.equal(sha256(fs.readFileSync(path.join(sb.target, "README.md"))), readmeBefore, "owner README must be untouched");
  const agentsAfter = fs.readFileSync(path.join(sb.target, "AGENTS.md"), "utf8");
  assert.ok(agentsAfter.includes("Hand-written notes that must survive."), "owner AGENTS.md prose must survive");
  assert.ok(agentsAfter.includes("BEE:START"), "BEE block must be merged into AGENTS.md");
});

// ── 4. dry-run: ZERO target/home writes AND ZERO mutating plugin calls ────────
check("dry-run performs zero target/home writes and zero mutating plugin calls (read-only probe only)", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  const before = managedDigest(sb);
  const stateBefore = fs.readFileSync(sb.statePath, "utf8");
  const r = run(sb, { args: ["-d", sb.target, "-y", "--dry-run", "--source", REPO_ROOT] });
  assert.equal(r.code, 0, `dry-run must succeed:\n${r.out}`);
  assert.equal(managedDigest(sb), before, "dry-run must not write to target or any home");
  assert.equal(fs.readFileSync(sb.statePath, "utf8"), stateBefore, "dry-run must not mutate plugin state");
  assert.equal(mutatingCalls(sb).length, 0, "dry-run must issue zero mutating plugin calls");
  const probes = readLog(sb.logPath);
  assert.ok(probes.length > 0 && probes.every((c) => c.verb === "list"), "dry-run may only issue read-only `plugin list` probes");
});

// ── 5. ordering: transition (mutating calls) happens ONLY after confirmation ──
check("plugin transition occurs only after confirmation: mutating calls absent in preview, present after -y", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  // Preview (dry-run) = everything before the confirmation gate.
  run(sb, { args: ["-d", sb.target, "-y", "--dry-run", "--source", REPO_ROOT] });
  const previewCalls = readLog(sb.logPath);
  assert.ok(previewCalls.length > 0, "preview must probe plugin state");
  assert.equal(previewCalls.filter((c) => c.mutating).length, 0, "preview must contain no mutating plugin calls");
  // Confirmed run appends transition calls after the probe.
  fs.writeFileSync(sb.logPath, "");
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(r.code, 0, `install must succeed:\n${r.out}`);
  const calls = readLog(sb.logPath);
  const firstMutation = calls.findIndex((c) => c.mutating);
  assert.ok(firstMutation > 0, "a mutating transition must appear after at least one read-only probe");
  assert.ok(calls.slice(0, firstMutation).every((c) => c.verb === "list"), "every call before the first mutation must be a read-only probe");
  // repo-copy transition is a removal.
  assert.ok(calls.some((c) => c.verb === "remove"), "repo-copy must remove the plugin as its transition");
});

// ── 6. noninteractive without --yes: refuses at the confirm gate, zero mutation ─
check("noninteractive without --yes refuses at confirmation with zero plugin mutation and zero target write", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  const before = managedDigest(sb);
  const r = run(sb, { args: ["-d", sb.target, "--source", REPO_ROOT], input: "" });
  assert.notEqual(r.code, 0, "must refuse without --yes and without a TTY");
  assert.equal(mutatingCalls(sb).length, 0, "a refused confirmation must not mutate any plugin");
  assert.equal(managedDigest(sb), before, "a refused confirmation must not write to target or home");
});

// ── 7. paths with spaces and Unicode ──────────────────────────────────────────
check("target path with spaces and Unicode onboards to one exact version", () => {
  const sb = sandbox({ targetName: "my proj ✓ дир", preinstalled: true });
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(r.code, 0, `install must succeed for a spaced/unicode path:\n${r.out}`);
  assertVersionParity(sb);
});

// ── 8. missing runtime CLI: repo-copy tolerates, still finishes ───────────────
check("missing runtime CLIs: repo-copy still finishes on one exact version", () => {
  const sb = sandbox();
  fs.mkdirSync(sb.target, { recursive: true });
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT], withFakes: false });
  assert.equal(r.code, 0, `repo-copy must tolerate absent runtime CLIs:\n${r.out}`);
  assertVersionParity(sb);
  assert.equal(mutatingCalls(sb).length, 0, "no fake CLI present, so no plugin calls should be logged");
});

// ── 9. unavailable source: fails loudly, nothing mutated ──────────────────────
check("unavailable --source fails loudly without mutation", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  const before = managedDigest(sb);
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", path.join(sb.root, "no-such-bee-checkout")] });
  assert.notEqual(r.code, 0, "must fail when the source path does not exist");
  assert.match(r.out, /source path not found/i, "must name the unavailable source");
  assert.equal(mutatingCalls(sb).length, 0, "an unavailable source must not mutate any plugin");
  assert.equal(managedDigest(sb), before, "an unavailable source must not write to target or home");
});

// ── 10. package-list shape drift: plugin-first probe returns non-JSON ─────────
check("package-list shape drift (malformed plugin list) fails loudly pre-confirmation with zero mutation", () => {
  const sb = sandbox();
  fs.mkdirSync(sb.target, { recursive: true });
  const before = managedDigest(sb);
  const r = run(sb, { args: ["-d", sb.target, "-y", "--no-git-init", "--distribution", "plugin-first", "--source", REPO_ROOT], extraEnv: { BEE_FAKE_MALFORMED: "1" } });
  assert.notEqual(r.code, 0, "malformed plugin-list output must fail the install");
  assert.match(r.out, /shape drift|probe/i, "must report the probe/shape failure");
  assert.equal(mutatingCalls(sb).length, 0, "shape drift is detected during the read-only probe, before any mutation");
  assert.equal(managedDigest(sb), before, "shape drift must not write to target or home");
});

// ── 11. mixed source tuple: refused before any confirmation/mutation ──────────
check("mixed source release tuple is refused before any confirmation, transition, or target write", () => {
  const staged = stagedSource();
  const mixedRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ivp-mixed-"));
  cleanups.push(mixedRoot);
  const mixedSrc = path.join(mixedRoot, "src");
  execFileSync("bash", ["-c", `cp -a "${staged.src}/." "${mixedSrc}/"`]);
  const codexManifest = path.join(mixedSrc, ".codex-plugin/plugin.json");
  const j = JSON.parse(fs.readFileSync(codexManifest, "utf8"));
  j.version = "9.9.9";
  fs.writeFileSync(codexManifest, `${JSON.stringify(j, null, 2)}\n`);
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  const before = managedDigest(sb);
  const r = run(sb, { args: ["-d", sb.target, "-y", "--no-git-init", "--source", mixedSrc] });
  assert.notEqual(r.code, 0, "a mixed source tuple must be refused");
  assert.match(r.out, /refused before any change|tuple/i, "must report a pre-mutation refusal");
  assert.equal(mutatingCalls(sb).length, 0, "a mixed tuple must be refused before any plugin transition");
  assert.equal(managedDigest(sb), before, "a mixed tuple must not write to target or home");
});

// ── 12. plugin-first: transition after confirm, package proven, up_to_date ────
check("plugin-first: installs package after confirmation, proves it, finishes up_to_date and cleans project fallbacks", () => {
  const staged = stagedSource();
  const sb = sandbox({ preinstalled: false, version: staged.version, pkgRoot: staged.pkg });
  fs.mkdirSync(sb.target, { recursive: true });
  // Seed a managed project fallback skill + a project-owned bee-custom that must survive.
  fs.mkdirSync(path.join(sb.target, ".claude/skills/bee-hive"), { recursive: true });
  fs.writeFileSync(path.join(sb.target, ".claude/skills/bee-hive/SKILL.md"), "stale legacy\n");
  fs.mkdirSync(path.join(sb.target, ".claude/skills/bee-custom"), { recursive: true });
  fs.writeFileSync(path.join(sb.target, ".claude/skills/bee-custom/SKILL.md"), "custom owned\n");
  const r = run(sb, { args: ["-d", sb.target, "-y", "--distribution", "plugin-first", "--source", staged.src] });
  assert.equal(r.code, 0, `plugin-first install must succeed:\n${r.out}`);
  // The transition installed the plugin only after a mutation-free preview.
  const calls = readLog(sb.logPath);
  const firstMutation = calls.findIndex((c) => c.mutating);
  assert.ok(firstMutation > 0, "plugin-first transition must follow a read-only probe");
  assert.ok(calls.some((c) => c.verb === "install"), "plugin-first transition must install the plugin");
  assert.ok(calls.some((c) => c.verb === "marketplace-add"), "plugin-first must register the marketplace before install");
  // Version parity against the STAGED source version.
  assertVersionParity(sb, staged.version, { sourceRoot: staged.src, onboardFlags: ["--plugin-source"] });
  // Managed fallback removed; project-owned bee-custom preserved.
  assert.equal(fs.existsSync(path.join(sb.target, ".claude/skills/bee-hive")), false, "managed project fallback must be cleaned");
  assert.equal(fs.readFileSync(path.join(sb.target, ".claude/skills/bee-custom/SKILL.md"), "utf8"), "custom owned\n", "project-owned bee-custom must survive");
});

// ── 12a. codex plugin-first: post-cleanup end state has hooks + no skill copies ─
// GH #22 P0-1 (cph-1) + cph-2's --runtime passthrough: a `--runtime codex
// --distribution plugin-first` install must land the codex-hybrid write
// (.codex/hooks.json + vendored .bee/bin/hooks/) and finish with skills
// resolving from the plugin only — no .claude/skills or .agents/skills copies
// left behind for Codex to load instead of the plugin package.
check("codex plugin-first: post-cleanup end state has codex-hybrid hooks and no repo-local skill copies", () => {
  const staged = stagedSource();
  const sb = sandbox({ preinstalled: false, version: staged.version, pkgRoot: staged.pkg });
  fs.mkdirSync(sb.target, { recursive: true });
  const r = run(sb, { args: ["-d", sb.target, "-y", "--runtime", "codex", "--distribution", "plugin-first", "--source", staged.src] });
  assert.equal(r.code, 0, `codex plugin-first install must succeed:\n${r.out}`);
  assertVersionParity(sb, staged.version, { sourceRoot: staged.src, onboardFlags: ["--plugin-source", "--runtime", "codex"] });

  // .codex/hooks.json exists with the required bee matchers.
  const hooksJsonPath = path.join(sb.target, ".codex", "hooks.json");
  assert.ok(fs.existsSync(hooksJsonPath), ".codex/hooks.json must exist after a codex plugin-first install");
  const hooksJson = JSON.parse(fs.readFileSync(hooksJsonPath, "utf8"));
  const requiredEvents = ["SessionStart", "UserPromptSubmit", "PreToolUse", "PostToolUse", "SubagentStart", "SubagentStop", "PreCompact", "Stop"];
  for (const event of requiredEvents) {
    assert.ok(Array.isArray(hooksJson.hooks?.[event]) && hooksJson.hooks[event].length > 0, `.codex/hooks.json must carry a ${event} entry`);
  }
  const allCommands = Object.values(hooksJson.hooks).flat().flatMap((group) => group.hooks || []).map((h) => h.command);
  assert.ok(allCommands.some((c) => /\.bee\/bin\/hooks\/bee-write-guard\.mjs/.test(c)), "bee-write-guard must be wired");
  assert.ok(allCommands.some((c) => /\.bee\/bin\/hooks\/bee-model-guard\.mjs/.test(c)), "bee-model-guard must be wired");
  const preToolUseMatchers = (hooksJson.hooks.PreToolUse || []).map((g) => g.matcher);
  assert.ok(preToolUseMatchers.includes("spawn_agent"), "PreToolUse must carry the Codex spawn_agent matcher for bee-model-guard");

  // Vendored .bee/bin/hooks/ exists with all 10 canonical hook handlers
  // (same VENDORED_HOOKS set the repo-copy greenfield test proves above —
  // codex-hybrid reuses the exact same listPluginHooks()/copy_repo_hook path).
  const vendoredDir = path.join(sb.target, ".bee", "bin", "hooks");
  assert.ok(fs.existsSync(vendoredDir), ".bee/bin/hooks/ must be vendored for a codex-hybrid install");
  for (const hook of VENDORED_HOOKS) {
    assert.ok(fs.existsSync(path.join(vendoredDir, hook)), `vendored hook ${hook} must exist in .bee/bin/hooks/`);
  }

  // Skills resolve from the plugin, not repo-local copies: plugin-first never
  // syncs skills, so neither project skill root should exist.
  assert.equal(fs.existsSync(path.join(sb.target, ".claude", "skills")), false, "plugin-first must not leave a .claude/skills copy");
  assert.equal(fs.existsSync(path.join(sb.target, ".agents", "skills")), false, "plugin-first must not leave an .agents/skills copy");

  // `bee.mjs doctor --runtime codex --json` in the sandbox target: hooks_file_present
  // must be ok; the trust/discovery rows stay unknown on codex-cli 0.145.0 (no
  // machine-readable hook-discovery/trust surface — CODEX_DOCTOR_TRUST_UNKNOWN_REASON),
  // which is EXPECTED, not fought. Since g22-3's three-state re-class, these four
  // rows carry `degrades: true` + a non-empty `degraded_reason` (never `blocking`
  // anymore). NOTE: this scenario does not assert doctor.overall_status —
  // unrelated blocking rows (capability_baseline_match, skills_installed; see
  // g22-4) independently hold the fresh-sandbox verdict at `blocked` here, so a
  // `degraded` expectation would not hold; that is a separate, out-of-scope
  // concern from the trust-row re-class this cell fixes.
  // Post-install verification call: same sandbox env seal as assertVersionParity above.
  const doctorResult = spawnSync("node", [".bee/bin/bee.mjs", "doctor", "--runtime", "codex", "--json"], { cwd: sb.target, encoding: "utf8", env: sandboxEnv(sb) });
  assert.equal(doctorResult.status, 0, `doctor must run cleanly in the sandbox target:\n${doctorResult.stdout}\n${doctorResult.stderr}`);
  let doctor;
  try {
    doctor = JSON.parse(doctorResult.stdout);
  } catch (error) {
    assert.fail(`doctor --json must produce parsable JSON: ${error.message}\nstdout:\n${doctorResult.stdout}`);
  }
  const rowsByName = Object.fromEntries(doctor.rows.map((row) => [row.row, row]));
  assert.equal(rowsByName.hooks_file_present?.status, "ok", `hooks_file_present must be ok: ${JSON.stringify(rowsByName.hooks_file_present)}`);
  for (const trustRow of ["hooks_discovered", "hooks_trusted", "project_trust", "pending_hook_review"]) {
    assert.equal(rowsByName[trustRow]?.status, "unknown", `${trustRow} must stay unknown on codex-cli 0.145.0: ${JSON.stringify(rowsByName[trustRow])}`);
    assert.equal(rowsByName[trustRow]?.degrades, true, `${trustRow} must be flagged degrades: ${JSON.stringify(rowsByName[trustRow])}`);
    assert.ok(
      typeof rowsByName[trustRow]?.degraded_reason === "string" && rowsByName[trustRow].degraded_reason.length > 0,
      `${trustRow} must carry a non-empty degraded_reason: ${JSON.stringify(rowsByName[trustRow])}`,
    );
    assert.notEqual(rowsByName[trustRow]?.blocking, true, `${trustRow} must NOT be flagged blocking: ${JSON.stringify(rowsByName[trustRow])}`);
  }
});

// ── 12b. hook-write-impossible fixture: typed refusal, plugin rollback, no skills-only end state ─
check("codex plugin-first refuses when .codex is a pre-existing file, rolls the plugin back, leaves no skills-only end state", () => {
  const staged = stagedSource();
  const sb = sandbox({ preinstalled: false, version: staged.version, pkgRoot: staged.pkg });
  fs.mkdirSync(sb.target, { recursive: true });
  // Pre-create .codex as a regular FILE — the write-impossible fixture: the
  // codex-hybrid hook write can never create .codex/hooks.json under it.
  fs.writeFileSync(path.join(sb.target, ".codex"), "not-a-directory\n");
  const r = run(sb, { args: ["-d", sb.target, "-y", "--no-git-init", "--runtime", "codex", "--distribution", "plugin-first", "--source", staged.src] });
  assert.notEqual(r.code, 0, "a pre-existing .codex file must refuse the install");
  // A pre-existing .codex FILE trips $DIST_HELPER's own project-cleanup probe
  // (it walks PROJECT_SKILL_ROOTS including .codex/skills) BEFORE onboarding's
  // apply step is ever reached, so the surfaced typed refusal is plugin_distribution.mjs's
  // caught {status:"blocked", error:...} JSON (an ENOTDIR lstat under the file),
  // not onboard_bee.mjs's own curated codexHookWriteBlocker message — that
  // message only fires for an obstacle collectProjectCleanup never touches
  // (e.g. .bee/bin/hooks pre-occupied by a file, proven in the doc comment
  // above the fix-options helper in install.sh). Both are typed refusals
  // (a status field, not a raw stack trace) that roll the plugin back.
  assert.match(r.out, /"status":\s*"blocked"/, "must surface a typed (status: blocked) refusal, not an uncaught crash");
  assert.match(r.out, /\.codex/, "must name .codex somewhere in the refusal (path under the pre-existing file)");
  assert.match(r.out, /Distribution preflight refused after transition/, "must report the distribution preflight as the failing step");
  assert.match(r.out, /fix options:/, "must name the fix options (repo-copy, or clear the obstacle and retry hybrid)");
  assert.match(r.out, /--distribution repo-copy/, "must name repo-copy as a fix option");
  assert.match(r.out, /retry.*hybrid|re-run.*plugin-first/i, "must name retrying hybrid after clearing the obstacle as a fix option");
  assert.match(r.out, /rollback: pre-run plugin state restored/i, "must roll the plugin transition back");

  // End state must NOT be skills-only: no plugin left installed, no repo
  // skills claiming success.
  const stateAfter = JSON.parse(fs.readFileSync(sb.statePath, "utf8"));
  assert.equal(stateAfter.codex.installed, false, "codex plugin must be rolled back to its pre-run not-installed state");
  assert.equal(fs.existsSync(path.join(sb.target, ".bee")), false, "the write-impossible preflight fires before any target write — no .bee/ at all");
  assert.equal(fs.existsSync(path.join(sb.target, ".claude", "skills")), false, "no .claude/skills copy may appear when the install refused");
  assert.equal(fs.existsSync(path.join(sb.target, ".agents", "skills")), false, "no .agents/skills copy may appear when the install refused");
  // The fixture file itself survives untouched (never coerced into a directory).
  assert.equal(fs.statSync(path.join(sb.target, ".codex")).isDirectory(), false, ".codex must remain the pre-existing plain file, never coerced into a directory");
  assert.equal(fs.readFileSync(path.join(sb.target, ".codex"), "utf8"), "not-a-directory\n", ".codex file content must be untouched");
});

// ── 12c. claude regression guard: exclusivity unchanged, no codex artifacts ───
check("claude plugin-first regression guard: repo-local claude hook entries stripped as today, no codex-hybrid artifacts appear", () => {
  const staged = stagedSource();
  const sb = sandbox({ preinstalled: false, version: staged.version, pkgRoot: staged.pkg });
  fs.mkdirSync(sb.target, { recursive: true });
  // Seed a stale repo-local bee hook entry in .claude/settings.json (as a prior
  // --repo-hooks or plugin-first run might have left behind) — plugin-first
  // cleanup must strip it exactly as it does today, --runtime claude included.
  fs.mkdirSync(path.join(sb.target, ".claude"), { recursive: true });
  fs.writeFileSync(
    path.join(sb.target, ".claude", "settings.json"),
    JSON.stringify({
      hooks: {
        SessionStart: [{ hooks: [{ type: "command", command: 'node "$CLAUDE_PROJECT_DIR"/.bee/bin/hooks/bee-session-init.mjs', statusMessage: "bee: session bootstrap" }] }],
      },
    }, null, 2) + "\n",
  );
  const r = run(sb, { args: ["-d", sb.target, "-y", "--runtime", "claude", "--distribution", "plugin-first", "--source", staged.src] });
  assert.equal(r.code, 0, `claude plugin-first install must succeed:\n${r.out}`);
  assertVersionParity(sb, staged.version, { sourceRoot: staged.src, onboardFlags: ["--plugin-source", "--runtime", "claude"] });

  // Exclusivity unchanged: the repo-local claude bee hook entry is stripped.
  const settings = JSON.parse(fs.readFileSync(path.join(sb.target, ".claude", "settings.json"), "utf8"));
  assert.equal(settings.hooks?.SessionStart, undefined, "the stale repo-local bee SessionStart entry must be stripped, exactly as today");

  // No codex-hybrid artifacts appear for a claude-only run.
  assert.equal(fs.existsSync(path.join(sb.target, ".codex", "hooks.json")), false, "--runtime claude must never write .codex/hooks.json");
  assert.equal(fs.existsSync(path.join(sb.target, ".bee", "bin", "hooks")), false, "--runtime claude must never vendor .bee/bin/hooks/ (no codex-hybrid, no --repo-hooks)");
});

// ── 13. injected post-transition/pre-onboarding failure: rollback + report ────
check("injected post-transition failure restores pre-run plugin state, leaves target unchanged, exits nonzero", () => {
  const sb = sandbox({ preinstalled: true }); // repo-copy pre-run state: bee INSTALLED
  fs.mkdirSync(sb.target, { recursive: true });
  const targetBefore = treeDigest(sb.target);
  const stateBefore = JSON.parse(fs.readFileSync(sb.statePath, "utf8"));
  assert.equal(stateBefore.codex.installed, true, "precondition: bee installed pre-run");
  const r = run(sb, { args: ["-d", sb.target, "-y", "--no-git-init", "--source", REPO_ROOT], extraEnv: { BEE_INSTALL_FAULT_AFTER_TRANSITION: "1" } });
  assert.notEqual(r.code, 0, "an injected post-transition failure must exit nonzero (never success)");
  assert.match(r.out, /injected post-transition fault/i, "must report the primary failure");
  assert.match(r.out, /rollback: pre-run plugin state restored/i, "must report a successful rollback");
  const stateAfter = JSON.parse(fs.readFileSync(sb.statePath, "utf8"));
  assert.equal(stateAfter.codex.installed, true, "codex plugin must be restored to its pre-run installed state");
  assert.equal(stateAfter.claude.installed, true, "claude plugin must be restored to its pre-run installed state");
  assert.equal(treeDigest(sb.target), targetBefore, "an injected pre-onboarding failure must leave the target unchanged");
});

// ── 14. rollback failure is reported alongside the primary, never swallowed ───
check("rollback failure is reported alongside the primary failure and never converted to success", () => {
  const sb = sandbox({ preinstalled: true }); // repo-copy: transition removes; rollback re-adds
  fs.mkdirSync(sb.target, { recursive: true });
  // Force the re-add path of rollback to fail (install/add verbs).
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT], extraEnv: { BEE_INSTALL_FAULT_AFTER_TRANSITION: "1", BEE_FAKE_FAIL: "codex:install,codex:add,claude:install,claude:add,codex:marketplace,claude:marketplace" } });
  assert.notEqual(r.code, 0, "must exit nonzero");
  assert.match(r.out, /injected post-transition fault/i, "must report the primary failure");
  assert.match(r.out, /rollback failed/i, "must report the rollback failure alongside the primary");
});

// ── 15b. never-installed no-op rollback: transition dies BEFORE installing ────
// The field regression: plugin-first on a machine with no pre-existing bee plugin,
// the transition fails at `marketplace add`, and the old rollback blindly ran
// `remove bee@bee` — which the real CLI rejects for a not-installed plugin (rc=1),
// misreporting "rollback failed to fully restore the pre-run plugin state" although
// nothing was ever installed. An honest rollback re-probes: current == pre-run for
// every runtime, so it is a NO-OP SUCCESS that never touches a never-installed plugin.
check("never-installed transition failure rolls back as an honest no-op (no remove-of-absent, no false rollback failure)", () => {
  const staged = stagedSource();
  const sb = sandbox({ preinstalled: false, version: staged.version, pkgRoot: staged.pkg });
  fs.mkdirSync(sb.target, { recursive: true });
  const stateBefore = JSON.parse(fs.readFileSync(sb.statePath, "utf8"));
  assert.equal(stateBefore.codex.installed, false, "precondition: bee NOT installed pre-run");
  // Kill the very first mutation (`marketplace add`) so nothing is ever installed.
  const r = run(sb, {
    args: ["-d", sb.target, "-y", "--distribution", "plugin-first", "--source", staged.src],
    extraEnv: { BEE_FAKE_FAIL: "codex:marketplace,claude:marketplace" },
  });
  assert.notEqual(r.code, 0, "a failed transition must exit nonzero");
  assert.match(r.out, /Plugin transition failed/i, "must report the primary transition failure");
  assert.match(r.out, /rollback: pre-run plugin state restored/i, "a never-installed rollback must be an honest no-op success");
  assert.doesNotMatch(r.out, /rollback failed/i, "must NOT misreport a failed rollback when nothing was ever installed");
  const calls = readLog(sb.logPath);
  assert.equal(calls.some((c) => c.verb === "remove"), false, "rollback must not remove a never-installed plugin");
  const stateAfter = JSON.parse(fs.readFileSync(sb.statePath, "utf8"));
  assert.equal(stateAfter.codex.installed, false, "codex plugin must remain not-installed (unchanged)");
  assert.equal(stateAfter.claude.installed, false, "claude plugin must remain not-installed (unchanged)");
});

// ── 15. repeat install: reports current, no timestamp-only managed rewrites ───
check("repeat install reports current without timestamp-only managed rewrites (byte-idempotent target)", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  const first = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(first.code, 0, `first install must succeed:\n${first.out}`);
  // Managed-file digest excludes bee's own runtime caches (.bee/manifest-hash.json
  // carries a checked_at timestamp refreshed on every status read).
  const afterFirst = treeDigest(sb.target, RUNTIME_CACHE);
  const onboardingBefore = sha256(fs.readFileSync(path.join(sb.target, ".bee/onboarding.json")));
  const second = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(second.code, 0, `repeat install must succeed:\n${second.out}`);
  assert.match(second.out, /already current|up_to_date/i, "repeat install must report already current");
  assert.equal(treeDigest(sb.target, RUNTIME_CACHE), afterFirst, "repeat install must not rewrite any managed file (no timestamp churn)");
  assert.equal(sha256(fs.readFileSync(path.join(sb.target, ".bee/onboarding.json"))), onboardingBefore, "onboarding.json must not be rewritten on a repeat install");
});

// ── 15a. extra-file-only drift (unmanaged .mjs in .bee/bin/lib/) is a warning,
//        not a hard-fail: installer-verify-extra-drift-1. computeRuntimeDrift
//        (packages/bee/bee.mjs) flags any .mjs file living in
//        .bee/bin/lib/ that the recorded managed.lib ledger does not name as
//        "(extra)" drift -- a much softer signal than a real hash mismatch,
//        because onboard_bee.mjs's plan/apply never touches a name outside
//        both the current template lib set AND the previously-recorded
//        managed.lib keys (it only adds/updates names the source still has,
//        or removes names the ledger used to record -- see 3c above). Planting
//        a foreign .mjs the ledger never recorded survives apply untouched, so
//        install.sh's final verify step must warn and continue instead of
//        hard-failing version parity.
check("extra unmanaged .mjs in .bee/bin/lib/ does not hard-fail install.sh's verify step, and the output names it", () => {
  const sb = sandbox({ preinstalled: true });
  fs.mkdirSync(sb.target, { recursive: true });
  const first = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(first.code, 0, `first install must succeed:\n${first.out}`);
  assertVersionParity(sb);

  const onboarding = JSON.parse(fs.readFileSync(path.join(sb.target, ".bee/onboarding.json"), "utf8"));
  const plantedName = "unmanaged-extra-drift-probe.mjs";
  assert.ok(!Object.prototype.hasOwnProperty.call(onboarding.managed?.lib ?? {}, plantedName),
    `sanity: planted file name must not already be a managed lib entry (got managed.lib: ${Object.keys(onboarding.managed?.lib ?? {}).join(", ")})`);
  fs.writeFileSync(path.join(sb.target, ".bee/bin/lib", plantedName), "export const extraDriftProbe = true;\n");

  const statusBefore = JSON.parse(execFileSync("node", [".bee/bin/bee.mjs", "status", "--json"], { cwd: sb.target, encoding: "utf8", env: sandboxEnv(sb) }));
  assert.equal(statusBefore.onboarding?.drift, true, "planting an unmanaged lib file must be visible as drift before the fix is proven");
  assert.ok((statusBefore.onboarding?.drift_detail ?? []).some((entry) => entry === `.bee/bin/lib/${plantedName} (extra)`),
    `drift_detail must name the planted file as "(extra)" (got: ${JSON.stringify(statusBefore.onboarding?.drift_detail)})`);

  const second = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT] });
  assert.equal(second.code, 0, `install.sh's verify step must not hard-fail on extra-file-only drift when versions already match:\n${second.out}`);
  assert.match(second.out, new RegExp(plantedName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
    `verify output must name the specific extra file path so a human/agent knows what to clear:\n${second.out}`);
  assert.doesNotMatch(second.out, /version parity failed/, "extra-file-only drift must not trigger the version-parity hard-fail message");

  assert.ok(fs.existsSync(path.join(sb.target, ".bee/bin/lib", plantedName)), "the planted extra file itself is untouched by this fix (apply never removes an unrecorded name)");
});

// ── 16. broken CLI on PATH: repo-copy warns and continues (field regression) ──
// Field report: a codex npm shim on PATH crashes when run (e.g. "Missing
// optional dependency @openai/codex-linux-x64" on Windows+WSL). Default
// repo-copy mode never needs codex to be runnable at all, so a broken-but-
// present CLI must degrade to a warning instead of hard-failing the probe.
check("broken codex on PATH: default repo-copy install succeeds with a warning", () => {
  const sb = sandbox();
  fs.mkdirSync(sb.target, { recursive: true });
  const r = run(sb, { args: ["-d", sb.target, "-y", "--source", REPO_ROOT], extraEnv: { BEE_FAKE_FAIL: "codex:list" } });
  assert.equal(r.code, 0, `repo-copy must tolerate a present-but-broken codex CLI:\n${r.out}`);
  assert.match(r.out, /codex/i, "must warn naming codex");
  assert.match(r.out, /not runnable/i, "must describe codex as not runnable");
  assert.match(r.out, /repo-copy/i, "must note repo-copy does not require it");
  assert.match(r.out, /fake codex list forced failure/, "captured probe stderr must be embedded in the warning, not lost");
  assert.ok(fs.existsSync(path.join(sb.target, ".bee", "onboarding.json")), "the target must still be onboarded despite the broken codex CLI");
});

// ── 16a. broken CLI on PATH: plugin-first still fails with the named way out ──
check("broken codex on PATH: plugin-first still fails and names the way out", () => {
  const staged = stagedSource();
  const sb = sandbox({ preinstalled: false, version: staged.version, pkgRoot: staged.pkg });
  fs.mkdirSync(sb.target, { recursive: true });
  const r = run(sb, {
    args: ["-d", sb.target, "-y", "--no-git-init", "--distribution", "plugin-first", "--source", staged.src],
    extraEnv: { BEE_FAKE_FAIL: "codex:list" },
  });
  assert.notEqual(r.code, 0, "plugin-first must still refuse when a required CLI is present but broken");
  assert.match(r.out, /codex/i, "must name codex as the broken CLI");
  assert.match(r.out, /repo-copy/i, "must name repo-copy as a way out");
  assert.match(r.out, /fake codex list forced failure/, "captured probe stderr must be surfaced before the refusal, not lost");
  assert.equal(mutatingCalls(sb).length, 0, "a broken required CLI must be caught before any mutation");
});

// ─── SUMMARY ───────────────────────────────────────────────────────────────────
for (const dir of cleanups) fs.rmSync(dir, { recursive: true, force: true });

console.log(`\ntest_installers_e2e (bash): ${passed} passed, ${failed} failed`);
if (failed) process.exit(1);
