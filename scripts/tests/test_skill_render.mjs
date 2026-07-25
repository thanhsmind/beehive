#!/usr/bin/env node
// test_skill_render.mjs — the D9 skill-runtime renderer + provenance regression
// net. Covers: marker-grammar refusals, per-runtime filtering, byte-preserving
// zero-marker passthrough, idempotent re-render, the no-marker no-op invariant,
// and the rendered-projection provenance refusal (own runtime included).
//
// Run suites with BEE_AGENT_NAME unset (the onboarding child processes below
// share the known env leak); the wrapper in commands.verify already unsets it.

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  renderSkillBytes,
  validateSkillMarkers,
  RENDER_RUNTIMES,
  RENDER_SIDECAR,
} from "../../packages/bee/scripts/onboard_bee.mjs";
import { classifySource } from "../../packages/bee/lib/source-identity.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const ONBOARD = path.join(REPO_ROOT, "packages", "bee", "scripts", "onboard_bee.mjs");
const TEMPLATES_LIB_DIR = path.join(REPO_ROOT, "packages", "bee", "lib");

let passed = 0;
let failed = 0;
function check(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`PASS ${name}`);
  } catch (error) {
    failed += 1;
    console.error(`FAIL ${name}: ${error && error.stack ? error.stack : error}`);
  }
}

const B = (s) => Buffer.from(s, "utf8");
const errorsFor = (text) => validateSkillMarkers(text);
const hasError = (text, needle) => errorsFor(text).some((e) => e.includes(needle));

// ─── marker grammar: refusals ─────────────────────────────────────────────

check("grammar: a clean marked doc is well-formed", () => {
  const doc = [
    "shared top",
    "<!-- bee:only claude -->",
    "claude line",
    "<!-- bee:end -->",
    "<!-- bee:only codex -->",
    "codex line",
    "<!-- bee:end -->",
    "shared bottom",
    "",
  ].join("\n");
  assert.deepEqual(errorsFor(doc), []);
});

check("grammar: no markers at all → no errors", () => {
  assert.deepEqual(errorsFor("# title\n\nplain body, mentions bee:only in prose\n"), []);
});

check("grammar: nested bee:only refused", () => {
  const doc = "<!-- bee:only claude -->\na\n<!-- bee:only codex -->\nb\n<!-- bee:end -->\n";
  assert.ok(hasError(doc, "nested"), errorsFor(doc).join(" | "));
});

check("grammar: unclosed block refused", () => {
  const doc = "before\n<!-- bee:only claude -->\ndangling\n";
  assert.ok(hasError(doc, "unclosed"), errorsFor(doc).join(" | "));
});

check("grammar: stray bee:end refused", () => {
  const doc = "before\n<!-- bee:end -->\nafter\n";
  assert.ok(hasError(doc, "stray"), errorsFor(doc).join(" | "));
});

check("grammar: unknown runtime label refused", () => {
  const doc = "<!-- bee:only python -->\nx\n<!-- bee:end -->\n";
  assert.ok(hasError(doc, "unknown runtime label"), errorsFor(doc).join(" | "));
});

check("grammar: ambiguous near-marker (missing space) refused", () => {
  const doc = "<!-- bee:only claude-->\nx\n<!-- bee:end -->\n";
  assert.ok(hasError(doc, "ambiguous near-marker"), errorsFor(doc).join(" | "));
});

check("grammar: indented (non-full-line) marker refused as ambiguous", () => {
  const doc = "  <!-- bee:only claude -->\nx\n  <!-- bee:end -->\n";
  assert.ok(hasError(doc, "ambiguous near-marker"), errorsFor(doc).join(" | "));
});

check("grammar: marker inside YAML frontmatter refused", () => {
  const doc = "---\ntitle: x\n<!-- bee:only claude -->\n---\nbody\n";
  assert.ok(hasError(doc, "frontmatter"), errorsFor(doc).join(" | "));
});

check("grammar: marker inside a fenced code block refused", () => {
  const doc = "text\n```\n<!-- bee:only claude -->\n```\nmore\n";
  assert.ok(hasError(doc, "fenced code block"), errorsFor(doc).join(" | "));
});

// ─── per-runtime filter ────────────────────────────────────────────────────

const MARKED = [
  "shared A",
  "<!-- bee:only claude -->",
  "CLAUDE ONLY",
  "<!-- bee:end -->",
  "shared B",
  "<!-- bee:only codex -->",
  "CODEX ONLY",
  "<!-- bee:end -->",
  "shared C",
  "",
].join("\n");

check("filter: claude keeps shared + claude, drops codex, strips markers", () => {
  const out = renderSkillBytes(B(MARKED), "claude").toString("utf8");
  assert.equal(out, "shared A\nCLAUDE ONLY\nshared B\nshared C\n");
  assert.ok(!out.includes("bee:only") && !out.includes("bee:end"));
  assert.ok(!out.includes("CODEX ONLY"));
});

check("filter: codex keeps shared + codex, drops claude, strips markers", () => {
  const out = renderSkillBytes(B(MARKED), "codex").toString("utf8");
  assert.equal(out, "shared A\nshared B\nCODEX ONLY\nshared C\n");
  assert.ok(!out.includes("CLAUDE ONLY"));
});

check("filter: every declared runtime renders without leaving markers", () => {
  for (const rt of RENDER_RUNTIMES) {
    const out = renderSkillBytes(B(MARKED), rt).toString("utf8");
    assert.ok(!out.includes("bee:only") && !out.includes("bee:end"), rt);
  }
});

check("filter: CRLF line endings preserved through rendering", () => {
  const doc = "a\r\n<!-- bee:only claude -->\r\nc\r\n<!-- bee:end -->\r\nz\r\n";
  const out = renderSkillBytes(B(doc), "claude").toString("utf8");
  assert.equal(out, "a\r\nc\r\nz\r\n");
});

// ─── idempotent re-render ──────────────────────────────────────────────────

check("idempotency: render(render(x)) === render(x)", () => {
  for (const rt of RENDER_RUNTIMES) {
    const once = renderSkillBytes(B(MARKED), rt);
    const twice = renderSkillBytes(once, rt);
    assert.equal(Buffer.compare(once, twice), 0, rt);
  }
});

// ─── byte-preserving zero-marker passthrough ───────────────────────────────

const PASSTHROUGH = {
  "plain LF": B("# Title\n\nbody\n"),
  "no final newline": B("abc"),
  "CRLF only": B("a\r\nb\r\n"),
  "UTF-8 BOM preserved": Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), B("# x\n")]),
  "arbitrary bytes": Buffer.from([0x00, 0x01, 0xff, 0x80, 0x0a, 0x7e]),
  "literal bee:only in prose, not a marker line": B("see the bee:only usage note\n"),
  "trailing bee:end word in prose": B("ends with bee:end\nand more\n"),
};

for (const [label, buf] of Object.entries(PASSTHROUGH)) {
  check(`byte-preservation: ${label} is byte-identical for every runtime`, () => {
    for (const rt of [...RENDER_RUNTIMES, "claude"]) {
      const out = renderSkillBytes(buf, rt);
      assert.equal(Buffer.compare(out, buf), 0, `${label} / ${rt}`);
      assert.equal(out.length, buf.length);
    }
  });
}

check("no-marker no-op invariant: a valid frontmatter doc without markers is untouched", () => {
  const buf = B("---\nname: x\n---\n\n# Body\n\ntext\n");
  assert.equal(Buffer.compare(renderSkillBytes(buf, "claude"), buf), 0);
  assert.deepEqual(errorsFor(buf.toString("utf8")), []);
});

// ─── provenance: rendered projection refused as a source ───────────────────

function mkdtemp() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "bee-render-test-"));
}

// Build a launchable skills projection under <base>/.claude/skills so it
// classifies as project_projection unless the render sidecar promotes it to
// rendered_projection. Copies the REAL onboarder + packages/bee/lib so the
// child process resolves its imports and the runtime version marker.
// packages-engine-move D1/D3: the engine no longer lives inside the skills
// tree it renders - the copied launcher goes to packagesBeeDir/scripts
// (never hive/scripts) so its own "../lib" relative import resolves to
// packagesBeeDir/lib, exactly like the real packages/bee/scripts/
// onboard_bee.mjs resolves against its real packages/bee/lib sibling. The
// copied launcher's own PLUGIN_ROOT arithmetic (dirname(dirname(ENGINE_DIR)))
// still lands on base/.claude, NOT base itself, so packagesBeeDir sits at
// that same nesting depth, outside the skills tree.
function buildProjection(base, { sidecar = false, extraSkills = {} } = {}) {
  const skillsRoot = path.join(base, ".claude", "skills");
  const hive = path.join(skillsRoot, "bee-hive");
  const packagesBeeDir = path.join(base, ".claude", "packages", "bee");
  const engineScripts = path.join(packagesBeeDir, "scripts");
  fs.mkdirSync(hive, { recursive: true });
  fs.mkdirSync(engineScripts, { recursive: true });
  fs.mkdirSync(path.join(packagesBeeDir, "lib"), { recursive: true });
  fs.writeFileSync(path.join(engineScripts, "onboard_bee.mjs"), fs.readFileSync(ONBOARD));
  for (const libName of fs.readdirSync(TEMPLATES_LIB_DIR)) {
    if (!libName.endsWith(".mjs")) continue;
    fs.writeFileSync(
      path.join(packagesBeeDir, "lib", libName),
      fs.readFileSync(path.join(TEMPLATES_LIB_DIR, libName)),
    );
  }
  for (const [skill, files] of Object.entries(extraSkills)) {
    for (const [rel, content] of Object.entries(files)) {
      const abs = path.join(skillsRoot, skill, ...rel.split("/"));
      fs.mkdirSync(path.dirname(abs), { recursive: true });
      fs.writeFileSync(abs, content);
    }
  }
  if (sidecar) {
    fs.writeFileSync(
      path.join(skillsRoot, RENDER_SIDECAR),
      `${JSON.stringify({ schema: "bee-render/1", target_runtime: "claude" }, null, 2)}\n`,
    );
  }
  return { skillsRoot, launcher: path.join(engineScripts, "onboard_bee.mjs") };
}

function runOnboard(launcher, args) {
  const env = { ...process.env };
  delete env.BEE_AGENT_NAME;
  try {
    const stdout = execFileSync("node", [launcher, ...args], { env, encoding: "utf8" });
    return { code: 0, json: JSON.parse(stdout) };
  } catch (error) {
    const stdout = error.stdout ? error.stdout.toString() : "";
    return { code: error.status ?? 1, json: stdout ? JSON.parse(stdout) : null };
  }
}

check("provenance: classifySource flags a skills root carrying the render sidecar", () => {
  const base = mkdtemp();
  try {
    const { skillsRoot } = buildProjection(base, { sidecar: true });
    const kind = classifySource({ hiveDir: path.join(skillsRoot, "bee-hive") }).kind;
    assert.equal(kind, "rendered_projection");
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
  }
});

check("provenance: a rendered projection WITHOUT the sidecar is not flagged rendered", () => {
  const base = mkdtemp();
  try {
    const { skillsRoot } = buildProjection(base, { sidecar: false });
    const kind = classifySource({ hiveDir: path.join(skillsRoot, "bee-hive") }).kind;
    assert.notEqual(kind, "rendered_projection");
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
  }
});

check("provenance: onboarding FROM a rendered projection is refused for any target (own runtime included)", () => {
  const base = mkdtemp();
  const repo = mkdtemp();
  try {
    const { launcher } = buildProjection(base, { sidecar: true });
    const { json } = runOnboard(launcher, ["--repo-root", repo, "--json"]);
    assert.ok(json, "expected a JSON payload");
    assert.equal(json.status, "blocked_no_source");
    assert.match(json.reason || "", /rendered per-runtime projection/);
    // own-runtime included: the sidecar records target_runtime "claude", yet the
    // refusal is unconditional — no target-filter re-admits it.
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
    fs.rmSync(repo, { recursive: true, force: true });
  }
});

// ─── whole-tree gate: a malformed marker refuses the whole apply, zero writes ─

check("gate: a malformed marker in one skill refuses the whole apply with zero writes", () => {
  const base = mkdtemp();
  const repo = mkdtemp();
  try {
    const { launcher } = buildProjection(base, {
      extraSkills: {
        "bee-badmark": { "SKILL.md": "# bad\n<!-- bee:only claude -->\ndangling, never closed\n" },
      },
    });
    // Plan mode reports the refusal (exit 0, blocked_render status).
    const plan = runOnboard(launcher, ["--repo-root", repo, "--json"]);
    assert.equal(plan.json.status, "blocked_render", JSON.stringify(plan.json).slice(0, 300));
    // Apply mode refuses (exit 1) and mutates nothing in the target repo.
    const applied = runOnboard(launcher, ["--repo-root", repo, "--apply", "--json"]);
    assert.equal(applied.code, 1);
    assert.equal(applied.json.status, "blocked_render");
    assert.ok(
      !fs.existsSync(path.join(repo, ".claude", "skills", "bee-hive")),
      "no skills should have been written on a refused apply",
    );
    assert.ok(!fs.existsSync(path.join(repo, ".bee", "onboarding.json")));
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
    fs.rmSync(repo, { recursive: true, force: true });
  }
});

console.log(`\nskill_render: ${passed} passed, ${failed} failed`);
if (failed) process.exit(1);
