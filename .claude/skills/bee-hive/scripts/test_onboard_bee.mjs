#!/usr/bin/env node
// test_onboard_bee.mjs - self-contained test for onboard_bee.mjs.
// Creates a temp repo, runs plan mode (expects changes_needed), applies,
// verifies AGENTS.md markers + .bee tree + vendored bin/lib, re-runs plan
// (expects up_to_date), checks AGENTS block idempotency and the never-
// overwrite rule, then exercises --repo-hooks. Exits 1 on any failure.

import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { runModuleWorker } from "../../../scripts/lib/run-module-worker.mjs";

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const SCRIPTS_DIR = path.dirname(SCRIPT_PATH);
const ONBOARD = path.join(SCRIPTS_DIR, "onboard_bee.mjs");
const REPO_ROOT = path.join(SCRIPTS_DIR, "..", "..", "..");
const TEMPLATES_DIR = path.join(REPO_ROOT, "packages", "bee");
const TEMPLATES_LIB_DIR = path.join(TEMPLATES_DIR, "lib");

let failures = 0;
let skips = 0;

function check(condition, label, extra = "") {
  if (condition) {
    process.stdout.write(`ok    - ${label}\n`);
  } else {
    failures += 1;
    process.stdout.write(`FAIL  - ${label}${extra ? ` :: ${extra}` : ""}\n`);
  }
}

function skip(label, why) {
  skips += 1;
  process.stdout.write(`skip  - ${label} (${why})\n`);
}

// --- hermetic per-case fake HOME/USERPROFILE isolation ----------------------
// The real home must be unreachable by construction: every launched onboard
// worker gets HOME and USERPROFILE pointed at a fake per-case temp dir, never
// at the developer's real home. Single-call cases get a fresh fake home per
// call (default param below); multi-call cases (apply-then-recheck, etc.)
// create ONE fake home explicitly and pass it to every call in that case.
const REAL_HOME = process.env.HOME;
const REAL_USERPROFILE = process.env.USERPROFILE;
const launchedHomes = [];

function makeFakeHome() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "bee-onboard-home-"));
}

async function runOnboardAt(scriptPath, args, fakeHome = makeFakeHome()) {
  const env = { ...process.env, HOME: fakeHome, USERPROFILE: fakeHome };
  launchedHomes.push({ HOME: env.HOME, USERPROFILE: env.USERPROFILE });
  const result = await runModuleWorker(scriptPath, { args, env, fakeHome });
  let payload = null;
  try {
    payload = JSON.parse(result.stdout || "null");
  } catch {
    payload = null;
  }
  return { ...result, payload };
}

async function runOnboard(args, fakeHome = makeFakeHome()) {
  return await runOnboardAt(ONBOARD, args, fakeHome);
}

function listMjs(dir) {
  if (!fs.existsSync(dir)) {
    return [];
  }
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith(".mjs"))
    .map((e) => e.name)
    .sort();
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-onboard-test-"));
process.stdout.write(`test repo: ${tmp}\n`);
// Main flow reuses `tmp` across many runOnboard calls -> one shared fake home.
const tmpHome = makeFakeHome();

try {
  // Two distinct properties, deliberately split so a slow/2-core CI runner
  // can't flake either one (rel1710rc-4):
  //  (1) timeout SEMANTICS (null status, SIGTERM, ETIMEDOUT) - a short
  //      timeout is fine here because nothing about output timing is
  //      asserted, so it stays cheap even on a starved runner.
  //  (2) capture PRESERVATION on timeout - needs the child to have actually
  //      printed before it's killed, which a short timeout cannot guarantee
  //      under CI scheduling pressure (the child can be SIGTERMed before
  //      node even boots). Use a generous timeout (>=5000ms) so the kill
  //      always lands well after the module's first-millisecond prints.
  const hangingModule = path.join(tmp, "hanging-module.mjs");
  fs.writeFileSync(
    hangingModule,
    "setInterval(() => {}, 1_000);\n",
  );
  const timedOut = await runModuleWorker(hangingModule, { timeout: 100 });
  check(timedOut.status === null, "shared Worker timeout returns null status");
  check(timedOut.signal === "SIGTERM", "shared Worker timeout returns SIGTERM signal");
  check(timedOut.error?.code === "ETIMEDOUT", "shared Worker timeout returns ETIMEDOUT error");

  const printsThenHangsModule = path.join(tmp, "prints-then-hangs-module.mjs");
  fs.writeFileSync(
    printsThenHangsModule,
    'process.stdout.write("hanging stdout\\n"); process.stderr.write("hanging stderr\\n"); setInterval(() => {}, 1_000);\n',
  );
  const timedOutWithOutput = await runModuleWorker(printsThenHangsModule, { timeout: 5_000 });
  check(timedOutWithOutput.status === null, "capture-preservation timeout returns null status");
  check(timedOutWithOutput.signal === "SIGTERM", "capture-preservation timeout returns SIGTERM signal");
  check(timedOutWithOutput.error?.code === "ETIMEDOUT", "capture-preservation timeout returns ETIMEDOUT error");
  check(
    timedOutWithOutput.stdout.includes("hanging stdout") && timedOutWithOutput.stderr.includes("hanging stderr"),
    "shared Worker timeout preserves captured stdout and stderr (generous timeout so the kill lands after startup prints)",
  );

  // --- 1. plan mode on empty repo -> changes_needed -----------------------
  const plan1 = await runOnboard(["--repo-root", tmp, "--json"], tmpHome);
  check(plan1.status === 0, "plan mode exits 0", plan1.stderr);
  check(plan1.payload?.status === "changes_needed", "empty repo reports changes_needed",
    `got: ${plan1.payload?.status}`);
  check(Array.isArray(plan1.payload?.plan) && plan1.payload.plan.length > 0,
    "plan mode lists planned actions");
  check(!fs.existsSync(path.join(tmp, "AGENTS.md")), "plan mode writes nothing");
  const plan1Actions = (plan1.payload?.plan || []).map((i) => i.action);
  check(plan1Actions.includes("create_agents_block") &&
    plan1Actions.includes("propose_agents_header"),
    "empty repo plans create_agents_block + propose_agents_header (D4)",
    JSON.stringify(plan1Actions));
  check(plan1Actions.indexOf("propose_agents_header") >
    plan1Actions.indexOf("create_agents_block"),
    "propose_agents_header ordered after create_agents_block");

  // --- 2. apply ------------------------------------------------------------
  const apply1 = await runOnboard(["--repo-root", tmp, "--apply", "--json"], tmpHome);
  check(apply1.status === 0, "apply exits 0", apply1.stderr);
  check(apply1.payload?.status === "applied", "apply reports applied");
  check(apply1.payload?.recheck === "up_to_date", "apply recheck is up_to_date",
    JSON.stringify(apply1.payload?.recheck_plan || []));

  // --- 2b. per-project skill roots (installer-hardening D2/D6) --------------
  // A fresh host onboard --apply materializes the full bee-* skill set under
  // BOTH <repo>/.claude/skills and <repo>/.agents/skills by default, never
  // touches ~/.claude/skills without --global-skills, and never gitignores
  // the committed trees (D4).
  const SOURCE_SKILLS_ROOT = path.join(REPO_ROOT, "skills");
  const sourceBeeSkills = fs.readdirSync(SOURCE_SKILLS_ROOT, { withFileTypes: true })
    .filter((e) => e.isDirectory() && e.name.startsWith("bee-"))
    .map((e) => e.name)
    .sort();
  check(sourceBeeSkills.length === 18,
    "source tree carries the expected 18 bee-* skills", JSON.stringify(sourceBeeSkills));
  for (const relRoot of [".claude/skills", ".agents/skills"]) {
    const rootAbs = path.join(tmp, ...relRoot.split("/"));
    const installed = fs.existsSync(rootAbs)
      ? fs.readdirSync(rootAbs, { withFileTypes: true })
          .filter((e) => e.isDirectory() && e.name.startsWith("bee-"))
          .map((e) => e.name)
          .sort()
      : [];
    check(JSON.stringify(installed) === JSON.stringify(sourceBeeSkills),
      `fresh host apply materializes all 18 bee-* dirs under ${relRoot}`,
      JSON.stringify(installed));
  }
  const applyTargetKinds = (apply1.payload?.skills?.targets || []).map((t) => t.kind);
  check(JSON.stringify(applyTargetKinds) === JSON.stringify(["repo-claude", "repo-agents"]),
    "default apply reports exactly the two in-repo targets (no global without --global-skills)",
    JSON.stringify(applyTargetKinds));
  check(!fs.existsSync(path.join(tmpHome, ".claude", "skills")),
    "without --global-skills, ~/.claude/skills is never written");
  const gitignoreAfterApply = fs.readFileSync(path.join(tmp, ".gitignore"), "utf8");
  check(!gitignoreAfterApply.includes(".claude/skills") &&
    !gitignoreAfterApply.includes(".agents/skills"),
    "per-project skill trees are never gitignored (D4: committed to the host repo)",
    gitignoreAfterApply);

  // --- 2c. bee agent files (config-rendered pinned agent types, Slice 3B) ---
  // Fresh repo, default config.json (extraction: haiku, generation: sonnet,
  // review unset -> AGENT_TIER_DEFAULTS_CLAUDE fallback opus). must_haves:
  // "three agent files render into .claude/agents/ with model values taken
  // from live config, never hardcoded in the template output path".
  const agentFilePath = (name) => path.join(tmp, ".claude", "agents", `${name}.md`);
  for (const [name, tier, model] of [
    ["bee-gather", "generation", "sonnet"],
    ["bee-extract", "extraction", "haiku"],
    ["bee-review", "review", "opus"],
  ]) {
    check(fs.existsSync(agentFilePath(name)), `${name}.md is rendered on fresh apply`);
    const text = fs.existsSync(agentFilePath(name)) ? fs.readFileSync(agentFilePath(name), "utf8") : "";
    check(text.includes(`name: ${name}`), `${name}.md frontmatter carries its own name`, text);
    check(text.includes(`model: ${model}`),
      `${name}.md frontmatter model comes from the configured ${tier} tier (${model}), not a hardcoded pin`,
      text);
    check(!text.includes("{{TIER_MODEL}}"), `${name}.md has no unrendered {{TIER_MODEL}} placeholder`, text);
    check(!/\bcost[- ]?reduc/i.test(text), `${name}.md makes no cost-reduction claim`, text);
  }

  // AO10: agents root is never joined to REPO_SKILL_TARGETS (regression guard
  // against the exact change plan.md flagged as forbidden).
  const onboardSourceForRegressionGuard = fs.readFileSync(path.join(SCRIPTS_DIR, "onboard_bee.mjs"), "utf8");
  const repoSkillTargetsMatch = onboardSourceForRegressionGuard.match(/REPO_SKILL_TARGETS = \[([\s\S]*?)\];/);
  // "repo-agents" (the existing .agents/skills target's `kind`) legitimately
  // contains the substring "agents" - the forbidden shape is a `segments`
  // array naming a plain "agents" directory (an agents ROOT, e.g. `[".claude",
  // "agents"]`), not the `kind` label. Match on `segments` arrays only.
  const repoSkillTargetsSegments = [...(repoSkillTargetsMatch?.[1] || "").matchAll(/segments:\s*\[([^\]]*)\]/g)]
    .map((m) => m[1]);
  check(Boolean(repoSkillTargetsMatch) && repoSkillTargetsSegments.length === 2 &&
    repoSkillTargetsSegments.every((s) => !/"agents"/.test(s)),
    "REPO_SKILL_TARGETS is untouched - no agents-root segments entry (AO10)",
    JSON.stringify(repoSkillTargetsSegments));

  // AO11: Codex gets no agent files at all - the asymmetry is recorded, not
  // silently absent.
  check(!fs.existsSync(path.join(tmp, ".agents", "agents")),
    ".agents/ receives no agent files (AO11 - no .agents/agents root at all)");
  const onboardingAfterApply1 = JSON.parse(fs.readFileSync(path.join(tmp, ".bee", "onboarding.json"), "utf8"));
  check(onboardingAfterApply1.agents_sync?.bee_version === apply1.payload?.bee_version,
    "onboarding.json agents_sync carries the sync's own bee_version marker",
    JSON.stringify(onboardingAfterApply1.agents_sync));
  check(JSON.stringify((onboardingAfterApply1.agents_sync?.files || []).slice().sort()) ===
    JSON.stringify([".claude/agents/bee-extract.md", ".claude/agents/bee-gather.md", ".claude/agents/bee-review.md"]),
    "onboarding.json agents_sync.files names all three rendered agent files",
    JSON.stringify(onboardingAfterApply1.agents_sync?.files));
  const sortedEntries = (o) => JSON.stringify(Object.entries(o || {}).sort());
  check(sortedEntries(onboardingAfterApply1.agents_sync?.rendered_from) ===
    sortedEntries({ generation: "sonnet", extraction: "haiku", review: "opus" }),
    "onboarding.json agents_sync.rendered_from records {tier: model} from live config",
    JSON.stringify(onboardingAfterApply1.agents_sync?.rendered_from));
  check(Array.isArray(onboardingAfterApply1.agents_sync?.codex?.agents) &&
    onboardingAfterApply1.agents_sync.codex.agents.length === 0 &&
    /AO11/.test(onboardingAfterApply1.agents_sync.codex.note || ""),
    "onboarding.json agents_sync records the Codex asymmetry (AO11) inline, no separate file",
    JSON.stringify(onboardingAfterApply1.agents_sync?.codex));

  // Idempotency: must_haves "re-running self-onboard is idempotent (same
  // bytes, marker refreshed)". Snapshot bytes, re-apply, compare.
  const agentBytesBefore = {
    "bee-gather": fs.readFileSync(agentFilePath("bee-gather")),
    "bee-extract": fs.readFileSync(agentFilePath("bee-extract")),
    "bee-review": fs.readFileSync(agentFilePath("bee-review")),
  };
  const reapplyForIdempotency = await runOnboard(["--repo-root", tmp, "--apply", "--json"], tmpHome);
  check(reapplyForIdempotency.payload?.status === "applied", "idempotency re-apply succeeds");
  const reapplyActions = (reapplyForIdempotency.payload?.applied || []).map((i) => i.action);
  check(!reapplyActions.includes("sync_agent_file") && !reapplyActions.includes("remove_agent_file"),
    "a no-drift re-apply plans zero agent-file actions (already current)",
    JSON.stringify(reapplyActions));
  for (const name of ["bee-gather", "bee-extract", "bee-review"]) {
    check(fs.readFileSync(agentFilePath(name)).equals(agentBytesBefore[name]),
      `${name}.md is byte-identical after a no-drift re-apply`);
  }
  const onboardingAfterReapply = JSON.parse(fs.readFileSync(path.join(tmp, ".bee", "onboarding.json"), "utf8"));
  check(onboardingAfterReapply.updated_at !== onboardingAfterApply1.updated_at,
    "onboarding.json marker (updated_at) refreshes on every apply, even a byte-idempotent one");

  // --- 2d. cli-shaped / explicit-null tier: skip render, remove stale copy --
  // must_haves: "a cli-shaped or null tier slot skips (and removes) its
  // agent file, recorded in the sync record".
  const cliTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-agentfiles-cli-"));
  const cliHome = makeFakeHome();
  await runOnboard(["--repo-root", cliTmp, "--apply", "--json"], cliHome);
  check(fs.existsSync(path.join(cliTmp, ".claude", "agents", "bee-review.md")),
    "precondition: bee-review.md renders under the default (opus) review tier");
  const cliCfgPath = path.join(cliTmp, ".bee", "config.json");
  const cliCfg = JSON.parse(fs.readFileSync(cliCfgPath, "utf8"));
  cliCfg.models.claude.review = { kind: "cli", command: "true" };
  fs.writeFileSync(cliCfgPath, `${JSON.stringify(cliCfg, null, 2)}\n`, "utf8");
  const cliPlan = await runOnboard(["--repo-root", cliTmp, "--json"], cliHome);
  const cliPlanActions = (cliPlan.payload?.plan || []).filter((i) => i.agent === "bee-review");
  check(cliPlanActions.length === 1 && cliPlanActions[0].action === "remove_agent_file",
    "a cli-shaped review tier plans remove_agent_file for bee-review.md",
    JSON.stringify(cliPlanActions));
  const cliApply = await runOnboard(["--repo-root", cliTmp, "--apply", "--json"], cliHome);
  check(cliApply.payload?.status === "applied", "cli-tier apply succeeds", JSON.stringify(cliApply.payload));
  check(!fs.existsSync(path.join(cliTmp, ".claude", "agents", "bee-review.md")),
    "bee-review.md is removed once its tier becomes cli-shaped");
  check(fs.existsSync(path.join(cliTmp, ".claude", "agents", "bee-gather.md")) &&
    fs.existsSync(path.join(cliTmp, ".claude", "agents", "bee-extract.md")),
    "the other two agent files are untouched by the cli-tier removal");
  const cliOnboarding = JSON.parse(fs.readFileSync(path.join(cliTmp, ".bee", "onboarding.json"), "utf8"));
  check(!("review" in (cliOnboarding.agents_sync?.rendered_from || {})) &&
    !(cliOnboarding.agents_sync?.files || []).includes(".claude/agents/bee-review.md"),
    "agents_sync record drops the cli-shaped review tier from files/rendered_from",
    JSON.stringify(cliOnboarding.agents_sync));
  const cliRecheck = await runOnboard(["--repo-root", cliTmp, "--json"], cliHome);
  check(cliRecheck.payload?.status === "up_to_date",
    "cli-tier repo settles to up_to_date after the removal apply",
    JSON.stringify(cliRecheck.payload?.plan));

  // --- 3. verify AGENTS.md markers ------------------------------------------
  const agentsText = fs.existsSync(path.join(tmp, "AGENTS.md"))
    ? fs.readFileSync(path.join(tmp, "AGENTS.md"), "utf8")
    : "";
  check(agentsText.includes("<!-- BEE:START -->") && agentsText.includes("<!-- BEE:END -->"),
    "AGENTS.md contains BEE:START/END markers");
  check(agentsText.includes("node .bee/bin/bee.mjs status --json"), "AGENTS block mentions bee.mjs status first step");
  check(agentsText.includes("commands.verify") && agentsText.includes("never build on red"),
    "AGENTS block carries the CI-status-gate startup step");

  // --- 3a. minimal header above the block (D4, propose_agents_header) -------
  check(agentsText.startsWith(`# ${path.basename(tmp)}\n`),
    "applied header opens with the repo folder title above the block");
  check(agentsText.includes("<!-- [unknown] one-line project description - replace me -->"),
    "header carries the loud [unknown] fill-me gap line");
  check(!agentsText.includes("- README.md") && !agentsText.includes("- docs/specs/"),
    "no pointer lines for files that do not exist");
  const applyHeaderAgain = await runOnboard(["--repo-root", tmp, "--apply", "--json"], tmpHome);
  check(applyHeaderAgain.payload?.status === "applied", "re-apply after header succeeds");
  check(fs.readFileSync(path.join(tmp, "AGENTS.md"), "utf8") === agentsText,
    "re-apply leaves header AGENTS.md byte-identical (idempotent)");

  // --- 3b. standard-commands capture notice (docs/09 item 1) ----------------
  check(Array.isArray(apply1.payload?.notices) &&
    apply1.payload.notices.some((n) => n.includes("standard commands")),
    "apply on repo without commands surfaces the capture notice");
  // No manifests in this repo -> the open-question notice, no detected candidates (D3).
  check(apply1.payload.notices.some((n) => n.includes("Ask the user")) &&
    !apply1.payload.notices.some((n) => n.includes("Detected candidates")),
    "notice stays the open question when detection finds nothing");
  // P1 / docs/09 item 6: first onboard without a build carries the init-lane offer.
  check(apply1.payload.notices.some((n) => n.includes("init lane") && n.includes("init cell")),
    "first onboard without a build surfaces the greenfield init-lane notice");
  const reapplyNotice = await runOnboard(["--repo-root", tmp, "--json"], tmpHome);
  check(!(reapplyNotice.payload?.notices || []).some((n) => n.includes("init lane")),
    "init-lane notice fires on the FIRST onboard only",
    JSON.stringify(reapplyNotice.payload?.notices || null));
  const cfgPath = path.join(tmp, ".bee", "config.json");
  const cfgRaw = JSON.parse(fs.readFileSync(cfgPath, "utf8"));
  cfgRaw.commands = { verify: "npm test" };
  fs.writeFileSync(cfgPath, `${JSON.stringify(cfgRaw, null, 2)}\n`, "utf8");
  const planNotice = await runOnboard(["--repo-root", tmp, "--json"], tmpHome);
  check(Array.isArray(planNotice.payload?.notices) && planNotice.payload.notices.length === 0,
    "notice disappears once commands are recorded",
    JSON.stringify(planNotice.payload?.notices || null));
  delete cfgRaw.commands;
  fs.writeFileSync(cfgPath, `${JSON.stringify(cfgRaw, null, 2)}\n`, "utf8");
  const stateSource = fs.readFileSync(path.join(TEMPLATES_LIB_DIR, "state.mjs"), "utf8");
  const stateKeys = stateSource.match(/COMMAND_KEYS = \[([^\]]+)\]/)?.[1] || "";
  const onboardSource = fs.readFileSync(path.join(SCRIPTS_DIR, "onboard_bee.mjs"), "utf8");
  const onboardKeys = onboardSource.match(/COMMAND_KEYS = \[([^\]]+)\]/)?.[1] || "";
  const normKeys = (s) => s.replace(/["'\s]/g, "");
  check(stateKeys && normKeys(stateKeys) === normKeys(onboardKeys),
    "onboard_bee.mjs COMMAND_KEYS matches lib/state.mjs (no drift)",
    `state: [${stateKeys}] vs onboard: [${onboardKeys}]`);

  // --- 3b-drift. STALE_ADVISOR_KEY_WARNING text must not drift (P2, fanout-4
  // review fix) --- onboard_bee.mjs deliberately does NOT import lib/state.mjs
  // (see the comment at its own STALE_ADVISOR_KEY_WARNING definition) — pin the
  // two literal warning strings with a text-scan instead, same pattern as the
  // COMMAND_KEYS drift check just above.
  const warningRe = /STALE_ADVISOR_KEY_WARNING\s*=\s*['"]([^'"]+)['"]/;
  const stateWarning = stateSource.match(warningRe)?.[1] || "";
  const onboardWarning = onboardSource.match(warningRe)?.[1] || "";
  check(stateWarning.length > 0 && stateWarning === onboardWarning,
    "onboard_bee.mjs STALE_ADVISOR_KEY_WARNING text matches lib/state.mjs (no drift)",
    `state: "${stateWarning}" vs onboard: "${onboardWarning}"`);

  // --- 3b-drift2. AGENT_TIER_DEFAULTS_CLAUDE must not drift from state.mjs
  // DEFAULT_MODELS.claude (same discipline as the two checks just above -
  // onboard_bee.mjs never import-depends on state.mjs's exports, so the
  // claude-runtime tier defaults are duplicated, and pinned here instead).
  const stateDefaultsMatch = stateSource.match(/DEFAULT_MODELS = \{[\s\S]*?claude:\s*\{([^}]+)\}/);
  const onboardDefaultsMatch = onboardSource.match(/AGENT_TIER_DEFAULTS_CLAUDE = \{([^}]+)\}/);
  const normDefaults = (s) => (s || "").replace(/["'\s]/g, "");
  check(Boolean(stateDefaultsMatch) && Boolean(onboardDefaultsMatch) &&
    normDefaults(stateDefaultsMatch[1]) === normDefaults(onboardDefaultsMatch[1]),
    "onboard_bee.mjs AGENT_TIER_DEFAULTS_CLAUDE matches lib/state.mjs DEFAULT_MODELS.claude (no drift)",
    `state: "${stateDefaultsMatch?.[1]}" vs onboard: "${onboardDefaultsMatch?.[1]}"`);

  // --- 3b-advisor. a host fixture with a stale advisor key surfaces the
  // stale-key notice (P1, fanout-4 review fix); the notice disappears once the
  // key is removed --------------------------------------------------------
  const advisorTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-advisor-notice-test-"));
  const advisorHome = makeFakeHome();
  try {
    const advisorApply = await runOnboardAt(ONBOARD, ["--repo-root", advisorTmp, "--apply", "--json"], advisorHome);
    check(advisorApply.payload?.status === "applied", "apply on fresh advisor-notice fixture succeeds");
    const advisorCfgPath = path.join(advisorTmp, ".bee", "config.json");
    const advisorCfgRaw = JSON.parse(fs.readFileSync(advisorCfgPath, "utf8"));
    advisorCfgRaw.advisor = { enabled: true, at: ["execution"], model: "opus" };
    fs.writeFileSync(advisorCfgPath, `${JSON.stringify(advisorCfgRaw, null, 2)}\n`, "utf8");
    const advisorPlanNotice = await runOnboardAt(ONBOARD, ["--repo-root", advisorTmp, "--json"], advisorHome);
    check(Array.isArray(advisorPlanNotice.payload?.notices) &&
      advisorPlanNotice.payload.notices.some((n) => n === stateWarning),
      "a host fixture whose config carries a stale advisor key surfaces the stale-key notice line",
      JSON.stringify(advisorPlanNotice.payload?.notices || null));
    delete advisorCfgRaw.advisor;
    fs.writeFileSync(advisorCfgPath, `${JSON.stringify(advisorCfgRaw, null, 2)}\n`, "utf8");
    const advisorCleanNotice = await runOnboardAt(ONBOARD, ["--repo-root", advisorTmp, "--json"], advisorHome);
    check(!(advisorCleanNotice.payload?.notices || []).some((n) => n === stateWarning),
      "stale-advisor notice disappears once the key is removed from config.json",
      JSON.stringify(advisorCleanNotice.payload?.notices || null));
  } finally {
    try {
      fs.rmSync(advisorTmp, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 3c. detected command candidates ride the notice, propose-only (D3) ---
  const detTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-detect-test-"));
  try {
    fs.writeFileSync(path.join(detTmp, "package.json"),
      `${JSON.stringify({ name: "fixture", scripts: { test: "vitest run" } }, null, 2)}\n`, "utf8");
    const detApply = await runOnboard(["--repo-root", detTmp, "--apply", "--json"]);
    check(detApply.payload?.status === "applied", "apply on manifest-bearing repo succeeds");
    const detNotices = detApply.payload?.notices || [];
    check(detNotices.some((n) => n.includes("Detected candidates") &&
      n.includes("test: npm test — package.json")),
      "notice lists detected candidates as key: value — source proposals",
      JSON.stringify(detNotices));
    check(detNotices.some((n) => n.includes("confirmation question") && n.includes("confirmed")),
      "candidate notice instructs confirm-before-write");
    check(!detNotices.some((n) => n.includes("init lane")),
      "a repo WITH a detectable build never gets the init-lane notice");
    const detConfig = JSON.parse(
      fs.readFileSync(path.join(detTmp, ".bee", "config.json"), "utf8"));
    check(detConfig.commands === undefined,
      "apply writes no detected values to config.json commands",
      JSON.stringify(detConfig.commands || null));
  } finally {
    try {
      fs.rmSync(detTmp, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 4. verify .bee tree ---------------------------------------------------
  for (const rel of [
    ".bee/onboarding.json",
    ".bee/state.json",
    ".bee/config.json",
    ".bee/reservations.json",
    ".bee/decisions.jsonl",
    ".bee/backlog.jsonl",
  ]) {
    check(fs.existsSync(path.join(tmp, rel)), `${rel} exists`);
  }
  for (const rel of [".bee/cells", ".bee/logs"]) {
    check(fs.existsSync(path.join(tmp, rel)) && fs.statSync(path.join(tmp, rel)).isDirectory(),
      `${rel}/ directory exists`);
  }
  check(fs.existsSync(path.join(tmp, "docs", "history", "learnings", "critical-patterns.md")),
    "docs/history/learnings/critical-patterns.md stub exists");

  const config = JSON.parse(fs.readFileSync(path.join(tmp, ".bee", "config.json"), "utf8"));
  check(config.hooks && Object.values(config.hooks).every((v) => v === true) &&
    Object.keys(config.hooks).length === 6, "config.json has all 6 hooks enabled");

  // --- 5. verify bin/lib copy (tolerate missing packages/bee: parallel INFRA) ---
  const helperNames = listMjs(TEMPLATES_DIR);
  if (helperNames.length === 0) {
    skip("helper copy to .bee/bin", "no packages/bee/*.mjs present yet");
  } else {
    for (const name of helperNames) {
      const src = fs.readFileSync(path.join(TEMPLATES_DIR, name), "utf8");
      const dst = path.join(tmp, ".bee", "bin", name);
      check(fs.existsSync(dst) && fs.readFileSync(dst, "utf8") === src,
        `.bee/bin/${name} copied verbatim`);
    }
  }
  const libNames = listMjs(TEMPLATES_LIB_DIR);
  if (libNames.length === 0) {
    skip("lib copy to .bee/bin/lib", "no packages/bee/lib/*.mjs present yet");
  } else {
    for (const name of libNames) {
      const src = fs.readFileSync(path.join(TEMPLATES_LIB_DIR, name), "utf8");
      const dst = path.join(tmp, ".bee", "bin", "lib", name);
      check(fs.existsSync(dst) && fs.readFileSync(dst, "utf8") === src,
        `.bee/bin/lib/${name} copied verbatim`);
    }
  }
  check(!fs.existsSync(path.join(tmp, ".bee", "bin", "AGENTS.block.md")),
    "AGENTS.block.md is NOT copied into .bee/bin");

  // --- 6. plan mode again -> up_to_date --------------------------------------
  const plan2 = await runOnboard(["--repo-root", tmp, "--json"], tmpHome);
  check(plan2.payload?.status === "up_to_date", "second plan run reports up_to_date",
    JSON.stringify(plan2.payload?.plan || []));

  // --- 7. AGENTS block idempotency -------------------------------------------
  // User content outside the markers must survive; a tampered block inside the
  // markers must be restored; a second apply must be a byte-level no-op.
  const userHeader = "# My project\n\nHand-written intro that bee must not touch.\n\n";
  const userFooter = "\n## Appendix\n\nMore hand-written content after the block.\n";
  const current = fs.readFileSync(path.join(tmp, "AGENTS.md"), "utf8");
  const tampered = current.replace(
    /<!-- BEE:START -->[\s\S]*?<!-- BEE:END -->/,
    "<!-- BEE:START -->\nTAMPERED CONTENT\n<!-- BEE:END -->",
  );
  fs.writeFileSync(path.join(tmp, "AGENTS.md"), userHeader + tampered + userFooter, "utf8");

  const plan3 = await runOnboard(["--repo-root", tmp, "--json"], tmpHome);
  check(plan3.payload?.status === "changes_needed", "tampered block detected as changes_needed");
  check(plan3.payload?.plan?.some((i) => i.action === "update_agents_block"),
    "plan includes update_agents_block");

  const apply2 = await runOnboard(["--repo-root", tmp, "--apply", "--json"], tmpHome);
  check(apply2.payload?.status === "applied", "re-apply after tamper succeeds");
  const restored = fs.readFileSync(path.join(tmp, "AGENTS.md"), "utf8");
  check(restored.includes("Hand-written intro that bee must not touch."),
    "user content before the block preserved");
  check(restored.includes("More hand-written content after the block."),
    "user content after the block preserved");
  check(!restored.includes("TAMPERED CONTENT"), "tampered block content restored");
  check(restored.includes("<!-- BEE:START -->") &&
    restored.indexOf("<!-- BEE:START -->") === restored.lastIndexOf("<!-- BEE:START -->"),
    "exactly one BEE block after re-apply");

  const apply3 = await runOnboard(["--repo-root", tmp, "--apply", "--json"], tmpHome);
  const afterThird = fs.readFileSync(path.join(tmp, "AGENTS.md"), "utf8");
  check(afterThird === restored, "third apply is byte-identical (idempotent)");
  check(apply3.payload?.recheck === "up_to_date", "third apply recheck up_to_date");

  // --- 7b. propose_agents_header semantics (D4) -------------------------------
  // Prose outside the markers -> never proposed, prose preserved byte-for-byte.
  const proseTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-header-prose-"));
  const proseHome = makeFakeHome();
  try {
    const prose = "# Handwritten\n\nThis project does X.";
    fs.writeFileSync(path.join(proseTmp, "AGENTS.md"), `${prose}\n`, "utf8");
    const prosePlan = await runOnboard(["--repo-root", proseTmp, "--json"], proseHome);
    check(!(prosePlan.payload?.plan || []).some((i) => i.action === "propose_agents_header"),
      "prose outside markers never yields propose_agents_header",
      JSON.stringify(prosePlan.payload?.plan || []));
    await runOnboard(["--repo-root", proseTmp, "--apply", "--json"], proseHome);
    const proseAfter = fs.readFileSync(path.join(proseTmp, "AGENTS.md"), "utf8");
    check(proseAfter.startsWith(prose),
      "existing prose preserved byte-for-byte ahead of the appended block");
    check(!proseAfter.includes("[unknown] one-line project description"),
      "no header injected into a prose-bearing AGENTS.md");
  } finally {
    try {
      fs.rmSync(proseTmp, { recursive: true, force: true });
      fs.rmSync(proseHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // Pointer lines appear only for files that exist at plan time.
  const ptrTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-header-ptr-"));
  try {
    fs.writeFileSync(path.join(ptrTmp, "README.md"), "# readme\n", "utf8");
    fs.mkdirSync(path.join(ptrTmp, "docs", "specs"), { recursive: true });
    fs.writeFileSync(path.join(ptrTmp, "docs", "specs", "reading-map.md"), "# map\n", "utf8");
    await runOnboard(["--repo-root", ptrTmp, "--apply", "--json"]);
    const ptrText = fs.readFileSync(path.join(ptrTmp, "AGENTS.md"), "utf8");
    check(ptrText.includes("- README.md") && ptrText.includes("- docs/specs/reading-map.md"),
      "header pointer lines present for files that exist");
    check(!ptrText.includes("- docs/specs/system-overview.md"),
      "no pointer line for the missing system-overview.md");
  } finally {
    try {
      fs.rmSync(ptrTmp, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // Block-only AGENTS.md (already-onboarded, pre-header) flips up_to_date ->
  // changes_needed with only the propose item: intended propose-only upgrade.
  const flipTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-header-flip-"));
  const flipHome = makeFakeHome();
  try {
    await runOnboard(["--repo-root", flipTmp, "--apply", "--json"], flipHome);
    const flipFull = fs.readFileSync(path.join(flipTmp, "AGENTS.md"), "utf8");
    const blockOnly = flipFull.slice(flipFull.indexOf("<!-- BEE:START -->"));
    fs.writeFileSync(path.join(flipTmp, "AGENTS.md"),
      `<!-- keep\nthis multi-line comment -->\n${blockOnly}`, "utf8");
    const flipPlan = await runOnboard(["--repo-root", flipTmp, "--json"], flipHome);
    check(flipPlan.payload?.status === "changes_needed" &&
      (flipPlan.payload?.plan || []).length > 0 &&
      flipPlan.payload.plan.every((i) => i.action === "propose_agents_header"),
      "block-only AGENTS.md flips up_to_date -> changes_needed with only propose_agents_header",
      JSON.stringify(flipPlan.payload?.plan || []));
    await runOnboard(["--repo-root", flipTmp, "--apply", "--json"], flipHome);
    const flipAfter = fs.readFileSync(path.join(flipTmp, "AGENTS.md"), "utf8");
    check(flipAfter.startsWith(`# ${path.basename(flipTmp)}\n`),
      "header prepended at the top of a block-only AGENTS.md");
    check(flipAfter.includes("<!-- keep\nthis multi-line comment -->"),
      "comment-only content outside markers preserved (comments are not prose)");
    const flipRecheck = await runOnboard(["--repo-root", flipTmp, "--json"], flipHome);
    check(flipRecheck.payload?.status === "up_to_date",
      "header apply settles the flip back to up_to_date");
  } finally {
    try {
      fs.rmSync(flipTmp, { recursive: true, force: true });
      fs.rmSync(flipHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 8. never overwrite existing state/decisions/cells ---------------------
  const customState = { schema_version: "1.0", phase: "swarming", marker: "user-owned" };
  fs.writeFileSync(path.join(tmp, ".bee", "state.json"),
    `${JSON.stringify(customState, null, 2)}\n`, "utf8");
  fs.appendFileSync(path.join(tmp, ".bee", "decisions.jsonl"),
    `${JSON.stringify({ event: "decide", decision: "keep me" })}\n`, "utf8");
  fs.writeFileSync(path.join(tmp, ".bee", "cells", "demo-1.json"),
    `${JSON.stringify({ id: "demo-1", status: "open" })}\n`, "utf8");

  await runOnboard(["--repo-root", tmp, "--apply", "--json"], tmpHome);
  const stateAfter = JSON.parse(fs.readFileSync(path.join(tmp, ".bee", "state.json"), "utf8"));
  check(stateAfter.marker === "user-owned" && stateAfter.phase === "swarming",
    "existing state.json never overwritten");
  check(fs.readFileSync(path.join(tmp, ".bee", "decisions.jsonl"), "utf8").includes("keep me"),
    "existing decisions.jsonl never overwritten");
  check(fs.existsSync(path.join(tmp, ".bee", "cells", "demo-1.json")),
    "existing cells never removed");

  // --- 9. --repo-hooks --------------------------------------------------------
  const hooksPlan = await runOnboard(["--repo-root", tmp, "--repo-hooks", "--json"], tmpHome);
  check(hooksPlan.payload?.status === "changes_needed", "--repo-hooks plan reports changes_needed");

  // Pre-seed a settings.json so the .bak backup path is exercised.
  fs.mkdirSync(path.join(tmp, ".claude"), { recursive: true });
  fs.writeFileSync(path.join(tmp, ".claude", "settings.json"),
    `${JSON.stringify({ permissions: { allow: ["Bash(ls:*)"] } }, null, 2)}\n`, "utf8");

  const hooksApply = await runOnboard(["--repo-root", tmp, "--apply", "--repo-hooks", "--json"], tmpHome);
  check(hooksApply.payload?.status === "applied", "--repo-hooks apply succeeds");
  check(hooksApply.payload?.recheck === "up_to_date", "--repo-hooks recheck up_to_date",
    JSON.stringify(hooksApply.payload?.recheck_plan || []));

  const settings = JSON.parse(
    fs.readFileSync(path.join(tmp, ".claude", "settings.json"), "utf8"));
  check(settings.permissions?.allow?.[0] === "Bash(ls:*)",
    "existing settings.json content preserved by merge");
  const settingsText = JSON.stringify(settings);
  for (const name of [
    "bee-session-init.mjs",
    "bee-prompt-context.mjs",
    "bee-write-guard.mjs",
    "bee-state-sync.mjs",
    "bee-chain-nudge.mjs",
    "bee-session-close.mjs",
    "bee-model-guard.mjs",
  ]) {
    check(settingsText.includes(`.bee/bin/hooks/${name}`), `settings.json wires ${name}`);
    check(fs.existsSync(path.join(tmp, ".bee", "bin", "hooks", name)),
      `.bee/bin/hooks/${name} copied`);
  }
  const codexAuditHandler = "bee-codex-subagent-audit.mjs";
  check(fs.existsSync(path.join(tmp, ".bee", "bin", "hooks", codexAuditHandler)),
    `fresh repo-hook host copies .bee/bin/hooks/${codexAuditHandler}`);
  check(!settingsText.includes(codexAuditHandler),
    "Claude repo settings do not wire the Codex-only subagent audit handler");
  check(settingsText.includes('\\"$CLAUDE_PROJECT_DIR\\"') ||
    settingsText.includes('"$CLAUDE_PROJECT_DIR"'),
    "hook commands use $CLAUDE_PROJECT_DIR-style paths");
  check(fs.existsSync(path.join(tmp, ".claude", "settings.json.bak")),
    "settings.json.bak backup created");

  // --- 9a. structural PreToolUse wiring (P1-4): parse, don't string-search ---
  // A wrong event, wrong matcher, or model-guard folded into the write-guard
  // entry must all turn this red; string containment (above) cannot tell.
  const preToolUse = Array.isArray(settings.hooks?.PreToolUse) ? settings.hooks.PreToolUse : [];
  const modelGuardEntries = preToolUse.filter((e) =>
    (e.hooks || []).some((h) => String(h.command || "").includes("bee-model-guard.mjs")));
  check(modelGuardEntries.length === 1 && modelGuardEntries[0].matcher === "Agent|Task",
    "exactly one PreToolUse entry wires bee-model-guard.mjs, matcher is exactly 'Agent|Task'",
    JSON.stringify(modelGuardEntries));

  const writeGuardEntries = preToolUse.filter((e) =>
    (e.hooks || []).some((h) => String(h.command || "").includes("bee-write-guard.mjs")));
  check(writeGuardEntries.length === 1 &&
    writeGuardEntries[0].matcher === "Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion",
    "exactly one PreToolUse entry wires bee-write-guard.mjs, matcher is byte-identical to the write-guard matcher",
    JSON.stringify(writeGuardEntries));

  // D4 (codex-native-runtime-v2): renderRepoHookEntries()'s PostToolUse
  // matcher for bee-state-sync.mjs must carry the update_plan superset
  // (never a swap) - parsed structurally, not string-contained, so a matcher
  // that dropped a legacy name would turn this red instead of hiding behind
  // "settings.json still mentions bee-state-sync.mjs somewhere".
  const postToolUse = Array.isArray(settings.hooks?.PostToolUse) ? settings.hooks.PostToolUse : [];
  const stateSyncEntries = postToolUse.filter((e) =>
    (e.hooks || []).some((h) => String(h.command || "").includes("bee-state-sync.mjs")));
  check(stateSyncEntries.length === 1 &&
    stateSyncEntries[0].matcher === "update_plan|TaskCreate|TaskUpdate|TodoWrite",
    "exactly one PostToolUse entry wires bee-state-sync.mjs, matcher is exactly the update_plan superset",
    JSON.stringify(stateSyncEntries));

  // model-guard must not be folded into any other event or entry anywhere in
  // the applied settings tree.
  const modelGuardSightings = [];
  for (const [eventName, entries] of Object.entries(settings.hooks || {})) {
    for (const entry of Array.isArray(entries) ? entries : []) {
      for (const hook of entry.hooks || []) {
        if (String(hook.command || "").includes("bee-model-guard.mjs")) {
          modelGuardSightings.push({ event: eventName, matcher: entry.matcher ?? null });
        }
      }
    }
  }
  check(modelGuardSightings.length === 1 &&
    modelGuardSightings[0].event === "PreToolUse" && modelGuardSightings[0].matcher === "Agent|Task",
    "bee-model-guard.mjs appears exactly once anywhere in settings.hooks: PreToolUse / Agent|Task",
    JSON.stringify(modelGuardSightings));

  // --- 9b. Claude plugin <-> repo parity: same (event, matcher, filename) ---
  // triples. hooks/claude-hooks.json (plugin, CLAUDE_PLUGIN_ROOT, wired by
  // .claude-plugin/plugin.json) and renderRepoHookEntries()'s applied output
  // (repo, CLAUDE_PROJECT_DIR) must expose an identical hook contract once
  // command roots are normalized away and statusMessage ignored. Since the
  // codex-parity-2 catalog inversion (commit d1777ed), hooks/hooks.json is
  // the CODEX default projection instead, so the Claude-settings parity
  // check compares against hooks/claude-hooks.json (codex-parity-2b).
  function hookFilenameFromCommand(command) {
    const m = String(command || "").match(/([A-Za-z0-9_.-]+\.mjs)/);
    return m ? m[1] : null;
  }
  function flattenHookTriples(hooksObj) {
    const triples = [];
    for (const [eventName, entries] of Object.entries(hooksObj || {})) {
      for (const entry of Array.isArray(entries) ? entries : []) {
        const matcher = entry.matcher ?? null;
        for (const hook of entry.hooks || []) {
          const filename = hookFilenameFromCommand(hook.command);
          if (filename) {
            triples.push(`${eventName}::${matcher}::${filename}`);
          }
        }
      }
    }
    return triples.sort();
  }
  const claudeHooksJsonPath = path.join(REPO_ROOT, "hooks", "claude-hooks.json");
  const claudeHooksJson = JSON.parse(fs.readFileSync(claudeHooksJsonPath, "utf8"));
  const claudeCatalogTriples = flattenHookTriples(claudeHooksJson.hooks);
  const repoTriples = flattenHookTriples(settings.hooks);
  check(claudeCatalogTriples.length > 0, "hooks/claude-hooks.json parsed at least one hook triple",
    JSON.stringify(claudeCatalogTriples));
  check(JSON.stringify(claudeCatalogTriples) === JSON.stringify(repoTriples),
    "hooks/claude-hooks.json and the applied repo settings expose identical (event, matcher, filename) triples",
    `claude-hooks.json: ${JSON.stringify(claudeCatalogTriples)}\nrepo:   ${JSON.stringify(repoTriples)}`);

  // --- 9b2. Codex <-> Claude projection parity via hooks/catalog.mjs -------
  // hooks/hooks.json (Codex default projection) and hooks/claude-hooks.json
  // (Claude projection) must differ ONLY by the differences hooks/catalog.mjs
  // declares as allowed (ALLOWED_DIFFERENCES export) — the boundary is
  // imported from the catalog, never re-hardcoded here, and this check must
  // never be dropped (codex-parity-2b; CONTEXT.md decisions D1/D2).
  const catalogModulePath = path.join(REPO_ROOT, "hooks", "catalog.mjs");
  const { ALLOWED_DIFFERENCES } = await import(pathToFileURL(catalogModulePath).href);
  const codexHooksJsonPath = path.join(REPO_ROOT, "hooks", "hooks.json");
  const codexHooksJson = JSON.parse(fs.readFileSync(codexHooksJsonPath, "utf8"));
  const codexProjectionTriples = flattenHookTriples(codexHooksJson.hooks);

  function tripleEvent(triple) {
    return triple.split("::")[0];
  }
  function tripleMatcher(triple) {
    const m = triple.split("::")[1];
    return m === "null" ? null : m;
  }
  function isAllowedDifference(triple) {
    return ALLOWED_DIFFERENCES.some((d) =>
      d.event === tripleEvent(triple) && (d.matcher ?? null) === tripleMatcher(triple));
  }

  const onlyInClaudeProjection = claudeCatalogTriples.filter((t) => !codexProjectionTriples.includes(t));
  const onlyInCodexProjection = codexProjectionTriples.filter((t) => !claudeCatalogTriples.includes(t));
  const allProjectionDiffs = [...onlyInClaudeProjection, ...onlyInCodexProjection];

  check(ALLOWED_DIFFERENCES.length > 0 &&
    allProjectionDiffs.length > 0 &&
    allProjectionDiffs.every(isAllowedDifference),
    "hooks/hooks.json (Codex) and hooks/claude-hooks.json (Claude) differ only by hooks/catalog.mjs ALLOWED_DIFFERENCES",
    JSON.stringify({ onlyInClaudeProjection, onlyInCodexProjection, ALLOWED_DIFFERENCES }));
  check(ALLOWED_DIFFERENCES.every((d) =>
    allProjectionDiffs.some((t) => tripleEvent(t) === d.event && tripleMatcher(t) === (d.matcher ?? null))),
    "every hooks/catalog.mjs ALLOWED_DIFFERENCES entry corresponds to an actual projection difference",
    JSON.stringify({ ALLOWED_DIFFERENCES, allProjectionDiffs }));

  // --- 9c. Codex repo projection (.codex/hooks.json) -------------------------
  // --repo-hooks must also wire the Codex side: without .codex/hooks.json a
  // Codex session in the host repo runs with NO bee guards (the gap that
  // motivated this projection). Structural checks, not string containment.
  const codexRepoPath = path.join(tmp, ".codex", "hooks.json");
  check(fs.existsSync(codexRepoPath), "--repo-hooks creates .codex/hooks.json");
  const codexRepo = JSON.parse(fs.readFileSync(codexRepoPath, "utf8"));
  const codexRepoText = JSON.stringify(codexRepo);
  check(!codexRepoText.includes("CLAUDE_PROJECT_DIR"),
    ".codex/hooks.json never uses $CLAUDE_PROJECT_DIR (Codex never sets it)");
  // cnr2-8 (codex-native-runtime-v2 D4): the Codex projection DOES wire
  // bee-model-guard.mjs, but on the spawn_agent matcher (Claude uses Agent|Task)
  // — a per-runtime matcher difference pinned in ALLOWED_DIFFERENCES, not a
  // Claude-only guard. Structural check: exactly one PreToolUse entry whose
  // matcher is "spawn_agent" wires the guard.
  const codexPreToolUse = Array.isArray(codexRepo.hooks?.PreToolUse) ? codexRepo.hooks.PreToolUse : [];
  const codexSpawnGuardEntries = codexPreToolUse.filter((e) =>
    e.matcher === "spawn_agent" &&
    (e.hooks || []).some((h) => String(h.command || "").includes("bee-model-guard.mjs")));
  check(codexSpawnGuardEntries.length === 1,
    ".codex/hooks.json wires bee-model-guard.mjs on exactly one PreToolUse spawn_agent entry (Codex spawn guard)",
    JSON.stringify(codexPreToolUse));
  let codexCommandCount = 0;
  let codexTransportOk = true;
  let codexStatusMessageOk = true;
  const codexReferencedHandlers = new Set();
  const codexAuditEvents = [];
  for (const [eventName, entries] of Object.entries(codexRepo.hooks || {})) {
    for (const entry of Array.isArray(entries) ? entries : []) {
      for (const hook of entry.hooks || []) {
        codexCommandCount += 1;
        const cmd = String(hook.command || "");
        const handler = hookFilenameFromCommand(cmd);
        if (handler?.startsWith("bee-")) {
          codexReferencedHandlers.add(handler);
        }
        if (handler === codexAuditHandler) {
          codexAuditEvents.push(eventName);
        }
        if (!cmd.startsWith('r="$(git rev-parse --show-toplevel 2>/dev/null)"') ||
            !cmd.includes("bee: hook transport unavailable (no git root)") ||
            !cmd.includes('exec node "$r"/.bee/bin/hooks/bee-') ||
            !cmd.includes("--source=repo")) {
          codexTransportOk = false;
        }
        if (typeof hook.statusMessage !== "string" || !hook.statusMessage.startsWith("bee:")) {
          codexStatusMessageOk = false;
        }
      }
    }
  }
  check(codexCommandCount === 13, ".codex/hooks.json wires exactly 13 hook commands",
    `count: ${codexCommandCount}`);
  check(codexTransportOk,
    "every .codex/hooks.json command uses the git-root transport with the pinned fail-open diagnostic");
  check(codexStatusMessageOk, "every .codex/hooks.json command carries a bee statusMessage");
  check([...codexReferencedHandlers].every((name) =>
    fs.existsSync(path.join(tmp, ".bee", "bin", "hooks", name))),
    "fresh repo-hook host copies every handler referenced by generated Codex hooks",
    JSON.stringify([...codexReferencedHandlers]));
  check(JSON.stringify(codexAuditEvents.sort()) ===
    JSON.stringify(["SubagentStart", "SubagentStop"]),
    "generated Codex SubagentStart and SubagentStop each resolve to the copied bounded audit handler",
    JSON.stringify(codexAuditEvents));

  // D4 (codex-native-runtime-v2): renderCodexHookEntries()'s PostToolUse
  // matcher for bee-state-sync.mjs must carry the update_plan superset -
  // parsed structurally (own row, independent of the triple-parity check
  // below), so a matcher regression here cannot hide behind that check
  // accidentally passing for an unrelated reason.
  const codexPostToolUse = Array.isArray(codexRepo.hooks?.PostToolUse) ? codexRepo.hooks.PostToolUse : [];
  const codexStateSyncEntries = codexPostToolUse.filter((e) =>
    (e.hooks || []).some((h) => String(h.command || "").includes("bee-state-sync.mjs")));
  check(codexStateSyncEntries.length === 1 &&
    codexStateSyncEntries[0].matcher === "update_plan|TaskCreate|TaskUpdate|TodoWrite",
    "exactly one .codex/hooks.json PostToolUse entry wires bee-state-sync.mjs, matcher is exactly the update_plan superset",
    JSON.stringify(codexStateSyncEntries));

  // Parity with the checked-in Codex plugin projection: identical
  // (event, matcher, filename) triples — only the command root differs.
  const codexRepoTriples = flattenHookTriples(codexRepo.hooks);
  check(JSON.stringify(codexRepoTriples) === JSON.stringify(codexProjectionTriples),
    ".codex/hooks.json and hooks/hooks.json (Codex plugin projection) expose identical triples",
    `repo:   ${JSON.stringify(codexRepoTriples)}\nplugin: ${JSON.stringify(codexProjectionTriples)}`);

  // Merge discipline: a user's non-bee Codex hook survives a re-apply, bee
  // entries do not duplicate, and the pre-existing file is backed up.
  const userCodexHook = {
    type: "command",
    command: "echo user-owned-codex-hook",
  };
  // Seed the user group AFTER the bee group: the merge normalizes to
  // [non-bee..., bee...], so this ordering forces changed=true and exercises
  // the rewrite + .bak path (user-before-bee is already canonical shape).
  const codexSeed = JSON.parse(fs.readFileSync(codexRepoPath, "utf8"));
  codexSeed.hooks.SessionStart = [
    ...codexSeed.hooks.SessionStart,
    { hooks: [userCodexHook] },
  ];
  // Stale bee wiring in historical transport shapes must be REPLACED, never
  // preserved beside the canonical render (a stale twin double-fires the
  // event): the bee source repo's own "$r"/hooks form, and the dead
  // hand-authored $CLAUDE_PROJECT_DIR form.
  codexSeed.hooks.Stop = [
    {
      hooks: [{
        type: "command",
        command: 'r="$(git rev-parse --show-toplevel 2>/dev/null)"\nexec node "$r"/hooks/bee-state-sync.mjs --source=repo',
      }],
    },
    ...(codexSeed.hooks.Stop || []),
  ];
  codexSeed.hooks.UserPromptSubmit = [
    { hooks: [{ type: "command", command: 'node "$CLAUDE_PROJECT_DIR"/.bee/bin/hooks/bee-prompt-context.mjs' }] },
    ...(codexSeed.hooks.UserPromptSubmit || []),
  ];
  fs.writeFileSync(codexRepoPath, `${JSON.stringify(codexSeed, null, 2)}\n`, "utf8");

  // --- 9d. state-layer skeletons (docs/specs) --------------------------------
  // The first apply (section 2) must have created both skeletons; a re-apply
  // must never touch an existing (user/scribing-owned) file.
  check(fs.existsSync(path.join(tmp, "docs", "specs", "reading-map.md")),
    "onboarding creates docs/specs/reading-map.md skeleton");
  check(fs.existsSync(path.join(tmp, "docs", "specs", "system-overview.md")),
    "onboarding creates docs/specs/system-overview.md skeleton");
  fs.writeFileSync(path.join(tmp, "docs", "specs", "reading-map.md"),
    "# My map\n\nscribing-owned content\n", "utf8");

  // --repo-hooks apply twice -> no duplicate bee entries.
  await runOnboard(["--repo-root", tmp, "--apply", "--repo-hooks", "--json"], tmpHome);
  const settings2 = JSON.parse(
    fs.readFileSync(path.join(tmp, ".claude", "settings.json"), "utf8"));
  const initCount = JSON.stringify(settings2).split("bee-session-init.mjs").length - 1;
  check(initCount === 1, "no duplicate hook entries after second --repo-hooks apply",
    `count: ${initCount}`);
  const preToolUse2 = Array.isArray(settings2.hooks?.PreToolUse) ? settings2.hooks.PreToolUse : [];
  const agentTaskEntries2 = preToolUse2.filter((e) => e.matcher === "Agent|Task");
  check(agentTaskEntries2.length === 1,
    "exactly one PreToolUse Agent|Task entry survives a second --repo-hooks apply",
    `count: ${agentTaskEntries2.length}`);

  // Codex side of the second apply: user entry preserved, no bee duplicates,
  // .bak written, and the scribing-owned specs file untouched.
  const codexRepo2 = JSON.parse(fs.readFileSync(codexRepoPath, "utf8"));
  const codexText2 = JSON.stringify(codexRepo2);
  check(codexText2.includes("user-owned-codex-hook"),
    "non-bee .codex/hooks.json entry survives a second --repo-hooks apply");
  // codex-command-windows-1: every codex-repo entry now carries the filename
  // TWICE by design — once in `command` (POSIX) and once in `commandWindows`
  // (Windows override) — so a flat substring count over the whole stringified
  // object would conflate "one entry, two fields" with "two duplicate
  // entries". Count each field separately instead: this is a STRONGER guard
  // than the old flat count, since it also proves commandWindows itself never
  // duplicates independently of command.
  function fieldOccurrences(hooksObj, filename, field) {
    let count = 0;
    for (const entries of Object.values(hooksObj || {})) {
      for (const entry of Array.isArray(entries) ? entries : []) {
        for (const hook of entry.hooks || []) {
          if (String(hook[field] || "").includes(filename)) count += 1;
        }
      }
    }
    return count;
  }
  const codexInitCommandCount = fieldOccurrences(codexRepo2.hooks, "bee-session-init.mjs", "command");
  const codexInitCommandWindowsCount = fieldOccurrences(codexRepo2.hooks, "bee-session-init.mjs", "commandWindows");
  check(codexInitCommandCount === 1 && codexInitCommandWindowsCount === 1,
    "no duplicate bee entries in .codex/hooks.json after second apply (command and commandWindows each carry bee-session-init.mjs exactly once)",
    `command: ${codexInitCommandCount}, commandWindows: ${codexInitCommandWindowsCount}`);
  check(!codexText2.includes('"$r"/hooks/bee-'),
    'stale source-repo-shape ("$r"/hooks) bee entry replaced, not preserved');
  check(!codexText2.includes("CLAUDE_PROJECT_DIR"),
    "stale $CLAUDE_PROJECT_DIR-shape bee entry replaced, not preserved");
  const codexStateSyncCommandCount = fieldOccurrences(codexRepo2.hooks, "bee-state-sync.mjs", "command");
  const codexStateSyncCommandWindowsCount = fieldOccurrences(codexRepo2.hooks, "bee-state-sync.mjs", "commandWindows");
  check(codexStateSyncCommandCount === 3 && codexStateSyncCommandWindowsCount === 3,
    "bee-state-sync.mjs appears exactly 3x in command AND 3x in commandWindows (canonical PostToolUse/SubagentStop/Stop; seeded stale Stop twin dropped)",
    `command: ${codexStateSyncCommandCount}, commandWindows: ${codexStateSyncCommandWindowsCount}`);
  // Regression guard for the Windows contract itself (hardening-1-7-10 D6,
  // tightened by cell 1710-6): every codex-repo entry's commandWindows is now
  // the git-root-resolving "node -e SCRIPT <script>" bootstrap, not the old
  // bare "node .bee/bin/hooks/<file>.mjs --source=repo" form (that form
  // assumed the session cwd IS the repo root and threw MODULE_NOT_FOUND from
  // a nested cwd). This mirrors the NODE_WINDOWS_BOOTSTRAP structural regex +
  // FORBIDDEN_WINDOWS_CHARS check hooks/test_hook_contracts.mjs's
  // runCatalogDriftChecks pins for the exact same contract — that file is not
  // importable here (its top-level `await main()` runs the whole suite as a
  // side effect of being loaded), so the same structural shape is duplicated
  // rather than shared, and must be kept in sync by hand with that file.
  const allCodexRepoHooks2 = Object.values(codexRepo2.hooks || {})
    .flat()
    .flatMap((e) => e.hooks || [])
    .filter((h) => String(h.command || "").includes("hooks/bee-"));
  const NODE_WINDOWS_BOOTSTRAP =
    /^node -e "var cp=require\('child_process'\);var path=require\('path'\);var hook=process\.argv\[1\];var root='';try\{root=cp\.execSync\('git rev-parse --show-toplevel',\{stdio:\['ignore','pipe','ignore'\]\}\)\.toString\(\)\.trim\(\);\}catch\(e\)\{root='';\}if\(!root\)\{process\.exit\(0\);\}var target=path\.join\(root,'\.bee','bin','hooks',hook\);var r=cp\.spawnSync\(process\.execPath,\[target,'--source=repo'\],\{stdio:'inherit'\}\);if\(r\.error\)\{process\.exit\(1\);\}process\.exit\(r\.status===null\?1:r\.status\);" bee-[a-z-]+\.mjs$/;
  const FORBIDDEN_WINDOWS_CHARS = /[$%`]/;
  const bareNodeNoShellMeta = allCodexRepoHooks2.every((h) =>
    typeof h.commandWindows === "string" &&
    NODE_WINDOWS_BOOTSTRAP.test(h.commandWindows) &&
    !FORBIDDEN_WINDOWS_CHARS.test(h.commandWindows));
  check(bareNodeNoShellMeta,
    "every bee .codex/hooks.json entry's commandWindows is the git-root-resolving \"node -e SCRIPT <script>\" bootstrap (D6), with no $, %, or backtick anywhere",
    JSON.stringify(allCodexRepoHooks2.map((h) => h.commandWindows)));

  // --- 9e. Codex user-config status line (machine-level, add-only) -----------
  // Uses its own temp repo + fake homes: the check targets ~/.codex/config.toml
  // via $HOME, which runOnboard already fakes per case.
  const cslTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-codex-sl-"));
  try {
    // (a) Codex absent: no item, and apply never creates the file.
    const cslHomeA = makeFakeHome();
    const cslPlanA = await runOnboard(["--repo-root", cslTmp, "--json"], cslHomeA);
    check(!(cslPlanA.payload?.plan || []).some((i) => i.action === "ensure_codex_statusline"),
      "no ~/.codex/config.toml -> no ensure_codex_statusline item");
    await runOnboard(["--repo-root", cslTmp, "--apply", "--json"], cslHomeA);
    check(!fs.existsSync(path.join(cslHomeA, ".codex", "config.toml")),
      "onboarding never creates ~/.codex/config.toml when Codex is absent");

    // (b) config with [tui] but no status_line: spliced under the header,
    // everything else preserved, .bak written, recheck clean.
    const cslHomeB = makeFakeHome();
    fs.mkdirSync(path.join(cslHomeB, ".codex"), { recursive: true });
    fs.writeFileSync(path.join(cslHomeB, ".codex", "config.toml"),
      'model = "gpt-x"\n\n[tui]\ntheme = "dark"\n\n[projects."/x"]\ntrust_level = "trusted"\n',
      "utf8");
    const cslPlanB = await runOnboard(["--repo-root", cslTmp, "--json"], cslHomeB);
    check((cslPlanB.payload?.plan || []).some((i) => i.action === "ensure_codex_statusline"),
      "config.toml without status_line -> ensure_codex_statusline planned");
    await runOnboard(["--repo-root", cslTmp, "--apply", "--json"], cslHomeB);
    const cslCfgB = fs.readFileSync(path.join(cslHomeB, ".codex", "config.toml"), "utf8");
    check(/\[tui\]\nstatus_line = \["current-dir"/.test(cslCfgB),
      "status_line spliced directly under the existing [tui] header");
    check(cslCfgB.includes("status_line_use_colors = true"),
      "status_line_use_colors rides the same splice");
    check(cslCfgB.includes('theme = "dark"') && cslCfgB.includes('trust_level = "trusted"') &&
      cslCfgB.includes('model = "gpt-x"'),
      "existing config content preserved around the splice");
    check(fs.existsSync(path.join(cslHomeB, ".codex", "config.toml.bak")),
      "config.toml.bak written before the splice");
    const cslPlanB2 = await runOnboard(["--repo-root", cslTmp, "--json"], cslHomeB);
    check(!(cslPlanB2.payload?.plan || []).some((i) => i.action === "ensure_codex_statusline"),
      "recheck after apply plans no further statusline item");

    // (c) config without a [tui] section (and no trailing newline): appended.
    const cslHomeC = makeFakeHome();
    fs.mkdirSync(path.join(cslHomeC, ".codex"), { recursive: true });
    fs.writeFileSync(path.join(cslHomeC, ".codex", "config.toml"), 'model = "gpt-x"', "utf8");
    await runOnboard(["--repo-root", cslTmp, "--apply", "--json"], cslHomeC);
    const cslCfgC = fs.readFileSync(path.join(cslHomeC, ".codex", "config.toml"), "utf8");
    check(cslCfgC.startsWith('model = "gpt-x"') && /\n\[tui\]\nstatus_line = \[/.test(cslCfgC),
      "[tui] section appended when absent, even after a no-trailing-newline file");

    // (d) a present status_line — custom segments — is preference, not drift.
    const cslHomeD = makeFakeHome();
    fs.mkdirSync(path.join(cslHomeD, ".codex"), { recursive: true });
    const cslCustomCfg = '[tui]\nstatus_line = ["model"]\n';
    fs.writeFileSync(path.join(cslHomeD, ".codex", "config.toml"), cslCustomCfg, "utf8");
    const cslPlanD = await runOnboard(["--repo-root", cslTmp, "--json"], cslHomeD);
    check(!(cslPlanD.payload?.plan || []).some((i) => i.action === "ensure_codex_statusline"),
      "custom status_line present -> no item planned");
    await runOnboard(["--repo-root", cslTmp, "--apply", "--json"], cslHomeD);
    check(fs.readFileSync(path.join(cslHomeD, ".codex", "config.toml"), "utf8") === cslCustomCfg,
      "custom status_line left untouched byte-for-byte by apply");
  } finally {
    fs.rmSync(cslTmp, { recursive: true, force: true });
  }
  check(fs.existsSync(`${codexRepoPath}.bak`),
    ".codex/hooks.json.bak backup created when merging into an existing file");
  check(fs.readFileSync(path.join(tmp, "docs", "specs", "reading-map.md"), "utf8")
    .startsWith("# My map"),
    "existing docs/specs/reading-map.md is never overwritten by a re-apply");

  // --claude-md: fresh repo -> created with header + bare import.
  const cmTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-claudemd-test-"));
  const cmHome = makeFakeHome();
  try {
    await runOnboard(["--repo-root", cmTmp, "--apply", "--claude-md", "--json"], cmHome);
    const created = fs.readFileSync(path.join(cmTmp, "CLAUDE.md"), "utf8");
    check(created.startsWith("# Project Rules"), "--claude-md creates CLAUDE.md with header");
    check(/^@AGENTS\.md\s*$/m.test(created), "created CLAUDE.md carries a bare @AGENTS.md import");
    const cmRecheck = await runOnboard(["--repo-root", cmTmp, "--claude-md", "--json"], cmHome);
    check(cmRecheck.payload && cmRecheck.payload.status === "up_to_date",
      "--claude-md recheck up_to_date");

    // existing CLAUDE.md without the import -> appended, user content preserved.
    fs.writeFileSync(path.join(cmTmp, "CLAUDE.md"), "# My rules\n\nDo X.\n", "utf8");
    await runOnboard(["--repo-root", cmTmp, "--apply", "--claude-md", "--json"], cmHome);
    const appended = fs.readFileSync(path.join(cmTmp, "CLAUDE.md"), "utf8");
    check(appended.startsWith("# My rules"), "--claude-md preserves existing CLAUDE.md content");
    check(/^@AGENTS\.md\s*$/m.test(appended), "--claude-md appends the import to existing CLAUDE.md");
    const importCount = appended.split("@AGENTS.md").length - 1;
    check(importCount === 1, "no duplicate @AGENTS.md import", `count: ${importCount}`);
  } finally {
    try {
      fs.rmSync(cmTmp, { recursive: true, force: true });
      fs.rmSync(cmHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 9b-default. CLAUDE.md is a default onboarding artifact (D1): no flag
  // needed -> fresh repo plans/creates it; existing CLAUDE.md without the
  // import gets the append; existing CLAUDE.md WITH the import is untouched.
  const cmDefaultTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-claudemd-default-test-"));
  const cmDefaultHome = makeFakeHome();
  try {
    const defaultPlan = await runOnboard(["--repo-root", cmDefaultTmp, "--json"], cmDefaultHome);
    const defaultPlanActions = (defaultPlan.payload?.plan || []).map((i) => i.action);
    check(defaultPlanActions.includes("create_claude_md"),
      "default (flag omitted): plan contains create_claude_md for a repo with no CLAUDE.md",
      JSON.stringify(defaultPlanActions));

    await runOnboard(["--repo-root", cmDefaultTmp, "--apply", "--json"], cmDefaultHome);
    const defaultCreated = fs.readFileSync(path.join(cmDefaultTmp, "CLAUDE.md"), "utf8");
    check(defaultCreated.startsWith("# Project Rules"),
      "default: apply creates CLAUDE.md with header");
    check(/^@AGENTS\.md\s*$/m.test(defaultCreated),
      "default: created CLAUDE.md carries a bare @AGENTS.md import");

    // Existing CLAUDE.md WITH the import already -> no CLAUDE.md plan item.
    const withImportPlan = await runOnboard(["--repo-root", cmDefaultTmp, "--json"], cmDefaultHome);
    const withImportActions = (withImportPlan.payload?.plan || []).map((i) => i.action);
    check(!withImportActions.includes("create_claude_md") &&
      !withImportActions.includes("append_claude_md_import"),
      "default: existing CLAUDE.md with the @AGENTS.md import produces no CLAUDE.md plan item",
      JSON.stringify(withImportActions));

    // Existing CLAUDE.md WITHOUT the import -> append_claude_md_import.
    fs.writeFileSync(path.join(cmDefaultTmp, "CLAUDE.md"), "# House rules\n\nDo Y.\n", "utf8");
    const noImportPlan = await runOnboard(["--repo-root", cmDefaultTmp, "--json"], cmDefaultHome);
    const noImportActions = (noImportPlan.payload?.plan || []).map((i) => i.action);
    check(noImportActions.includes("append_claude_md_import"),
      "default: existing CLAUDE.md without the @AGENTS.md import plans append_claude_md_import",
      JSON.stringify(noImportActions));
    await runOnboard(["--repo-root", cmDefaultTmp, "--apply", "--json"], cmDefaultHome);
    const defaultAppended = fs.readFileSync(path.join(cmDefaultTmp, "CLAUDE.md"), "utf8");
    check(defaultAppended.startsWith("# House rules"),
      "default: append preserves existing CLAUDE.md content");
    check(/^@AGENTS\.md\s*$/m.test(defaultAppended),
      "default: append adds the @AGENTS.md import");
  } finally {
    try {
      fs.rmSync(cmDefaultTmp, { recursive: true, force: true });
      fs.rmSync(cmDefaultHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 9b-no-claude-md. --no-claude-md suppresses every CLAUDE.md plan item --
  const cmOptOutTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-claudemd-optout-test-"));
  const cmOptOutHome = makeFakeHome();
  try {
    const optOutPlan = await runOnboard(["--repo-root", cmOptOutTmp, "--no-claude-md", "--json"], cmOptOutHome);
    const optOutActions = (optOutPlan.payload?.plan || []).map((i) => i.action);
    check(!optOutActions.includes("create_claude_md") &&
      !optOutActions.includes("append_claude_md_import"),
      "--no-claude-md: plan carries no CLAUDE.md items for a fresh repo",
      JSON.stringify(optOutActions));
    await runOnboard(["--repo-root", cmOptOutTmp, "--apply", "--no-claude-md", "--json"], cmOptOutHome);
    check(!fs.existsSync(path.join(cmOptOutTmp, "CLAUDE.md")),
      "--no-claude-md: apply never creates CLAUDE.md");
  } finally {
    try {
      fs.rmSync(cmOptOutTmp, { recursive: true, force: true });
      fs.rmSync(cmOptOutHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 9c. statusline opt-in vendor -----------------------------------------
  // The pair packages/bee/statusline/{statusline-command.sh,statusline-usage.mjs}
  // is synced ONLY into repos whose .claude/settings.json already points
  // statusLine at .claude/statusline-command.sh. Onboarding never creates the
  // opt-in and never mutates settings.json in this stage.
  const SL_NAMES = ["statusline-command.sh", "statusline-usage.mjs"];
  const SL_TEMPLATES_DIR = path.join(TEMPLATES_DIR, "statusline");

  // opted-in repo: settings entry present BEFORE first onboard
  const slTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-statusline-test-"));
  const slHome = makeFakeHome();
  try {
    fs.mkdirSync(path.join(slTmp, ".claude"), { recursive: true });
    const slSettingsPath = path.join(slTmp, ".claude", "settings.json");
    const slSettingsText = `${JSON.stringify({
      statusLine: {
        type: "command",
        command: 'bash "${CLAUDE_PROJECT_DIR:-.}/.claude/statusline-command.sh"',
      },
      permissions: { allow: ["Bash(ls:*)"] },
    }, null, 2)}\n`;
    fs.writeFileSync(slSettingsPath, slSettingsText, "utf8");

    const slPlan = await runOnboard(["--repo-root", slTmp, "--json"], slHome);
    const slPlanItems = (slPlan.payload?.plan || []).filter((i) => i.action === "copy_statusline");
    check(slPlanItems.length === SL_NAMES.length &&
      SL_NAMES.every((n) => slPlanItems.some((i) => i.path === `.claude/${n}`)),
      "opted-in repo: plan carries one copy_statusline item per missing pair file",
      JSON.stringify(slPlanItems));

    const slApply = await runOnboard(["--repo-root", slTmp, "--apply", "--json"], slHome);
    check(slApply.payload?.status === "applied", "opted-in apply succeeds");
    check(slApply.payload?.recheck === "up_to_date", "opted-in recheck up_to_date",
      JSON.stringify(slApply.payload?.recheck_plan || []));
    for (const n of SL_NAMES) {
      const vendoredPath = path.join(slTmp, ".claude", n);
      const vendored = fs.existsSync(vendoredPath) ? fs.readFileSync(vendoredPath, "utf8") : null;
      const template = fs.readFileSync(path.join(SL_TEMPLATES_DIR, n), "utf8");
      check(vendored === template, `.claude/${n} vendored byte-identical to template`);
    }
    check(fs.readFileSync(slSettingsPath, "utf8") === slSettingsText,
      "settings.json byte-untouched by the statusline stage");
    check(!fs.existsSync(`${slSettingsPath}.bak`),
      "no settings.json.bak from the statusline stage");
    const slOnboarding = JSON.parse(
      fs.readFileSync(path.join(slTmp, ".bee", "onboarding.json"), "utf8"));
    check(SL_NAMES.every((n) => typeof slOnboarding.managed?.statusline?.[n] === "string"),
      "onboarding.json managed.statusline records a hash per pair file",
      JSON.stringify(slOnboarding.managed?.statusline || null));

    // drift exactly one file -> exactly one item, apply heals, recheck clean
    fs.appendFileSync(path.join(slTmp, ".claude", "statusline-command.sh"), "# local drift\n");
    const slDriftPlan = await runOnboard(["--repo-root", slTmp, "--json"], slHome);
    const slDriftItems = (slDriftPlan.payload?.plan || []).filter((i) => i.action === "copy_statusline");
    check(slDriftItems.length === 1 && slDriftItems[0].path === ".claude/statusline-command.sh",
      "drifted pair file: exactly the drifted file is re-planned",
      JSON.stringify(slDriftItems));
    const slHeal = await runOnboard(["--repo-root", slTmp, "--apply", "--json"], slHome);
    check(slHeal.payload?.recheck === "up_to_date", "drift healed: recheck up_to_date");
  } finally {
    try {
      fs.rmSync(slTmp, { recursive: true, force: true });
      fs.rmSync(slHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // non-opted variants: user-level statusLine path, unparseable settings,
  // non-string command. None throws, none plans or writes the pair, and the
  // managed manifest never grows a statusline key.
  const slCases = [
    ["statusline points at the user-level script", JSON.stringify({
      statusLine: { type: "command", command: "bash /home/someone/.claude/statusline-command.sh" },
    })],
    ["settings.json is unparseable", "{ not json"],
    ["statusLine.command is not a string", JSON.stringify({ statusLine: { command: 42 } })],
    // review P2-1: CLAUDE_PROJECT_DIR appearing anywhere must not opt in when
    // the script path itself is user-level.
    ["CLAUDE_PROJECT_DIR present but the script path is user-level", JSON.stringify({
      statusLine: {
        type: "command",
        command: 'test -n "$CLAUDE_PROJECT_DIR" && bash ~/.claude/statusline-command.sh',
      },
    })],
  ];
  for (const [why, settingsText] of slCases) {
    const noTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-statusline-no-"));
    const noHome = makeFakeHome();
    try {
      fs.mkdirSync(path.join(noTmp, ".claude"), { recursive: true });
      fs.writeFileSync(path.join(noTmp, ".claude", "settings.json"), settingsText, "utf8");
      const noApply = await runOnboard(["--repo-root", noTmp, "--apply", "--json"], noHome);
      check(noApply.payload?.status === "applied" &&
        !(noApply.payload?.applied || []).some((i) => i.action === "copy_statusline"),
        `non-opted (${why}): apply succeeds with zero copy_statusline items`,
        JSON.stringify((noApply.payload?.applied || []).map((i) => i.action)));
      check(SL_NAMES.every((n) => !fs.existsSync(path.join(noTmp, ".claude", n))),
        `non-opted (${why}): no pair file created`);
      const noOnboarding = JSON.parse(
        fs.readFileSync(path.join(noTmp, ".bee", "onboarding.json"), "utf8"));
      check(!("statusline" in (noOnboarding.managed || {})),
        `non-opted (${why}): managed manifest carries no statusline key`);
      const noRecheck = await runOnboard(["--repo-root", noTmp, "--json"], noHome);
      check(noRecheck.payload?.status === "up_to_date",
        `non-opted (${why}): recheck up_to_date`);
    } finally {
      try {
        fs.rmSync(noTmp, { recursive: true, force: true });
        fs.rmSync(noHome, { recursive: true, force: true });
      } catch {
        // best-effort cleanup
      }
    }
  }

  // --- 9d. .gitignore managed block (D1, footprint-1) -------------------------
  // Mirrors the AGENTS.md marker-splice idiom (section 7 above) but with
  // '#'-comment markers and a fixed D1 pattern list. Fresh repo -> create;
  // idempotent re-apply; onboarding.json carries a gitignore_block hash;
  // tampered block detected + restored, user content preserved byte-for-byte.
  const giTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-test-"));
  const giHome = makeFakeHome();
  try {
    const giPlan1 = await runOnboard(["--repo-root", giTmp, "--json"], giHome);
    const giPlan1Actions = (giPlan1.payload?.plan || []).map((i) => i.action);
    check(giPlan1Actions.includes("create_gitignore_block"),
      "fresh repo plans create_gitignore_block", JSON.stringify(giPlan1Actions));
    check(!fs.existsSync(path.join(giTmp, ".gitignore")), "plan mode writes no .gitignore");

    const giApply1 = await runOnboard(["--repo-root", giTmp, "--apply", "--json"], giHome);
    check(giApply1.payload?.status === "applied", "apply on fresh repo succeeds");
    check(giApply1.payload?.recheck === "up_to_date", "fresh apply recheck up_to_date",
      JSON.stringify(giApply1.payload?.recheck_plan || []));

    const giText1 = fs.readFileSync(path.join(giTmp, ".gitignore"), "utf8");
    check(giText1.includes("# BEE:START") && giText1.includes("# BEE:END"),
      ".gitignore contains # BEE:START / # BEE:END markers (gitignore-comment syntax)");
    check(!giText1.includes("<!--"), ".gitignore markers are never HTML comments");
    for (const pattern of [
      ".bee/state.json",
      ".bee/reservations.json",
      ".bee/workers/",
      ".bee/logs/",
      ".bee/capture-queue.jsonl",
      ".bee/feedback-digest.json",
      ".bee/.inject-cache.json",
      ".bee/HANDOFF.json",
      ".bee/spikes/",
    ]) {
      check(giText1.includes(pattern), `.gitignore block includes ${pattern}`);
    }
    for (const teamDurable of [
      ".bee/bin",
      ".bee/config.json",
      ".bee/config-sample.json",
      ".bee/onboarding.json",
      ".bee/decisions.jsonl",
      ".bee/backlog.jsonl",
      ".bee/cells",
    ]) {
      check(!giText1.includes(teamDurable),
        `.gitignore block never ignores team-durable path ${teamDurable}`);
    }

    const giApply2 = await runOnboard(["--repo-root", giTmp, "--apply", "--json"], giHome);
    const giText2 = fs.readFileSync(path.join(giTmp, ".gitignore"), "utf8");
    check(giText2 === giText1, "second apply on a clean .gitignore is byte-identical (idempotent)");
    check(giApply2.payload?.recheck === "up_to_date", "second apply recheck up_to_date");

    const giOnboarding = JSON.parse(
      fs.readFileSync(path.join(giTmp, ".bee", "onboarding.json"), "utf8"));
    check(typeof giOnboarding.managed?.gitignore_block === "string" &&
      giOnboarding.managed.gitignore_block.length === 64,
      "onboarding.json managed.gitignore_block records a sha256 hex hash",
      JSON.stringify(giOnboarding.managed?.gitignore_block || null));
    check(giOnboarding.managed?.gitignore_block !== giOnboarding.managed?.agents_block,
      "gitignore_block hash is distinct from agents_block (computed from its own template content)");

    // tamper the block body -> detected + restored, user content preserved.
    // The footer carries EXTRA trailing blank lines on purpose (review P2:
    // the old `${updated.replace(/\s*$/, "")}\n` normalized the whole file's
    // trailing whitespace, which would silently eat these) -- exact equality
    // below catches that regression, not just substring presence.
    const giUserHeader = "node_modules/\ndist/\n";
    const giUserFooter = "\n*.local\n\n\n";
    const giTampered = giText1.replace(
      /# BEE:START[\s\S]*?# BEE:END/,
      "# BEE:START\nTAMPERED\n# BEE:END",
    );
    fs.writeFileSync(path.join(giTmp, ".gitignore"), giUserHeader + giTampered + giUserFooter, "utf8");

    const giPlan3 = await runOnboard(["--repo-root", giTmp, "--json"], giHome);
    check(giPlan3.payload?.status === "changes_needed",
      "tampered gitignore block detected as changes_needed");
    check((giPlan3.payload?.plan || []).some((i) => i.action === "update_gitignore_block"),
      "plan includes update_gitignore_block", JSON.stringify(giPlan3.payload?.plan || []));

    const giApply3 = await runOnboard(["--repo-root", giTmp, "--apply", "--json"], giHome);
    check(giApply3.payload?.status === "applied", "re-apply after gitignore tamper succeeds");
    const giRestored = fs.readFileSync(path.join(giTmp, ".gitignore"), "utf8");
    const giExpectedRestored = giUserHeader + giText1 + giUserFooter;
    check(giRestored === giExpectedRestored,
      "user header AND footer bytes preserved EXACTLY (full equality, not substring) around the restored block -- includes extra trailing blank lines the old whole-file trim would have eaten",
      JSON.stringify({ giRestored, giExpectedRestored }));
    check(!giRestored.includes("TAMPERED"), "tampered gitignore block content restored");
    check(giRestored.indexOf("# BEE:START") === giRestored.lastIndexOf("# BEE:START"),
      "exactly one gitignore BEE block after re-apply");

    const giApply4 = await runOnboard(["--repo-root", giTmp, "--apply", "--json"], giHome);
    const giAfterFourth = fs.readFileSync(path.join(giTmp, ".gitignore"), "utf8");
    check(giAfterFourth === giRestored, "next apply after restore is byte-identical (idempotent)");
    check(giApply4.payload?.recheck === "up_to_date", "post-restore recheck up_to_date");
  } finally {
    try {
      fs.rmSync(giTmp, { recursive: true, force: true });
      fs.rmSync(giHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 9e. .gitignore append case: pre-existing file WITHOUT markers and
  // WITHOUT a trailing newline - the exact corrupt-merge bug class this
  // feature fixes (D1/D3 origin: `.bee/feedback-digest.json.spikes/` merged
  // onto one line because no separator was ever inserted). -------------------
  const giAppendTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-append-"));
  const giAppendHome = makeFakeHome();
  try {
    fs.writeFileSync(path.join(giAppendTmp, ".gitignore"), "node_modules/\ndist/", "utf8"); // no trailing \n
    const giAppendPlan = await runOnboard(["--repo-root", giAppendTmp, "--json"], giAppendHome);
    check((giAppendPlan.payload?.plan || []).some((i) => i.action === "append_gitignore_block"),
      ".gitignore without markers plans append_gitignore_block",
      JSON.stringify(giAppendPlan.payload?.plan || []));

    await runOnboard(["--repo-root", giAppendTmp, "--apply", "--json"], giAppendHome);
    const giAppended = fs.readFileSync(path.join(giAppendTmp, ".gitignore"), "utf8");
    check(giAppended.includes("node_modules/\ndist/"),
      "pre-existing lines survive untouched", JSON.stringify(giAppended));
    check(!giAppended.includes("dist/#") && !giAppended.includes("dist/.bee"),
      "no trailing-newline corruption: dist/ never merges onto one line with the appended block",
      JSON.stringify(giAppended));
    check(giAppended.includes("# BEE:START") && giAppended.includes("# BEE:END"),
      "appended .gitignore contains the BEE markers");

    const giAppendRecheck = await runOnboard(["--repo-root", giAppendTmp, "--json"], giAppendHome);
    check(giAppendRecheck.payload?.status === "up_to_date", "appended gitignore recheck up_to_date");
  } finally {
    try {
      fs.rmSync(giAppendTmp, { recursive: true, force: true });
      fs.rmSync(giAppendHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 9f. gitignore review fixes (footprint-4): hash tie, marker-lookalike,
  // CRLF drift, and the tracked-paths advisory --------------------------------

  function runGit(cwd, args) {
    return spawnSync("git", args, { cwd, encoding: "utf8" });
  }
  const gitAvailable = spawnSync("git", ["--version"]).status === 0;

  // (i) managed.gitignore_block ties to an INDEPENDENTLY reconstructed sha256
  // of the rendered block source, not just "any 64-char hex string" (review
  // P3 test-coverage).
  const GITIGNORE_PATTERNS_FOR_HASH = [
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
    ".bee/sessions/",
    ".bee/claims/",
    ".bee/runtime/",
    ".bee/cache/",
    ".bee/intent/",
    ".bee/locks/",
    ".bee/doctor-attest.json",
    ".bee/native-transport-probe.json",
    ".bee/config.local.json",
    ".bee/tmp/",
    ".bee/backups/",
    "*.[0-9]*-*-*.tmp",
    ".claude-plugin/skills.tmp-*/",
    ".claude-plugin/skills.old-*/",
    ".codex-plugin/skills.tmp-*/",
    ".codex-plugin/skills.old-*/",
    ".bee/tmp_*",
    ".bee/patch-*.json",
  ];
  const expectedGitignoreBlockSource =
    `# BEE:START\n${GITIGNORE_PATTERNS_FOR_HASH.join("\n")}\n# BEE:END\n`;
  const expectedGitignoreHash = crypto
    .createHash("sha256")
    .update(expectedGitignoreBlockSource)
    .digest("hex");

  const hashTieTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-hash-"));
  const hashTieHome = makeFakeHome();
  try {
    await runOnboard(["--repo-root", hashTieTmp, "--apply", "--json"], hashTieHome);
    const hashTieOnboarding = JSON.parse(
      fs.readFileSync(path.join(hashTieTmp, ".bee", "onboarding.json"), "utf8"));
    check(hashTieOnboarding.managed?.gitignore_block === expectedGitignoreHash,
      "managed.gitignore_block ties to an independently-reconstructed sha256 of the true rendered block source",
      `${hashTieOnboarding.managed?.gitignore_block} !== ${expectedGitignoreHash}`);
  } finally {
    try {
      fs.rmSync(hashTieTmp, { recursive: true, force: true });
      fs.rmSync(hashTieHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // (ii) marker-lookalike: a line containing the marker text as a substring
  // (trailing prose after START) must never be adopted as the managed block
  // -> treated as absent -> append (never update), and the user's original
  // lines are never deleted (review P2/P3 security-anchor).
  const lookalikeTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-lookalike-"));
  const lookalikeHome = makeFakeHome();
  try {
    const lookalikeContent = "# BEE:START custom notes\nkeep-me/\n# BEE:END\n";
    fs.writeFileSync(path.join(lookalikeTmp, ".gitignore"), lookalikeContent, "utf8");

    const lookalikePlan = await runOnboard(["--repo-root", lookalikeTmp, "--json"], lookalikeHome);
    const lookalikeActions = (lookalikePlan.payload?.plan || []).map((i) => i.action);
    check(lookalikeActions.includes("append_gitignore_block"),
      "marker-lookalike (trailing prose after START) reads as absent -> append",
      JSON.stringify(lookalikeActions));
    check(!lookalikeActions.includes("update_gitignore_block"),
      "marker-lookalike is never mistaken for an already-present block to update",
      JSON.stringify(lookalikeActions));

    await runOnboard(["--repo-root", lookalikeTmp, "--apply", "--json"], lookalikeHome);
    const lookalikeApplied = fs.readFileSync(path.join(lookalikeTmp, ".gitignore"), "utf8");
    check(lookalikeApplied.includes("# BEE:START custom notes"),
      "the fake marker-lookalike line is never deleted", JSON.stringify(lookalikeApplied));
    check(lookalikeApplied.includes("keep-me/"),
      "user content under the fake marker-lookalike is never deleted", JSON.stringify(lookalikeApplied));
    check(lookalikeApplied.includes("\n# BEE:START\n") && lookalikeApplied.includes("\n# BEE:END\n"),
      "a real managed block is appended alongside the untouched fake one",
      JSON.stringify(lookalikeApplied));
  } finally {
    try {
      fs.rmSync(lookalikeTmp, { recursive: true, force: true });
      fs.rmSync(lookalikeHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // (iii) CRLF-saved block reads up_to_date: a CRLF-normalized (content-
  // identical) block must never cause a perpetual update_gitignore_block loop
  // (review P3 CRLF). Writes still stay LF -- this only relaxes the compare.
  const crlfTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-crlf-"));
  const crlfHome = makeFakeHome();
  try {
    await runOnboard(["--repo-root", crlfTmp, "--apply", "--json"], crlfHome);
    const crlfOriginal = fs.readFileSync(path.join(crlfTmp, ".gitignore"), "utf8");
    fs.writeFileSync(path.join(crlfTmp, ".gitignore"), crlfOriginal.replace(/\n/g, "\r\n"), "utf8");

    const crlfPlan = await runOnboard(["--repo-root", crlfTmp, "--json"], crlfHome);
    check(crlfPlan.payload?.status === "up_to_date",
      "a CRLF-saved (content-identical) gitignore block reads up_to_date, no perpetual update loop",
      JSON.stringify(crlfPlan.payload?.plan || []));
  } finally {
    try {
      fs.rmSync(crlfTmp, { recursive: true, force: true });
      fs.rmSync(crlfHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // (iv) tracked-paths advisory: a git repo with a tracked managed path emits
  // the notice with the exact untrack command, in both plan mode and the
  // post-apply recheck; the advisory never actually runs git rm itself
  // (review P2 code-quality).
  if (gitAvailable) {
    const trackedTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-tracked-"));
    const trackedHome = makeFakeHome();
    try {
      runGit(trackedTmp, ["init", "-q"]);
      fs.mkdirSync(path.join(trackedTmp, ".bee"), { recursive: true });
      fs.writeFileSync(path.join(trackedTmp, ".bee", "state.json"), "{}\n", "utf8");
      const addResult = runGit(trackedTmp, ["add", ".bee/state.json"]);
      check(addResult.status === 0, "git add .bee/state.json succeeds in the fixture repo",
        addResult.stderr);

      const trackedPlan = await runOnboard(["--repo-root", trackedTmp, "--json"], trackedHome);
      const trackedNotices = trackedPlan.payload?.notices || [];
      check(
        trackedNotices.some(
          (n) => n.includes("git-tracked") && n.includes(".bee/state.json") &&
            n.includes("git rm -r --cached"),
        ),
        "plan mode notices carry the tracked-paths advisory with the exact untrack command",
        JSON.stringify(trackedNotices),
      );

      const trackedApply = await runOnboard(["--repo-root", trackedTmp, "--apply", "--json"], trackedHome);
      const trackedApplyNotices = trackedApply.payload?.notices || [];
      check(trackedApplyNotices.some((n) => n.includes("git-tracked")),
        "post-apply recheck notices also carry the tracked-paths advisory (the ignore block alone can't silence an already-tracked file)",
        JSON.stringify(trackedApplyNotices));

      const stillTracked = runGit(trackedTmp, ["show", ":.bee/state.json"]);
      check(stillTracked.status === 0,
        "the advisory never actually ran git rm -- the file is still tracked in the index",
        JSON.stringify(stillTracked));
    } finally {
      try {
        fs.rmSync(trackedTmp, { recursive: true, force: true });
        fs.rmSync(trackedHome, { recursive: true, force: true });
      } catch {
        // best-effort cleanup
      }
    }
  } else {
    skip("tracked-paths advisory (git repo fixture)", "git binary not available");
  }

  // non-git dir -> no notice, no crash (graceful degradation).
  const nonGitTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-nongit-"));
  const nonGitHome = makeFakeHome();
  try {
    const nonGitPlan = await runOnboard(["--repo-root", nonGitTmp, "--json"], nonGitHome);
    check(nonGitPlan.status === 0, "non-git repo root: onboard never crashes", nonGitPlan.stderr);
    check(!(nonGitPlan.payload?.notices || []).some((n) => n.includes("git-tracked")),
      "non-git repo root: no tracked-paths notice is ever emitted",
      JSON.stringify(nonGitPlan.payload?.notices || []));
  } finally {
    try {
      fs.rmSync(nonGitTmp, { recursive: true, force: true });
      fs.rmSync(nonGitHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // --- 9g. crash-leak scratch ignore patterns (tree-hygiene D3, th-5):
  // .bee/tmp/, .bee/backups/, the atomic-tmp SHAPE (never a bare '*.tmp'),
  // and the four plugin swap-dir patterns join the managed block. Shape-only
  // matching is asserted in BOTH directions via `git check-ignore` so a
  // human's own hand-made notes.tmp never disappears from git status. -------
  const scratchGiTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-gitignore-scratch-"));
  const scratchGiHome = makeFakeHome();
  try {
    await runOnboard(["--repo-root", scratchGiTmp, "--apply", "--json"], scratchGiHome);
    const scratchGiText = fs.readFileSync(path.join(scratchGiTmp, ".gitignore"), "utf8");
    for (const pattern of [
      ".bee/tmp/",
      ".bee/backups/",
      ".claude-plugin/skills.tmp-*/",
      ".claude-plugin/skills.old-*/",
      ".codex-plugin/skills.tmp-*/",
      ".codex-plugin/skills.old-*/",
      // Legacy-flow debris (bee-gitignore-debris gi-1): pre-`--evidence-stdin`
      // verify/evidence/deviations scratch and one-off patch files that used
      // to land tracked at the .bee root (six such files removed by gi-1).
      ".bee/tmp_*",
      ".bee/patch-*.json",
    ]) {
      check(scratchGiText.includes(pattern), `.gitignore block includes crash-leak pattern ${pattern}`,
        scratchGiText);
    }
    check(
      !scratchGiText.split("\n").some((line) => line.trim() === "*.tmp"),
      ".gitignore block never adds a greedy bare *.tmp rule (D3 prohibition: a human's own notes.tmp must stay visible)",
      scratchGiText,
    );

    if (gitAvailable) {
      runGit(scratchGiTmp, ["init", "-q"]);
      fs.writeFileSync(path.join(scratchGiTmp, "notes.tmp"), "human scratch note\n", "utf8");
      fs.writeFileSync(path.join(scratchGiTmp, "x.json.12345-0-abcdef.tmp"), "{}\n", "utf8");
      const checkIgnoreHuman = runGit(scratchGiTmp, ["check-ignore", "-q", "notes.tmp"]);
      check(checkIgnoreHuman.status !== 0,
        "a hand-made notes.tmp is NEVER matched by the atomic-tmp shape pattern -- stays visible to git status",
        JSON.stringify(checkIgnoreHuman));
      const checkIgnoreAtomic = runGit(scratchGiTmp, ["check-ignore", "-q", "x.json.12345-0-abcdef.tmp"]);
      check(checkIgnoreAtomic.status === 0,
        "an atomic-tmp-SHAPED file (<pid>-<counter>-<random>.tmp) IS matched by the managed ignore pattern",
        JSON.stringify(checkIgnoreAtomic));

      // Legacy-flow debris shapes (bee-gitignore-debris gi-1): a fresh
      // '.bee/tmp_foo.txt' or '.bee/patch-cz1.json' must be untrack-able the
      // moment the managed block lands, so a re-onboarded host never lets
      // this class of scratch reappear as tracked git-status noise.
      fs.mkdirSync(path.join(scratchGiTmp, ".bee"), { recursive: true });
      fs.writeFileSync(path.join(scratchGiTmp, ".bee", "tmp_foo.txt"), "scratch\n", "utf8");
      fs.writeFileSync(path.join(scratchGiTmp, ".bee", "patch-cz1.json"), "{}\n", "utf8");
      const checkIgnoreTmpDebris = runGit(scratchGiTmp, ["check-ignore", "-q", ".bee/tmp_foo.txt"]);
      check(checkIgnoreTmpDebris.status === 0,
        "a .bee/tmp_* legacy-flow scratch file IS matched by the managed ignore pattern",
        JSON.stringify(checkIgnoreTmpDebris));
      const checkIgnorePatchDebris = runGit(scratchGiTmp, ["check-ignore", "-q", ".bee/patch-cz1.json"]);
      check(checkIgnorePatchDebris.status === 0,
        "a .bee/patch-*.json one-off patch file IS matched by the managed ignore pattern",
        JSON.stringify(checkIgnorePatchDebris));
    } else {
      skip("atomic-tmp shape ignore (git check-ignore)", "git binary not available");
    }
  } finally {
    try {
      fs.rmSync(scratchGiTmp, { recursive: true, force: true });
      fs.rmSync(scratchGiHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
} finally {
  try {
    fs.rmSync(tmp, { recursive: true, force: true });
    fs.rmSync(tmpHome, { recursive: true, force: true });
  } catch {
    // best-effort cleanup
  }
}

// --- 9b. codex-hybrid (GH #22 P0-1) ------------------------------------------
// --plugin-source alone installs plugin skills but no hooks: codex-cli has no
// plugin-hook mechanism (capability matrix row B1), so a plugin-first-onboarded
// repo used to report itself onboarded while carrying ZERO mechanical
// enforcement for Codex sessions. --runtime codex|both (default both) now
// ALWAYS also vendors .bee/bin/hooks/ and merges .codex/hooks.json under
// --plugin-source, reusing the exact --repo-hooks codex projection. Uses the
// REAL script directly (not a version-pinned fixture), same as the plain
// --repo-hooks tests above — plugin-source's own release-identity gate
// resolves fine against the running repo's own self-consistent tree.
{
  const hybridTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-codex-hybrid-"));

  // (a) plugin-source + runtime codex -> .codex/hooks.json AND vendored
  // .bee/bin/hooks/ are written; project skill projections stay absent
  // (plugin-source's existing behavior for skills is untouched).
  {
    const repo = path.join(hybridTmp, "runtime-codex");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    const apply = await runOnboard(
      ["--repo-root", repo, "--apply", "--plugin-source", "--runtime", "codex", "--json"], home);
    check(apply.payload?.status === "applied",
      "codex-hybrid: plugin-source + runtime codex apply succeeds",
      JSON.stringify(apply.payload));
    check(fs.existsSync(path.join(repo, ".codex", "hooks.json")),
      "codex-hybrid: .codex/hooks.json is written");
    const hooksDir = path.join(repo, ".bee", "bin", "hooks");
    check(listMjs(hooksDir).length > 0,
      "codex-hybrid: .bee/bin/hooks/ is vendored");
    const codexHooksText = fs.readFileSync(path.join(repo, ".codex", "hooks.json"), "utf8");
    check(codexHooksText.includes(".bee/bin/hooks/bee-write-guard.mjs"),
      "codex-hybrid: .codex/hooks.json wires bee-write-guard.mjs");
    check(!fs.existsSync(path.join(repo, ".claude", "skills")) &&
      !fs.existsSync(path.join(repo, ".agents", "skills")),
      "codex-hybrid: plugin-source still skips project skill projections");
    check(apply.payload?.onboarding?.managed?.codex_hooks &&
      Object.keys(apply.payload.onboarding.managed.codex_hooks).length > 0,
      "codex-hybrid: onboarding.json managed set records codex_hooks",
      JSON.stringify(apply.payload?.onboarding?.managed));
    check(!apply.payload?.onboarding?.managed?.repo_hooks,
      "codex-hybrid: onboarding.json managed set never records repo_hooks (that key is --repo-hooks-only)");

    const recheck = await runOnboard(
      ["--repo-root", repo, "--plugin-source", "--runtime", "codex", "--json"], home);
    check(recheck.payload?.status === "up_to_date",
      "codex-hybrid: immediate recheck is up_to_date",
      JSON.stringify(recheck.payload?.plan || []));
  }

  // (a2) --runtime both (the default) covers codex too — same effect as
  // explicit --runtime codex, and matches install.sh's own default.
  {
    const repo = path.join(hybridTmp, "runtime-both-default");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    const apply = await runOnboard(["--repo-root", repo, "--apply", "--plugin-source", "--json"], home);
    check(apply.payload?.status === "applied",
      "codex-hybrid: plugin-source with no --runtime (default both) apply succeeds");
    check(fs.existsSync(path.join(repo, ".codex", "hooks.json")),
      "codex-hybrid: default --runtime both also writes .codex/hooks.json");
  }

  // (b) plugin-source + runtime claude -> NO codex files, NO claude repo-local
  // hook entries either — exclusivity byte-identical to plain --plugin-source
  // before --runtime existed.
  {
    const repo = path.join(hybridTmp, "runtime-claude");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    const apply = await runOnboard(
      ["--repo-root", repo, "--apply", "--plugin-source", "--runtime", "claude", "--json"], home);
    check(apply.payload?.status === "applied",
      "codex-hybrid: plugin-source + runtime claude apply succeeds");
    check(!fs.existsSync(path.join(repo, ".codex", "hooks.json")),
      "codex-hybrid: runtime claude writes no .codex/hooks.json");
    check(!fs.existsSync(path.join(repo, ".bee", "bin", "hooks")),
      "codex-hybrid: runtime claude vendors no .bee/bin/hooks/");
    check(!fs.existsSync(path.join(repo, ".claude", "settings.json")),
      "codex-hybrid: runtime claude writes no .claude/settings.json (plugin-first relies on the plugin's own Claude hooks)");
    check(!apply.payload?.onboarding?.managed?.codex_hooks &&
      !apply.payload?.onboarding?.managed?.repo_hooks,
      "codex-hybrid: runtime claude records neither codex_hooks nor repo_hooks",
      JSON.stringify(apply.payload?.onboarding?.managed));

    // (5) managed-set gating (advisor R5): a claude-only run's recheck must
    // never report codex drift — a phantom changes_needed would mean the
    // managed-set gate leaked codex_hooks expectations into a claude-only run.
    const recheck = await runOnboard(
      ["--repo-root", repo, "--plugin-source", "--runtime", "claude", "--json"], home);
    check(recheck.payload?.status === "up_to_date",
      "codex-hybrid: runtime claude reports no phantom codex drift on recheck",
      JSON.stringify(recheck.payload?.plan || []));
  }

  // (c) typed blocked apply (advisor R3): a `.codex` path occupied by a plain
  // FILE must refuse the WHOLE apply with a typed {status, reason, forceable}
  // result — never an untyped {error: ...} crash — and mutate nothing (D3
  // fail-closed: skills/doctrine must never be reported applied without the
  // hooks that make them mechanically enforced for Codex).
  {
    const repo = path.join(hybridTmp, "blocked-codex-file");
    fs.mkdirSync(repo, { recursive: true });
    fs.writeFileSync(path.join(repo, ".codex"), "not a directory\n", "utf8");
    const home = makeFakeHome();
    const before = hashTree(repo);

    const apply = await runOnboard(
      ["--repo-root", repo, "--apply", "--plugin-source", "--runtime", "codex", "--json"], home);
    check(apply.status === 1, "codex-hybrid blocked: apply exits nonzero", String(apply.status));
    check(apply.payload !== null, "codex-hybrid blocked: output is valid JSON (never an untyped crash)",
      apply.stdout || "");
    check(apply.payload?.status === "blocked",
      "codex-hybrid blocked: typed status is 'blocked'", JSON.stringify(apply.payload));
    check(typeof apply.payload?.reason === "string" &&
      /repo-copy|hybrid apply/i.test(apply.payload.reason),
      "codex-hybrid blocked: reason names repo-copy or a hybrid retry",
      String(apply.payload?.reason));
    check(apply.payload?.error === undefined,
      "codex-hybrid blocked: never falls through to the generic untyped {error} shape",
      JSON.stringify(apply.payload));
    check(hashTree(repo) === before,
      "codex-hybrid blocked: zero mutation — the whole apply refuses, not just the hook write");
  }

  fs.rmSync(hybridTmp, { recursive: true, force: true });
}

// --- 9c. sticky repo-hooks record under --plugin-source (point 6) -----------
// A repo that once recorded a full --repo-hooks install (Claude + Codex
// repo-local wiring) and is re-onboarded as --plugin-source must not have
// that record silently carried forward while the mechanism underneath it
// quietly changed shape. The record is allowed to lapse (repoHooks is always
// false under --plugin-source), but the transition is surfaced as a notice
// in the SAME run, naming what survives (codex-hybrid, when --runtime covers
// codex) and what does not (Claude repo-local entries, always).
{
  const stickyPluginTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-sticky-plugin-source-"));

  // (a) --runtime codex: the codex portion is honestly replaced by the
  // codex-hybrid projection; the notice says so.
  {
    const repo = path.join(stickyPluginTmp, "codex-covered");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    await runOnboard(["--repo-root", repo, "--apply", "--repo-hooks", "--json"], home);
    check(JSON.parse(fs.readFileSync(path.join(repo, ".bee", "onboarding.json"), "utf8"))
      .managed?.repo_hooks, "sticky/plugin-source precondition: repo_hooks recorded after --repo-hooks");

    const apply = await runOnboard(
      ["--repo-root", repo, "--apply", "--plugin-source", "--runtime", "codex", "--json"], home);
    check(apply.payload?.status === "applied",
      "sticky/plugin-source: re-onboard as plugin-source + runtime codex applies cleanly");
    const onboarding = JSON.parse(fs.readFileSync(path.join(repo, ".bee", "onboarding.json"), "utf8"));
    check(!onboarding.managed?.repo_hooks,
      "sticky/plugin-source: the stale repo_hooks record is NOT silently carried forward",
      JSON.stringify(onboarding.managed));
    check(!!onboarding.managed?.codex_hooks,
      "sticky/plugin-source: codex_hooks now records the surviving codex-hybrid coverage");
    check((apply.payload?.notices || []).some((n) => /previously opted into --repo-hooks/.test(n) &&
      /codex-hybrid/.test(n)),
      "sticky/plugin-source: a notice surfaces the transition and names codex-hybrid as the replacement",
      JSON.stringify(apply.payload?.notices));
  }

  // (b) --runtime claude: NO replacement covers Codex any more — the notice
  // says so plainly instead of staying silent about the regression.
  {
    const repo = path.join(stickyPluginTmp, "claude-only");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    await runOnboard(["--repo-root", repo, "--apply", "--repo-hooks", "--json"], home);
    const apply = await runOnboard(
      ["--repo-root", repo, "--apply", "--plugin-source", "--runtime", "claude", "--json"], home);
    check(apply.payload?.status === "applied",
      "sticky/plugin-source: re-onboard as plugin-source + runtime claude applies cleanly");
    check((apply.payload?.notices || []).some((n) => /previously opted into --repo-hooks/.test(n) &&
      /retires ALL repo-local hook entries, including Codex/.test(n)),
      "sticky/plugin-source: runtime claude's notice names the Codex coverage loss plainly",
      JSON.stringify(apply.payload?.notices));
  }

  fs.rmSync(stickyPluginTmp, { recursive: true, force: true });
}

// --- 10. skill-sync (D1-D5): safety-critical behavioral cases ---------------
// Fixture authority (F4): source authority = the EXECUTING file's own tree, so
// fake-source cases copy the launcher + its relative deps into the fake
// skills/bee-hive tree and run THAT copy; only cases about the real tree run
// the real launcher. The real ~/.claude is never read or written: every spawn
// goes through runOnboardAt's fake HOME/USERPROFILE.

const REAL_ONBOARD_SRC = fs.readFileSync(ONBOARD, "utf8");
const REAL_AGENTS_BLOCK_SRC = fs.readFileSync(
  path.join(TEMPLATES_DIR, "AGENTS.block.md"), "utf8");

function fakeStateSource(version) {
  return `export const BEE_VERSION = '${version}';\n` +
    `export const COMMAND_KEYS = ['setup', 'start', 'test', 'verify'];\n`;
}

function writeSkillFiles(skillsRoot, skill, files) {
  for (const [rel, content] of Object.entries(files)) {
    const abs = path.join(skillsRoot, skill, ...rel.split("/"));
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    fs.writeFileSync(abs, content, "utf8");
  }
}

// Build a fake bee source tree at skillsRoot whose bee-hive dir carries a REAL
// copy of the launcher + its relative deps (F4), with a controlled version.
function makeFakeSkillsRoot(skillsRoot, {
  version = "0.1.19",
  hiveDirName = "bee-hive",
  skills = { "bee-alpha": { "SKILL.md": "# alpha v1\n" } },
  stateText = null,
  claudeManifest = undefined,
  codexManifest = undefined,
} = {}) {
  const pluginRoot = path.dirname(skillsRoot);
  const hive = path.join(skillsRoot, hiveDirName);
  // D3 (packages-restructure): the real launcher resolves its vendored payload
  // PLUGIN_ROOT-relative (packages/bee), not self-relative to its own scripts
  // dir. pluginRoot here already equals what the fake launcher itself computes
  // as PLUGIN_ROOT (dirname(dirname(hive))), so the fixture's packages/bee
  // copy must live OUTSIDE the fake skills tree, as pluginRoot's sibling of
  // skillsRoot — never nested under hive/templates anymore.
  const packagesBee = path.join(pluginRoot, "packages", "bee");
  fs.mkdirSync(path.join(hive, "scripts"), { recursive: true });
  fs.mkdirSync(path.join(packagesBee, "lib"), { recursive: true });
  fs.writeFileSync(path.join(hive, "scripts", "onboard_bee.mjs"), REAL_ONBOARD_SRC, "utf8");
  // Vendor EVERY real packages/bee/lib/*.mjs into the fixture launcher, derived
  // via readdirSync — never a hand-list (crit-pattern 20260714: a curated subset
  // rots silently the moment onboard imports a new lib module, e.g. fsutil for
  // the shared hashFile). state.mjs stays version-controlled below; every other
  // module is copied verbatim so the real onboard's imports all resolve.
  for (const libName of fs.readdirSync(TEMPLATES_LIB_DIR)) {
    if (!libName.endsWith(".mjs") || libName === "state.mjs") continue;
    fs.writeFileSync(
      path.join(packagesBee, "lib", libName),
      fs.readFileSync(path.join(TEMPLATES_LIB_DIR, libName), "utf8"), "utf8");
  }
  fs.writeFileSync(
    path.join(packagesBee, "lib", "state.mjs"),
    stateText !== null ? stateText : fakeStateSource(version), "utf8");
  fs.writeFileSync(path.join(packagesBee, "AGENTS.block.md"), REAL_AGENTS_BLOCK_SRC, "utf8");
  fs.writeFileSync(path.join(hive, "SKILL.md"), "# fake bee-hive\n", "utf8");
  const writeManifest = (dirName, spec) => {
    if (spec === false) return;
    const manifestDir = path.join(pluginRoot, dirName);
    fs.mkdirSync(manifestDir, { recursive: true });
    const content = typeof spec === "string"
      ? spec
      : `${JSON.stringify(spec || { name: "bee", version }, null, 2)}\n`;
    fs.writeFileSync(path.join(manifestDir, "plugin.json"), content, "utf8");
  };
  writeManifest(".claude-plugin", claudeManifest);
  writeManifest(".codex-plugin", codexManifest);
  for (const [skill, files] of Object.entries(skills)) {
    writeSkillFiles(skillsRoot, skill, files);
  }
  return { skillsRoot, launcher: path.join(hive, "scripts", "onboard_bee.mjs") };
}

function makeInstalledSkills(fakeHome, { version = "0.1.19", stateText = null, skills = {} } = {}) {
  const root = path.join(fakeHome, ".claude", "skills");
  fs.mkdirSync(root, { recursive: true });
  if (version !== null || stateText !== null) {
    writeSkillFiles(root, "bee-hive", {
      "SKILL.md": "# installed hive\n",
      "templates/lib/state.mjs": stateText !== null ? stateText : fakeStateSource(version),
    });
  }
  for (const [skill, files] of Object.entries(skills)) {
    writeSkillFiles(root, skill, files);
  }
  return root;
}

function readInstalled(fakeHome, rel) {
  const abs = path.join(fakeHome, ".claude", "skills", ...rel.split("/"));
  return fs.existsSync(abs) ? fs.readFileSync(abs, "utf8") : null;
}

// --- per-target helpers (installer-hardening) --------------------------------
const REPO_TARGET_ROOTS = [".claude/skills", ".agents/skills"];

function readRepoTarget(repo, relRoot, rel) {
  const abs = path.join(repo, ...relRoot.split("/"), ...rel.split("/"));
  return fs.existsSync(abs) ? fs.readFileSync(abs, "utf8") : null;
}

// Seed the two managed in-repo targets with a version-marked bee-hive so a
// per-target downgrade refusal resolves all three versions numeric (forceable)
// - without this, a fresh in-repo target reads installed_skills "absent" and a
// host_helpers-driven downgrade refusal is never forceable for that target.
function seedRepoSkillTargets(repo, version) {
  for (const relRoot of REPO_TARGET_ROOTS) {
    writeSkillFiles(path.join(repo, ...relRoot.split("/")), "bee-hive", {
      "SKILL.md": "# installed hive\n",
      "templates/lib/state.mjs": fakeStateSource(version),
    });
  }
}

function skillTarget(payload, kind) {
  return (payload?.skills?.targets || []).find((t) => t.kind === kind) || null;
}

function flatSkillItems(payload) {
  return (payload?.skills?.targets || []).flatMap((t) => t.items || []);
}

// Stable full-tree digest (lstat semantics: symlinks recorded by target, never
// followed) for byte-identical / zero-mutation assertions.
function hashTree(dir) {
  if (!fs.existsSync(dir)) {
    return "ABSENT";
  }
  const lines = [];
  const walk = (d, prefix) => {
    const entries = fs.readdirSync(d, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name));
    for (const e of entries) {
      const rel = prefix ? `${prefix}/${e.name}` : e.name;
      const abs = path.join(d, e.name);
      if (e.isSymbolicLink()) {
        lines.push(`link ${rel} -> ${fs.readlinkSync(abs)}`);
      } else if (e.isDirectory()) {
        lines.push(`dir ${rel}`);
        walk(abs, rel);
      } else {
        lines.push(
          `file ${rel} ${crypto.createHash("sha256").update(fs.readFileSync(abs)).digest("hex")}`);
      }
    }
  };
  walk(dir, "");
  return lines.join("\n");
}

// --- 10a. fresh install: absent targets -> full per-target sync, no refusal --
// Default targets are the two in-repo roots; ~/.claude/skills participates
// only under --global-skills and is otherwise never written (D2/D3).
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-fresh-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha v1\n" },
        "bee-beta": { "SKILL.md": "# beta\n", "references/notes.md": "beta notes\n" },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "changes_needed",
      "fresh install: plan reports changes_needed",
      `exit ${plan.status} status ${plan.payload?.status}`);
    for (const kind of ["repo-claude", "repo-agents"]) {
      const syncSkills = (plan.payload?.plan || [])
        .filter((i) => i.action === "sync_skill" && i.target === kind)
        .map((i) => i.skill).sort();
      check(JSON.stringify(syncSkills) === JSON.stringify(["bee-alpha", "bee-beta", "bee-hive"]),
        `fresh install: plan lists sync_skill for every source bee-* skill on ${kind}`,
        JSON.stringify(syncSkills));
    }
    check(!(plan.payload?.plan || []).some((i) => i.target === "global"),
      "fresh install: no global-target item without --global-skills");
    check(!fs.existsSync(path.join(home, ".claude")),
      "fresh install: plan mode writes nothing to the fake home");
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "fresh install: absent targets proceed as fresh install, no refusal (D3)",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(apply.payload?.recheck === "up_to_date",
      "fresh install: recheck lands up_to_date on content-hash parity (D5)",
      JSON.stringify(apply.payload?.recheck_plan || []));
    const onboarding = JSON.parse(
      fs.readFileSync(path.join(repo, ".bee", "onboarding.json"), "utf8"),
    );
    check(onboarding.bee_version === "0.1.19",
      "fresh install: onboarding.json records the validated release tuple");
    check(
      fs.readFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"), "utf8") ===
        fakeStateSource("0.1.19"),
      "fresh install: vendored runtime state equals the validated release tuple",
    );
    for (const relRoot of REPO_TARGET_ROOTS) {
      check(readRepoTarget(repo, relRoot, "bee-alpha/SKILL.md") === "# alpha v1\n",
        `fresh install: bee-alpha synced byte-exact into ${relRoot}`);
      check(readRepoTarget(repo, relRoot, "bee-beta/references/notes.md") === "beta notes\n",
        `fresh install: nested skill files synced into ${relRoot}`);
      // D1 (packages-restructure): skills are instruction-only now - the
      // synced bee-hive skill dir never carries templates/lib/state.mjs. The
      // per-target release tuple now lives in the sync-owned version stamp
      // (SKILLS_VERSION_STAMP) written at the projection root instead.
      const stampRaw = readRepoTarget(repo, relRoot, ".bee-skills-version.json");
      check(
        stampRaw !== null && JSON.parse(stampRaw).version === "0.1.19",
        `fresh install: ${relRoot} projection equals the validated release tuple`,
        String(stampRaw),
      );
    }
    check(!fs.existsSync(path.join(home, ".claude")),
      "fresh install: apply without --global-skills never writes ~/.claude/skills");

    // --global-skills adds the legacy global target (D3): repo targets are
    // already in parity, so the remaining drift is exactly the global root.
    const gPlan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    const gItems = (gPlan.payload?.plan || []).filter((i) => i.action === "sync_skill");
    check(gItems.length > 0 && gItems.every((i) => i.target === "global"),
      "--global-skills: remaining sync_skill items all target the global root",
      JSON.stringify(gItems));
    const gApply = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(gApply.status === 0 && gApply.payload?.status === "applied" &&
      gApply.payload?.recheck === "up_to_date",
      "--global-skills: apply syncs the fake home global root",
      `exit ${gApply.status} status ${gApply.payload?.status}`);
    check(readInstalled(home, "bee-alpha/SKILL.md") === "# alpha v1\n" &&
      readInstalled(home, "bee-beta/references/notes.md") === "beta notes\n",
      "--global-skills: global root synced byte-exact (old behavior restored)");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10a1. authoritative source tuple refuses before any target mutation ---
// The running state marker and both package manifests are one release identity.
// Missing, unreadable, non-numeric, or unequal members are source failures,
// never a downgrade that --force-downgrade may override.
for (const scenario of [
  {
    label: "missing Claude manifest",
    options: { claudeManifest: false },
    reasonNeedle: ".claude-plugin/plugin.json",
  },
  {
    label: "unreadable Claude manifest",
    options: { claudeManifest: "{ not-json\n" },
    reasonNeedle: ".claude-plugin/plugin.json",
  },
  {
    label: "mixed numeric tuple",
    options: { codexManifest: { name: "bee", version: "0.1.20" } },
    reasonNeedle: "tuple members disagree",
  },
]) {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-release-tuple-refuse-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "source", "skills"), {
      version: "0.1.19",
      ...scenario.options,
    });
    const repo = path.join(base, "target");
    fs.mkdirSync(repo, { recursive: true });
    fs.writeFileSync(path.join(repo, "owner.txt"), "must survive\n", "utf8");
    const repoBefore = hashTree(repo);
    const homeBefore = hashTree(home);
    const apply = await runOnboardAt(
      launcher,
      ["--repo-root", repo, "--apply", "--force-downgrade", "--json"],
      home,
    );
    check(
      apply.status === 1 && apply.payload?.status === "blocked_no_source",
      `source tuple: ${scenario.label} refuses apply`,
      `exit ${apply.status} payload ${JSON.stringify(apply.payload)}`,
    );
    check(
      typeof apply.payload?.reason === "string" &&
        apply.payload.reason.includes("authoritative source release tuple") &&
        apply.payload.reason.includes(scenario.reasonNeedle),
      `source tuple: ${scenario.label} names the strict release-identity failure`,
      String(apply.payload?.reason),
    );
    check(
      hashTree(repo) === repoBefore && hashTree(home) === homeBefore,
      `source tuple: ${scenario.label} leaves repo and runtime targets byte-identical`,
    );
    check(
      apply.payload?.forced_downgrade === undefined,
      `source tuple: ${scenario.label} is never forceable`,
    );
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
    fs.rmSync(home, { recursive: true, force: true });
  }
}

// --- 10a2. drift in ONE in-repo root -> sync_skill for that root only --------
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-onedrift-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      skills: { "bee-alpha": { "SKILL.md": "# alpha v1\n" } },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    // A host repo that merely contains its own skills/ dir never trips the
    // self-onboard skip: onboarding runs from an external source.
    fs.mkdirSync(path.join(repo, "skills", "bee-decoy"), { recursive: true });
    fs.writeFileSync(path.join(repo, "skills", "bee-decoy", "SKILL.md"), "# decoy\n", "utf8");
    await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    const clean = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    check(clean.payload?.status === "up_to_date",
      "one-root drift: immediate re-run reports up_to_date (repo skills/ dir never trips self-skip)",
      JSON.stringify(clean.payload?.plan || []));
    fs.writeFileSync(
      path.join(repo, ".agents", "skills", "bee-alpha", "SKILL.md"), "# drifted\n", "utf8");
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    const driftItems = (plan.payload?.plan || []).filter((i) => i.action === "sync_skill");
    check(plan.payload?.status === "changes_needed" &&
      driftItems.length === 1 && driftItems[0].skill === "bee-alpha" &&
      driftItems[0].target === "repo-agents",
      "drift planted in ONE root yields sync_skill for exactly that root",
      JSON.stringify(driftItems));
    const heal = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(heal.payload?.status === "applied" && heal.payload?.recheck === "up_to_date",
      "one-root drift: apply heals it, recheck up_to_date");
    check(readRepoTarget(repo, ".agents/skills", "bee-alpha/SKILL.md") === "# alpha v1\n",
      "one-root drift: drifted file mirrored back to source bytes");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10a2b. .codex/hooks.json has TWO renderers; the catalog owns it in bee's repo -
// hooks/catalog.mjs (TARGETS.REPO) renders `"$r"/hooks/<script>` because bee's hooks
// LIVE in hooks/. onboard_bee.mjs's renderCodexHookEntries() is the HOST projection:
// `.bee/bin/hooks/<script>`, right for a host (no hooks/ dir there) and WRONG for bee.
// Regression guard: a repo that ships hooks/catalog.mjs must NEVER have its
// .codex/hooks.json rewritten by onboarding. This broke twice — the drift check fired,
// the file was repaired by hand, and the next self-onboard silently undid the repair.
// A host repo (no catalog) must still get the projection, or Codex runs with no guards.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-codexhooks-owner-"));
  const home = makeFakeHome();
  try {
    const repo = path.join(base, "repo");
    const { launcher } = makeFakeSkillsRoot(path.join(repo, "skills"), {
      skills: { "bee-alpha": { "SKILL.md": "# alpha\n" } },
    });
    // This repo owns the catalog, and already carries the catalog's rendering.
    fs.mkdirSync(path.join(repo, "hooks"), { recursive: true });
    fs.writeFileSync(path.join(repo, "hooks", "catalog.mjs"), "// catalog\n");
    const codexHooks = path.join(repo, ".codex", "hooks.json");
    fs.mkdirSync(path.dirname(codexHooks), { recursive: true });
    const catalogRendering = JSON.stringify(
      {
        UserPromptSubmit: [
          {
            hooks: [
              {
                type: "command",
                command: 'exec node "$r"/hooks/bee-prompt-context.mjs --source=repo',
                statusMessage: "bee: phase reminder",
              },
            ],
          },
        ],
      },
      null,
      2,
    );
    fs.writeFileSync(codexHooks, catalogRendering);

    // The DRY RUN is where the plan is visible — asserting against an --apply
    // payload's plan would be vacuous (proven: that row stayed green with the fix
    // deliberately broken, while the byte-identity rows below correctly went red).
    const dry = await runOnboardAt(launcher, ["--repo-root", repo, "--repo-hooks", "--json"], home);
    const planned = (dry.payload?.plan || []).some((p) => p.action === "merge_codex_hooks");
    check(
      !planned,
      "codex-hooks: a repo owning hooks/catalog.mjs never PLANS merge_codex_hooks (the catalog is the authority)",
      JSON.stringify(dry.payload?.plan || []),
    );

    await runOnboardAt(launcher, ["--repo-root", repo, "--repo-hooks", "--apply", "--json"], home);
    check(
      fs.readFileSync(codexHooks, "utf8") === catalogRendering,
      "codex-hooks: the catalog's rendering survives self-onboard BYTE-FOR-BYTE (no clobber, no .bak dance)",
      fs.readFileSync(codexHooks, "utf8").slice(0, 160),
    );
    check(
      !fs.existsSync(`${codexHooks}.bak`),
      "codex-hooks: no .bak is written for a file onboarding must not touch",
    );
    check(
      !fs.readFileSync(codexHooks, "utf8").includes(".bee/bin/hooks/"),
      "codex-hooks: the HOST projection (.bee/bin/hooks/) never leaks into bee's own repo",
    );

    // Falsifiability: a HOST repo (no hooks/catalog.mjs) must STILL get the projection.
    const host = path.join(base, "host");
    const { launcher: hostLauncher } = makeFakeSkillsRoot(path.join(base, "src", "skills"), {
      skills: { "bee-alpha": { "SKILL.md": "# alpha\n" } },
    });
    fs.mkdirSync(host, { recursive: true });
    const hostRes = await runOnboardAt(
      hostLauncher,
      ["--repo-root", host, "--repo-hooks", "--apply", "--json"],
      makeFakeHome(),
    );
    const hostCodex = path.join(host, ".codex", "hooks.json");
    check(
      fs.existsSync(hostCodex) && fs.readFileSync(hostCodex, "utf8").includes(".bee/bin/hooks/"),
      "codex-hooks: a HOST repo still receives the .bee/bin/hooks/ projection (the skip is scoped, not a kill)",
      `exists=${fs.existsSync(hostCodex)} status=${hostRes.payload?.status}`,
    );
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
    fs.rmSync(home, { recursive: true, force: true });
  }
}

// --- 10a3. self-onboard refreshes discoverable project projections ---------
// A canonical source checkout still exposes .claude/skills and .agents/skills
// to assistants. Those projections must therefore be upgraded from the same
// validated release tuple and settle to an immediate up_to_date repeat.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-selfonboard-"));
  const home = makeFakeHome();
  try {
    const repo = path.join(base, "repo");
    const { launcher } = makeFakeSkillsRoot(path.join(repo, "skills"), {
      skills: { "bee-alpha": { "SKILL.md": "# alpha self\n" } },
    });
    for (const relRoot of REPO_TARGET_ROOTS) {
      const targetRoot = path.join(repo, ...relRoot.split("/"));
      writeSkillFiles(targetRoot, "bee-hive", {
        "SKILL.md": "# stale projected hive\n",
        "templates/lib/state.mjs": fakeStateSource("0.1.18"),
      });
      writeSkillFiles(targetRoot, "bee-alpha", { "SKILL.md": "# stale alpha\n" });
    }
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "self-onboard: stale project projections upgrade successfully",
      `exit ${apply.status} status ${apply.payload?.status}`);
    for (const kind of ["repo-claude", "repo-agents"]) {
      check(skillTarget(apply.payload, kind)?.mode === "sync",
        `self-onboard: ${kind} target runs the ordinary fail-closed sync path`,
        JSON.stringify(skillTarget(apply.payload, kind)));
    }
    for (const relRoot of REPO_TARGET_ROOTS) {
      // D1 (packages-restructure): the stale seeded legacy marker
      // (bee-hive/templates/lib/state.mjs) is pruned by the sync itself
      // (skills are instruction-only, no templates/ in the source walk); the
      // refreshed release tuple now lives in the sync-owned version stamp.
      const stampRaw = readRepoTarget(repo, relRoot, ".bee-skills-version.json");
      check(readRepoTarget(repo, relRoot, "bee-alpha/SKILL.md") === "# alpha self\n" &&
        stampRaw !== null && JSON.parse(stampRaw).version === "0.1.19",
        `self-onboard: refreshes ${relRoot} to the validated source tuple`,
        String(stampRaw));
    }
    check(apply.payload?.recheck === "up_to_date",
      "self-onboard: immediate recheck is up_to_date",
      JSON.stringify(apply.payload?.recheck_plan || []));
    // Global sync behavior is unchanged there: --global-skills still syncs.
    const gApply = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(gApply.status === 0 && gApply.payload?.status === "applied" &&
      skillTarget(gApply.payload, "global")?.mode === "fresh" &&
      readInstalled(home, "bee-alpha/SKILL.md") === "# alpha self\n",
      "self-onboard: --global-skills still syncs the global root",
      JSON.stringify(gApply.payload?.skills?.targets || null));
    for (const relRoot of REPO_TARGET_ROOTS) {
      check(readRepoTarget(repo, relRoot, "bee-alpha/SKILL.md") === "# alpha self\n",
        `self-onboard: --global-skills run keeps ${relRoot} current`);
    }
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10a4. plugin-first distribution mode: release identity still gates, and
// the distribution-applicable surface reaches the validated tuple (D1) --------
// --plugin-source vendors the .bee runtime but intentionally skips project skill
// projections. The strict release-identity tuple check runs BEFORE any plan that
// can be applied, plugin-first included (see readSourceReleaseIdentity) — nothing
// else in this suite exercises that mode, so a regression that skipped the tuple
// gate (or the runtime parity) whenever syncSkills is off would ship green.
{
  // (a) greenfield plugin-first: the .bee runtime lib and onboarding marker reach
  // the validated tuple; project projections are legitimately absent for this
  // distribution mode; the immediate repeat is up_to_date.
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-plugin-source-green-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), { version: "0.1.19" });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const apply = await runOnboardAt(
      launcher, ["--repo-root", repo, "--apply", "--plugin-source", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "plugin-source: greenfield apply succeeds (release identity validated for the plugin-first mode)",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(apply.payload?.recheck === "up_to_date",
      "plugin-source: immediate recheck is up_to_date",
      JSON.stringify(apply.payload?.recheck_plan || []));
    check(
      fs.readFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"), "utf8") ===
        fakeStateSource("0.1.19"),
      "plugin-source: vendored runtime state.mjs equals the validated release tuple");
    check(JSON.parse(fs.readFileSync(path.join(repo, ".bee", "onboarding.json"), "utf8"))
      .bee_version === "0.1.19",
      "plugin-source: onboarding.json records the validated release tuple");
    // Distribution-applicable surface: plugin-first skips project projections,
    // so neither in-repo skill root is written and no project target is enumerated.
    for (const relRoot of REPO_TARGET_ROOTS) {
      check(!fs.existsSync(path.join(repo, ...relRoot.split("/"))),
        `plugin-source: ${relRoot} project projection is not written in the plugin-first mode`);
    }
    check((apply.payload?.skills?.targets || []).length === 0,
      "plugin-source: no project skill targets are enumerated in the plugin-first mode",
      JSON.stringify(apply.payload?.skills?.targets || null));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }

  // (b) plugin-first mode does NOT exempt the release-identity gate: a mixed
  // numeric tuple refuses BEFORE the .bee runtime is vendored — zero mutation,
  // never forceable — exactly as the project-projection modes do.
  const mixBase = fs.mkdtempSync(path.join(os.tmpdir(), "bee-plugin-source-mixed-"));
  const mixHome = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(mixBase, "skills"), {
      version: "0.1.19",
      codexManifest: { name: "bee", version: "0.1.20" },
    });
    const repo = path.join(mixBase, "repo");
    fs.mkdirSync(repo, { recursive: true });
    fs.writeFileSync(path.join(repo, "owner.txt"), "must survive\n", "utf8");
    const repoBefore = hashTree(repo);
    const homeBefore = hashTree(mixHome);
    const apply = await runOnboardAt(
      launcher,
      ["--repo-root", repo, "--apply", "--plugin-source", "--force-downgrade", "--json"],
      mixHome);
    check(apply.status === 1 && apply.payload?.status === "blocked_no_source",
      "plugin-source: a mixed release tuple refuses even in the plugin-first mode",
      `exit ${apply.status} payload ${JSON.stringify(apply.payload)}`);
    check(typeof apply.payload?.reason === "string" &&
      apply.payload.reason.includes("authoritative source release tuple") &&
      apply.payload.reason.includes("tuple members disagree"),
      "plugin-source: refusal names the strict release-identity failure",
      String(apply.payload?.reason));
    check(!fs.existsSync(path.join(repo, ".bee")),
      "plugin-source: refused apply vendors no .bee runtime (zero mutation before the tuple gate)");
    check(hashTree(repo) === repoBefore && hashTree(mixHome) === homeBefore,
      "plugin-source: refused apply leaves repo and runtime targets byte-identical");
    check(apply.payload?.forced_downgrade === undefined,
      "plugin-source: a source-tuple failure is never forceable");
  } finally {
    try {
      fs.rmSync(mixBase, { recursive: true, force: true });
      fs.rmSync(mixHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10b. fence payload + equal-version drift + removal (D4/D5) -------------
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-fence-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      skills: { "bee-alpha": { "SKILL.md": "# alpha v2\n" } },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const installedRoot = makeInstalledSkills(home, {
      version: "0.1.19",
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha v1 STALE\n" },
        "bee-obsolete": { "SKILL.md": "# obsolete\n", "references/old.md": "old\n" },
      },
    });
    // Non-bee payload: must be byte-identical after a deletion-bearing sync.
    writeSkillFiles(installedRoot, "agent-browser", {
      "SKILL.md": "# not bee's business\n",
      "references/deep/data.md": "precious user data\n",
    });
    const payloadBefore = hashTree(path.join(installedRoot, "agent-browser"));
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check((plan.payload?.plan || []).some((i) => i.action === "sync_skill" && i.skill === "bee-alpha" && i.target === "global"),
      "equal-version byte drift produces a sync_skill item (D5)",
      JSON.stringify(plan.payload?.plan || []));
    check((plan.payload?.plan || []).some((i) => i.action === "remove_skill" && i.skill === "bee-obsolete"),
      "skill absent from the anchored source planned as remove_skill (D2/D4)");
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "fence: deletion-bearing apply succeeds", `exit ${apply.status}`);
    check(readInstalled(home, "bee-alpha/SKILL.md") === "# alpha v2\n",
      "drifted skill mirrored back to source bytes (D5)");
    check(!fs.existsSync(path.join(installedRoot, "bee-obsolete")),
      "removed-from-source skill deleted from the install (D4 mirror)");
    check(hashTree(path.join(installedRoot, "agent-browser")) === payloadBefore,
      "non-bee sibling byte-identical after a deletion-bearing sync (D4 fence payload)");
    check(apply.payload?.recheck === "up_to_date", "fence: recheck up_to_date");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10c. zero-mutation downgrade refusal (D3) -------------------------------
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-refuse-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), { version: "0.1.18" });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    makeInstalledSkills(home, { version: "0.1.19" });
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "blocked_downgrade",
      "downgrade: plan mode reports blocked_downgrade with exit 0",
      `exit ${plan.status} status ${plan.payload?.status}`);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 1, "downgrade: apply exits 1", `exit ${apply.status}`);
    check(apply.payload?.status === "blocked_downgrade",
      "downgrade: apply reports blocked_downgrade", JSON.stringify(apply.payload));
    const v = apply.payload?.versions || {};
    check(v.source === "0.1.18" && v.host_helpers === "absent" && v.installed_skills === "0.1.19",
      "refusal reports all three versions (source/host_helpers/installed_skills)",
      JSON.stringify(v));
    check(typeof apply.payload?.reason === "string" && apply.payload.reason.length > 0,
      "refusal carries a one-line reason");
    check(hashTree(home) === homeBefore,
      "refused apply leaves the target tree byte-identical (zero mutations)");
    check(hashTree(repo) === repoBefore,
      "refused apply leaves the repo byte-identical (post-loop onboarding.json write unreachable)");
    check(!fs.existsSync(path.join(repo, ".bee")),
      "refused apply creates no .bee dir at all");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10d. existing-but-unreadable tree = unknown = refuse, never forceable ---
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-unknown-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), { version: "0.1.19" });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    makeInstalledSkills(home, { stateText: "// corrupt: no version constant here\n" });
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_downgrade",
      "existing-but-unreadable installed tree refuses (unknown, D3)",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(apply.payload?.versions?.installed_skills === "unknown",
      "unreadable installed version reported as unknown",
      JSON.stringify(apply.payload?.versions || {}));
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 1 && forced.payload?.status === "blocked_downgrade",
      "unknown is NEVER forceable: --force-downgrade still refuses",
      `exit ${forced.status} status ${forced.payload?.status}`);
    check(forced.payload?.forced_downgrade === undefined,
      "refused force reports no forced_downgrade");
    check(hashTree(home) === homeBefore && hashTree(repo) === repoBefore,
      "unforceable refusal keeps repo and target byte-identical");
    // P49 (advisor finding 1/4): a non-forceable refusal never invites a
    // force - host_items is omitted entirely, not an empty array, so a
    // consumer can't mistake "nothing to force" for "forceable with none".
    check(apply.payload?.host_items === undefined,
      "unknown-version refusal carries NO host_items field",
      JSON.stringify(apply.payload?.host_items));
    check(forced.payload?.host_items === undefined,
      "unknown-version refusal via --force-downgrade also carries no host_items",
      JSON.stringify(forced.payload?.host_items));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10e. --force-downgrade with all three versions numeric (F9) ------------
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-force-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), { version: "0.1.18" });
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"),
      fakeStateSource("0.1.19"), "utf8");
    // In-repo targets must also resolve all three versions numeric, or the
    // host_helpers-driven refusal is unforceable for them (blocked-first:
    // EVERY blocked target must be forceable for --force-downgrade to apply).
    seedRepoSkillTargets(repo, "0.1.19");
    makeInstalledSkills(home, { version: "0.1.19" });
    const refused = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(refused.status === 1 && refused.payload?.status === "blocked_downgrade",
      "all-numeric downgrade still refuses by default");
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 0 && forced.payload?.status === "applied",
      "--force-downgrade proceeds when all three versions resolved numeric",
      `exit ${forced.status} status ${forced.payload?.status}`);
    check(forced.payload?.forced_downgrade === true,
      "forced apply reports forced_downgrade: true in its JSON (F9)");
    const fv = forced.payload?.versions || {};
    check(fv.source === "0.1.18" && fv.host_helpers === "0.1.19" && fv.installed_skills === "0.1.19",
      "forced apply reports the versions triple alongside the flag (F9)",
      JSON.stringify(fv));
    // D1 (packages-restructure): skills are instruction-only - the forced
    // sync prunes the seeded legacy marker along with the rest of the stale
    // bee-hive skill dir; the sync-owned version stamp is what now carries
    // the applied release.
    const forcedStampRaw = readInstalled(home, ".bee-skills-version.json");
    check(forcedStampRaw !== null && JSON.parse(forcedStampRaw).version === "0.1.18",
      "forced apply actually syncs the older source into the install",
      String(forcedStampRaw));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10f. symlink fail-closed at BOTH levels (F6, panel-2 NEW-2) -------------
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-symlink-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha v2\n" },
        "bee-beta": { "SKILL.md": "# beta v2\n" },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const outsideA = path.join(base, "outside-a");
    const outsideB = path.join(base, "outside-b");
    fs.mkdirSync(outsideA, { recursive: true });
    fs.mkdirSync(outsideB, { recursive: true });
    fs.writeFileSync(path.join(outsideA, "real-work.md"), "do not touch A\n", "utf8");
    fs.writeFileSync(path.join(outsideB, "real-work.md"), "do not touch B\n", "utf8");
    const installedRoot = makeInstalledSkills(home, { version: "0.1.19" });
    // (i) top-level bee-* entry that IS a symlink to an outside dir
    fs.symlinkSync(outsideA, path.join(installedRoot, "bee-alpha"));
    // (ii) managed dir with a NESTED symlink pointing outside
    writeSkillFiles(installedRoot, "bee-beta", { "SKILL.md": "# beta v1\n" });
    fs.symlinkSync(outsideB, path.join(installedRoot, "bee-beta", "link"));
    // (iii) symlinked bee-* entry ABSENT from source: removal path must not unlink
    fs.symlinkSync(outsideA, path.join(installedRoot, "bee-gone"));
    const outsideABefore = hashTree(outsideA);
    const outsideBBefore = hashTree(outsideB);
    const betaBefore = hashTree(path.join(installedRoot, "bee-beta"));
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    const planBlocked = (plan.payload?.plan || [])
      .filter((i) => i.action === "blocked_symlink").map((i) => i.skill).sort();
    check(["bee-alpha", "bee-beta", "bee-gone"].every((s) => planBlocked.includes(s)),
      "plan reports blocked_symlink loudly for every affected skill",
      JSON.stringify(planBlocked));
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "symlink: apply still proceeds for unaffected skills", `exit ${apply.status}`);
    const skipped = (apply.payload?.skills?.skipped || []).map((s) => s.skill).sort();
    check(["bee-alpha", "bee-beta", "bee-gone"].every((s) => skipped.includes(s)),
      "apply reports each symlinked skill as skipped (loud per-skill report)",
      JSON.stringify(apply.payload?.skills || null));
    check(fs.lstatSync(path.join(installedRoot, "bee-alpha")).isSymbolicLink() &&
      fs.readlinkSync(path.join(installedRoot, "bee-alpha")) === outsideA,
      "top-level symlinked skill entry never unlinked or replaced");
    check(fs.lstatSync(path.join(installedRoot, "bee-gone")).isSymbolicLink(),
      "symlinked entry absent from source never unlinked (removal path fail-closed)");
    check(hashTree(outsideA) === outsideABefore,
      "top-level link target contents byte-identical (never written through)");
    check(hashTree(path.join(installedRoot, "bee-beta")) === betaBefore,
      "skill with a nested symlink left byte-identical (skipped whole, never traversed)");
    check(hashTree(outsideB) === outsideBBefore,
      "nested link target contents byte-identical");
    check(readInstalled(home, "bee-hive/SKILL.md") === "# fake bee-hive\n",
      "unaffected sibling skill still synced in the same run");
    check(apply.payload?.recheck === "changes_needed",
      "recheck stays changes_needed while a skill is symlink-blocked (parity unresolved)");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10g. ancestor overlap of source/target roots fails closed (F6) ---------
{
  // Direction 1: source root strictly inside the target root.
  const home = fs.realpathSync(makeFakeHome());
  const repoBase = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-ovl1-"));
  try {
    const nestedRoot = path.join(home, ".claude", "skills", "bee-dev", "checkout", "skills");
    const { launcher } = makeFakeSkillsRoot(nestedRoot, {});
    const repo = path.join(repoBase, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_no_source",
      "ancestor overlap (source inside target) fails closed on apply",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(hashTree(home) === homeBefore && hashTree(repo) === repoBefore,
      "overlap refusal (source inside target) mutates nothing anywhere");
    const v1 = apply.payload?.versions || {};
    check(v1.source === "unknown" && v1.host_helpers === "unknown" && v1.installed_skills === "unknown",
      "overlap (source inside target) reports the version triple as unknown (review P1-8)",
      JSON.stringify(v1));
  } finally {
    try {
      fs.rmSync(home, { recursive: true, force: true });
      fs.rmSync(repoBase, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}
{
  // Direction 2: target root strictly inside the source root.
  const base = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-ovl2-")));
  try {
    const skillsRoot = path.join(base, "skills");
    const { launcher } = makeFakeSkillsRoot(skillsRoot, {});
    const innerHome = path.join(skillsRoot, "home");
    fs.mkdirSync(innerHome, { recursive: true });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const sourceBefore = hashTree(skillsRoot);
    const repoBefore = hashTree(repo);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], innerHome);
    check(apply.status === 1 && apply.payload?.status === "blocked_no_source",
      "ancestor overlap (target inside source) fails closed on apply",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(hashTree(skillsRoot) === sourceBefore && hashTree(repo) === repoBefore,
      "overlap refusal (target inside source) mutates nothing anywhere");
    const v2 = apply.payload?.versions || {};
    check(v2.source === "unknown" && v2.host_helpers === "unknown" && v2.installed_skills === "unknown",
      "overlap (target inside source) reports the version triple as unknown (review P1-8)",
      JSON.stringify(v2));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10h. installed-copy self-invocation = verify-only NOOP (D2) -------------
{
  const home = makeFakeHome();
  const repoBase = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-noop-"));
  try {
    const skillsRoot = path.join(home, ".claude", "skills");
    const { launcher } = makeFakeSkillsRoot(skillsRoot, {
      skills: { "bee-alpha": { "SKILL.md": "# alpha installed\n" } },
    });
    const repo = path.join(repoBase, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const before = hashTree(skillsRoot);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "installed-copy run applies host-repo onboarding normally",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(skillTarget(apply.payload, "global")?.mode === "noop",
      "source==target realpath resolves the global target to verify-only NOOP (D2)",
      JSON.stringify(apply.payload?.skills || null));
    check(!(apply.payload?.applied || []).some(
      (i) => (i.action === "sync_skill" || i.action === "remove_skill") && i.target === "global"),
      "NOOP run emits no global-target skill mutations");
    check(hashTree(skillsRoot) === before,
      "NOOP run leaves the installed skill tree byte-identical");
    check(fs.existsSync(path.join(repo, "AGENTS.md")),
      "host-repo onboarding still lands during a NOOP skill stage");
    check(apply.payload?.recheck === "up_to_date", "NOOP recheck up_to_date",
      JSON.stringify(apply.payload?.recheck_plan || []));
  } finally {
    try {
      fs.rmSync(home, { recursive: true, force: true });
      fs.rmSync(repoBase, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10i. realpath identity anchor: misplaced launcher never adopts a tree ---
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-ident-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      hiveDirName: "bee-hive-moved",
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "blocked_no_source",
      "identity failure: plan reports blocked_no_source with exit 0",
      `exit ${plan.status} status ${plan.payload?.status}`);
    const planV = plan.payload?.versions || {};
    check(planV.source === "unknown" && planV.host_helpers === "unknown" &&
      planV.installed_skills === "unknown",
      "identity failure: plan mode reports the version triple as unknown (review P1-8)",
      JSON.stringify(planV));
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_no_source",
      "identity failure aborts the whole apply with exit 1 (F2)",
      `exit ${apply.status} status ${apply.payload?.status}`);
    const applyV = apply.payload?.versions || {};
    check(applyV.source === "unknown" && applyV.host_helpers === "unknown" &&
      applyV.installed_skills === "unknown",
      "identity failure: apply reports the version triple as unknown (review P1-8)",
      JSON.stringify(applyV));
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--force-downgrade", "--json"], home);
    check(forced.status === 1 && forced.payload?.status === "blocked_no_source",
      "blocked_no_source is NEVER forceable");
    check(hashTree(home) === homeBefore && hashTree(repo) === repoBefore,
      "no-source refusal mutates nothing anywhere (repo and target byte-identical)");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10j. source<helpers only refusal, driven solely by host_helpers (F3) ---
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-hostonly-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), { version: "0.1.18" });
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"),
      fakeStateSource("0.1.19"), "utf8"); // host_helpers newer than source
    seedRepoSkillTargets(repo, "0.1.17"); // numeric per-target installs keep the refusal forceable
    makeInstalledSkills(home, { version: "0.1.17" }); // installed OLDER: never triggers
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "blocked_downgrade",
      "source<helpers only: plan mode reports blocked_downgrade (F3)",
      `exit ${plan.status} status ${plan.payload?.status}`);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_downgrade",
      "source<helpers only: apply refuses driven solely by host_helpers (independent branch)",
      `exit ${apply.status} status ${apply.payload?.status}`);
    const v = apply.payload?.versions || {};
    check(v.source === "0.1.18" && v.host_helpers === "0.1.19" && v.installed_skills === "0.1.17",
      "source<helpers only: an OLDER installed_skills never masks the host_helpers refusal",
      JSON.stringify(v));
    check(hashTree(home) === homeBefore && hashTree(repo) === repoBefore,
      "source<helpers only: refusal mutates nothing anywhere");
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 0 && forced.payload?.status === "applied" &&
      forced.payload?.forced_downgrade === true,
      "source<helpers only: --force-downgrade proceeds once all three versions resolved numeric",
      `exit ${forced.status} status ${forced.payload?.status}`);
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10k. host_helpers existing-but-unreadable -> unknown -> refuse, never forceable ---
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-hostunknown-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), { version: "0.1.19" });
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"),
      "// corrupt: no version constant here\n", "utf8");
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_downgrade",
      "existing-but-unreadable vendored state.mjs refuses (host_helpers unknown, D3)",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(apply.payload?.versions?.host_helpers === "unknown",
      "unreadable host_helpers version reported as unknown",
      JSON.stringify(apply.payload?.versions || {}));
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--force-downgrade", "--json"], home);
    check(forced.status === 1 && forced.payload?.status === "blocked_downgrade",
      "host_helpers unknown is NEVER forceable: --force-downgrade still refuses",
      `exit ${forced.status} status ${forced.payload?.status}`);
    check(forced.payload?.forced_downgrade === undefined,
      "refused force reports no forced_downgrade");
    check(hashTree(home) === homeBefore && hashTree(repo) === repoBefore,
      "unforceable host_helpers-unknown refusal keeps repo and target byte-identical");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10l. source EXISTING-but-unreadable -> unknown -> refuse, never forceable (F3) ---
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-srcunknown-"));
  const home = makeFakeHome();
  try {
    // The source tree's state.mjs is imported as a real ESM module (by
    // commands_detect.mjs, for COMMAND_KEYS), unlike the installed/host
    // state.mjs which is only regex-read - so "corrupt" here must stay valid
    // JS with a working COMMAND_KEYS export; only BEE_VERSION is malformed.
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      stateText: "export const COMMAND_KEYS = ['setup', 'start', 'test', 'verify'];\n" +
        "export const BEE_VERSION = 'not-a-version';\n",
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_no_source",
      "existing-but-unreadable SOURCE state.mjs refuses as invalid release identity",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(apply.payload?.versions?.source === "unknown",
      "unreadable source version reported as unknown",
      JSON.stringify(apply.payload?.versions || {}));
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--force-downgrade", "--json"], home);
    check(forced.status === 1 && forced.payload?.status === "blocked_no_source",
      "source unknown is NEVER forceable: --force-downgrade still refuses",
      `exit ${forced.status} status ${forced.payload?.status}`);
    check(forced.payload?.forced_downgrade === undefined,
      "refused force reports no forced_downgrade");
    check(hashTree(home) === homeBefore && hashTree(repo) === repoBefore,
      "unforceable source-unknown refusal keeps repo and target byte-identical");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10m. source newer than both host_helpers and installed_skills -> proceeds ----
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-srcnewer-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.2.0",
      skills: { "bee-alpha": { "SKILL.md": "# alpha v3\n" } },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"),
      fakeStateSource("0.1.19"), "utf8");
    makeInstalledSkills(home, {
      version: "0.1.19",
      skills: { "bee-alpha": { "SKILL.md": "# alpha v3\n" } },
    });
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "source newer than both host_helpers and installed_skills proceeds without --force-downgrade",
      `exit ${apply.status} status ${apply.payload?.status}`);
    const v = skillTarget(apply.payload, "global")?.versions || {};
    check(v.source === "0.2.0" && v.host_helpers === "0.1.19" && v.installed_skills === "0.1.19",
      "source-newer apply reports the versions triple with source strictly ahead",
      JSON.stringify(v));
    check(apply.payload?.forced_downgrade === undefined,
      "source-newer apply carries no forced_downgrade marker (force was never needed)");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10n. repo never onboarded (host_helpers absent) + an older installed tree ---
// -> proceeds. Distinct from 10c: there, host absent still refuses because
// installed_skills is NEWER; here installed_skills is OLDER, proving "absent"
// is never itself a refusal trigger - only a resolved comparison is.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-neveronboarded-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.19",
      skills: { "bee-alpha": { "SKILL.md": "# alpha v2\n" } },
    });
    const repo = path.join(base, "repo"); // never onboarded: no .bee dir at all
    fs.mkdirSync(repo, { recursive: true });
    makeInstalledSkills(home, {
      version: "0.1.18", // older than source, but present -> not "absent"
      skills: { "bee-alpha": { "SKILL.md": "# alpha v1\n" } },
    });
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "repo never onboarded (host_helpers absent) proceeds as first onboard, no refusal",
      `exit ${apply.status} status ${apply.payload?.status}`);
    const v = skillTarget(apply.payload, "global")?.versions || {};
    check(v.host_helpers === "absent" && v.installed_skills === "0.1.18" && v.source === "0.1.19",
      "first-onboard apply reports host_helpers absent distinctly from unknown",
      JSON.stringify(v));
    check(readInstalled(home, "bee-alpha/SKILL.md") === "# alpha v2\n",
      "first-onboard apply still syncs the newer skill content");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10o. deep mirror: nested file removed, new skill appears, stale skill gone ---
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-deepmirror-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      skills: {
        "bee-kept": {
          "SKILL.md": "# kept\n",
          "references/keep-me.md": "keep\n",
        },
        "bee-new": { "SKILL.md": "# brand new\n", "scripts/run.mjs": "// new\n" },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const installedRoot = makeInstalledSkills(home, {
      version: "0.1.19",
      skills: {
        "bee-kept": {
          "SKILL.md": "# kept\n",
          "references/keep-me.md": "keep\n",
          "references/deep/stale.md": "stale nested file to remove\n",
        },
        "bee-stale": { "SKILL.md": "# going away\n", "references/old.md": "old\n" },
      },
    });
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    const planActions = (plan.payload?.plan || []).map((i) => `${i.action}:${i.skill}`).sort();
    check(planActions.includes("sync_skill:bee-kept"),
      "deep mirror: plan syncs bee-kept (a nested file differs)", JSON.stringify(planActions));
    check(planActions.includes("sync_skill:bee-new"),
      "deep mirror: plan syncs the brand-new bee-new skill", JSON.stringify(planActions));
    check(planActions.includes("remove_skill:bee-stale"),
      "deep mirror: plan removes the stale bee-stale skill", JSON.stringify(planActions));
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "deep mirror apply succeeds", `exit ${apply.status}`);
    check(!fs.existsSync(path.join(installedRoot, "bee-kept", "references", "deep", "stale.md")),
      "deep mirror: a file removed from nested references/ of a KEPT skill disappears");
    check(readInstalled(home, "bee-kept/references/keep-me.md") === "keep\n",
      "deep mirror: sibling nested file in the kept skill left untouched");
    check(readInstalled(home, "bee-new/scripts/run.mjs") === "// new\n",
      "deep mirror: a brand-new bee skill's nested file synced in full");
    check(!fs.existsSync(path.join(installedRoot, "bee-stale")),
      "deep mirror: a bee skill removed from source is fully deleted from the install");
    check(apply.payload?.recheck === "up_to_date", "deep mirror: recheck up_to_date",
      JSON.stringify(apply.payload?.recheck_plan || []));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10p. idempotency: second apply -> up_to_date, zero items, manifest parity ---
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-idempotent-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha\n", "references/notes.md": "notes\n" },
        "bee-beta": { "SKILL.md": "# beta\n" },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const sourceRoot = path.join(base, "skills");
    const installedRoot = path.join(home, ".claude", "skills");
    await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    const plan2 = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check(plan2.status === 0 && plan2.payload?.status === "up_to_date",
      "idempotency: second run's plan mode reports up_to_date",
      `exit ${plan2.status} status ${plan2.payload?.status}`);
    check(Array.isArray(plan2.payload?.plan) && plan2.payload.plan.length === 0,
      "idempotency: second run's plan carries zero items",
      JSON.stringify(plan2.payload?.plan || []));
    for (const skill of ["bee-alpha", "bee-beta", "bee-hive"]) {
      const sourceHash = hashTree(path.join(sourceRoot, skill));
      const installedHash = hashTree(path.join(installedRoot, skill));
      check(sourceHash === installedHash,
        `idempotency: ${skill} manifest hash parity between source and installed`,
        `source: ${sourceHash} installed: ${installedHash}`);
      for (const relRoot of REPO_TARGET_ROOTS) {
        check(hashTree(path.join(repo, ...relRoot.split("/"), skill)) === sourceHash,
          `idempotency: ${skill} manifest hash parity between source and ${relRoot}`);
      }
    }
    const apply2 = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply2.payload?.status === "applied" && apply2.payload?.recheck === "up_to_date",
      "idempotency: second apply is a no-op, recheck up_to_date");
    check(Array.isArray(apply2.payload?.applied) &&
      !apply2.payload.applied.some((i) => i.action === "sync_skill" || i.action === "remove_skill"),
      "idempotency: second apply performs no skill mutations",
      JSON.stringify(apply2.payload?.applied || []));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10q. partial install: bee-* present without readable bee-hive = unknown ---
// (review P1-1) A target holding bee-* skills but NO readable bee-hive version
// marker must resolve installed_skills to "unknown" (refuse, never forceable) -
// never "absent": an older source would otherwise overwrite/delete newer
// foreign bee-* skills with no refusal. "absent" is earned only by a target
// with zero lstat-visible bee-* entries.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-partial-"));
  const home = makeFakeHome();
  const home2 = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.19",
      skills: {},
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    const installedRoot = makeInstalledSkills(home, {
      version: null, // NO bee-hive at all - only a foreign bee-* skill
      skills: { "bee-newer": { "SKILL.md": "# from a newer layout\n" } },
    });
    const homeBefore = hashTree(home);
    const repoBefore = hashTree(repo);
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "blocked_downgrade",
      "partial install (bee-* without bee-hive) refuses as unknown, never fresh (review P1-1)",
      `exit ${plan.status} status ${plan.payload?.status}`);
    check(plan.payload?.versions?.installed_skills === "unknown",
      "partial install reports installed_skills unknown, not absent",
      JSON.stringify(plan.payload?.versions || null));
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_downgrade",
      "partial install: apply refuses with exit 1",
      `exit ${apply.status} status ${apply.payload?.status}`);
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 1 && forced.payload?.status === "blocked_downgrade",
      "partial-install unknown is NEVER forceable",
      `exit ${forced.status} status ${forced.payload?.status}`);
    check(fs.existsSync(path.join(installedRoot, "bee-newer")) &&
      readInstalled(home, "bee-newer/SKILL.md") === "# from a newer layout\n",
      "the foreign bee-* skill survives byte-identical (never deleted as 'fresh')");
    check(hashTree(home) === homeBefore && hashTree(repo) === repoBefore,
      "partial-install refusal mutates nothing anywhere");
    // Contrast: an EXISTING target dir with ONLY non-bee entries is still a
    // fresh install ("absent" = no lstat-visible bee-* entry at all).
    fs.mkdirSync(path.join(home2, ".claude", "skills", "agent-browser"), { recursive: true });
    fs.writeFileSync(path.join(home2, ".claude", "skills", "agent-browser", "SKILL.md"),
      "# not bee's\n", "utf8");
    const fresh = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home2);
    check(fresh.status === 0 && fresh.payload?.status === "applied",
      "a target with only non-bee entries still reads absent -> fresh install proceeds",
      `exit ${fresh.status} status ${fresh.payload?.status}`);
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
      fs.rmSync(home2, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10r. version reader: ONE line-anchored declaration on a regular file ----
// (review P1-2) The preflight reader must accept exactly one line-anchored
// `export const BEE_VERSION = 'x.y.z'` read from a REGULAR, non-symlinked file
// (every path component under the managed target lstat'ed; source side lstats
// the marker file itself). Comment decoys never resolve, multiple declarations
// are unknown, symlinked markers or components are unknown - and unknown
// always refuses, never forceable.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-reader-"));
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.19",
      skills: {},
    });
    let repoN = 0;
    const nextRepo = () => {
      repoN += 1;
      const r = path.join(base, `repo-${repoN}`);
      fs.mkdirSync(r, { recursive: true });
      return r;
    };

    // (a) decoy comment above the real declaration -> the REAL version wins
    {
      const home = makeFakeHome();
      try {
        makeInstalledSkills(home, {
          stateText: "// BEE_VERSION = '0.1.18' decoy comment - never the version\n" +
            "export const BEE_VERSION = '0.1.20';\n",
        });
        const homeBefore = hashTree(home);
        const apply = await runOnboardAt(launcher, ["--repo-root", nextRepo(), "--apply", "--global-skills", "--json"], home);
        check(apply.status === 1 && apply.payload?.status === "blocked_downgrade",
          "decoy BEE_VERSION comment never resolves: the real 0.1.20 refuses a 0.1.19 source (review P1-2)",
          `exit ${apply.status} status ${apply.payload?.status}`);
        check(apply.payload?.versions?.installed_skills === "0.1.20",
          "reader reports the real line-anchored version, never the decoy",
          JSON.stringify(apply.payload?.versions || null));
        check(hashTree(home) === homeBefore,
          "decoy-refused apply mutates nothing in the target");
      } finally {
        try {
          fs.rmSync(home, { recursive: true, force: true });
        } catch {
          // best-effort cleanup
        }
      }
    }

    // (b) multiple line-anchored declarations -> unknown, never forceable
    {
      const home = makeFakeHome();
      try {
        makeInstalledSkills(home, {
          stateText: "export const BEE_VERSION = '0.1.18';\n" +
            "export const BEE_VERSION = '0.1.20';\n",
        });
        const homeBefore = hashTree(home);
        const forced = await runOnboardAt(launcher,
          ["--repo-root", nextRepo(), "--apply", "--global-skills", "--force-downgrade", "--json"], home);
        check(forced.status === 1 && forced.payload?.status === "blocked_downgrade" &&
          forced.payload?.versions?.installed_skills === "unknown",
          "multiple BEE_VERSION declarations resolve to unknown and refuse even --force-downgrade",
          `exit ${forced.status} versions ${JSON.stringify(forced.payload?.versions || null)}`);
        check(hashTree(home) === homeBefore,
          "multi-declaration refusal mutates nothing in the target");
      } finally {
        try {
          fs.rmSync(home, { recursive: true, force: true });
        } catch {
          // best-effort cleanup
        }
      }
    }

    // (c) symlinked marker FILE -> unknown (the reader lstats the marker)
    {
      const home = makeFakeHome();
      try {
        const root = makeInstalledSkills(home, {
          version: null,
          skills: { "bee-hive": { "SKILL.md": "# installed hive\n" } },
        });
        const outsideMarker = path.join(base, "outside-marker-c.mjs");
        fs.writeFileSync(outsideMarker, fakeStateSource("0.1.18"), "utf8");
        fs.mkdirSync(path.join(root, "bee-hive", "templates", "lib"), { recursive: true });
        fs.symlinkSync(outsideMarker,
          path.join(root, "bee-hive", "templates", "lib", "state.mjs"));
        const apply = await runOnboardAt(launcher, ["--repo-root", nextRepo(), "--apply", "--global-skills", "--json"], home);
        check(apply.status === 1 && apply.payload?.status === "blocked_downgrade" &&
          apply.payload?.versions?.installed_skills === "unknown",
          "symlinked version marker is never followed: unknown, refused",
          `exit ${apply.status} versions ${JSON.stringify(apply.payload?.versions || null)}`);
        check(fs.readFileSync(outsideMarker, "utf8") === fakeStateSource("0.1.18"),
          "symlink target file untouched by the refused apply");
      } finally {
        try {
          fs.rmSync(home, { recursive: true, force: true });
        } catch {
          // best-effort cleanup
        }
      }
    }

    // (d) symlinked path COMPONENT (templates/) -> unknown
    {
      const home = makeFakeHome();
      try {
        const root = makeInstalledSkills(home, {
          version: null,
          skills: { "bee-hive": { "SKILL.md": "# installed hive\n" } },
        });
        const outsideTemplates = path.join(base, "outside-templates-d");
        fs.mkdirSync(path.join(outsideTemplates, "lib"), { recursive: true });
        fs.writeFileSync(path.join(outsideTemplates, "lib", "state.mjs"),
          fakeStateSource("0.1.18"), "utf8");
        fs.symlinkSync(outsideTemplates, path.join(root, "bee-hive", "templates"));
        const apply = await runOnboardAt(launcher, ["--repo-root", nextRepo(), "--apply", "--global-skills", "--json"], home);
        check(apply.status === 1 && apply.payload?.status === "blocked_downgrade" &&
          apply.payload?.versions?.installed_skills === "unknown",
          "symlinked path component under the managed target is never trusted: unknown, refused",
          `exit ${apply.status} versions ${JSON.stringify(apply.payload?.versions || null)}`);
      } finally {
        try {
          fs.rmSync(home, { recursive: true, force: true });
        } catch {
          // best-effort cleanup
        }
      }
    }
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10s. dir<->file transitions converge without deleting fresh output -----
// (review P1-3) The old apply materialized the source shape FIRST and ran
// stale-entry cleanup from the pre-write snapshot AFTERWARDS: on a dir->file
// transition the stale-dirs pass rm -rf'ed the freshly written file; on
// file->dir the stale-files pass hit a directory mid-apply. Both directions
// (including nested paths) must converge to full manifest parity (D5).
{
  // Direction A: installed DIRECTORY -> source FILE (top-level and nested)
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-dir2file-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.19",
      skills: {
        "bee-doc": {
          "SKILL.md": "# doc\n",
          "guide": "# guide is now a FILE\n",
          "refs/data": "# data is now a FILE\n",
        },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    makeInstalledSkills(home, {
      version: "0.1.19",
      skills: {
        "bee-doc": {
          "SKILL.md": "# doc\n",
          "guide/intro.md": "# old intro\n",
          "guide/deep/note.md": "# old nested note\n",
          "refs/data/x.md": "# old nested data\n",
        },
      },
    });
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "dir->file transition: apply succeeds",
      `exit ${apply.status} stdout ${apply.stdout}`);
    check(readInstalled(home, "bee-doc/guide") === "# guide is now a FILE\n",
      "dir->file: freshly materialized top-level file survives cleanup (review P1-3)",
      String(readInstalled(home, "bee-doc/guide")));
    check(readInstalled(home, "bee-doc/refs/data") === "# data is now a FILE\n",
      "dir->file: freshly materialized NESTED file survives cleanup",
      String(readInstalled(home, "bee-doc/refs/data")));
    check(apply.payload?.recheck === "up_to_date",
      "dir->file: recheck lands up_to_date (full manifest parity, D5)",
      JSON.stringify(apply.payload?.recheck_plan || []));
    check(hashTree(path.join(base, "skills", "bee-doc")) ===
      hashTree(path.join(home, ".claude", "skills", "bee-doc")),
      "dir->file: installed bee-doc byte-identical to source");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}
{
  // Direction B: installed FILE -> source DIRECTORY (top-level and nested)
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-file2dir-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.19",
      skills: {
        "bee-doc": {
          "SKILL.md": "# doc\n",
          "guide/intro.md": "# new intro\n",
          "guide/deep/note.md": "# new nested note\n",
          "refs/data/x.md": "# new nested data\n",
        },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    makeInstalledSkills(home, {
      version: "0.1.19",
      skills: {
        "bee-doc": {
          "SKILL.md": "# doc\n",
          "guide": "# guide was a file\n",
          "refs/data": "# data was a file\n",
        },
      },
    });
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "file->dir transition: apply succeeds without throwing mid-apply (review P1-3)",
      `exit ${apply.status} stdout ${apply.stdout}`);
    check(readInstalled(home, "bee-doc/guide/intro.md") === "# new intro\n" &&
      readInstalled(home, "bee-doc/guide/deep/note.md") === "# new nested note\n",
      "file->dir: directory content fully materialized");
    check(readInstalled(home, "bee-doc/refs/data/x.md") === "# new nested data\n",
      "file->dir: NESTED file->dir transition materialized");
    check(apply.payload?.recheck === "up_to_date",
      "file->dir: recheck lands up_to_date (full manifest parity, D5)",
      JSON.stringify(apply.payload?.recheck_plan || []));
    check(hashTree(path.join(base, "skills", "bee-doc")) ===
      hashTree(path.join(home, ".claude", "skills", "bee-doc")),
      "file->dir: installed bee-doc byte-identical to source");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10t. repo root and skills root overlap (either direction) refuses ------
// (review P1-4) A repo living under ~/.claude/skills must never be deletable
// by its own onboard: the remove_skill pass would erase the live checkout,
// git history included. realpath(repoRoot)<->targetRoot overlap refuses
// blocked_no_source at preflight, never forceable, zero mutations.
{
  // Direction 1: repo INSIDE the target skills root
  const home = fs.realpathSync(makeFakeHome());
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-repoingt-"));
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.19",
      skills: {},
    });
    const installedRoot = makeInstalledSkills(home, { version: "0.1.19" });
    const repo = path.join(installedRoot, "bee-local");
    fs.mkdirSync(path.join(repo, ".git"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".git", "HEAD"), "ref: refs/heads/main\n", "utf8");
    fs.writeFileSync(path.join(repo, "work.md"), "irreplaceable checkout\n", "utf8");
    const homeBefore = hashTree(home);
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "blocked_no_source",
      "repo inside the skills root: plan reports blocked_no_source (review P1-4)",
      `exit ${plan.status} status ${plan.payload?.status}`);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 1 && apply.payload?.status === "blocked_no_source",
      "repo inside the skills root: apply refuses pre-write with exit 1",
      `exit ${apply.status} status ${apply.payload?.status}`);
    const repoOverlapV = apply.payload?.versions || {};
    check(repoOverlapV.source === "unknown" && repoOverlapV.host_helpers === "unknown" &&
      repoOverlapV.installed_skills === "unknown",
      "repo-in-target overlap reports the version triple as unknown (review P1-8)",
      JSON.stringify(repoOverlapV));
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 1 && forced.payload?.status === "blocked_no_source",
      "repo-in-target refusal is NEVER forceable",
      `exit ${forced.status} status ${forced.payload?.status}`);
    check(fs.existsSync(path.join(repo, ".git", "HEAD")) &&
      fs.readFileSync(path.join(repo, "work.md"), "utf8") === "irreplaceable checkout\n",
      "the live checkout (git history included) survives untouched");
    check(hashTree(home) === homeBefore,
      "repo-in-target refusal performs zero mutations anywhere");
  } finally {
    try {
      fs.rmSync(home, { recursive: true, force: true });
      fs.rmSync(base, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}
{
  // Direction 2: target skills root INSIDE the repo
  const base = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-tgtinrepo-")));
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.19",
      skills: {},
    });
    const repo = path.join(base, "repo");
    const innerHome = path.join(repo, "nested-home");
    fs.mkdirSync(innerHome, { recursive: true });
    const repoBefore = hashTree(repo);
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], innerHome);
    check(apply.status === 1 && apply.payload?.status === "blocked_no_source",
      "target skills root inside the repo refuses fail-closed (review P1-4)",
      `exit ${apply.status} status ${apply.payload?.status}`);
    check(hashTree(repo) === repoBefore,
      "target-in-repo refusal mutates nothing (no skills written into the repo)");
    const tgtOverlapV = apply.payload?.versions || {};
    check(tgtOverlapV.source === "unknown" && tgtOverlapV.host_helpers === "unknown" &&
      tgtOverlapV.installed_skills === "unknown",
      "target-in-repo overlap reports the version triple as unknown (review P1-8)",
      JSON.stringify(tgtOverlapV));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10u. case-insensitive fs: alias collision fails closed (review P1-5) ---
// Exact-case name comparison would sync via one name and delete via the other
// - one physical directory. Detection must be by canonical filesystem
// identity, simulated on the running platform (probe: does bee-Hive resolve
// to a created bee-hive?) and skipped with a named reason when the fs is
// case-sensitive. BEE_CASE_FS_DIR points the case at a case-insensitive
// mount when the default tmpdir is case-sensitive (e.g. /mnt/c on WSL).
{
  const caseBase = process.env.BEE_CASE_FS_DIR || os.tmpdir();
  let caseInsensitive = false;
  let probe = null;
  try {
    probe = fs.mkdtempSync(path.join(caseBase, "bee-caseprobe-"));
    fs.mkdirSync(path.join(probe, "bee-hive"));
    caseInsensitive = fs.existsSync(path.join(probe, "bee-Hive"));
  } catch {
    caseInsensitive = false;
  } finally {
    if (probe) {
      try {
        fs.rmSync(probe, { recursive: true, force: true });
      } catch {
        // best-effort cleanup
      }
    }
  }
  if (!caseInsensitive) {
    skip("case-alias fail-closed (review P1-5)",
      `filesystem at ${caseBase} is case-sensitive - bee-Hive does not resolve to bee-hive; set BEE_CASE_FS_DIR to a case-insensitive mount to exercise`);
  } else {
    const srcBase = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-aliassrc-"));
    const aliasBase = fs.mkdtempSync(path.join(caseBase, "bee-skillsync-alias-"));
    const home = path.join(aliasBase, "home");
    fs.mkdirSync(home, { recursive: true });
    try {
      const { launcher } = makeFakeSkillsRoot(path.join(srcBase, "skills"), {
        version: "0.1.19",
        skills: { "bee-alpha": { "SKILL.md": "# alpha source\n" } },
      });
      const repo = path.join(srcBase, "repo");
      fs.mkdirSync(repo, { recursive: true });
      const installedRoot = makeInstalledSkills(home, {
        version: "0.1.19",
        skills: { "bee-Alpha": { "SKILL.md": "# alpha installed, other case\n" } },
      });
      const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
      // The alias lives in the GLOBAL target; the fresh in-repo targets plan
      // their own (clean) sync_skill items, so the guarantee is per-target.
      const planItems = (plan.payload?.plan || []).filter((i) => i.target === "global");
      check(!planItems.some((i) =>
        (i.action === "sync_skill" || i.action === "remove_skill") &&
        ["bee-alpha", "bee-Alpha"].includes(i.skill)),
        "alias collision: neither sync_skill nor remove_skill planned for the aliased names",
        JSON.stringify(planItems));
      check(planItems.some((i) => i.action === "blocked_alias"),
        "alias collision is reported loudly as a blocked plan item",
        JSON.stringify(planItems));
      const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
      check(apply.status === 0 && apply.payload?.status === "applied",
        "alias: apply still proceeds for unaffected skills", `exit ${apply.status}`);
      check((apply.payload?.skills?.skipped || []).some((s) =>
        ["bee-alpha", "bee-Alpha"].includes(s.skill)),
        "apply reports the alias-blocked skill as skipped (loud per-skill report)",
        JSON.stringify(apply.payload?.skills || null));
      check(fs.existsSync(path.join(installedRoot, "bee-Alpha")),
        "the physical skill directory survives (never sync-then-delete)");
      check(readInstalled(home, "bee-Alpha/SKILL.md") === "# alpha installed, other case\n",
        "aliased skill content byte-identical (never written through the alias)",
        String(readInstalled(home, "bee-Alpha/SKILL.md")));
      check(apply.payload?.recheck === "changes_needed",
        "recheck stays changes_needed while a skill is alias-blocked (parity unresolved)",
        JSON.stringify(apply.payload?.recheck_plan || []));
    } finally {
      try {
        fs.rmSync(srcBase, { recursive: true, force: true });
        fs.rmSync(aliasBase, { recursive: true, force: true });
      } catch {
        // best-effort cleanup
      }
    }
  }
}

// --- 10v. forced-apply transparency: blocked-forceable dry-run and refused ---
// apply both enumerate the skills a --force-downgrade would overwrite/delete
// (review P1-6, D2). Before the fix neither the plain dry-run (`--json`, no
// --apply) nor the refused `--apply` response carried the computed items -
// a human deciding whether to pass --force-downgrade could not see which
// skills get overwritten (sync_skill) or DELETED (remove_skill) until AFTER
// authorizing it and reading the forced apply's own report.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-forcedvis-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.18",
      skills: { "bee-alpha": { "SKILL.md": "# alpha v2 from older source\n" } },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"),
      fakeStateSource("0.1.19"), "utf8");
    seedRepoSkillTargets(repo, "0.1.19"); // numeric per-target installs keep the refusal forceable
    makeInstalledSkills(home, {
      version: "0.1.19",
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha v1 - about to be overwritten\n" },
        "bee-doomed": { "SKILL.md": "# about to be deleted\n" },
      },
    });
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "blocked_downgrade",
      "forced-vis: plan mode reports blocked_downgrade (forceable)",
      `exit ${plan.status} status ${plan.payload?.status}`);
    const planItems = flatSkillItems(plan.payload);
    check(planItems.some((i) => i.action === "sync_skill" && i.skill === "bee-alpha" && i.target === "global"),
      "forced-vis: blocked dry-run still enumerates the sync_skill a force would overwrite (P1-6)",
      JSON.stringify(planItems));
    check(planItems.some((i) => i.action === "remove_skill" && i.skill === "bee-doomed" && i.target === "global"),
      "forced-vis: blocked dry-run still enumerates the remove_skill a force would DELETE (P1-6)",
      JSON.stringify(planItems));
    const refused = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(refused.status === 1 && refused.payload?.status === "blocked_downgrade",
      "forced-vis: refused apply (no --force-downgrade) still reports blocked_downgrade");
    const refusedItems = flatSkillItems(refused.payload);
    check(refusedItems.some((i) => i.action === "sync_skill" && i.skill === "bee-alpha" && i.target === "global") &&
      refusedItems.some((i) => i.action === "remove_skill" && i.skill === "bee-doomed" && i.target === "global"),
      "forced-vis: the refused --apply response ALSO enumerates the items per target, not only the plain dry-run (P1-6)",
      JSON.stringify(refusedItems));
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 0 && forced.payload?.status === "applied" &&
      forced.payload?.forced_downgrade === true,
      "forced-vis: forcing actually applies", `exit ${forced.status} status ${forced.payload?.status}`);
    const previewedSkills = [...new Set(refusedItems.map((i) => i.skill))].sort();
    const appliedSkills = [...new Set((forced.payload?.applied || [])
      .filter((i) => i.action === "sync_skill" || i.action === "remove_skill")
      .map((i) => i.skill))].sort();
    check(JSON.stringify(previewedSkills.filter((s) =>
      forced.payload.applied.some((i) => i.skill === s))) === JSON.stringify(appliedSkills) ||
      appliedSkills.every((s) => previewedSkills.includes(s)),
      "forced-vis: the forced apply touches exactly the reviewed set previewed before authorization",
      JSON.stringify({ previewedSkills, appliedSkills }));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10w. recheck honesty: a blocked skill stage can never report up_to_date -
// (review P1-7, D5) The post-apply recheck previously used `plan.length`
// only, which is empty whenever computePlan() withholds skill items because
// its skillSync stage is blocked - a false parity claim. Reachable after a
// forced downgrade that leaves ONE skill mid-refusal (a nested symlink
// elsewhere in bee-hive, off the templates/lib/state.mjs path so the version
// marker itself still resolves): the whole-stage version compare sees the
// installed bee-hive skill's marker still un-synced (older source, newer
// installed) and is genuinely blocked again at recheck time, while the
// general (non-skill) plan items are all freshly up to date and contribute
// zero items - so plan.length alone reads as up_to_date. Blocked-first
// precedence must override that.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-recheckhonesty-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.18",
      skills: {},
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"),
      fakeStateSource("0.1.19"), "utf8");
    seedRepoSkillTargets(repo, "0.1.19"); // numeric per-target installs keep the refusal forceable
    const installedRoot = makeInstalledSkills(home, { version: "0.1.19" });
    const outside = path.join(base, "outside-rogue");
    fs.mkdirSync(outside, { recursive: true });
    fs.writeFileSync(path.join(outside, "real-work.md"), "do not touch\n", "utf8");
    // Nested symlink INSIDE installed bee-hive, off the templates/lib path -
    // version resolution still succeeds, but the whole skill is blocked_symlink
    // and never gets synced, so its marker stays stuck at the OLDER install.
    fs.symlinkSync(outside, path.join(installedRoot, "bee-hive", "rogue-link"));
    const refused = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(refused.status === 1 && refused.payload?.status === "blocked_downgrade",
      "recheck-honesty: unforced apply refuses as blocked_downgrade (setup sanity)",
      `exit ${refused.status} status ${refused.payload?.status}`);
    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 0 && forced.payload?.status === "applied" &&
      forced.payload?.forced_downgrade === true,
      "recheck-honesty: forced apply proceeds", `exit ${forced.status} status ${forced.payload?.status}`);
    check((forced.payload?.skills?.skipped || []).some((s) => s.skill === "bee-hive"),
      "recheck-honesty: bee-hive itself is skipped (nested rogue symlink), left un-synced",
      JSON.stringify(forced.payload?.skills || null));
    check(forced.payload?.recheck !== "up_to_date",
      "recheck-honesty: a residual blocked skill stage can NEVER report recheck up_to_date (P1-7)",
      JSON.stringify({ recheck: forced.payload?.recheck, recheck_skills: forced.payload?.recheck_skills }));
    check(forced.payload?.recheck_skills?.blocked === true &&
      typeof forced.payload?.recheck_skills?.reason === "string" &&
      forced.payload?.recheck_skills?.reason.length > 0 &&
      forced.payload?.recheck_skills?.versions?.installed_skills === "0.1.19",
      "recheck-honesty: recheck exposes the blocked skill stage's status/reason/versions (P1-7)",
      JSON.stringify(forced.payload?.recheck_skills));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10z. plan-item scope discriminator (review P1-9): every skill-stage ----
// item carries scope: "installed" | "source" so a consumer never resolves a
// global deletion/overwrite against repoRoot; legacy (repo-relative) items
// carry no `scope` field at all, unchanged.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-skillsync-scope-"));
  const home = makeFakeHome();
  const outside = path.join(base, "outside-src-link");
  try {
    const { launcher, skillsRoot } = makeFakeSkillsRoot(path.join(base, "skills"), {
      skills: { "bee-normal": { "SKILL.md": "# normal\n" } },
    });
    fs.mkdirSync(outside, { recursive: true });
    // Source-level symlinked skill entry -> scope: "source".
    fs.symlinkSync(outside, path.join(skillsRoot, "bee-linked"));
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    makeInstalledSkills(home, {
      version: "0.1.19",
      skills: { "bee-remove-me": { "SKILL.md": "# will be removed\n" } },
    });
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--global-skills", "--json"], home);
    check(plan.status === 0, "scope: plan mode exits 0", `exit ${plan.status}`);
    const items = plan.payload?.plan || [];
    const byKey = (action, skill) => items.find((i) => i.action === action && i.skill === skill);
    const syncNormal = byKey("sync_skill", "bee-normal");
    check(syncNormal?.scope === "installed",
      "scope: sync_skill (writes a managed install root) carries scope: installed",
      JSON.stringify(syncNormal));
    const removeMe = byKey("remove_skill", "bee-remove-me");
    check(removeMe?.scope === "installed" && removeMe?.target === "global",
      "scope: remove_skill (deletes from the global install) carries scope: installed + target: global",
      JSON.stringify(removeMe));
    const linked = byKey("blocked_symlink", "bee-linked");
    check(linked?.scope === "source",
      "scope: a source-side symlinked skill entry carries scope: source, not installed",
      JSON.stringify(linked));
    const syncHive = byKey("sync_skill", "bee-hive");
    check(syncHive?.scope === "installed",
      "scope: bee-hive's own sync_skill also carries scope: installed",
      JSON.stringify(syncHive));
    const skillStageItems = items.filter((i) =>
      ["sync_skill", "remove_skill", "blocked_symlink", "blocked_alias"].includes(i.action));
    check(skillStageItems.length > 0 && skillStageItems.every((i) =>
      ["repo-claude", "repo-agents", "global"].includes(i.target)),
      "scope: every skill-stage item carries the target-kind discriminator",
      JSON.stringify(skillStageItems));
    const legacyItem = items.find((i) => i.action === "create_agents_block");
    check(legacyItem && !("scope" in legacyItem) && !("target" in legacyItem),
      "scope: legacy repo-relative items carry no scope/target fields at all (unchanged)",
      JSON.stringify(legacyItem));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10z1. host_items: refused-apply payload enumerates the copy_lib/ -------
// copy_helper blast radius a --force-downgrade would overwrite under
// .bee/bin (P49; docs/history/p49-force-downgrade-blast-radius/reports/
// advisor-verdict.md findings 1-3, 5, 7). Follows 10v's three-step shape
// (dry-run enumerates -> refused apply enumerates -> forced apply touches
// exactly the previewed set) but STRENGTHENED: exact normalized
// {action, path} array equality across all three steps (10v's :3311 discards
// fields and tolerates a subset - not repeated here), and the fixture is
// seeded so BOTH action classes fire. makeFakeSkillsRoot vendors every real
// packages/bee/lib/*.mjs (readdirSync) into the fake source but never a
// top-level packages/bee/*.mjs helper, so reusing it unchanged only ever
// exercises copy_lib - the real top-level helper set is vendored here too,
// itself via readdirSync of the real TEMPLATES_DIR (never a hand-kept
// filename list - critical-patterns fixture-list-rot).
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-hostitems-enum-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.18",
    });
    const hiveDir = path.dirname(path.dirname(launcher));
    // D3 (packages-restructure): the fake source's vendored payload lives
    // OUTSIDE the fake skills tree now, at pluginRoot/packages/bee (sibling of
    // skillsRoot) - never nested under hiveDir/templates.
    const fakePluginRoot = path.dirname(path.dirname(hiveDir));
    const fakePackagesBeeDir = path.join(fakePluginRoot, "packages", "bee");
    const realHelperNames = fs
      .readdirSync(TEMPLATES_DIR, { withFileTypes: true })
      .filter((e) => e.isFile() && e.name.endsWith(".mjs"))
      .map((e) => e.name);
    for (const name of realHelperNames) {
      fs.writeFileSync(
        path.join(fakePackagesBeeDir, name),
        fs.readFileSync(path.join(TEMPLATES_DIR, name), "utf8"),
        "utf8",
      );
    }
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", "state.mjs"),
      fakeStateSource("0.1.19"), "utf8"); // newer than source -> forceable downgrade block
    seedRepoSkillTargets(repo, "0.1.19"); // numeric per-target installs keep the refusal forceable

    // No --global-skills here (unlike 10v/10e): the global target's own
    // installed_skills would resolve "absent" while host_helpers (shared,
    // repo-side) reads 0.1.19 > source 0.1.18 - a genuine but unforceable
    // block on that ONE target (not all versions resolved numeric), which
    // would drag the aggregate forceable to false for a reason unrelated to
    // what this case tests. Repo-local targets alone keep the scenario clean.
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "blocked_downgrade",
      "host-items: plan mode reports blocked_downgrade (forceable)",
      `exit ${plan.status} status ${plan.payload?.status}`);

    // Expected set, derived the SAME way onboard derives it (readdirSync,
    // never hardcoded names): every real helper is absent from the repo's
    // .bee/bin, every real lib module besides state.mjs is absent from
    // .bee/bin/lib, and state.mjs itself is version-drifted - so every one
    // of them is a pending copy_helper/copy_lib item.
    const expectedHostItems = [
      ...realHelperNames.slice().sort()
        .map((name) => ({ action: "copy_helper", path: `.bee/bin/${name}` })),
      ...fs.readdirSync(path.join(fakePackagesBeeDir, "lib"), { withFileTypes: true })
        .filter((e) => e.isFile() && e.name.endsWith(".mjs"))
        .map((e) => e.name).sort()
        .map((name) => ({ action: "copy_lib", path: `.bee/bin/lib/${name}` })),
    ];
    check(expectedHostItems.some((i) => i.action === "copy_helper") &&
      expectedHostItems.some((i) => i.action === "copy_lib"),
      "host-items: fixture exercises BOTH copy_lib and copy_helper action classes",
      JSON.stringify([...new Set(expectedHostItems.map((i) => i.action))]));

    const planHostItems = (plan.payload?.plan || [])
      .filter((i) => i.action === "copy_lib" || i.action === "copy_helper")
      .map((i) => ({ action: i.action, path: i.path }));
    check(JSON.stringify(planHostItems) === JSON.stringify(expectedHostItems),
      "host-items: dry-run plan enumerates exactly the expected copy_lib/copy_helper set, order-preserved",
      JSON.stringify({ planHostItems, expectedHostItems }));

    const refused = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(refused.status === 1 && refused.payload?.status === "blocked_downgrade",
      "host-items: refused apply (no --force-downgrade) still reports blocked_downgrade");
    const refusedHostItemsRaw = refused.payload?.host_items;
    const refusedHostItems = (refusedHostItemsRaw || [])
      .map((i) => ({ action: i.action, path: i.path }));
    check(JSON.stringify(refusedHostItems) === JSON.stringify(expectedHostItems),
      "host-items: refused apply's host_items is EXACT normalized {action,path} array equality with the dry-run plan (no subset tolerance, unlike 10v :3311)",
      JSON.stringify({ refusedHostItems, expectedHostItems }));
    check(Array.isArray(refusedHostItemsRaw) && refusedHostItemsRaw.every((i) =>
      Object.keys(i).sort().join(",") === "action,path"),
      "host-items: host items carry NO scope/target fields (advisor finding 7 - lib/helper paths are always repo-root-relative)",
      JSON.stringify(refusedHostItemsRaw));

    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--force-downgrade", "--json"], home);
    check(forced.status === 0 && forced.payload?.status === "applied" &&
      forced.payload?.forced_downgrade === true,
      "host-items: forcing actually applies", `exit ${forced.status} status ${forced.payload?.status}`);
    const appliedHostItems = (forced.payload?.applied || [])
      .filter((i) => i.action === "copy_lib" || i.action === "copy_helper")
      .map((i) => ({ action: i.action, path: i.path }));
    check(JSON.stringify(appliedHostItems) === JSON.stringify(expectedHostItems),
      "host-items: forced apply touches EXACTLY the previewed host_items set - exact equality, no subset tolerance",
      JSON.stringify({ appliedHostItems, expectedHostItems }));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 10z2. host_items: forceable refusal with zero lib/helper drift ---------
// carries host_items: [] (present, empty) - a forceable downgrade can be
// driven purely by a target's own installed_skills version, with the host's
// .bee/bin/lib and .bee/bin already byte-identical to source. Absence would
// be indistinguishable from the non-forceable omission case (10d); presence
// as an empty array tells the operator "nothing to force here" precisely.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-hostitems-empty-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "0.1.18",
    });
    const hiveDir = path.dirname(path.dirname(launcher));
    // D3 (packages-restructure): fake source's vendored payload lives OUTSIDE
    // the fake skills tree now, at pluginRoot/packages/bee.
    const fakePluginRoot = path.dirname(path.dirname(hiveDir));
    const fakeTemplatesDir = path.join(fakePluginRoot, "packages", "bee");
    const fakeTemplatesLibDir = path.join(fakeTemplatesDir, "lib");
    const realHelperNames = fs
      .readdirSync(TEMPLATES_DIR, { withFileTypes: true })
      .filter((e) => e.isFile() && e.name.endsWith(".mjs"))
      .map((e) => e.name);
    for (const name of realHelperNames) {
      fs.writeFileSync(
        path.join(fakeTemplatesDir, name),
        fs.readFileSync(path.join(TEMPLATES_DIR, name), "utf8"),
        "utf8",
      );
    }
    const repo = path.join(base, "repo");
    fs.mkdirSync(path.join(repo, ".bee", "bin", "lib"), { recursive: true });
    // Vendor the repo's .bee/bin + .bee/bin/lib byte-for-byte from the fake
    // SOURCE tree (never the real repo tree, in case the two ever diverge) -
    // zero drift by construction, discovered via readdirSync on each side.
    for (const name of realHelperNames) {
      fs.writeFileSync(path.join(repo, ".bee", "bin", name),
        fs.readFileSync(path.join(fakeTemplatesDir, name), "utf8"), "utf8");
    }
    for (const name of fs.readdirSync(fakeTemplatesLibDir, { withFileTypes: true })
      .filter((e) => e.isFile() && e.name.endsWith(".mjs"))
      .map((e) => e.name)) {
      fs.writeFileSync(path.join(repo, ".bee", "bin", "lib", name),
        fs.readFileSync(path.join(fakeTemplatesLibDir, name), "utf8"), "utf8");
    }
    // The block itself comes solely from the in-repo targets' own
    // installed_skills version being newer than source - host_helpers stays
    // equal to source (0.1.18), so hostLibDowngradeBlock contributes nothing.
    seedRepoSkillTargets(repo, "0.1.19");

    const refused = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(refused.status === 1 && refused.payload?.status === "blocked_downgrade",
      "host-items-empty: refused apply reports blocked_downgrade (forceable, target-version-driven)",
      `exit ${refused.status} status ${refused.payload?.status}`);
    check(refused.payload?.host_items !== undefined && Array.isArray(refused.payload?.host_items) &&
      refused.payload.host_items.length === 0,
      "host-items-empty: forceable refusal with zero lib/helper drift carries host_items: [] (present, empty)",
      JSON.stringify(refused.payload?.host_items));

    const forced = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--force-downgrade", "--json"], home);
    check(forced.status === 0 && forced.payload?.status === "applied" &&
      forced.payload?.forced_downgrade === true,
      "host-items-empty: forcing still applies (the skill-target downgrade)",
      `exit ${forced.status} status ${forced.payload?.status}`);
    const appliedHostItems = (forced.payload?.applied || [])
      .filter((i) => i.action === "copy_lib" || i.action === "copy_helper");
    check(appliedHostItems.length === 0,
      "host-items-empty: forced apply touches no copy_lib/copy_helper items either (nothing was ever drifted)",
      JSON.stringify(appliedHostItems));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 11. sticky repo-hooks opt-in -------------------------------------------
// The bug this pins: --repo-hooks used to be re-consent owed on EVERY upgrade.
// A bare --apply refreshed the doctrine block, helpers, and version stamp while
// leaving first-onboard guards vendored in .bee/bin/hooks/ — and still reported
// up_to_date, because subsetManaged() ignores repo_hooks when the flag is absent.
// Eight host repos ran current doctrine against stale guards for many versions.
// The opt-in is now sticky: the recorded repo_hooks marker keeps upgrades honest.
{
  const stickyTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-sticky-hooks-"));

  // (a) opted in once -> a BARE --apply refreshes hooks, no flag needed
  {
    const repo = path.join(stickyTmp, "opted-in");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    await runOnboard(["--repo-root", repo, "--apply", "--repo-hooks", "--json"], home);
    const hooksDir = path.join(repo, ".bee", "bin", "hooks");
    check(listMjs(hooksDir).length > 0, "sticky: first --repo-hooks install vendors hooks");

    // Simulate the stale state the real hosts were in: gut a vendored guard, then
    // upgrade WITHOUT the flag. Pre-fix, the corruption survived and status said up_to_date.
    const guard = path.join(hooksDir, "bee-write-guard.mjs");
    fs.writeFileSync(guard, "// stale first-onboard guard\n");
    const bare = await runOnboard(["--repo-root", repo, "--apply", "--json"], home);

    const refreshed = fs.readFileSync(guard, "utf8");
    check(!refreshed.includes("stale first-onboard guard") && refreshed.length > 200,
      "sticky: a bare --apply refreshes vendored hooks once the repo has opted in",
      `guard length after: ${refreshed.length}`);
    check(bare.payload?.status === "applied", "sticky: the bare upgrade still applies cleanly",
      JSON.stringify(bare.payload?.status));
  }

  // (b) never opted in -> a bare --apply must NOT start vendoring hooks
  {
    const repo = path.join(stickyTmp, "never-opted-in");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    await runOnboard(["--repo-root", repo, "--apply", "--json"], home);
    await runOnboard(["--repo-root", repo, "--apply", "--json"], home);

    check(listMjs(path.join(repo, ".bee", "bin", "hooks")).length === 0,
      "sticky: a repo that never opted in is never silently given vendored hooks");
  }

  // (c) a hook file ADDED to source since the last onboard is picked up on the sticky path
  //     (this is exactly how adapter.mjs and bee-model-guard.mjs went missing on all 8 hosts)
  {
    const repo = path.join(stickyTmp, "new-hook-appears");
    fs.mkdirSync(repo, { recursive: true });
    const home = makeFakeHome();

    await runOnboard(["--repo-root", repo, "--apply", "--repo-hooks", "--json"], home);
    const hooksDir = path.join(repo, ".bee", "bin", "hooks");
    const before = listMjs(hooksDir);

    // Delete one, as if it had never existed in the older source the repo onboarded from.
    const victim = before.includes("bee-model-guard.mjs") ? "bee-model-guard.mjs" : before[0];
    fs.rmSync(path.join(hooksDir, victim));
    check(!listMjs(hooksDir).includes(victim), "sticky: precondition — hook removed from the repo");

    await runOnboard(["--repo-root", repo, "--apply", "--json"], home);
    check(listMjs(hooksDir).includes(victim),
      "sticky: a hook missing from the repo is restored by a bare --apply (new source hooks land)",
      `missing: ${victim}`);
  }

  fs.rmSync(stickyTmp, { recursive: true, force: true });
}

// --- 12. encoding regression guard: scripts/*.ps1 must be ASCII-only --------
// Root cause (installer-hardening CONTEXT.md E2): scripts/install.ps1 was
// UTF-8 without BOM; Windows PowerShell 5.1 decodes it as cp1252, so the
// em-dash byte sequence's trailing 0x94 became a smart right-double-quote,
// which PowerShell treats as a string terminator and the file failed to
// parse. Keeping scripts/*.ps1 ASCII-only makes the file encoding-proof.
// This check is pure node fs + a byte scan, no pwsh required, so it runs on
// any platform.
{
  function findNonAsciiBytes(buf) {
    const hits = [];
    for (let i = 0; i < buf.length; i++) {
      if (buf[i] > 0x7f) hits.push({ offset: i, byte: buf[i] });
    }
    return hits;
  }

  const scriptsDir = path.join(REPO_ROOT, "scripts");
  const ps1Files = fs.existsSync(scriptsDir)
    ? fs.readdirSync(scriptsDir).filter((f) => f.endsWith(".ps1")).sort()
    : [];
  check(ps1Files.length > 0, "encoding guard: scripts/*.ps1 files found to check",
    `dir: ${scriptsDir}, files: ${JSON.stringify(ps1Files)}`);

  const violations = [];
  for (const name of ps1Files) {
    const hits = findNonAsciiBytes(fs.readFileSync(path.join(scriptsDir, name)));
    if (hits.length > 0) {
      violations.push({ file: name, count: hits.length, first: hits[0] });
    }
  }
  check(violations.length === 0,
    "encoding guard: no non-ASCII bytes in any scripts/*.ps1",
    JSON.stringify(violations));

  // Self-test the detector on a scratch file OUTSIDE the repo's scripts/ dir,
  // so a vacuously-true check (e.g. an empty glob) cannot pass silently.
  const scratchDir = fs.mkdtempSync(path.join(os.tmpdir(), "bee-encoding-guard-"));
  try {
    const scratchFile = path.join(scratchDir, "scratch.ps1");
    fs.writeFileSync(scratchFile, "Write-Host 'em dash — here'\n", "utf8");
    const scratchHits = findNonAsciiBytes(fs.readFileSync(scratchFile));
    check(scratchHits.length > 0,
      "encoding guard: detector flags a non-ASCII byte planted in a scratch .ps1",
      JSON.stringify(scratchHits));

    const cleanFile = path.join(scratchDir, "clean.ps1");
    fs.writeFileSync(cleanFile, "Write-Host 'plain ascii'\n", "utf8");
    const cleanHits = findNonAsciiBytes(fs.readFileSync(cleanFile));
    check(cleanHits.length === 0,
      "encoding guard: detector reports clean on an ASCII-only scratch .ps1",
      JSON.stringify(cleanHits));
  } finally {
    fs.rmSync(scratchDir, { recursive: true, force: true });
  }
}

// --- 13. retired helper shims (D2, shim-retire): removal pass ---------------
// bbc6bcea D2: a host with leftover bee_*.mjs shims in its own .bee/bin/ gets
// them removed on the next --apply; a second run plans nothing further; a
// brand-new host never sees them appear in the first place (listTemplateHelpers()
// already stopped copying them once shim-retire-1 deleted the source packages/bee
// - this section only proves the explicit *removal* pass, which is the part
// that would otherwise never happen for an already-onboarded host).
const RETIRED_HELPER_NAMES = [
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

{
  const staleTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-onboard-stale-shims-"));
  const staleHome = makeFakeHome();
  try {
    fs.mkdirSync(path.join(staleTmp, ".bee", "bin"), { recursive: true });
    for (const name of RETIRED_HELPER_NAMES) {
      fs.writeFileSync(path.join(staleTmp, ".bee", "bin", name), "// legacy shim\n", "utf8");
    }
    // A live, non-retired file in the same directory must never be touched -
    // the removal pass targets exact retired basenames only (must_haves
    // prohibition: never a generic rm of unmanaged files).
    fs.writeFileSync(path.join(staleTmp, ".bee", "bin", "bee.mjs"), "// dispatcher\n", "utf8");

    const stalePlan = await runOnboard(["--repo-root", staleTmp, "--json"], staleHome);
    const stalePlanActions = (stalePlan.payload?.plan || [])
      .filter((i) => i.action === "remove_helper")
      .map((i) => i.path)
      .sort();
    check(
      JSON.stringify(stalePlanActions) ===
        JSON.stringify(RETIRED_HELPER_NAMES.map((n) => `.bee/bin/${n}`).sort()),
      "stale host: plan lists a remove_helper item for every leftover retired shim",
      JSON.stringify(stalePlanActions),
    );
    check(fs.existsSync(path.join(staleTmp, ".bee", "bin", "bee_status.mjs")),
      "stale host: plan mode writes nothing (shim still on disk before apply)");

    const staleApply = await runOnboard(["--repo-root", staleTmp, "--apply", "--json"], staleHome);
    check(staleApply.payload?.status === "applied", "stale host: apply succeeds", staleApply.stderr);
    for (const name of RETIRED_HELPER_NAMES) {
      check(!fs.existsSync(path.join(staleTmp, ".bee", "bin", name)),
        `stale host: ${name} deleted from .bee/bin on --apply`);
    }
    check(fs.existsSync(path.join(staleTmp, ".bee", "bin", "bee.mjs")),
      "stale host: an unrelated .bee/bin/bee.mjs survives the removal pass untouched");

    // --- idempotence: a second run plans zero remove_helper items -----------
    const staleReplan = await runOnboard(["--repo-root", staleTmp, "--json"], staleHome);
    const replanActions = (staleReplan.payload?.plan || [])
      .filter((i) => i.action === "remove_helper");
    check(replanActions.length === 0,
      "idempotence: re-running onboarding after removal plans zero remove_helper items",
      JSON.stringify(replanActions));
    check(staleReplan.payload?.status === "up_to_date",
      "idempotence: repo with shims already removed reports up_to_date (no residual drift)",
      JSON.stringify(staleReplan.payload));
  } finally {
    fs.rmSync(staleTmp, { recursive: true, force: true });
    fs.rmSync(staleHome, { recursive: true, force: true });
  }
}

// --- fresh onboard: a brand-new host never gets the retired shims ----------
{
  const freshTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-onboard-fresh-noshims-"));
  const freshHome = makeFakeHome();
  try {
    const freshApply = await runOnboard(["--repo-root", freshTmp, "--apply", "--json"], freshHome);
    check(freshApply.payload?.status === "applied", "fresh onboard: apply succeeds", freshApply.stderr);
    const binNames = listMjs(path.join(freshTmp, ".bee", "bin"));
    const leftoverShims = binNames.filter((n) => RETIRED_HELPER_NAMES.includes(n));
    check(leftoverShims.length === 0,
      "fresh onboard: a brand-new host's .bee/bin ends with no retired bee_*.mjs shims",
      JSON.stringify(binNames));
    check(binNames.includes("bee.mjs"),
      "fresh onboard: .bee/bin/bee.mjs (the sole dispatcher) is vendored");
  } finally {
    fs.rmSync(freshTmp, { recursive: true, force: true });
    fs.rmSync(freshHome, { recursive: true, force: true });
  }
}

// --- retired lib module removal (self-derived from the ledger diff) --------
// A lib module the recorded ledger's managed.lib still names, but current
// source packages/bee/lib no longer has, is an orphan on every host that
// installed it before the removal: copy_lib only ever adds/updates names the
// source still has, so nothing before this removed the stale file, and
// computeRuntimeDrift (bee.mjs) flags it as "(extra)" forever. Unlike
// RETIRED_HELPERS (a hand-maintained allowlist), this removal is derived from
// the ledger diff itself - no future lib retirement needs its own entry here.
{
  const libTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-onboard-stale-lib-"));
  const libHome = makeFakeHome();
  try {
    const firstApply = await runOnboard(["--repo-root", libTmp, "--apply", "--json"], libHome);
    check(firstApply.payload?.status === "applied", "stale lib: initial onboard applies", firstApply.stderr);

    // Simulate a host onboarded while a since-removed lib module still shipped:
    // the ledger records it (arbitrary hash - only presence-as-a-key matters)
    // and the installed copy sits on disk, exactly as copy_lib would have left
    // it before the module was retired from source.
    const onboardingPath = path.join(libTmp, ".bee", "onboarding.json");
    const onboarding = JSON.parse(fs.readFileSync(onboardingPath, "utf8"));
    onboarding.managed.lib["zz-test-retired-lib.mjs"] = "0".repeat(64);
    fs.writeFileSync(onboardingPath, `${JSON.stringify(onboarding, null, 2)}\n`, "utf8");
    fs.writeFileSync(
      path.join(libTmp, ".bee", "bin", "lib", "zz-test-retired-lib.mjs"),
      "// orphaned lib module\n",
      "utf8",
    );

    const stalePlan = await runOnboard(["--repo-root", libTmp, "--json"], libHome);
    const removeLibItems = (stalePlan.payload?.plan || []).filter((i) => i.action === "remove_lib");
    check(
      JSON.stringify(removeLibItems.map((i) => i.path)) ===
        JSON.stringify([".bee/bin/lib/zz-test-retired-lib.mjs"]),
      "stale lib: plan lists a remove_lib item for the orphaned module only",
      JSON.stringify(removeLibItems),
    );
    check(
      fs.existsSync(path.join(libTmp, ".bee", "bin", "lib", "zz-test-retired-lib.mjs")),
      "stale lib: plan mode writes nothing (orphan still on disk before apply)",
    );

    const staleApply = await runOnboard(["--repo-root", libTmp, "--apply", "--json"], libHome);
    check(staleApply.payload?.status === "applied", "stale lib: apply succeeds", staleApply.stderr);
    check(
      !fs.existsSync(path.join(libTmp, ".bee", "bin", "lib", "zz-test-retired-lib.mjs")),
      "stale lib: orphaned module deleted from .bee/bin/lib on --apply",
    );
    const survivingLibNames = listMjs(path.join(libTmp, ".bee", "bin", "lib"));
    check(
      survivingLibNames.length > 0 && !survivingLibNames.includes("zz-test-retired-lib.mjs"),
      "stale lib: current lib modules survive the removal pass untouched",
      JSON.stringify(survivingLibNames),
    );

    // --- idempotence: a second run plans zero remove_lib items -------------
    const libReplan = await runOnboard(["--repo-root", libTmp, "--json"], libHome);
    const replanLibItems = (libReplan.payload?.plan || []).filter((i) => i.action === "remove_lib");
    check(replanLibItems.length === 0,
      "idempotence: re-running onboarding after lib removal plans zero remove_lib items",
      JSON.stringify(replanLibItems));
    check(libReplan.payload?.status === "up_to_date",
      "idempotence: repo with orphaned lib module already removed reports up_to_date",
      JSON.stringify(libReplan.payload));
  } finally {
    fs.rmSync(libTmp, { recursive: true, force: true });
    fs.rmSync(libHome, { recursive: true, force: true });
  }
}

// --- legacy-global version-parity refresh (installer-version-parity-1-3-1) ---
// WITHOUT --global-skills, a managed skill that ALREADY EXISTS under the legacy
// global ~/.claude/skills root is refreshed in place to current source content
// (action refresh_legacy_global_skill). A managed skill ABSENT there is never
// created; a non-managed dir (bee-custom, foreign) is never touched; nothing is
// ever deleted; and the parity pass never flips drift/up_to_date once applied.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-legacy-refresh-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "1.0.0",
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha CURRENT\n" },
        "bee-beta": { "SKILL.md": "# beta CURRENT\n" },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    // Stale legacy global install: OLD bee-hive + OLD bee-alpha content, NO
    // bee-beta at all, plus a non-managed bee-custom skill and a foreign
    // (non-bee) dir that must both survive byte-identical.
    const installedRoot = makeInstalledSkills(home, {
      version: "0.1.42", // pre-1.0 global install (older than the 1.0.0 source)
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha STALE 0.1.42\n" },
        "bee-custom": { "SKILL.md": "# user's own skill, not managed\n" },
      },
    });
    writeSkillFiles(installedRoot, "agent-browser", {
      "SKILL.md": "# foreign non-bee skill\n",
      "references/keep.md": "precious\n",
    });
    const customBefore = hashTree(path.join(installedRoot, "bee-custom"));
    const foreignBefore = hashTree(path.join(installedRoot, "agent-browser"));

    // (1) plan mode lists refresh_legacy_global_skill for the EXISTING managed
    //     skills (bee-hive, bee-alpha) and never a create for the absent bee-beta.
    const plan = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    check(plan.status === 0 && plan.payload?.status === "changes_needed",
      "legacy refresh: plan reports changes_needed (stale global drift present)",
      `exit ${plan.status} status ${plan.payload?.status}`);
    const refreshItems = (plan.payload?.plan || [])
      .filter((i) => i.action === "refresh_legacy_global_skill")
      .map((i) => i.skill).sort();
    check(JSON.stringify(refreshItems) === JSON.stringify(["bee-alpha", "bee-hive"]),
      "legacy refresh: plan refreshes exactly the managed skills already present in the global root",
      JSON.stringify(refreshItems));
    check((plan.payload?.plan || []).every(
      (i) => i.target !== "legacy-global" || i.skill !== "bee-beta"),
      "legacy refresh: a managed skill absent from the global root is never planned (never created)");
    check(!(plan.payload?.plan || []).some(
      (i) => i.action === "remove_skill" && i.target === "legacy-global"),
      "legacy refresh: never a remove_skill item against the legacy global root");
    check(readInstalled(home, "bee-alpha/SKILL.md") === "# alpha STALE 0.1.42\n",
      "legacy refresh: plan mode mutates nothing in the global root");

    // (2) apply refreshes the stale copies to source content, distinctly labeled.
    const apply = await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "legacy refresh: apply succeeds", `exit ${apply.status}`);
    const applied = (apply.payload?.applied || [])
      .filter((i) => i.action === "refresh_legacy_global_skill").map((i) => i.skill).sort();
    check(JSON.stringify(applied) === JSON.stringify(["bee-alpha", "bee-hive"]),
      "legacy refresh: apply output labels the refresh action distinctly",
      JSON.stringify(applied));
    check(readInstalled(home, "bee-alpha/SKILL.md") === "# alpha CURRENT\n",
      "legacy refresh: stale bee-alpha refreshed to current source content",
      String(readInstalled(home, "bee-alpha/SKILL.md")));
    // D1 (packages-restructure): skills are instruction-only - there is no
    // version marker left inside a synced bee-hive dir to compare (the
    // legacy-global refresh path is a side channel that predates
    // SKILLS_VERSION_STAMP and stays out of scope for it here). Content
    // parity against the fake source's OWN bee-hive dir is the honest
    // structural proof that the refresh actually landed current content.
    const sourceHiveDir = path.dirname(path.dirname(launcher));
    check(
      hashTree(path.join(installedRoot, "bee-hive")) === hashTree(sourceHiveDir),
      "legacy refresh: stale bee-hive marker refreshed to the current source version",
    );

    // (3) never created: absent bee-beta stays absent in the global root.
    check(!fs.existsSync(path.join(installedRoot, "bee-beta")),
      "legacy refresh: a managed skill absent from the global root is never created");

    // (4) never touched: non-managed bee-custom + foreign agent-browser survive.
    check(hashTree(path.join(installedRoot, "bee-custom")) === customBefore,
      "legacy refresh: non-managed bee-custom skill survives byte-identical");
    check(hashTree(path.join(installedRoot, "agent-browser")) === foreignBefore,
      "legacy refresh: foreign non-bee dir survives byte-identical");

    // (5) drift/up_to_date consistency: repeat onboard is up_to_date, no refresh.
    check(apply.payload?.recheck === "up_to_date",
      "legacy refresh: recheck lands up_to_date after refresh",
      JSON.stringify(apply.payload?.recheck_plan || []));
    const replan = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    check(replan.payload?.status === "up_to_date" &&
      !(replan.payload?.plan || []).some((i) => i.action === "refresh_legacy_global_skill"),
      "legacy refresh: a refreshed global never re-drifts (repeat onboard up_to_date, zero refresh items)",
      JSON.stringify(replan.payload?.plan || []));

    // (6) core-parity repo + a freshly-stale global: the legacy refresh is a
    //     side pass - it is listed (and would be applied) but NEVER flips the
    //     repo's up_to_date status to changes_needed (mirrors the un-isolated
    //     installer recheck against a stale real ~/.claude/skills).
    fs.writeFileSync(path.join(installedRoot, "bee-alpha", "SKILL.md"), "# alpha re-STALED\n", "utf8");
    const sideplan = await runOnboardAt(launcher, ["--repo-root", repo, "--json"], home);
    check(sideplan.payload?.status === "up_to_date",
      "legacy refresh: a stale global never flips a core-parity repo to changes_needed (side pass)",
      JSON.stringify(sideplan.payload?.status));
    check((sideplan.payload?.plan || []).some(
      (i) => i.action === "refresh_legacy_global_skill" && i.skill === "bee-alpha"),
      "legacy refresh: the stale global skill is still LISTED in the plan for transparency",
      JSON.stringify(sideplan.payload?.plan || []));
    await runOnboardAt(launcher, ["--repo-root", repo, "--apply", "--json"], home);
    check(readInstalled(home, "bee-alpha/SKILL.md") === "# alpha CURRENT\n",
      "legacy refresh: --apply still refreshes the freshly-stale global even while status read up_to_date",
      String(readInstalled(home, "bee-alpha/SKILL.md")));
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- legacy refresh: --global-skills path is unchanged (full management) -----
// WITH the flag the global root is a fully managed target: an absent managed
// skill IS created there and items are labeled sync_skill (target global), never
// refresh_legacy_global_skill.
{
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "bee-legacy-refresh-gs-"));
  const home = makeFakeHome();
  try {
    const { launcher } = makeFakeSkillsRoot(path.join(base, "skills"), {
      version: "1.0.0",
      skills: {
        "bee-alpha": { "SKILL.md": "# alpha CURRENT\n" },
        "bee-beta": { "SKILL.md": "# beta CURRENT\n" },
      },
    });
    const repo = path.join(base, "repo");
    fs.mkdirSync(repo, { recursive: true });
    makeInstalledSkills(home, {
      version: "0.1.42",
      skills: { "bee-alpha": { "SKILL.md": "# alpha STALE\n" } },
    });
    const apply = await runOnboardAt(launcher,
      ["--repo-root", repo, "--apply", "--global-skills", "--json"], home);
    check(apply.status === 0 && apply.payload?.status === "applied",
      "legacy refresh (--global-skills): apply succeeds", `exit ${apply.status}`);
    check(!(apply.payload?.applied || []).some(
      (i) => i.action === "refresh_legacy_global_skill"),
      "legacy refresh (--global-skills): never emits refresh_legacy_global_skill (full-management path unchanged)");
    check(readInstalled(home, "bee-beta/SKILL.md") === "# beta CURRENT\n",
      "legacy refresh (--global-skills): an absent managed skill IS created in the global root (unchanged)");
    check(readInstalled(home, "bee-alpha/SKILL.md") === "# alpha CURRENT\n",
      "legacy refresh (--global-skills): stale global skill synced to source (unchanged)");
  } finally {
    try {
      fs.rmSync(base, { recursive: true, force: true });
      fs.rmSync(home, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 14. onboarding-generator drift check (H1) ------------------------------
// hooks/catalog.mjs is the single logical source of truth for bee's hook set
// (its own header). Three onboarding generators mirror it by hand: the
// vendored-hook list (HOOK_FILENAMES, copied into .bee/bin/hooks/), the
// Claude settings template (renderRepoHookEntries) and the Codex
// repo-projection template (renderCodexHookEntries). A hook added to the
// catalog without teaching all three generators is exactly the clobber class
// from learnings/20260717-guard-membership-escape-routes.md Addendum 2 (P2
// friction): the checked-in plugin projections self-correct via the
// hooks/test_hook_contracts.mjs drift row, but these hand-authored generator
// templates silently lagged and a live settings edit was clobbered by
// self-onboard. This row fails RED naming the missing hook and the lagging
// inventory, the moment a generator falls behind the catalog.
//
// Every side is read from what onboarding actually RENDERS into a fresh
// fixture (never parsed from source text), so this row survives refactors of
// the generator functions' internal shape. Per-runtime catalog sets come
// straight from hooks/catalog.mjs renderProjection(), which already filters
// per runtime (catalog.mjs:224) — no ALLOWED_DIFFERENCES arithmetic needed.
{
  function scriptNameFromCommand(command) {
    const m = String(command || "").match(/([A-Za-z0-9_.-]+\.mjs)/);
    return m ? m[1] : null;
  }
  function scriptNamesFromHooksObject(hooksObj) {
    const names = new Set();
    for (const entries of Object.values(hooksObj || {})) {
      for (const entry of Array.isArray(entries) ? entries : []) {
        for (const hook of entry.hooks || []) {
          const name = scriptNameFromCommand(hook.command);
          if (name) names.add(name);
        }
      }
    }
    return names;
  }

  const catalogModulePath = path.join(REPO_ROOT, "hooks", "catalog.mjs");
  const { renderProjection, RUNTIMES } = await import(pathToFileURL(catalogModulePath).href);
  const claudeCatalogNames = scriptNamesFromHooksObject(renderProjection(RUNTIMES.CLAUDE).hooks);
  const codexCatalogNames = scriptNamesFromHooksObject(renderProjection(RUNTIMES.CODEX).hooks);
  const allCatalogNames = new Set([...claudeCatalogNames, ...codexCatalogNames]);

  const driftTmp = fs.mkdtempSync(path.join(os.tmpdir(), "bee-onboard-drift-"));
  const driftHome = makeFakeHome();
  try {
    await runOnboard(["--repo-root", driftTmp, "--apply", "--repo-hooks", "--json"], driftHome);

    // generator 1: HOOK_FILENAMES (vendored-hook list) must cover every
    // catalog script, either runtime — it is the single dir both runtime
    // projections' commands resolve against on a host repo.
    const hooksDir = path.join(driftTmp, ".bee", "bin", "hooks");
    const vendoredNames = new Set(fs.existsSync(hooksDir) ? fs.readdirSync(hooksDir) : []);
    const missingFromVendored = [...allCatalogNames].filter((n) => !vendoredNames.has(n));
    check(missingFromVendored.length === 0,
      "vendored-hook list (HOOK_FILENAMES) covers every hooks/catalog.mjs script",
      JSON.stringify({ missingFromVendored }));

    // generator 2: renderRepoHookEntries (Claude settings template).
    const driftSettingsPath = path.join(driftTmp, ".claude", "settings.json");
    const driftSettings = fs.existsSync(driftSettingsPath)
      ? JSON.parse(fs.readFileSync(driftSettingsPath, "utf8"))
      : { hooks: {} };
    const claudeGeneratorNames = scriptNamesFromHooksObject(driftSettings.hooks);
    const missingFromClaudeGenerator =
      [...claudeCatalogNames].filter((n) => !claudeGeneratorNames.has(n));
    check(missingFromClaudeGenerator.length === 0,
      "Claude settings template (renderRepoHookEntries) covers every hooks/catalog.mjs claude-runtime script",
      JSON.stringify({ missingFromClaudeGenerator }));

    // generator 3: renderCodexHookEntries (Codex repo-projection template).
    const driftCodexPath = path.join(driftTmp, ".codex", "hooks.json");
    const driftCodex = fs.existsSync(driftCodexPath)
      ? JSON.parse(fs.readFileSync(driftCodexPath, "utf8"))
      : { hooks: {} };
    const codexGeneratorNames = scriptNamesFromHooksObject(driftCodex.hooks);
    const missingFromCodexGenerator =
      [...codexCatalogNames].filter((n) => !codexGeneratorNames.has(n));
    check(missingFromCodexGenerator.length === 0,
      "Codex repo-projection template (renderCodexHookEntries) covers every hooks/catalog.mjs codex-runtime script",
      JSON.stringify({ missingFromCodexGenerator }));
  } finally {
    try {
      fs.rmSync(driftTmp, { recursive: true, force: true });
      fs.rmSync(driftHome, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

// --- 15. worktree-local coordination migration (msn-18d, advisor condition
// F4): a linked worktree carrying pre-existing worktree-local sessions/
// claims/runtime records (stranded by the msn-18a-c control-plane re-root
// onto controlRootFor(root)) must MIGRATE them into main's control store, or
// FAIL LOUD naming every conflicting record — never silently orphan them.
// Same lightweight manual .git-worktree-marker fixture test_state.mjs's
// resolveContext tests use (main .git/worktrees/<id> + reverse gitdir
// pointer under work/.git) — no real `git worktree add` spawn needed.
{
  function makeLinkedWorktreeFixture(prefix) {
    const main = fs.mkdtempSync(path.join(os.tmpdir(), `bee-msn18d-main-${prefix}-`));
    const work = fs.mkdtempSync(path.join(os.tmpdir(), `bee-msn18d-work-${prefix}-`));
    fs.mkdirSync(path.join(main, ".git"));
    const id = `msn18d-${prefix}-${crypto.randomBytes(4).toString("hex")}`;
    const gitdir = path.join(main, ".git", "worktrees", id);
    fs.mkdirSync(gitdir, { recursive: true });
    fs.writeFileSync(path.join(work, ".git"), `gitdir: ${gitdir}\n`);
    fs.writeFileSync(path.join(gitdir, "gitdir"), `${path.join(work, ".git")}\n`);
    return { main, work, id };
  }

  function writeJson(filePath, value) {
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
  }

  function cleanupFixture(...dirs) {
    for (const dir of dirs) {
      try {
        fs.rmSync(dir, { recursive: true, force: true });
      } catch {
        // best-effort cleanup
      }
    }
  }

  // --- 15a. migrate happy path: a live worktree-local claim + session
  // migrate idempotently into main's store; local copies removed. ----------
  {
    const { main, work } = makeLinkedWorktreeFixture("happy");
    const home = makeFakeHome();
    const sessionId = "sess-happy-1";
    const cellId = "cell-happy-1";
    const sessionRecord = { id: sessionId, started_at: "2026-07-01T00:00:00.000Z", last_heartbeat: "2026-07-01T00:00:00.000Z" };
    const claimRecord = { cell: cellId, session: sessionId, epoch: 1 };
    writeJson(path.join(work, ".bee", "sessions", `${sessionId}.json`), sessionRecord);
    writeJson(path.join(work, ".bee", "claims", `${cellId}.json`), claimRecord);

    try {
      const plan = await runOnboard(["--repo-root", work, "--json"], home);
      check(plan.payload?.status === "changes_needed",
        "msn-18d happy path: plan mode reports changes_needed (pending migration + fresh onboard)",
        JSON.stringify(plan.payload?.status));
      const planActions = (plan.payload?.plan || []).map((i) => i.action);
      check(planActions.includes("migrate_worktree_records"),
        "msn-18d happy path: plan lists migrate_worktree_records",
        JSON.stringify(planActions));
      check(plan.payload?.worktree_migration?.blocked === false &&
        plan.payload?.worktree_migration?.pending === 2,
        "msn-18d happy path: worktree_migration reports 2 pending, not blocked",
        JSON.stringify(plan.payload?.worktree_migration));

      const apply = await runOnboard(["--repo-root", work, "--apply", "--json"], home);
      check(apply.payload?.status === "applied", "msn-18d happy path: apply succeeds", JSON.stringify(apply.payload));
      const appliedActions = (apply.payload?.applied || []).map((i) => i.action);
      check(appliedActions.includes("migrate_worktree_records"),
        "msn-18d happy path: applied list includes migrate_worktree_records",
        JSON.stringify(appliedActions));

      const mainSessionPath = path.join(main, ".bee", "sessions", `${sessionId}.json`);
      const mainClaimPath = path.join(main, ".bee", "claims", `${cellId}.json`);
      const workSessionPath = path.join(work, ".bee", "sessions", `${sessionId}.json`);
      const workClaimPath = path.join(work, ".bee", "claims", `${cellId}.json`);
      check(fs.existsSync(mainSessionPath), "msn-18d happy path: session record now lives in main's store");
      check(fs.existsSync(mainClaimPath), "msn-18d happy path: claim record now lives in main's store");
      check(!fs.existsSync(workSessionPath), "msn-18d happy path: local session copy removed");
      check(!fs.existsSync(workClaimPath), "msn-18d happy path: local claim copy removed");
      check(JSON.parse(fs.readFileSync(mainSessionPath, "utf8")).id === sessionId,
        "msn-18d happy path: migrated session content preserved");
      check(JSON.parse(fs.readFileSync(mainClaimPath, "utf8")).session === sessionId,
        "msn-18d happy path: migrated claim content preserved");

      // --- 15c. re-run is a no-op (idempotency): no local copies left to
      // find, so a second run neither re-migrates nor reports blocked. -----
      const replan = await runOnboard(["--repo-root", work, "--json"], home);
      const replanActions = (replan.payload?.plan || []).map((i) => i.action);
      check(!replanActions.includes("migrate_worktree_records"),
        "msn-18d re-run: no more migration items once local copies are gone",
        JSON.stringify(replanActions));
      check(replan.payload?.worktree_migration?.pending === 0 &&
        replan.payload?.worktree_migration?.blocked === false,
        "msn-18d re-run: worktree_migration reports zero pending, still not blocked",
        JSON.stringify(replan.payload?.worktree_migration));
    } finally {
      cleanupFixture(main, work, home);
    }
  }

  // --- 15b. conflict variant: main already holds a DIFFERENT record under
  // the same id -> loud abort, zero mutations anywhere in the whole apply
  // (not just the migration). ------------------------------------------------
  {
    const { main, work } = makeLinkedWorktreeFixture("conflict");
    const home = makeFakeHome();
    const sessionId = "sess-conflict-1";
    const localRecord = { id: sessionId, started_at: "2026-07-01T00:00:00.000Z", last_heartbeat: "2026-07-01T00:00:00.000Z" };
    const mainRecord = { id: sessionId, started_at: "2026-07-01T00:00:00.000Z", last_heartbeat: "2026-07-02T09:00:00.000Z" };
    const workSessionPath = path.join(work, ".bee", "sessions", `${sessionId}.json`);
    const mainSessionPath = path.join(main, ".bee", "sessions", `${sessionId}.json`);
    writeJson(workSessionPath, localRecord);
    writeJson(mainSessionPath, mainRecord);

    try {
      const plan = await runOnboard(["--repo-root", work, "--json"], home);
      check(plan.payload?.status === "blocked_worktree_migration_conflict",
        "msn-18d conflict: plan mode reports blocked_worktree_migration_conflict",
        JSON.stringify(plan.payload?.status));
      check(plan.payload?.worktree_migration?.blocked === true &&
        Array.isArray(plan.payload?.worktree_migration?.stranded) &&
        plan.payload.worktree_migration.stranded.length === 1 &&
        plan.payload.worktree_migration.stranded[0].id === sessionId &&
        plan.payload.worktree_migration.stranded[0].kind === "session",
        "msn-18d conflict: worktree_migration names the exact stranded record",
        JSON.stringify(plan.payload?.worktree_migration));
      check(typeof plan.payload?.reason === "string" && plan.payload.reason.includes(sessionId),
        "msn-18d conflict: plan reason names the stranded record id", plan.payload?.reason);

      const apply = await runOnboard(["--repo-root", work, "--apply", "--json"], home);
      check(apply.status === 1, "msn-18d conflict: apply exits nonzero", JSON.stringify(apply.payload));
      check(apply.payload?.status === "blocked_worktree_migration_conflict",
        "msn-18d conflict: apply refusal reports blocked_worktree_migration_conflict",
        JSON.stringify(apply.payload?.status));
      check(apply.payload?.worktree_migration?.stranded?.length === 1,
        "msn-18d conflict: apply refusal names the stranded record too",
        JSON.stringify(apply.payload?.worktree_migration));

      // Zero mutations anywhere: neither copy touched, nothing else onboarded.
      check(JSON.stringify(JSON.parse(fs.readFileSync(workSessionPath, "utf8"))) === JSON.stringify(localRecord),
        "msn-18d conflict: local record left byte-for-byte untouched");
      check(JSON.stringify(JSON.parse(fs.readFileSync(mainSessionPath, "utf8"))) === JSON.stringify(mainRecord),
        "msn-18d conflict: main's conflicting record left byte-for-byte untouched (never overwritten)");
      check(!fs.existsSync(path.join(work, "AGENTS.md")),
        "msn-18d conflict: the ENTIRE apply is refused, not just the migration (AGENTS.md never written)");
      check(!fs.existsSync(path.join(work, ".bee", "onboarding.json")),
        "msn-18d conflict: onboarding.json never written on a migration-conflict refusal");
    } finally {
      cleanupFixture(main, work, home);
    }
  }

  // --- 15d. identical-content duplicate: already-migrated -> local copy
  // removed silently, main's copy never rewritten. --------------------------
  {
    const { main, work } = makeLinkedWorktreeFixture("dup");
    const home = makeFakeHome();
    const cellId = "cell-dup-1";
    // Same logical content, deliberately different KEY ORDER — proves the
    // duplicate check is structural (canonicalJson), not a raw byte compare.
    const workClaimPath = path.join(work, ".bee", "claims", `${cellId}.json`);
    const mainClaimPath = path.join(main, ".bee", "claims", `${cellId}.json`);
    fs.mkdirSync(path.dirname(workClaimPath), { recursive: true });
    fs.mkdirSync(path.dirname(mainClaimPath), { recursive: true });
    fs.writeFileSync(workClaimPath, `${JSON.stringify({ cell: cellId, session: "s1", epoch: 1 })}\n`, "utf8");
    const mainOriginal = `${JSON.stringify({ epoch: 1, session: "s1", cell: cellId })}\n`;
    fs.writeFileSync(mainClaimPath, mainOriginal, "utf8");

    try {
      const apply = await runOnboard(["--repo-root", work, "--apply", "--json"], home);
      check(apply.payload?.status === "applied",
        "msn-18d duplicate: apply succeeds (never blocked by an identical duplicate)",
        JSON.stringify(apply.payload?.status));
      check(!fs.existsSync(workClaimPath), "msn-18d duplicate: local duplicate copy removed");
      check(fs.readFileSync(mainClaimPath, "utf8") === mainOriginal,
        "msn-18d duplicate: main's copy is never rewritten (byte-identical to before)");
    } finally {
      cleanupFixture(main, work, home);
    }
  }

  // --- 15e. main/solo checkouts: zero behavior change (must_have truth) ----
  {
    const solo = fs.mkdtempSync(path.join(os.tmpdir(), "bee-msn18d-solo-"));
    fs.mkdirSync(path.join(solo, ".git"));
    const home = makeFakeHome();
    try {
      const plan = await runOnboard(["--repo-root", solo, "--json"], home);
      check(plan.payload !== null && !("worktree_migration" in plan.payload),
        "msn-18d: an ordinary/main checkout's payload carries no worktree_migration key at all",
        JSON.stringify(plan.payload?.worktree_migration));
    } finally {
      cleanupFixture(solo, home);
    }
  }
}

// --- suite-wide isolation invariant -----------------------------------------
// Helper-level check: not a single spawn across the whole suite inherited the
// real HOME/USERPROFILE unmodified. launchedHomes is populated by runOnboard
// itself, so this covers every call site regardless of case.
check(launchedHomes.length > 0, "at least one onboard worker was launched",
  `count: ${launchedHomes.length}`);
check(launchedHomes.every((h) => h.HOME !== REAL_HOME && h.HOME && h.HOME.length > 0),
  "no worker ever inherited the real HOME unmodified",
  JSON.stringify({ real: REAL_HOME, count: launchedHomes.length }));
check(launchedHomes.every((h) => h.USERPROFILE !== REAL_USERPROFILE &&
  h.USERPROFILE && h.USERPROFILE.length > 0),
  "no worker ever inherited the real USERPROFILE unmodified",
  JSON.stringify({ real: REAL_USERPROFILE, count: launchedHomes.length }));

process.stdout.write(`\n${failures === 0 ? "PASS" : "FAIL"} - failures: ${failures}, skipped: ${skips}\n`);
process.exitCode = failures === 0 ? 0 : 1;
