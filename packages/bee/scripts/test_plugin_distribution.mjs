#!/usr/bin/env node

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import {
  applyDistributionPlan,
  buildDistributionPlan,
  discoverBeePlugin,
  loadPackageInventory,
  managedSkillNames,
  proveInstalledPackage,
  provePluginInactive,
} from "./plugin_distribution.mjs";
import { renderSkillBytes, RENDER_RUNTIMES, RENDER_SIDECAR, walkSkillTree, skillDigest } from "./onboard_bee.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "../../..");
let passed = 0;
let failed = 0;

function check(name, fn) {
  try { fn(); passed += 1; console.log(`PASS ${name}`); }
  catch (error) { failed += 1; console.error(`FAIL ${name}: ${error.stack ?? error.message}`); }
}

function hash(file) { return createHash("sha256").update(fs.readFileSync(file)).digest("hex"); }
function mode(file) { return (fs.statSync(file).mode & 0o777).toString(8).padStart(3, "0"); }
function write(file, value) { fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, value); }
function treeDigest(root) {
  const rows = [];
  function walk(current) {
    if (!fs.existsSync(current)) return;
    for (const entry of fs.readdirSync(current, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const target = path.join(current, entry.name);
      const rel = path.relative(root, target).split(path.sep).join("/");
      if (entry.isSymbolicLink()) rows.push([rel, "link", fs.readlinkSync(target)]);
      else if (entry.isDirectory()) { rows.push([rel, "dir"]); walk(target); }
      else rows.push([rel, "file", hash(target), mode(target)]);
    }
  }
  walk(root);
  return JSON.stringify(rows);
}

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "bee-distribution-"));
  const repo = path.join(root, "repo");
  const pkg = path.join(root, "installed", "bee");
  write(path.join(pkg, "skills/bee-hive/SKILL.md"), "hive\n");
  write(path.join(pkg, "skills/bee-planning/SKILL.md"), "planning\n");
  write(path.join(pkg, "hooks/hooks.json"), "{}\n");
  write(path.join(pkg, "hooks/bee-write-guard.mjs"), "export {};\n");
  write(path.join(pkg, ".codex-plugin/plugin.json"), '{"name":"bee","version":"1.3.0"}\n');
  write(path.join(pkg, ".claude-plugin/plugin.json"), '{"name":"bee","version":"1.3.0"}\n');
  write(path.join(pkg, ".claude-plugin/marketplace.json"), '{"name":"bee"}\n');
  const inventory = [
    ["skills/bee-hive/SKILL.md", "plugin_skill"],
    ["skills/bee-planning/SKILL.md", "plugin_skill"],
    ["hooks/hooks.json", "plugin_hook"],
    ["hooks/bee-write-guard.mjs", "plugin_hook"],
    [".codex-plugin/plugin.json", "plugin_manifest"],
    [".claude-plugin/plugin.json", "plugin_manifest"],
    [".claude-plugin/marketplace.json", "plugin_marketplace"],
  ].map(([relative, role]) => ({ path: relative, role, sha256: hash(path.join(pkg, relative)), mode: mode(path.join(pkg, relative)) }));
  write(path.join(repo, ".claude/skills/bee-hive/SKILL.md"), "legacy\n");
  write(path.join(repo, ".agents/skills/bee-planning/SKILL.md"), "legacy\n");
  write(path.join(repo, ".codex/skills/user-skill/SKILL.md"), "user\n");
  write(path.join(repo, ".claude/skills/bee-custom/SKILL.md"), "custom\n");
  const beeCommand = 'node "$CLAUDE_PROJECT_DIR"/.bee/bin/hooks/bee-write-guard.mjs';
  write(path.join(repo, ".claude/settings.json"), `${JSON.stringify({ permissions: { allow: ["Read"] }, hooks: { PreToolUse: [{ matcher: "Bash", hooks: [{ type: "command", command: beeCommand }, { type: "command", command: "node user-hook.mjs" }] }] } }, null, 2)}\n`);
  write(path.join(repo, ".codex/hooks.json"), `${JSON.stringify({ hooks: { SessionStart: [{ hooks: [{ type: "command", command: beeCommand }] }] }, foreign: { keep: true } }, null, 2)}\n`);
  const states = ["claude", "codex"].map((runtime) => ({ runtime, installed: true, enabled: true, root: pkg, version: "1.3.0", sourceKind: "installed" }));
  return { root, repo, pkg, inventory, states };
}

function planFor(f, extra = {}) {
  return buildDistributionPlan({ mode: "plugin-first", runtimes: ["claude", "codex"], repoRoot: f.repo, pluginStates: f.states, inventory: f.inventory, ...extra });
}

check("discovers bee from Codex list JSON without trusting exit status", () => {
  const state = discoverBeePlugin({ plugins: [{ name: "bee", enabled: true, installed: true, install_path: "/tmp/pkg", version: "1.3.0" }] }, "codex");
  assert.equal(state.enabled, true); assert.equal(state.root, "/tmp/pkg");
});

check("listed package without explicit enabled status is not accepted", () => {
  const state = discoverBeePlugin({ plugins: [{ name: "bee", install_path: "/tmp/pkg" }] }, "codex");
  assert.equal(state.installed, true); assert.equal(state.enabled, false);
});

check("complete installed inventory passes", () => {
  const f = fixture();
  assert.equal(proveInstalledPackage(f.states[0], f.inventory).files, f.inventory.length);
  fs.rmSync(f.root, { recursive: true, force: true });
});

for (const [name, mutate] of [
  ["missing package file refuses", (f) => fs.rmSync(path.join(f.pkg, "hooks/hooks.json"))],
  ["changed package file refuses", (f) => fs.appendFileSync(path.join(f.pkg, "hooks/hooks.json"), "x")],
  ["unexpected package file refuses", (f) => write(path.join(f.pkg, "skills/extra/SKILL.md"), "x")],
  ["package symlink refuses", (f) => fs.symlinkSync(path.join(f.pkg, "hooks/hooks.json"), path.join(f.pkg, "hooks/link.json"))],
]) check(name, () => {
  const f = fixture(); mutate(f); assert.throws(() => proveInstalledPackage(f.states[0], f.inventory)); fs.rmSync(f.root, { recursive: true, force: true });
});

check("source checkout cannot stand in for installed package", () => {
  const f = fixture(); assert.throws(() => proveInstalledPackage({ ...f.states[0], sourceKind: "source_checkout" }, f.inventory), /source checkout/); fs.rmSync(f.root, { recursive: true, force: true });
});

check("failed package proof leaves every repository byte unchanged", () => {
  const f = fixture(); const before = treeDigest(f.repo); f.states[0].enabled = false;
  assert.throws(() => planFor(f)); assert.equal(treeDigest(f.repo), before); fs.rmSync(f.root, { recursive: true, force: true });
});

check("plugin-first removes only direct bee copies and recognized hooks", () => {
  const f = fixture(); const plan = planFor(f); const result = applyDistributionPlan(plan);
  assert.equal(result.status, "applied");
  assert.equal(fs.existsSync(path.join(f.repo, ".claude/skills/bee-hive")), false);
  assert.equal(fs.existsSync(path.join(f.repo, ".agents/skills/bee-planning")), false);
  assert.equal(fs.readFileSync(path.join(f.repo, ".codex/skills/user-skill/SKILL.md"), "utf8"), "user\n");
  assert.equal(fs.readFileSync(path.join(f.repo, ".claude/skills/bee-custom/SKILL.md"), "utf8"), "custom\n");
  const claude = JSON.parse(fs.readFileSync(path.join(f.repo, ".claude/settings.json")));
  assert.deepEqual(claude.permissions, { allow: ["Read"] });
  assert.equal(claude.hooks.PreToolUse[0].hooks.length, 1);
  assert.equal(claude.hooks.PreToolUse[0].hooks[0].command, "node user-hook.mjs");
  const codex = JSON.parse(fs.readFileSync(path.join(f.repo, ".codex/hooks.json")));
  assert.deepEqual(codex, { foreign: { keep: true } });
  fs.rmSync(f.root, { recursive: true, force: true });
});

// ─── GH #22 P0-1 / advisor R4: codex-hybrid cleanup scoping ────────────────
// onboard_bee.mjs's codex-hybrid path (--plugin-source --runtime codex|both)
// writes .codex/hooks.json bee entries that are byte-shape-identical to what
// this file's own cleanHookConfig() strips — without scoping, plugin-first
// cleanup would immediately erase the only mechanical enforcement Codex
// sessions get, right after onboarding reported the apply successful.

check("codex-hybrid: .codex/hooks.json bee entries survive cleanup; .claude/settings.json is still stripped", () => {
  const f = fixture();
  const result = applyDistributionPlan(planFor(f, { codexHybrid: true }));
  assert.equal(result.status, "applied");
  const codex = JSON.parse(fs.readFileSync(path.join(f.repo, ".codex/hooks.json")));
  assert.equal(
    codex.hooks.SessionStart[0].hooks[0].command,
    'node "$CLAUDE_PROJECT_DIR"/.bee/bin/hooks/bee-write-guard.mjs',
    "codex-hybrid bee entry survives cleanup",
  );
  assert.deepEqual(codex.foreign, { keep: true }, "non-bee .codex/hooks.json content survives untouched");
  const claude = JSON.parse(fs.readFileSync(path.join(f.repo, ".claude/settings.json")));
  assert.equal(claude.hooks.PreToolUse[0].hooks.length, 1, "claude settings.json bee entry is still stripped under hybrid");
  assert.equal(claude.hooks.PreToolUse[0].hooks[0].command, "node user-hook.mjs");
  fs.rmSync(f.root, { recursive: true, force: true });
});

check("codex-hybrid flag omitted (default false): legacy behavior stays byte-identical — both hook files stripped", () => {
  const f = fixture();
  const result = applyDistributionPlan(planFor(f));
  assert.equal(result.status, "applied");
  const codex = JSON.parse(fs.readFileSync(path.join(f.repo, ".codex/hooks.json")));
  assert.deepEqual(codex, { foreign: { keep: true } }, "without codexHybrid, .codex/hooks.json bee entry is stripped exactly as before this flag existed");
  fs.rmSync(f.root, { recursive: true, force: true });
});

check("cleanup candidates derive from exact plugin_skill inventory names only", () => {
  const f = fixture();
  const names = managedSkillNames(f.inventory);
  assert.deepEqual([...names].sort(), ["bee-hive", "bee-planning"]);
  assert.equal(names.has("bee-custom"), false);
  const plan = planFor(f);
  assert.ok(plan.dirs.some((d) => d.endsWith(path.join(".claude", "skills", "bee-hive"))), "plan targets managed bee-hive");
  assert.ok(plan.dirs.some((d) => d.endsWith(path.join(".agents", "skills", "bee-planning"))), "plan targets managed bee-planning");
  assert.ok(!plan.dirs.some((d) => d.includes("bee-custom")), "plan never targets bee-custom");
  fs.rmSync(f.root, { recursive: true, force: true });
});

check("project-owned bee-custom survives plugin-first cleanup untouched", () => {
  const f = fixture();
  const before = hash(path.join(f.repo, ".claude/skills/bee-custom/SKILL.md"));
  const result = applyDistributionPlan(planFor(f));
  assert.equal(result.status, "applied");
  assert.equal(fs.existsSync(path.join(f.repo, ".claude/skills/bee-hive")), false);
  assert.equal(hash(path.join(f.repo, ".claude/skills/bee-custom/SKILL.md")), before);
  fs.rmSync(f.root, { recursive: true, force: true });
});

for (const [name, mutate] of [
  ["release inventory with no managed plugin skills refuses before mutation", (inv) => inv.filter((r) => r.role !== "plugin_skill")],
  ["release inventory naming a non-bee managed skill refuses before mutation", (inv) => inv.map((r) => (r.role === "plugin_skill" && r.path.startsWith("skills/bee-hive/") ? { ...r, path: r.path.replace("skills/bee-hive/", "skills/rogue-tool/") } : r))],
]) check(name, () => {
  const f = fixture();
  const before = treeDigest(f.repo);
  assert.throws(() => planFor(f, { inventory: mutate(f.inventory) }));
  assert.equal(treeDigest(f.repo), before);
  fs.rmSync(f.root, { recursive: true, force: true });
});

check("post-plan drift on a managed target refuses apply and spares bee-custom", () => {
  const f = fixture(); const plan = planFor(f);
  write(path.join(f.repo, ".agents/skills/bee-planning/extra.txt"), "drift");
  const before = treeDigest(f.repo);
  assert.throws(() => applyDistributionPlan(plan), /changed after preflight/);
  assert.equal(treeDigest(f.repo), before);
  assert.equal(fs.readFileSync(path.join(f.repo, ".claude/skills/bee-custom/SKILL.md"), "utf8"), "custom\n");
  fs.rmSync(f.root, { recursive: true, force: true });
});

check("repeat managed cleanup is byte-idempotent and keeps bee-custom", () => {
  const f = fixture(); applyDistributionPlan(planFor(f)); const after1 = treeDigest(f.repo);
  const repeat = applyDistributionPlan(planFor(f));
  assert.equal(repeat.status, "up_to_date");
  assert.equal(treeDigest(f.repo), after1);
  assert.equal(fs.readFileSync(path.join(f.repo, ".claude/skills/bee-custom/SKILL.md"), "utf8"), "custom\n");
  fs.rmSync(f.root, { recursive: true, force: true });
});

check("repeat plugin-first apply is idempotent", () => {
  const f = fixture(); applyDistributionPlan(planFor(f)); const before = treeDigest(f.repo);
  const repeat = applyDistributionPlan(planFor(f)); assert.equal(repeat.status, "up_to_date"); assert.equal(treeDigest(f.repo), before); fs.rmSync(f.root, { recursive: true, force: true });
});

for (const [name, mutate] of [
  ["project bee symlink refuses whole plan", (f) => { fs.rmSync(path.join(f.repo, ".claude/skills/bee-hive"), { recursive: true }); fs.symlinkSync(path.join(f.repo, ".agents/skills/bee-planning"), path.join(f.repo, ".claude/skills/bee-hive")); }],
  ["project bee file refuses whole plan", (f) => { fs.rmSync(path.join(f.repo, ".claude/skills/bee-hive"), { recursive: true }); write(path.join(f.repo, ".claude/skills/bee-hive"), "file"); }],
  ["malformed hook JSON refuses whole plan", (f) => write(path.join(f.repo, ".codex/hooks.json"), "{")],
  ["malformed hook shape refuses whole plan", (f) => write(path.join(f.repo, ".codex/hooks.json"), '{"hooks":{"SessionStart":{}}}')],
]) check(name, () => {
  const f = fixture(); mutate(f); const before = treeDigest(f.repo); assert.throws(() => planFor(f)); assert.equal(treeDigest(f.repo), before); fs.rmSync(f.root, { recursive: true, force: true });
});

check("snapshot drift refuses apply before mutation", () => {
  const f = fixture(); const plan = planFor(f); write(path.join(f.repo, ".claude/skills/bee-hive/new.txt"), "race"); const before = treeDigest(f.repo);
  assert.throws(() => applyDistributionPlan(plan), /changed after preflight/); assert.equal(treeDigest(f.repo), before); fs.rmSync(f.root, { recursive: true, force: true });
});

check("empty-directory drift also refuses apply before mutation", () => {
  const f = fixture(); const plan = planFor(f); fs.mkdirSync(path.join(f.repo, ".claude/skills/bee-hive/empty")); const before = treeDigest(f.repo);
  assert.throws(() => applyDistributionPlan(plan), /changed after preflight/); assert.equal(treeDigest(f.repo), before); fs.rmSync(f.root, { recursive: true, force: true });
});

check("repo-copy requires proven inactive plugin", () => {
  const f = fixture(); assert.throws(() => provePluginInactive(f.states));
  const plan = buildDistributionPlan({ mode: "repo-copy", runtimes: ["claude", "codex"], repoRoot: f.repo, pluginStates: f.states.map((state) => ({ ...state, installed: false, enabled: false })), inventory: f.inventory });
  assert.equal(applyDistributionPlan(plan).status, "ready_for_onboarding"); fs.rmSync(f.root, { recursive: true, force: true });
});

check("missing ownership ledger refuses user-root cleanup with zero mutation", () => {
  const f = fixture(); const userRoot = path.join(f.root, "home/skills"); write(path.join(userRoot, "bee-hive/SKILL.md"), "owned"); const before = treeDigest(f.root);
  assert.throws(() => planFor(f, { userSkillRoots: [userRoot] }), /ownership ledger/); assert.equal(treeDigest(f.root), before); fs.rmSync(f.root, { recursive: true, force: true });
});

check("exact ownership ledger authorizes only named global bee directories", () => {
  const f = fixture(); const userRoot = path.join(f.root, "home/skills"); const ledger = path.join(f.root, "home/ledger.json");
  write(path.join(userRoot, "bee-hive/SKILL.md"), "owned"); write(path.join(userRoot, "bee-personal/SKILL.md"), "foreign");
  write(ledger, `${JSON.stringify({ schemaVersion: 1, roots: [{ path: userRoot, skills: ["bee-hive"] }] }, null, 2)}\n`);
  applyDistributionPlan(planFor(f, { userSkillRoots: [userRoot], ledgerPath: ledger }));
  assert.equal(fs.existsSync(path.join(userRoot, "bee-hive")), false); assert.equal(fs.existsSync(path.join(userRoot, "bee-personal/SKILL.md")), true);
  assert.deepEqual(JSON.parse(fs.readFileSync(ledger)).roots[0].skills, []); fs.rmSync(f.root, { recursive: true, force: true });
});

check("Bash and PowerShell expose identical distribution modes and helper", () => {
  const bash = fs.readFileSync(path.join(REPO_ROOT, "scripts/install.sh"), "utf8");
  const ps = fs.readFileSync(path.join(REPO_ROOT, "scripts/install.ps1"), "utf8");
  for (const token of ["plugin-first", "repo-copy", "plugin_distribution.mjs"]) { assert.ok(bash.includes(token), `bash missing ${token}`); assert.ok(ps.includes(token), `powershell missing ${token}`); }
  assert.ok(Buffer.from(ps).every((byte) => byte < 128), "install.ps1 must remain ASCII");
});

check("cachebuster staging changes one staged version and never canonical tuple", () => {
  const canonicalPath = path.join(REPO_ROOT, ".codex-plugin/plugin.json"); const canonicalBytes = fs.readFileSync(canonicalPath); const canonical = JSON.parse(canonicalBytes);
  const stage = fs.mkdtempSync(path.join(os.tmpdir(), "bee-cachebuster-")); const stagedPath = path.join(stage, ".codex-plugin/plugin.json");
  write(stagedPath, `${JSON.stringify({ ...canonical, version: `${canonical.version.split("+")[0]}+codex.fixture` }, null, 2)}\n`);
  const staged = JSON.parse(fs.readFileSync(stagedPath)); const differing = Object.keys(staged).filter((key) => JSON.stringify(staged[key]) !== JSON.stringify(canonical[key]));
  assert.deepEqual(differing, ["version"]); assert.equal(staged.version.split("+codex.")[0], canonical.version.split("+")[0]); assert.deepEqual(fs.readFileSync(canonicalPath), canonicalBytes);
  fs.rmSync(stage, { recursive: true, force: true });
});

// ─── D9/cnr2-12: committed per-runtime plugin skill-route trees ───────────
//
// Each plugin manifest now routes to a COMMITTED rendered tree
// (.claude-plugin/skills/ = render(canonical, claude), .codex-plugin/skills/
// = render(canonical, codex)) generated only through the cnr2-9 renderer
// (scripts/render_plugin_skill_trees.mjs). These checks recompute the render
// from the REAL canonical skills/ tree at test time and compare it against
// what is actually committed — a drift pin, not a trust of a prior run.

const SKILL_DIR_RE = /^bee-/;
const SKILLS_ROOT = path.join(REPO_ROOT, "skills");
const PLUGIN_RENDER_ROOTS = {
  claude: path.join(REPO_ROOT, ".claude-plugin", "skills"),
  codex: path.join(REPO_ROOT, ".codex-plugin", "skills"),
};

function listRelFiles(root) {
  const out = [];
  const walk = (dir, relPrefix) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const rel = relPrefix ? `${relPrefix}/${entry.name}` : entry.name;
      const abs = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(abs, rel);
      else out.push(rel);
    }
  };
  walk(root, "");
  return out.sort();
}

function canonicalSkillFiles() {
  const files = [];
  for (const name of fs.readdirSync(SKILLS_ROOT, { withFileTypes: true }).filter((e) => e.isDirectory() && SKILL_DIR_RE.test(e.name)).map((e) => e.name).sort()) {
    for (const rel of listRelFiles(path.join(SKILLS_ROOT, name))) files.push(`${name}/${rel}`);
  }
  return files.sort();
}

for (const runtime of RENDER_RUNTIMES) {
  check(`plugin skill tree (${runtime}): every file byte-equals render(canonical, ${runtime})`, () => {
    const canonical = canonicalSkillFiles();
    const targetRoot = PLUGIN_RENDER_ROOTS[runtime];
    const committed = listRelFiles(targetRoot).filter((rel) => rel !== RENDER_SIDECAR);
    assert.deepEqual(committed, canonical, `committed ${runtime} tree file set must equal canonical`);
    for (const rel of canonical) {
      const sourceBytes = fs.readFileSync(path.join(SKILLS_ROOT, rel));
      const expected = renderSkillBytes(sourceBytes, runtime);
      const actual = fs.readFileSync(path.join(targetRoot, ...rel.split("/")));
      assert.equal(Buffer.compare(actual, expected), 0, `${runtime}/${rel} must equal render(canonical, ${runtime})`);
    }
  });

  check(`plugin skill tree (${runtime}): carries the D9/D7 render inventory sidecar (bee-render/2)`, () => {
    const sidecar = JSON.parse(fs.readFileSync(path.join(PLUGIN_RENDER_ROOTS[runtime], RENDER_SIDECAR), "utf8"));
    assert.equal(sidecar.target_runtime, runtime);
    assert.equal(sidecar.schema, "bee-render/2");
    assert.ok(Array.isArray(sidecar.skills) && sidecar.skills.length > 0, "sidecar must list at least one skill");
    const names = sidecar.skills.map((s) => s.name);
    assert.deepEqual(names, [...names].sort(), "skills[] must be sorted by name");
    for (const s of sidecar.skills) {
      assert.match(s.sha256, /^[0-9a-f]{64}$/, `${s.name} sha256 must be a hex digest`);
    }
  });

  check(`plugin skill tree (${runtime}): sidecar sha256 recomputes from the committed rendered files (drift pin)`, () => {
    const targetRoot = PLUGIN_RENDER_ROOTS[runtime];
    const sidecar = JSON.parse(fs.readFileSync(path.join(targetRoot, RENDER_SIDECAR), "utf8"));
    for (const { name, sha256: expected } of sidecar.skills) {
      const walk = walkSkillTree(path.join(targetRoot, name));
      assert.equal(walk.blocked, null, `${runtime}/${name} unexpectedly blocked: ${JSON.stringify(walk.blocked)}`);
      assert.equal(
        skillDigest(walk.files),
        expected,
        `${runtime}/${name} sidecar sha256 must match the digest recomputed from the committed rendered files`,
      );
    }
  });
}

// A genuine, well-formed, full marker LINE — same shape as onboard_bee.mjs's
// own MARKER_ONLY_RE/MARKER_END_RE grammar (column-0, trailing whitespace
// only). Deliberately NOT a substring search: onboard_bee.mjs's own comments
// document the marker syntax inline (e.g. "//   <!-- bee:only claude -->
// ...") — legitimate prose about the mechanism, never a real marker line, and
// must not false-positive here the way a bare `.includes()` would.
const REAL_MARKER_LINE_RE = /^<!-- bee:(?:only (?:claude|codex)|end) -->[ \t]*$/;

function realMarkerLines(text) {
  return text.split(/\r\n|\n/).filter((line) => REAL_MARKER_LINE_RE.test(line));
}

check("runtime-clean: claude plugin tree carries no unstripped marker line", () => {
  for (const rel of listRelFiles(PLUGIN_RENDER_ROOTS.claude).filter((r) => r !== RENDER_SIDECAR)) {
    const text = fs.readFileSync(path.join(PLUGIN_RENDER_ROOTS.claude, ...rel.split("/")), "utf8");
    assert.deepEqual(realMarkerLines(text), [], `claude tree left an unstripped marker line in ${rel}`);
  }
});

check("runtime-clean: codex plugin tree carries no unstripped marker line", () => {
  for (const rel of listRelFiles(PLUGIN_RENDER_ROOTS.codex).filter((r) => r !== RENDER_SIDECAR)) {
    const text = fs.readFileSync(path.join(PLUGIN_RENDER_ROOTS.codex, ...rel.split("/")), "utf8");
    assert.deepEqual(realMarkerLines(text), [], `codex tree left an unstripped marker line in ${rel}`);
  }
});

check("release inventory covers both committed plugin skill trees with matching sha256", () => {
  const inventory = loadPackageInventory(path.join(REPO_ROOT, "docs/history/codex-harness-hardening/release-manifest.json"));
  for (const [runtime, role] of [["claude", "plugin_skill_claude_render"], ["codex", "plugin_skill_codex_render"]]) {
    const targetRoot = PLUGIN_RENDER_ROOTS[runtime];
    const pluginDir = runtime === "claude" ? ".claude-plugin" : ".codex-plugin";
    const expectedPaths = listRelFiles(targetRoot).map((rel) => `${pluginDir}/skills/${rel}`).sort();
    const records = inventory.filter((r) => r.role === role);
    assert.deepEqual(records.map((r) => r.path).sort(), expectedPaths, `${role} inventory paths must match committed tree`);
    for (const record of records) {
      const abs = path.join(REPO_ROOT, ...record.path.split("/"));
      assert.equal(createHash("sha256").update(fs.readFileSync(abs)).digest("hex"), record.sha256, `${record.path} sha256 must match release manifest`);
    }
  }
});

check("plugin manifests route to their own committed rendered skill tree", () => {
  const claudeManifest = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, ".claude-plugin/plugin.json"), "utf8"));
  const codexManifest = JSON.parse(fs.readFileSync(path.join(REPO_ROOT, ".codex-plugin/plugin.json"), "utf8"));
  assert.equal(claudeManifest.skills, "./.claude-plugin/skills/");
  assert.equal(codexManifest.skills, "./.codex-plugin/skills/");
});

console.log(`\nplugin_distribution: ${passed} passed, ${failed} failed`);
if (failed) process.exit(1);
