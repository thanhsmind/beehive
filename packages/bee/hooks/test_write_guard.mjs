#!/usr/bin/env node
// test_write_guard.mjs - fixture test for the checkWrite direct-edit deny
// rule (cell cli-mutations-4, plan.md §Approach step 4): .bee/state.json and
// .bee/backlog.jsonl must never be hand-edited — bee.mjs state / bee.mjs
// backlog own them (shim-retire decision bbc6bcea D1: bee.mjs is the sole
// canonical and sole shipped CLI). Spawns hooks/bee-write-guard.mjs as a
// child process (same pattern as hooks/test_model_guard.mjs), feeds it a
// JSON payload on stdin, and asserts exit code + stderr for each row. Builds
// isolated fixture repos so no test run ever touches this project's real
// .bee/state.json or hooks.jsonl.
// Exits 1 on any failure.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

import { runModuleWorker } from "../../../scripts/lib/run-module-worker.mjs";
// wcg-1: isSharedNestedCheckoutTarget imported from the SAME vendored
// location (`.bee/bin/lib`) as acquireLeases below, matching this file's
// existing convention of testing against whatever is actually vendored there.
import { isSharedNestedCheckoutTarget } from "../../../.bee/bin/lib/guards.mjs";
// multisession-native-16: a fixture reservation must be seeded as a REAL
// lease-store lease, not written to `.bee/reservations.json` (which is now
// only a rebuildable projection — see reservations.mjs's own module header;
// the hook's reservation guard reads lease-store's files directly). Imported
// from the SAME vendored location copyLib() below copies fixtures from
// (`.bee/bin/lib`), matching this file's existing convention of testing
// against whatever is actually vendored there. acquireLeases is synchronous,
// so the fixture builders below stay synchronous too.
import { acquireLeases } from "../../../.bee/bin/lib/lease-store.mjs";

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const HOOKS_DIR = path.dirname(SCRIPT_PATH);
const REPO_ROOT = path.dirname(path.dirname(path.dirname(HOOKS_DIR)));
const HOOK_PATH = path.join(HOOKS_DIR, "bee-write-guard.mjs");
const REAL_LIB_DIR = path.join(REPO_ROOT, ".bee", "bin", "lib");

// Deliberate small duplicate of reservations.mjs's own reserve()<->lease
// field mapping (see that module's header for the canonical explanation) —
// this file's fixtures seed a lease directly via lease-store.mjs's
// synchronous acquireLeases rather than reservations.mjs's async reserve().
function seedReservationLease(root, { path: reservedPath, agent, cell = "other", ttl = 3600 }) {
  acquireLeases(root, [
    {
      type: "path",
      id: reservedPath,
      mode: "write",
      workflow_id: cell,
      session_id: "\u0000bee-reservation-sessionless\u0000",
      workspace_id: `agent:${agent}`,
      epoch: 0,
      ttl,
      kind: "lease",
    },
  ]);
}

let failures = 0;

function check(condition, label, extra = "") {
  if (condition) {
    process.stdout.write(`ok    - ${label}\n`);
  } else {
    failures += 1;
    process.stdout.write(`FAIL  - ${label}${extra ? ` :: ${extra}` : ""}\n`);
  }
}

// --- fixture builders --------------------------------------------------

function mkFixture(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

// guards.mjs pulls in reservations.mjs (findConflicts) and state.mjs
// (readConfig, resolvePipeline); state.mjs itself pulls in claims.mjs
// (fresh-session-handoff fsh-3, session/lane primitives); reservations.mjs
// pulls in fsutil.mjs; and check (d)'s CLI-shape validation dynamically
// imports validate-args.mjs + command-registry.mjs, which in turn pull in
// reviews.mjs, cells.mjs, and backlog.mjs. Copy the WHOLE lib directory
// (readdirSync, name-agnostic — matches hooks/test_hook_contracts.mjs's own
// copyLib) rather than a hardcoded name list: a hardcoded list silently goes
// stale every time a new transitive dependency ships (exactly what happened
// here — state.mjs's claims.mjs import shipped after this list was last
// updated, so every fixture row hit ERR_MODULE_NOT_FOUND at import and the
// hook fail-opened universally, masking real deny-rule regressions behind a
// false "still passing" or a false "still failing" signal depending on the
// row's expected status). Bug found and fixed while updating this cell's
// guard-message assertions (auto-fix per worker rule-1: a bug in touched
// code).
function copyLib(fixtureRoot) {
  const libDir = path.join(fixtureRoot, ".bee", "bin", "lib");
  fs.mkdirSync(libDir, { recursive: true });
  for (const name of fs.readdirSync(REAL_LIB_DIR)) {
    if (!name.endsWith(".mjs")) continue;
    fs.copyFileSync(path.join(REAL_LIB_DIR, name), path.join(libDir, name));
  }
}

function writeState(root, state) {
  fs.writeFileSync(path.join(root, ".bee", "state.json"), `${JSON.stringify(state, null, 2)}\n`);
}

// A working fixture at a given phase (default: swarming with execution
// approved — the most permissive phase, so a deny proves the first-hit rule
// fires regardless of phase logic, not because the phase itself would deny).
function buildFixture(prefix, { phase = "swarming", executionApproved = true } = {}) {
  const root = mkFixture(prefix);
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  copyLib(root);
  writeState(root, {
    phase,
    mode: "standard",
    feature: "demo",
    approved_gates: { context: true, shape: true, execution: executionApproved, review: false },
  });
  return root;
}

// A fixture whose vendored guards.mjs (the file this cell modified) throws on
// import — exercising the hook's own try/catch fail-open path, independent of
// the pure checkWrite rule. Proves the new deny rule doesn't turn a broken
// import into a session-breaking hook crash.
function buildThrowingGuardsFixture() {
  const root = buildFixture("bee-write-guard-throwguards-", { phase: "swarming" });
  fs.writeFileSync(
    path.join(root, ".bee", "bin", "lib", "guards.mjs"),
    "throw new Error('boom: fixture guards.mjs deliberately throws on import');\n",
  );
  return root;
}

// A working fixture whose vendored guards.mjs imports cleanly but whose
// isSharedNestedCheckoutTarget THROWS WHEN CALLED (not on import — that is
// buildThrowingGuardsFixture / row7's outer-catch fail-open path). It
// re-exports the real guards module verbatim and shadows only the one
// primitive with a throwing stub, so checkWrite and every other guard still
// behave exactly as in production. Models a real filesystem error raised from
// inside the detection primitive (unreadable nested .git, EACCES realpath).
function buildCallThrowingGuardsFixture(prefix) {
  const root = buildFixture(prefix, { phase: "swarming" });
  const realGuardsUrl = pathToFileURL(path.join(REAL_LIB_DIR, "guards.mjs")).href;
  fs.writeFileSync(
    path.join(root, ".bee", "bin", "lib", "guards.mjs"),
    `export * from ${JSON.stringify(realGuardsUrl)};\n` +
      "export function isSharedNestedCheckoutTarget() {\n" +
      "  throw new Error('boom: forced detection error inside isSharedNestedCheckoutTarget');\n" +
      "}\n",
  );
  return root;
}

// A working fixture pre-seeded with one active reservation — a real
// lease-store lease (multisession-native-16; see seedReservationLease above)
// — so the apply_patch reservation rows below can prove a real
// conflict/no-conflict decision instead of asserting on string presence
// alone.
function buildReservationFixture(prefix, reservedPath, holderAgent) {
  const root = buildFixture(prefix);
  seedReservationLease(root, { path: reservedPath, agent: holderAgent, cell: "other-cell" });
  return root;
}

function buildLinkedFixture(prefix, { invalid = false, reservedPath = null, holderAgent = null } = {}) {
  const mainRoot = mkFixture(`${prefix}-main-`);
  const workRoot = mkFixture(`${prefix}-work-`);
  const gitdir = path.join(mainRoot, ".git", "worktrees", "fixture");
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(workRoot, ".git"), `gitdir: ${gitdir}\n`);
  if (!invalid) {
    fs.writeFileSync(path.join(gitdir, "gitdir"), `${path.join(workRoot, ".git")}\n`);
  }
  fs.mkdirSync(path.join(mainRoot, ".bee"), { recursive: true });
  copyLib(mainRoot);
  writeState(mainRoot, {
    phase: "swarming",
    mode: "high-risk",
    feature: "worktree-isolation",
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  if (reservedPath && holderAgent) {
    // multisession-native-16: seeded as a real lease under mainRoot (the
    // store this linked-but-ungranted worktree's checkWrite call actually
    // resolves to), not written to mainRoot's `.bee/reservations.json`.
    seedReservationLease(mainRoot, { path: reservedPath, agent: holderAgent, cell: "other" });
  }
  fs.mkdirSync(path.join(workRoot, "src"), { recursive: true });
  return { mainRoot, workRoot };
}

// --- git-exemption fixture builders (D1/D3/D4, cell ige-2, P46 / GH #1) ----
// A REAL git repo (not just a fixture directory) so `git diff --cached
// --name-only` resolves against actual index state — the whole point of D4
// (eligibility computed from real staged paths, never from wording).
function runGit(cwd, args) {
  execFileSync("git", args, { cwd, stdio: ["ignore", "ignore", "ignore"] });
}

function buildGitFixture(prefix, { phase = "idle" } = {}) {
  const root = mkFixture(prefix);
  runGit(root, ["init", "-q"]);
  runGit(root, ["config", "user.email", "ige2@example.com"]);
  runGit(root, ["config", "user.name", "ige2 fixture"]);
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  copyLib(root);
  writeState(root, {
    phase,
    mode: "standard",
    feature: "demo",
    approved_gates: { context: true, shape: true, execution: false, review: false },
  });
  return root;
}

function stageFile(root, relPath, content = "x\n") {
  const abs = path.join(root, relPath);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, content);
  runGit(root, ["add", relPath]);
}

function writePayloadFor(toolName, target) {
  if (toolName === "Bash") {
    return { tool_name: toolName, tool_input: { command: `printf x > "${target}"` } };
  }
  if (toolName === "apply_patch") {
    return {
      tool_name: toolName,
      tool_input: { input: `*** Begin Patch\n*** Add File: ${target}\n+x\n*** End Patch` },
    };
  }
  return { tool_name: toolName, tool_input: { file_path: target } };
}

// --- hook invocation -----------------------------------------------------

async function runHookPayload(payload, cwd) {
  const body = { ...payload, cwd };
  const input = JSON.stringify(body);
  return await runModuleWorker(HOOK_PATH, { input, cwd });
}

function readLastJsonl(file) {
  if (!fs.existsSync(file)) return null;
  const lines = fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .filter((l) => l.trim());
  if (lines.length === 0) return null;
  try {
    return JSON.parse(lines[lines.length - 1]);
  } catch {
    return null;
  }
}

async function main() {
  const root = buildFixture("bee-write-guard-swarming-");
  process.stdout.write(`fixture: ${root}\n`);

  // --- 1. Edit .bee/state.json -> denied (exit 2), message names the CLI verb
  const r1 = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: ".bee/state.json" } },
    root,
  );
  check(r1.status === 2, "row1: Edit .bee/state.json is denied (exit 2)", `status=${r1.status} stderr=${r1.stderr}`);

  // --- AskUserQuestion schema pre-validation (ask-guard-autofix D1/D2): an
  // over-long header is now a FIXABLE violation — the hook auto-truncates it,
  // allows the call (exit 0), and emits an updatedInput JSON on stdout
  // instead of denying with the harness's opaque "Invalid tool parameters".
  const rAskFix = await runHookPayload(
    { tool_name: "AskUserQuestion", tool_input: { questions: [{ question: "q", header: "Worktree switch", options: [{ label: "A", description: "x" }, { label: "B", description: "y" }] }] } },
    root,
  );
  check(rAskFix.status === 0, "AskUserQuestion with a 16-char header is auto-fixed and allowed (exit 0)", `status=${rAskFix.status} stderr=${rAskFix.stderr}`);
  let rAskFixOutput = null;
  try { rAskFixOutput = JSON.parse(rAskFix.stdout); } catch { /* left null, checked below */ }
  check(rAskFixOutput !== null, "the auto-fix emits parseable JSON on stdout", rAskFix.stdout);
  check(
    !!rAskFixOutput && rAskFixOutput.hookSpecificOutput?.permissionDecision === "allow",
    "the auto-fix stdout carries permissionDecision 'allow'",
    rAskFix.stdout,
  );
  check(
    !!rAskFixOutput && rAskFixOutput.hookSpecificOutput?.updatedInput?.questions?.[0]?.header === "Worktree sw…",
    "the auto-fix stdout carries the whole rewritten toolInput with the truncated header",
    rAskFix.stdout,
  );

  // A mixed fixable+unfixable call still denies (exit 2) with the unfixable reason.
  const rAskMixedDeny = await runHookPayload(
    { tool_name: "AskUserQuestion", tool_input: { questions: [{ question: "q", header: "Worktree switch", options: [{ label: "only-one", description: "x" }] }] } },
    root,
  );
  check(rAskMixedDeny.status === 2, "a fixable header alongside an unfixable violation still denies (exit 2)", `status=${rAskMixedDeny.status} stderr=${rAskMixedDeny.stderr}`);
  check(/option/.test(rAskMixedDeny.stderr), "the deny message names the unfixable option-count violation", rAskMixedDeny.stderr);

  const rAskOk = await runHookPayload(
    { tool_name: "AskUserQuestion", tool_input: { questions: [{ question: "q", header: "Approach", options: [{ label: "A", description: "x" }, { label: "B", description: "y" }] }] } },
    root,
  );
  check(rAskOk.status === 0, "a valid AskUserQuestion is allowed (exit 0)", `status=${rAskOk.status} stderr=${rAskOk.stderr}`);
  check(r1.stderr.includes("bee.mjs state"), "row1: stderr names bee.mjs state", r1.stderr);
  check(r1.stderr.includes("FIX"), "row1: stderr has a FIX element", r1.stderr);
  check(r1.stderr.includes("direct-edit"), "row1: stderr identifies the direct-edit guard", r1.stderr);

  // --- 2. Write .bee/backlog.jsonl -> denied (exit 2), message names bee_backlog.mjs add
  const r2 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: ".bee/backlog.jsonl", content: "{}\n" } },
    root,
  );
  check(r2.status === 2, "row2: Write .bee/backlog.jsonl is denied (exit 2)", `status=${r2.status} stderr=${r2.stderr}`);
  check(r2.stderr.includes("bee.mjs backlog add"), "row2: stderr names bee.mjs backlog add", r2.stderr);

  // --- 3. bash-redirect row: `cat foo.txt >> .bee/backlog.jsonl` -> denied,
  // proving the deny reaches Bash-extracted targets, not just Edit/Write.
  const r3 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "cat notes.txt >> .bee/backlog.jsonl" } },
    root,
  );
  check(r3.status === 2, "row3: bash redirect into .bee/backlog.jsonl is denied (exit 2)",
    `status=${r3.status} stderr=${r3.stderr}`);
  check(r3.stderr.includes("bee.mjs backlog add"), "row3: stderr names bee.mjs backlog add", r3.stderr);

  // --- 3b. bash-redirect row for state.json (sed -i) -> denied
  const r3b = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'sed -i "s/idle/swarming/" .bee/state.json' } },
    root,
  );
  check(r3b.status === 2, "row3b: sed -i on .bee/state.json is denied (exit 2)",
    `status=${r3b.status} stderr=${r3b.stderr}`);
  check(r3b.stderr.includes("bee.mjs state"), "row3b: stderr names bee.mjs state", r3b.stderr);

  // --- 3c. Write docs/backlog.md -> denied (exit 2), message names the
  // owning verbs (backlog-unification bu-3, D3): docs/backlog.md is the
  // generated view over .bee/backlog.jsonl, CLI-owned same as the two rows
  // above.
  const r3c = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: "docs/backlog.md", content: "x\n" } },
    root,
  );
  check(r3c.status === 2, "row3c: Write docs/backlog.md is denied (exit 2)", `status=${r3c.status} stderr=${r3c.stderr}`);
  check(r3c.stderr.includes("bee.mjs backlog pbi add"), "row3c: stderr names bee.mjs backlog pbi add", r3c.stderr);
  check(r3c.stderr.includes("bee.mjs backlog pbi status"), "row3c: stderr names bee.mjs backlog pbi status", r3c.stderr);
  check(r3c.stderr.includes("bee.mjs backlog pbi amend"), "row3c: stderr names bee.mjs backlog pbi amend", r3c.stderr);
  check(r3c.stderr.includes("bee.mjs backlog render --write"), "row3c: stderr names bee.mjs backlog render --write", r3c.stderr);
  check(r3c.stderr.includes("direct-edit"), "row3c: stderr identifies the direct-edit guard", r3c.stderr);

  // --- 3d. Edit docs/history/demo/CONTEXT.md still passes: the exact-path
  // deny on docs/backlog.md must not spill onto the rest of docs/ (the
  // docs-lane exemption used by scribing/capture stays untouched).
  const r3d = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: "docs/history/demo/CONTEXT.md" } },
    root,
  );
  check(r3d.status === 0, "row3d: Edit docs/history/demo/CONTEXT.md still passes (rest of docs/ unaffected)", `status=${r3d.status} stderr=${r3d.stderr}`);

  // --- 4. pass row: Edit .bee/cells/x.json still passes (untouched verdict)
  const r4 = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: ".bee/cells/demo-1.json" } },
    root,
  );
  check(r4.status === 0, "row4: Edit .bee/cells/demo-1.json still passes", `status=${r4.status} stderr=${r4.stderr}`);

  // --- 5. pass row: a plain bee CLI invocation extracts no bash target and
  // passes untouched (extractBashTargets behavior validated in validation-1.md)
  const r5 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "node .bee/bin/bee_state.mjs set --phase swarming" } },
    root,
  );
  check(r5.status === 0, "row5: plain bee_state.mjs CLI invocation still passes",
    `status=${r5.status} stderr=${r5.stderr}`);

  // --- 5b. same for bee_backlog.mjs add
  const r5b = await runHookPayload(
    {
      tool_name: "Bash",
      tool_input: { command: 'node .bee/bin/bee_backlog.mjs add --type bug --title "x" --severity P2' },
    },
    root,
  );
  check(r5b.status === 0, "row5b: plain bee_backlog.mjs add CLI invocation still passes",
    `status=${r5b.status} stderr=${r5b.stderr}`);

  // --- 5c/5d. check (d) CLI-shape validation (D3 transition guard): BOTH the
  // legacy shim shape and the sole shipped bee.mjs dispatcher shape must
  // resolve the same command name against the shared registry — a call
  // missing the required --id proves resolution actually happened (a
  // shape the guard doesn't recognize fails open at exit 0 instead, per
  // checkCliShape's documented "unrecognized shapes are left alone" rule).
  const r5c = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'node .bee/bin/bee_cells.mjs cap --outcome "done"' } },
    root,
  );
  check(r5c.status === 2, "row5c: legacy bee_cells.mjs cap (missing --id) resolves to the registry and is denied",
    `status=${r5c.status} stderr=${r5c.stderr}`);
  check(r5c.stderr.includes("cells.cap"), "row5c: stderr names the resolved cells.cap command", r5c.stderr);
  check(r5c.stderr.includes("field: id"), "row5c: stderr names the missing id field", r5c.stderr);

  const r5d = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'node .bee/bin/bee.mjs cells cap --outcome "done"' } },
    root,
  );
  check(r5d.status === 2, "row5d: dispatcher bee.mjs cells cap (missing --id) resolves to the registry and is denied",
    `status=${r5d.status} stderr=${r5d.stderr}`);
  check(r5d.stderr.includes("cells.cap"), "row5d: stderr names the resolved cells.cap command", r5d.stderr);
  check(r5d.stderr.includes("field: id"), "row5d: stderr names the missing id field", r5d.stderr);

  // --- 6. deny rule fires in every phase, not only swarming: idle phase too
  // (idle is otherwise the most permissive phase for .bee/ writes — this
  // proves the deny rule really runs before GATE_ALLOWED_PREFIXES / phase logic)
  const idleRoot = buildFixture("bee-write-guard-idle-", { phase: "idle" });
  const r6 = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: ".bee/state.json" } },
    idleRoot,
  );
  check(r6.status === 2, "row6: Edit .bee/state.json is denied even while idle (.bee/ is normally allowed)",
    `status=${r6.status} stderr=${r6.stderr}`);
  check(r6.stderr.includes("bee.mjs state"), "row6: idle-phase denial still names bee.mjs state", r6.stderr);
  // control: an unrelated .bee/ path keeps its current (allowed) idle verdict
  const r6b = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: ".bee/cells/demo-1.json" } },
    idleRoot,
  );
  check(r6b.status === 0, "row6b: unrelated .bee/ path keeps its current allowed-at-idle verdict",
    `status=${r6b.status} stderr=${r6b.stderr}`);

  // --- 7. fail-open row: guards.mjs itself throws on import -> hook still
  // exits 0 with empty stderr, and a crash line lands in that fixture's
  // hooks.jsonl (HOOK-level try/catch, not the pure checkWrite rule).
  const throwRoot = buildThrowingGuardsFixture();
  const r7 = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: ".bee/state.json" } },
    throwRoot,
  );
  check(r7.status === 0, "row7: guards.mjs import throwing still fails open (exit 0)",
    `status=${r7.status} stderr=${r7.stderr}`);
  check(r7.stderr === "", "row7: fail-open path produces empty stderr", JSON.stringify(r7.stderr));
  const crashLog = path.join(throwRoot, ".bee", "logs", "hooks.jsonl");
  const crashEvent = readLastJsonl(crashLog);
  check(!!crashEvent, "row7: a crash line was appended to that fixture's hooks.jsonl", String(crashEvent));
  check(crashEvent && crashEvent.hook === "write-guard", "row7: crash line's hook is write-guard",
    JSON.stringify(crashEvent));
  check(
    crashEvent && typeof crashEvent.error === "string" && crashEvent.error.includes("boom"),
    "row7: crash line carries the underlying error",
    JSON.stringify(crashEvent),
  );

  // ======================================================================
  // 8+. apply_patch matrix (cell codex-parity-4, plan-review third bullet):
  // Add/Update/Delete/Move x multi-target/Unicode/space/escape/malformed
  // rows, gate-policy rows, reservation rows, and the unknown-target deny
  // row. Every row spawns the real hook process and asserts exit code +
  // stderr shape — no string-presence-only assertions.
  // ======================================================================

  // --- 8. Add File, single safe target -> passes
  const patchAdd = "*** Begin Patch\n*** Add File: src/new-file.txt\n+hello world\n*** End Patch";
  const r8 = await runHookPayload({ tool_name: "apply_patch", tool_input: { input: patchAdd } }, root);
  check(r8.status === 0, "row8: apply_patch Add File to a safe path passes", `status=${r8.status} stderr=${r8.stderr}`);

  // --- 9. Update File, single target denied via direct-edit (.bee/state.json)
  const patchUpdateDenied =
    "*** Begin Patch\n*** Update File: .bee/state.json\n@@\n-old\n+new\n*** End Patch";
  const r9 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchUpdateDenied } },
    root,
  );
  check(r9.status === 2, "row9: apply_patch Update File .bee/state.json is denied (exit 2)",
    `status=${r9.status} stderr=${r9.stderr}`);
  check(r9.stderr.includes("bee.mjs state"), "row9: stderr names bee.mjs state", r9.stderr);

  // --- 10. Delete File, single target denied via direct-edit (.bee/backlog.jsonl)
  const patchDeleteDenied = "*** Begin Patch\n*** Delete File: .bee/backlog.jsonl\n*** End Patch";
  const r10 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchDeleteDenied } },
    root,
  );
  check(r10.status === 2, "row10: apply_patch Delete File .bee/backlog.jsonl is denied (exit 2)",
    `status=${r10.status} stderr=${r10.stderr}`);
  check(r10.stderr.includes("bee.mjs backlog add"), "row10: stderr names bee.mjs backlog add", r10.stderr);

  // --- 11. Move (Update File + Move to), both targets safe -> passes
  const patchMoveSafe =
    "*** Begin Patch\n*** Update File: src/old-name.txt\n*** Move to: src/new-name.txt\n@@\n-old\n+new\n*** End Patch";
  const r11 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchMoveSafe } },
    root,
  );
  check(r11.status === 0, "row11: apply_patch Move (Update File + Move to) with safe paths passes",
    `status=${r11.status} stderr=${r11.stderr}`);

  // --- 12. Move destination is the direct-edit-denied file -> whole patch denied
  const patchMoveDenied =
    "*** Begin Patch\n*** Update File: src/old-name.txt\n*** Move to: .bee/state.json\n@@\n-old\n+new\n*** End Patch";
  const r12 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchMoveDenied } },
    root,
  );
  check(r12.status === 2, "row12: apply_patch Move destination .bee/state.json is denied (exit 2)",
    `status=${r12.status} stderr=${r12.stderr}`);
  check(r12.stderr.includes("bee.mjs state"), "row12: stderr names bee.mjs state", r12.stderr);

  // --- 13. Multi-target (Add + Update + Delete), one target denied -> whole patch denied
  const patchMultiOneDenied =
    "*** Begin Patch\n" +
    "*** Add File: src/a.txt\n+content\n" +
    "*** Update File: src/b.txt\n@@\n-x\n+y\n" +
    "*** Delete File: .bee/state.json\n" +
    "*** End Patch";
  const r13 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchMultiOneDenied } },
    root,
  );
  check(r13.status === 2,
    "row13: multi-target apply_patch denies when any target hits a policy deny (.bee/state.json)",
    `status=${r13.status} stderr=${r13.stderr}`);
  check(r13.stderr.includes("bee.mjs state"), "row13: stderr names bee.mjs state", r13.stderr);

  // --- 14. Multi-target, every target safe -> passes
  const patchMultiSafe =
    "*** Begin Patch\n" +
    "*** Add File: src/a.txt\n+content\n" +
    "*** Update File: src/b.txt\n@@\n-x\n+y\n" +
    "*** Delete File: src/c.txt\n" +
    "*** End Patch";
  const r14 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchMultiSafe } },
    root,
  );
  check(r14.status === 0, "row14: multi-target apply_patch with every target safe passes",
    `status=${r14.status} stderr=${r14.stderr}`);

  // --- 15. Unicode path, reserved by another agent -> denied naming the
  // exact resolved path (proves Unicode target extraction/resolution is
  // correct, not merely "didn't crash").
  const unicodePath = "café/résumé.md";
  const uniRoot = buildReservationFixture("bee-write-guard-applypatch-unicode-", unicodePath, "otto");
  const patchUnicode = `*** Begin Patch\n*** Add File: ${unicodePath}\n+hello\n*** End Patch`;
  const r15 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchUnicode }, agent_name: "mel" },
    uniRoot,
  );
  check(r15.status === 2,
    "row15: apply_patch Add File to a Unicode path reserved by another agent is denied (exit 2)",
    `status=${r15.status} stderr=${r15.stderr}`);
  check(r15.stderr.includes(unicodePath), "row15: stderr names the exact Unicode path", r15.stderr);
  check(r15.stderr.includes("otto"), "row15: stderr names the reservation holder", r15.stderr);

  // --- 16. Path with spaces, reserved by another agent -> denied naming the
  // exact resolved path.
  const spacedPath = "my folder/file name.txt";
  const spaceRoot = buildReservationFixture("bee-write-guard-applypatch-space-", spacedPath, "otto");
  const patchSpace = `*** Begin Patch\n*** Update File: ${spacedPath}\n@@\n-a\n+b\n*** End Patch`;
  const r16 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchSpace }, agent_name: "mel" },
    spaceRoot,
  );
  check(r16.status === 2,
    "row16: apply_patch Update File to a space-containing path reserved by another agent is denied",
    `status=${r16.status} stderr=${r16.stderr}`);
  check(r16.stderr.includes(spacedPath), "row16: stderr names the exact space-containing path", r16.stderr);

  // --- 17. Escape-sequence path (literal backslash-escaped space in the
  // target line) reserved by another agent -> still resolved to a concrete
  // target and denied (proves the parser doesn't silently drop/mangle an
  // escaped-looking path into a false "unprovable" pass-through).
  const escapedPath = "my\\ folder/escaped.txt";
  const escRoot = buildReservationFixture("bee-write-guard-applypatch-escape-", escapedPath, "otto");
  const patchEscaped = `*** Begin Patch\n*** Add File: ${escapedPath}\n+hi\n*** End Patch`;
  const r17 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchEscaped }, agent_name: "mel" },
    escRoot,
  );
  check(r17.status === 2,
    "row17: apply_patch Add File with a backslash-escaped-space path resolves to a concrete target and is denied",
    `status=${r17.status} stderr=${r17.stderr}`);
  check(r17.stderr.includes("otto"), "row17: stderr names the reservation holder", r17.stderr);

  // --- 18. Malformed patch body: a verb line with no colon/path at all ->
  // zero targets extracted -> denied (P1 repair: unprovable target set).
  const patchMalformedNoColon = "*** Begin Patch\n*** Add File\n+content\n*** End Patch";
  const r18 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchMalformedNoColon } },
    root,
  );
  check(r18.status === 2,
    "row18: apply_patch with a malformed verb line (no colon/path) denies (unprovable target set)",
    `status=${r18.status} stderr=${r18.stderr}`);
  check(r18.stderr.trim().length > 0, "row18: stderr carries a corrective reason", r18.stderr);

  // --- 19. Unknown-target deny row: an unrecognized verb (not Add/Update/
  // Delete/Move) parses to zero targets -> denied, never silently allowed
  // through an unexamined operation.
  const patchUnknownVerb = "*** Begin Patch\n*** Rename File: src/a.txt -> src/b.txt\n*** End Patch";
  const r19 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchUnknownVerb } },
    root,
  );
  check(r19.status === 2,
    "row19: apply_patch with an unrecognized verb line denies rather than silently allowing an unexamined operation",
    `status=${r19.status} stderr=${r19.stderr}`);
  check(r19.stderr.includes("FIX"), "row19: stderr carries the corrective FIX guidance", r19.stderr);

  // --- 20. Empty/whitespace-only path after the verb colon -> denied (the
  // extraction bug fix: a lone leftover whitespace char must not count as a
  // proved target).
  const patchEmptyPath = "*** Begin Patch\n*** Add File:    \n+content\n*** End Patch";
  const r20 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchEmptyPath } },
    root,
  );
  check(r20.status === 2,
    "row20: apply_patch Add File with a whitespace-only path denies (empty target is never proved)",
    `status=${r20.status} stderr=${r20.stderr}`);
  check(r20.stderr.includes("FIX"), "row20: stderr carries the corrective FIX guidance", r20.stderr);

  // --- 21. Path traversal escape (relative .. outside the repo) -> denied
  // (unprovable target: toRelPath returns null for an escaping path).
  const patchTraversal = "*** Begin Patch\n*** Add File: ../../outside-repo.txt\n+x\n*** End Patch";
  const r21 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchTraversal } },
    root,
  );
  check(r21.status === 2,
    "row21: apply_patch Add File escaping the repo via .. traversal denies (unprovable target)",
    `status=${r21.status} stderr=${r21.stderr}`);
  check(r21.stderr.includes("FIX"), "row21: stderr carries the corrective FIX guidance", r21.stderr);

  // --- 22. Absolute path outside the repo -> denied (unprovable target).
  const patchAbsoluteOutside = "*** Begin Patch\n*** Add File: /etc/passwd\n+x\n*** End Patch";
  const r22 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchAbsoluteOutside } },
    root,
  );
  check(r22.status === 2,
    "row22: apply_patch Add File to an absolute path outside the repo denies (unprovable target)",
    `status=${r22.status} stderr=${r22.stderr}`);
  check(r22.stderr.includes("FIX"), "row22: stderr carries the corrective FIX guidance", r22.stderr);

  // --- 23. Malformed OUTER payload: apply_patch tool_input carries no
  // canonical "*** Begin Patch" envelope at all -> stays D2's visible
  // fail-open (this class is explicitly NOT the deny-on-unprovable P1
  // repair -- nothing was genuinely intercepted).
  const r23 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: "not a patch at all" } },
    root,
  );
  check(r23.status === 0,
    "row23: apply_patch with no canonical patch envelope in tool_input stays fail-open (malformed OUTER payload, D2)",
    `status=${r23.status} stderr=${r23.stderr}`);

  // --- 24. Gate-policy row: apply_patch write to a source path during a
  // gated phase with execution unapproved is denied by the gate guard (not
  // direct-edit, not reservation) -- proves apply_patch runs the SAME gate
  // decision as Edit/Write/Bash. Uses "planning" (a phase the state machine
  // can actually produce, and a live member of guards.mjs's GATED_PHASES)
  // rather than the retired "validating" value -- this row must exercise
  // checkWrite's real gate policy, not readState's legacy-phase coercion.
  const gateRoot = buildFixture("bee-write-guard-applypatch-gate-", {
    phase: "planning",
    executionApproved: false,
  });
  const patchGateSrc = "*** Begin Patch\n*** Add File: src/feature.txt\n+new code\n*** End Patch";
  const r24 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchGateSrc } },
    gateRoot,
  );
  check(r24.status === 2,
    "row24: apply_patch Add File to a source path during a gated phase (execution unapproved) is denied by the gate guard",
    `status=${r24.status} stderr=${r24.stderr}`);
  check(r24.stderr.includes("bee gate"), "row24: stderr identifies the gate guard", r24.stderr);

  // --- 25. Gate-allowed-prefix control: the same gated phase still allows a
  // docs/ target, proving row24 denied on gate policy, not a broad apply_patch
  // block.
  const patchGateDocs = "*** Begin Patch\n*** Add File: docs/notes.md\n+notes\n*** End Patch";
  const r25 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchGateDocs } },
    gateRoot,
  );
  check(r25.status === 0,
    "row25: apply_patch Add File under docs/ (gate-allowed prefix) passes even in the gated phase",
    `status=${r25.status} stderr=${r25.stderr}`);

  // --- 26. Reservation-row control: a target reserved by the SAME requesting
  // agent has no self-conflict and passes.
  const selfRoot = buildReservationFixture("bee-write-guard-applypatch-self-", "src/mine.txt", "mel");
  const patchSelf = "*** Begin Patch\n*** Update File: src/mine.txt\n@@\n-a\n+b\n*** End Patch";
  const r26 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchSelf }, agent_name: "mel" },
    selfRoot,
  );
  check(r26.status === 0,
    "row26: apply_patch Update File to a path reserved by the SAME agent passes (no self-conflict)",
    `status=${r26.status} stderr=${r26.stderr}`);

  // --- 27+. Partial-unprovable matrix (review finding F1, P1): an
  // apply_patch envelope that mixes a PROVABLE target (a normal, otherwise-
  // allowed path) with an UNPROVABLE target must deny the WHOLE request,
  // never allow the safe half through. This pins bee-write-guard.mjs's
  // `relPaths.length < targets.length` branch (line 176) specifically for
  // the *mixed* case -- rows 18-22 above only ever exercise all-unprovable
  // patches, so a regression that started evaluating just the resolved
  // subset of targets (dropping the escaping one silently) would still pass
  // every existing row while allowing an unchecked write.

  // --- 27. Ordering A: provable target FIRST, unprovable target SECOND
  // (safe Add File, then a blank/whitespace-only-path Update File).
  const patchPartialProvableFirst =
    "*** Begin Patch\n" +
    "*** Add File: src/safe-first.txt\n+hello\n" +
    "*** Update File:    \n@@\n-old\n+new\n" +
    "*** End Patch";
  const r27 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchPartialProvableFirst } },
    root,
  );
  check(r27.status === 2,
    "row27: apply_patch mixing a safe Add (provable, FIRST) with a blank-path Update (unprovable, SECOND) denies the WHOLE patch",
    `status=${r27.status} stderr=${r27.stderr}`);
  check(r27.stderr.includes("FIX"), "row27: stderr carries the corrective FIX guidance", r27.stderr);

  // --- 28. Ordering B: unprovable target FIRST, provable target SECOND --
  // proves the whole-patch deny does not depend on scan order (guards
  // against a future short-circuit that stops once a proved target is
  // already recorded).
  const patchPartialUnprovableFirst =
    "*** Begin Patch\n" +
    "*** Update File:    \n@@\n-old\n+new\n" +
    "*** Add File: src/safe-second.txt\n+hello\n" +
    "*** End Patch";
  const r28 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchPartialUnprovableFirst } },
    root,
  );
  check(r28.status === 2,
    "row28: apply_patch mixing a blank-path Update (unprovable, FIRST) with a safe Add (provable, SECOND) still denies the WHOLE patch",
    `status=${r28.status} stderr=${r28.stderr}`);
  check(r28.stderr.includes("FIX"), "row28: stderr carries the corrective FIX guidance", r28.stderr);

  // --- 29. Second mixed combo named in the review finding: one valid Update
  // (provable) plus a second operation whose Move destination escapes the
  // repo outright (unprovable via path traversal, not a blank path) -- proves
  // the whole-deny rule generalizes across unprovable *kinds*, not just the
  // blank-path extraction bug.
  const patchPartialMoveOutside =
    "*** Begin Patch\n" +
    "*** Update File: src/valid.txt\n@@\n-old\n+new\n" +
    "*** Update File: src/other.txt\n*** Move to: ../../outside-repo.txt\n@@\n-a\n+b\n" +
    "*** End Patch";
  const r29 = await runHookPayload(
    { tool_name: "apply_patch", tool_input: { input: patchPartialMoveOutside } },
    root,
  );
  check(r29.status === 2,
    "row29: apply_patch mixing a valid Update (provable) with a second operation's outside-repo Move destination (unprovable) denies the WHOLE patch",
    `status=${r29.status} stderr=${r29.stderr}`);
  check(r29.stderr.includes("FIX"), "row29: stderr carries the corrective FIX guidance", r29.stderr);

  // --- 30+. Worktree isolation matrix (D2/D4). Every write-capable tool
  // uses the physical workRoot for canonical containment and the validated
  // main checkout storeRoot for state/reservations.
  const writeToolClasses = ["Edit", "Write", "MultiEdit", "Bash", "apply_patch"];
  const linked = buildLinkedFixture("bee-write-guard-linked", {
    reservedPath: "src/held.txt",
    holderAgent: "otto",
  });
  for (const toolName of writeToolClasses) {
    const foreign = await runHookPayload(
      { ...writePayloadFor(toolName, "src/held.txt"), agent_name: "mel" },
      linked.workRoot,
    );
    check(
      foreign.status === 2 && foreign.stderr.includes("otto"),
      `row30[${toolName}]: linked worktree reads foreign reservation from main store and denies`,
      `status=${foreign.status} stderr=${foreign.stderr}`,
    );
    const owner = await runHookPayload(
      { ...writePayloadFor(toolName, "src/held.txt"), agent_name: "otto" },
      linked.workRoot,
    );
    check(
      owner.status === 0,
      `row31[${toolName}]: linked worktree reservation owner remains allowed`,
      `status=${owner.status} stderr=${owner.stderr}`,
    );
  }

  const invalidLinked = buildLinkedFixture("bee-write-guard-linked-invalid", { invalid: true });
  for (const toolName of writeToolClasses) {
    const result = await runHookPayload(writePayloadFor(toolName, "src/new.txt"), invalidLinked.workRoot);
    check(
      result.status === 2 && result.stderr.includes("WORKTREE_LINK_INVALID"),
      `row32[${toolName}]: linked-invalid resolution denies before mutation`,
      `status=${result.status} stderr=${result.stderr}`,
    );
  }

  const outside = mkFixture("bee-write-guard-outside-");
  const outsideExisting = path.join(outside, "existing.txt");
  fs.writeFileSync(outsideExisting, "outside\n");
  const symlink = path.join(linked.workRoot, "src", "escape-link");
  fs.symlinkSync(outside, symlink, "dir");
  const escapeRows = [
    ["traversal", "../outside.txt"],
    ["absolute-main", path.join(linked.mainRoot, "src", "main-only.txt")],
    ["symlink-existing", path.join(symlink, "existing.txt")],
    ["symlink-new", path.join(symlink, "new", "nested.txt")],
    ["windows-separator-traversal", "..\\outside-win.txt"],
    ["case-alias", path.join(path.dirname(linked.workRoot), path.basename(linked.workRoot).toUpperCase(), "src", "case.txt")],
  ];
  for (const [kind, target] of escapeRows) {
    for (const toolName of writeToolClasses) {
      const result = await runHookPayload(writePayloadFor(toolName, target), linked.workRoot);
      check(
        result.status === 2,
        `row33[${kind}][${toolName}]: canonical containment denies target escape`,
        `status=${result.status} stderr=${result.stderr}`,
      );
    }
  }

  for (const toolName of writeToolClasses) {
    const result = await runHookPayload(writePayloadFor(toolName, "src\\nested\\new.txt"), linked.workRoot);
    check(
      result.status === 0,
      `row34[${toolName}]: contained Windows separators normalize to the logical reservation namespace`,
      `status=${result.status} stderr=${result.stderr}`,
    );
  }

  // --- 34h+ (cell gmr-1, GH #71 / CONTEXT guard-memory-roots D8): the
  // SPELLING of an out-of-worktree destination must never decide the verdict.
  // One destination, four spellings — absolute, `~/…`, `$HOME/…`, `${HOME}/…`
  // — must produce the SAME exit status AND the SAME reason on every
  // write-capable tool. Before this cell the absolute spelling denied (exit 2)
  // while all three home spellings silently ALLOWED (exit 0): a leading `~/`
  // is neither absolute nor contains `..`, so canonicalRelPath resolved it
  // against cwd into an in-repo relative path under a literal directory named
  // `~` or `$HOME`, containment passed, and the write flowed on to checkWrite
  // under a repo-relative identity that has nothing to do with where the
  // shell would actually put the bytes.
  const homeSpellings = [
    ["absolute", path.join(os.homedir(), ".claude", "bee-gmr1-probe.md")],
    ["tilde", "~/.claude/bee-gmr1-probe.md"],
    ["env-home", "$HOME/.claude/bee-gmr1-probe.md"],
    ["braced-env-home", "${HOME}/.claude/bee-gmr1-probe.md"],
  ];
  for (const toolName of writeToolClasses) {
    const verdicts = [];
    for (const [kind, target] of homeSpellings) {
      const result = await runHookPayload(writePayloadFor(toolName, target), linked.workRoot);
      check(
        result.status === 2,
        `row34h[${kind}][${toolName}]: an out-of-worktree home destination is denied`,
        `status=${result.status} stderr=${result.stderr}`,
      );
      verdicts.push([kind, result]);
    }
    const baseline = verdicts[0][1];
    for (const [kind, result] of verdicts.slice(1)) {
      check(
        result.status === baseline.status && result.stderr === baseline.stderr,
        `row34h[${kind}][${toolName}]: the home spelling gets the SAME verdict and reason as the absolute spelling`,
        `absolute: status=${baseline.status} stderr=${JSON.stringify(baseline.stderr)} | ${kind}: status=${result.status} stderr=${JSON.stringify(result.stderr)}`,
      );
    }
  }

  // --- 34i: a BARE `~` is NOT swept into the new deny. It stays exactly what
  // BROAD_TARGETS (guards.mjs) already made it — this cell closes the
  // `~<separator>…` bypass, it does not redefine the bare token.
  const rBareTilde = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "rm -rf ~" } },
    linked.workRoot,
  );
  check(
    !rBareTilde.stderr.includes("could not be canonically contained"),
    "row34i: a bare `~` keeps today's behavior — never denied by the containment check",
    `status=${rBareTilde.status} stderr=${rBareTilde.stderr}`,
  );

  // ======================================================================
  // 35+. scratch-shape guard (cell th-6, CONTEXT tree-hygiene D4/D5): a
  // scratch-SHAPED write (.tmp/.log/.bak extension, a dotfile whose name
  // contains debug/stress/scratch, or a verdict-/probe-/digest- prefixed
  // payload) landing in a TRACKED directory is denied and the refusal names
  // .bee/tmp/. The scratch homes (.bee/tmp/, .bee/spikes/, .bee/logs/,
  // .bee/workers/) and every deliverable store (docs/**, .bee/cells/,
  // .bee/decisions.jsonl, plugin skill renders) stay allowed even when a
  // filename would otherwise look scratch-shaped — a false deny on a
  // deliverable is worse than the garbage this guard prevents. Uses `root`
  // (swarming, execution approved) — the same permissive fixture rows 1-5
  // use, so a deny here proves the rule, not phase gating.
  // ======================================================================

  // --- 35. the cell's literal DENY example: a debug script into .bee/bin/
  const r35 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: ".bee/bin/.foo_stress_debug.sh", content: "#!/bin/sh\n" } },
    root,
  );
  check(r35.status === 2, "row35: a debug script written into .bee/bin/ is denied", `status=${r35.status} stderr=${r35.stderr}`);
  check(r35.stderr.includes(".bee/tmp/"), "row35: the deny message names .bee/tmp/", r35.stderr);

  // --- 36. the SAME file into .bee/tmp/ is allowed
  const r36 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: ".bee/tmp/th6/.foo_stress_debug.sh", content: "#!/bin/sh\n" } },
    root,
  );
  check(r36.status === 0, "row36: the same debug script into .bee/tmp/ is allowed", `status=${r36.status} stderr=${r36.stderr}`);

  // --- 37. a report into docs/history/<feature>/reports/ is allowed, even
  // named to otherwise look scratch-shaped (verdict- prefix) — proves the
  // docs/** deliverable exemption, not just "this filename was never scratch"
  const r37 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: "docs/history/tree-hygiene/reports/verdict-th6.md", content: "# report\n" } },
    root,
  );
  check(r37.status === 0, "row37: a docs/history/<feature>/reports/ write is allowed even with a scratch-shaped name", `status=${r37.status} stderr=${r37.stderr}`);

  // --- 38. a .bee/cells/<id>.json write is allowed, same non-trivial shape
  const r38 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: ".bee/cells/probe-th-6.json", content: "{}\n" } },
    root,
  );
  check(r38.status === 0, "row38: a .bee/cells/ write is allowed even with a scratch-shaped name", `status=${r38.status} stderr=${r38.stderr}`);

  // --- 39. .bee/decisions.jsonl stays allowed (append-only decisions store)
  const r39 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'printf "x" >> .bee/decisions.jsonl' } },
    root,
  );
  check(r39.status === 0, "row39: .bee/decisions.jsonl append stays allowed", `status=${r39.status} stderr=${r39.stderr}`);

  // --- 40. a plugin skill render stays allowed, again with a scratch-shaped
  // basename so the assertion proves the render-tree exemption fires
  const r40 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: ".claude-plugin/skills/bee-swarming/probe-render.json", content: "{}\n" } },
    root,
  );
  check(r40.status === 0, "row40: a plugin skill render write is allowed even with a scratch-shaped name", `status=${r40.status} stderr=${r40.stderr}`);

  // --- 41. false-deny protection: a project's OWN .log file inside a
  // recognized test/fixture directory is not bee scratch and stays allowed
  const r41 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: "test/fixtures/sample.log", content: "log line\n" } },
    root,
  );
  check(r41.status === 0, "row41: a project's own test-fixture .log file is not denied as bee scratch", `status=${r41.status} stderr=${r41.stderr}`);

  // --- 42. the same bare .log extension OUTSIDE a fixture dir, in a tracked
  // non-.bee directory, is denied (the rule fires; row41 is the exemption,
  // not the rule failing to apply)
  const r42 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: "results.log", content: "log line\n" } },
    root,
  );
  check(r42.status === 2, "row42: a bare .log file in a tracked non-fixture directory is denied", `status=${r42.status} stderr=${r42.stderr}`);
  check(r42.stderr.includes(".bee/tmp/"), "row42: the deny message names .bee/tmp/", r42.stderr);

  // --- 43. a repo-root scratch dotfile (the exact D1 evidence shape: a
  // crashed worker's .<slug>_stress_debug.sh) is denied, not just inside .bee/
  const r43 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: ".rel9999_stress_debug.sh", content: "#!/bin/sh\n" } },
    root,
  );
  check(r43.status === 2, "row43: a repo-root scratch dotfile is denied, not only inside .bee/", `status=${r43.status} stderr=${r43.stderr}`);

  // --- 44. a .tmp extension in a tracked non-.bee directory (scripts/) is denied
  const r44 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: "scripts/scratch-notes.tmp", content: "x\n" } },
    root,
  );
  check(r44.status === 2, "row44: a .tmp file in scripts/ is denied", `status=${r44.status} stderr=${r44.stderr}`);

  // --- 45. a verdict-/probe-/digest- prefixed payload in a tracked non-.bee
  // directory is denied, proving the naming-pattern rules aren't .bee-only
  const r45 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: "scripts/probe-foo.mjs", content: "// x\n" } },
    root,
  );
  check(r45.status === 2, "row45: a probe-*.mjs payload in scripts/ is denied", `status=${r45.status} stderr=${r45.stderr}`);

  // --- 46. fail-open reinforcement: guards.mjs throwing on import still
  // fails open (exit 0) even when the target is scratch-shaped (row7 proves
  // this generically; this row proves it for the new rule's own code path)
  const r46 = await runHookPayload(
    { tool_name: "Write", tool_input: { file_path: ".bee/bin/.foo_stress_debug.sh", content: "#!/bin/sh\n" } },
    throwRoot,
  );
  check(r46.status === 0, "row46: scratch-shaped target still fails open when guards.mjs throws on import", `status=${r46.status} stderr=${r46.stderr}`);

  // ======================================================================
  // 47+. Intake-gate git exemption (D1/D3/D4, cell ige-2, closes P46 / GH
  // #1): a mutating git command at a terminal phase is exempt from the
  // intake gate ONLY when every path it would actually change is bookkeeping
  // (.bee/, docs/, plans/, AGENTS.md) — computed from REAL git state (D4),
  // never the command's wording. Read-only git is always allowed; an
  // unrecognized subcommand fails closed; `git push` never gets the
  // exemption. All rows use `phase: "idle"` (a TERMINAL_PHASES member) —
  // the exact phase the incident (commit a7d2069) hit.
  // ======================================================================

  // --- 47. git status / git log: always allowed at a terminal phase.
  const gitRoot = buildGitFixture("bee-write-guard-git-", { phase: "idle" });
  const r47a = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "git status" } },
    gitRoot,
  );
  check(r47a.status === 0, "row47a: `git status` is allowed at a terminal phase", `status=${r47a.status} stderr=${r47a.stderr}`);
  const r47b = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "git log --oneline -5" } },
    gitRoot,
  );
  check(r47b.status === 0, "row47b: `git log` is allowed at a terminal phase", `status=${r47b.status} stderr=${r47b.stderr}`);

  // --- 48. terminal phase, staged paths ALL bookkeeping (.bee/ + docs/) ->
  // `git commit` is ALLOWED (the exemption firing).
  const bkRoot = buildGitFixture("bee-write-guard-git-bookkeeping-", { phase: "idle" });
  stageFile(bkRoot, ".bee/cells/demo-1.json", "{}\n");
  stageFile(bkRoot, "docs/notes.md", "# notes\n");
  const r48 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'git commit -m "bookkeeping only"' } },
    bkRoot,
  );
  check(r48.status === 0, "row48: `git commit` with only .bee/+docs staged is allowed at a terminal phase (exemption)", `status=${r48.status} stderr=${r48.stderr}`);

  // --- 49. terminal phase, ONE staged source path among the bookkeeping
  // ones -> `git commit` is REFUSED with today's semantics (D1: a single
  // source path keeps the refusal exactly as today).
  const srcRoot = buildGitFixture("bee-write-guard-git-source-", { phase: "idle" });
  stageFile(srcRoot, ".bee/cells/demo-1.json", "{}\n");
  stageFile(srcRoot, "src/feature.js", "console.log('x');\n");
  const r49 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'git commit -m "mixed change"' } },
    srcRoot,
  );
  check(r49.status === 2, "row49: `git commit` staging a source path (mixed with bookkeeping) is refused at a terminal phase", `status=${r49.status} stderr=${r49.stderr}`);
  check(/intake gate/.test(r49.stderr), "row49: stderr identifies the intake gate", r49.stderr);
  check(r49.stderr.includes("src/feature.js"), "row49: stderr names the actual offending source path", r49.stderr);

  // --- 50. D4: a misleading commit MESSAGE never changes the outcome — the
  // SAME staged source path, but the message claims "just bookkeeping".
  const misleadRoot = buildGitFixture("bee-write-guard-git-mislead-", { phase: "idle" });
  stageFile(misleadRoot, "src/feature.js", "console.log('x');\n");
  const r50 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'git commit -m "just bookkeeping"' } },
    misleadRoot,
  );
  check(r50.status === 2, "row50: a `-m \"just bookkeeping\"` message does NOT change the outcome — still refused (D4)", `status=${r50.status} stderr=${r50.stderr}`);

  // --- 51. `git push` gets NO exemption at a terminal phase, even with
  // nothing outstanding to commit — outward-facing, always refused.
  const pushRoot = buildGitFixture("bee-write-guard-git-push-", { phase: "idle" });
  const r51 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "git push origin main" } },
    pushRoot,
  );
  check(r51.status === 2, "row51: `git push` is refused at a terminal phase (no exemption)", `status=${r51.status} stderr=${r51.stderr}`);
  check(/push/i.test(r51.stderr) && /(never|no) exempt/i.test(r51.stderr), "row51: stderr says push is never exempted", r51.stderr);

  // --- 52. an unrecognized git subcommand fails closed at a terminal phase
  // (never assumed read-only, never assumed a safe mutation).
  const unkRoot = buildGitFixture("bee-write-guard-git-unknown-", { phase: "idle" });
  const r52 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "git bisect start" } },
    unkRoot,
  );
  check(r52.status === 2, "row52: an unrecognized git subcommand (`git bisect`) is refused, not assumed safe", `status=${r52.status} stderr=${r52.stderr}`);

  // --- 53. the refusal's FIX line names the bookkeeping-only route and
  // bee-hive BEFORE it ever mentions guards.idle_gate (D3) — checked on the
  // row49 mixed-commit refusal, the exact incident shape.
  const fixText = r49.stderr;
  const idleGateIdx = fixText.indexOf("guards.idle_gate");
  const bookkeepingIdx = fixText.search(/commit or write bookkeeping|bookkeeping.{0,40}directly/i);
  check(idleGateIdx > -1 && bookkeepingIdx > -1 && bookkeepingIdx < idleGateIdx,
    "row53: the FIX line names the bookkeeping/bee-hive route before guards.idle_gate (D3)",
    `bookkeepingIdx=${bookkeepingIdx} idleGateIdx=${idleGateIdx} :: ${fixText}`);

  // --- 54. explicit-pathspec form: `git add <source path>` at a terminal
  // phase is refused (D1's "explicit pathspecs for git add" branch).
  const addSrcRoot = buildGitFixture("bee-write-guard-git-add-src-", { phase: "idle" });
  fs.mkdirSync(path.join(addSrcRoot, "src"), { recursive: true });
  fs.writeFileSync(path.join(addSrcRoot, "src", "new.js"), "x\n");
  const r54 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "git add src/new.js" } },
    addSrcRoot,
  );
  check(r54.status === 2, "row54: `git add` with an explicit source pathspec is refused at a terminal phase", `status=${r54.status} stderr=${r54.stderr}`);

  // --- 55. explicit-pathspec form: `git add <bookkeeping path>` at a
  // terminal phase is allowed (the exemption for add/rm/mv/checkout/restore
  // with an explicit pathspec).
  const addBkRoot = buildGitFixture("bee-write-guard-git-add-bk-", { phase: "idle" });
  fs.mkdirSync(path.join(addBkRoot, "docs"), { recursive: true });
  fs.writeFileSync(path.join(addBkRoot, "docs", "new.md"), "# x\n");
  const r55 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "git add docs/new.md" } },
    addBkRoot,
  );
  check(r55.status === 0, "row55: `git add` with an explicit bookkeeping pathspec is allowed at a terminal phase", `status=${r55.status} stderr=${r55.stderr}`);

  // --- 56. scope control (Boundary): outside a terminal phase (swarming),
  // this new git-awareness never fires — a plain `git commit` with a staged
  // source path behaves exactly as it did before this cell (unaffected;
  // proves the fix stays confined to the intake gate, never reopening
  // gated/swarming behavior).
  const swarmRoot = buildGitFixture("bee-write-guard-git-swarm-", { phase: "swarming" });
  writeState(swarmRoot, {
    phase: "swarming",
    mode: "standard",
    feature: "demo",
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  stageFile(swarmRoot, "src/feature.js", "console.log('x');\n");
  const r56 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: 'git commit -m "normal work"' } },
    swarmRoot,
  );
  check(r56.status === 0, "row56: `git commit` with a staged source path is unaffected outside a terminal phase (scope control)", `status=${r56.status} stderr=${r56.stderr}`);

  // --- 57-64. large-read guard (router-cost rc-1, D1/D2/D3/D4): a Read of a
  // big file with neither offset nor limit is denied; a slice read, a small
  // file, a directory, a nonexistent path, a disabled hook, and a custom
  // threshold all behave as specified.
  const readRoot = buildFixture("bee-write-guard-read-size-");
  const bigFile = path.join(readRoot, "big.md");
  const bigLines = 900;
  fs.writeFileSync(bigFile, `${Array.from({ length: bigLines }, (_, i) => `line ${i}`).join("\n")}\n`);
  const smallFile = path.join(readRoot, "small.md");
  fs.writeFileSync(smallFile, "line 1\nline 2\nline 3\n");
  const dirTarget = path.join(readRoot, "a-directory");
  fs.mkdirSync(dirTarget, { recursive: true });

  // --- 57. over-threshold + no offset/limit -> DENY, stderr names both escapes
  const r57 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "big.md" } }, readRoot);
  check(r57.status === 2, "row57: a Read of an over-threshold file with no offset/limit is denied", `status=${r57.status} stderr=${r57.stderr}`);
  check(/big\.md/.test(r57.stderr), "row57: stderr names the file", r57.stderr);
  check(new RegExp(`${bigLines + 1}`).test(r57.stderr) || new RegExp(`${bigLines}`).test(r57.stderr), "row57: stderr names the line count", r57.stderr);
  check(/800/.test(r57.stderr), "row57: stderr names the threshold (800 default)", r57.stderr);
  check(/limit/.test(r57.stderr), "row57: stderr names the `limit` escape", r57.stderr);
  check(/bee-extract/.test(r57.stderr), "row57: stderr names the `bee-extract` worker escape", r57.stderr);

  // --- 58. over-threshold + limit -> allow
  const r58 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "big.md", limit: 50 } }, readRoot);
  check(r58.status === 0, "row58: a Read of the same over-threshold file WITH `limit` is allowed", `status=${r58.status} stderr=${r58.stderr}`);

  // --- 59. over-threshold + offset -> allow
  const r59 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "big.md", offset: 100 } }, readRoot);
  check(r59.status === 0, "row59: a Read of the same over-threshold file WITH `offset` is allowed", `status=${r59.status} stderr=${r59.stderr}`);

  // --- 60. under-threshold + neither -> allow
  const r60 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "small.md" } }, readRoot);
  check(r60.status === 0, "row60: a Read of an under-threshold file with no offset/limit is allowed", `status=${r60.status} stderr=${r60.stderr}`);

  // --- 61. directory path -> allow (fail-open: not a regular file)
  const r61 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "a-directory" } }, readRoot);
  check(r61.status === 0, "row61: a Read of a directory path is allowed (fail-open)", `status=${r61.status} stderr=${r61.stderr}`);

  // --- 62. nonexistent path -> allow (fail-open: stat throws ENOENT)
  const r62 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "does-not-exist.md" } }, readRoot);
  check(r62.status === 0, "row62: a Read of a nonexistent path is allowed (fail-open)", `status=${r62.status} stderr=${r62.stderr}`);

  // --- 63. hooks.write-guard=false -> allow, even for the over-threshold file
  const disabledReadRoot = buildFixture("bee-write-guard-read-size-disabled-");
  fs.writeFileSync(
    path.join(disabledReadRoot, "big.md"),
    `${Array.from({ length: bigLines }, (_, i) => `line ${i}`).join("\n")}\n`,
  );
  fs.writeFileSync(
    path.join(disabledReadRoot, ".bee", "config.json"),
    `${JSON.stringify({ hooks: { "write-guard": false } }, null, 2)}\n`,
  );
  const r63 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "big.md" } }, disabledReadRoot);
  check(r63.status === 0, "row63: hooks.write-guard=false disables the size check along with the rest of the guard", `status=${r63.status} stderr=${r63.stderr}`);

  // --- 64. a custom guards.max_read_lines is honoured (small.md has 3 lines;
  // a threshold of 2 must trip on a file that the 800 default would allow).
  const customThresholdRoot = buildFixture("bee-write-guard-read-size-custom-");
  fs.writeFileSync(path.join(customThresholdRoot, "small.md"), "line 1\nline 2\nline 3\n");
  fs.writeFileSync(
    path.join(customThresholdRoot, ".bee", "config.json"),
    `${JSON.stringify({ guards: { max_read_lines: 2 } }, null, 2)}\n`,
  );
  const r64 = await runHookPayload({ tool_name: "Read", tool_input: { file_path: "small.md" } }, customThresholdRoot);
  check(r64.status === 2, "row64: a custom guards.max_read_lines=2 trips on a 3-line file the 800 default would allow", `status=${r64.status} stderr=${r64.stderr}`);
  check(/2/.test(r64.stderr), "row64: stderr names the custom threshold", r64.stderr);

  // --- 65-69. worktree-companion-hook mount recognition (fix-write-guard-
  // symlink): `bee worktree new --with-companion` symlinks a nested repo's
  // own worktree into this one at a `mountPath`, recorded in
  // `<root>/.bee/companion-session.json` as `{sessionId, worktreePath,
  // mountPath}`. A target that lexically escapes the physical worktree
  // PURELY by crossing that specific, marker-declared symlink is the
  // companion's own working tree, not an escape.
  const companionRoot = buildFixture("bee-write-guard-companion-");
  const companionMountTarget = mkFixture("bee-write-guard-companion-mount-");
  fs.writeFileSync(path.join(companionMountTarget, "foo.js"), "// companion file\n");
  fs.symlinkSync(companionMountTarget, path.join(companionRoot, "repo"));
  fs.writeFileSync(
    path.join(companionRoot, ".bee", "companion-session.json"),
    `${JSON.stringify({ sessionId: "s1", worktreePath: companionMountTarget, mountPath: "repo" }, null, 2)}\n`,
  );

  const r65 = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: "repo/foo.js", old_string: "x", new_string: "y" } },
    companionRoot,
  );
  check(r65.status === 0, "row65: an Edit inside the matched companion mount is allowed (denied pre-fix)", `status=${r65.status} stderr=${r65.stderr}`);

  const r66 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "cp new.js repo/foo.js" } },
    companionRoot,
  );
  check(r66.status === 0, "row66: a Bash-extracted target under the mount is also mapped and allowed", `status=${r66.status} stderr=${r66.stderr}`);

  const noMarkerRoot = buildFixture("bee-write-guard-companion-no-marker-");
  fs.symlinkSync(companionMountTarget, path.join(noMarkerRoot, "repo"));
  const r67 = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: "repo/foo.js", old_string: "x", new_string: "y" } },
    noMarkerRoot,
  );
  check(r67.status === 2, "row67: the same symlinked mount with NO marker file is still denied (generic containment)", `status=${r67.status} stderr=${r67.stderr}`);

  const mismatchRoot = buildFixture("bee-write-guard-companion-mismatch-");
  const otherReal = mkFixture("bee-write-guard-companion-other-");
  fs.symlinkSync(companionMountTarget, path.join(mismatchRoot, "repo"));
  fs.writeFileSync(
    path.join(mismatchRoot, ".bee", "companion-session.json"),
    `${JSON.stringify({ sessionId: "s1", worktreePath: otherReal, mountPath: "repo" }, null, 2)}\n`,
  );
  const r68 = await runHookPayload(
    { tool_name: "Edit", tool_input: { file_path: "repo/foo.js", old_string: "x", new_string: "y" } },
    mismatchRoot,
  );
  check(
    r68.status === 2,
    "row68: a marker whose declared worktreePath does NOT match the live symlink target is still denied",
    `status=${r68.status} stderr=${r68.stderr}`,
  );

  const r69 = await runHookPayload(
    { tool_name: "Read", tool_input: { file_path: "repo/foo.js" } },
    noMarkerRoot,
  );
  check(r69.status === 0, "row69: a companion-mounted Read stays allowed regardless of the marker (read tools are untouched)", `status=${r69.status} stderr=${r69.stderr}`);

  // --- 70-77. worktree-concurrency-guard, cell wcg-1: the shared detection
  // primitive isSharedNestedCheckoutTarget (guards.mjs). This is Epic 1's
  // UNWIRED primitive — tested directly (imported, in-process) rather than
  // through the hook, since it is not yet consulted by checkWrite/the hook
  // dispatch. D2 (widened by supersession 0ccc1cf3) fixes the shapes; the
  // three confirmed baselines from the validating spike (reports/
  // validation-e1.md) are locked here as regression assertions alongside the
  // primitive's intended flag/no-flag behavior.

  // Row 70 regression (existing containment, NOT this primitive): an
  // unrecognized symlink escape (a real external git repo symlinked in, no
  // companion marker) is denied TODAY by canonicalRelPath/
  // describeCrossWorktreeTarget alone, independent of concurrency — spike
  // case A, status 2. Locks the baseline this feature must not regress.
  {
    const root = buildFixture("bee-wcg1-baseline-symlink-");
    const external = mkFixture("bee-wcg1-baseline-external-");
    execFileSync("git", ["init", "-q"], { cwd: external, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(external, "foo.js"), "// external\n");
    fs.symlinkSync(external, path.join(root, "repo"));
    const r70 = await runHookPayload(
      { tool_name: "Edit", tool_input: { file_path: "repo/foo.js", old_string: "x", new_string: "y" } },
      root,
    );
    check(r70.status === 2, "row70: unverified symlink escape denied by EXISTING containment, independent of the new primitive", `status=${r70.status} stderr=${r70.stderr}`);
  }

  // Row 71-72: PLAIN nested `.git` physically inside the checkout's own tree
  // (spike case B, status 0 — unguarded today, STR65's incident shape).
  //   71: concurrent + plain nested  -> primitive FLAGS it (true).
  //   72: solo (no live session) + plain nested -> NOT flagged (D6 no-op).
  {
    const root = mkWcgRoot("bee-wcg1-plain-nested-");
    const nested = path.join(root, "repo");
    fs.mkdirSync(nested, { recursive: true });
    execFileSync("git", ["init", "-q"], { cwd: nested, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(nested, "foo.js"), "// nested plain\n");
    const target = path.join(nested, "foo.js");

    check(isSharedNestedCheckoutTarget(root, target) === false, "row72: plain nested .git with NO concurrent session is NOT flagged (D6 backward-compat no-op)", `solo`);
    addLiveSession(root);
    check(isSharedNestedCheckoutTarget(root, target) === true, "row71: plain nested .git + concurrent session IS flagged (STR65's unguarded incident shape)", `concurrent`);
  }

  // Row 73: registered git submodule (spike case C, status 0 — structurally
  // identical to case B). Even concurrent, the primitive must NOT flag it —
  // the exclusion keys off `.gitmodules` registration, not the `.git` shape.
  {
    const root = mkWcgRoot("bee-wcg1-submodule-");
    const gitOpts = { cwd: root, stdio: ["ignore", "pipe", "pipe"] };
    execFileSync("git", ["init", "-q"], gitOpts);
    execFileSync("git", ["config", "user.email", "wcg@example.com"], gitOpts);
    execFileSync("git", ["config", "user.name", "wcg fixture"], gitOpts);
    fs.writeFileSync(path.join(root, "README.md"), "root\n");
    execFileSync("git", ["add", "README.md"], gitOpts);
    execFileSync("git", ["commit", "-q", "-m", "root init"], gitOpts);
    const subRemote = mkFixture("bee-wcg1-submodule-remote-");
    execFileSync("git", ["init", "-q", "--bare"], { cwd: subRemote, stdio: ["ignore", "pipe", "pipe"] });
    const seed = mkFixture("bee-wcg1-submodule-seed-");
    const seedOpts = { cwd: seed, stdio: ["ignore", "pipe", "pipe"] };
    execFileSync("git", ["init", "-q"], seedOpts);
    execFileSync("git", ["config", "user.email", "wcg@example.com"], seedOpts);
    execFileSync("git", ["config", "user.name", "wcg fixture"], seedOpts);
    fs.writeFileSync(path.join(seed, "foo.js"), "// seed\n");
    execFileSync("git", ["add", "foo.js"], seedOpts);
    execFileSync("git", ["commit", "-q", "-m", "seed"], seedOpts);
    execFileSync("git", ["push", "-q", subRemote, "HEAD:refs/heads/main"], seedOpts);
    execFileSync("git", ["symbolic-ref", "HEAD", "refs/heads/main"], { cwd: subRemote, stdio: ["ignore", "pipe", "pipe"] });
    execFileSync("git", ["-c", "protocol.file.allow=always", "submodule", "add", "-q", subRemote, "repo"], gitOpts);
    addLiveSession(root);
    const target = path.join(root, "repo", "foo.js");
    check(isSharedNestedCheckoutTarget(root, target) === false, "row73: a real .gitmodules-registered submodule is NOT flagged even when concurrent (registration-based exclusion)", `submodule`);
  }

  // Row 74-77: VERIFIED companion mount (spike case A's positive sibling) — a
  // `.bee/companion-session.json` marker whose declared worktreePath realpath
  // matches the live mount symlink. Allowed unconditionally today
  // (resolveCompanionMountedRelPath); the primitive flags it so a concurrency
  // check can gate it.
  //   74: concurrent + verified mount -> FLAGGED (true).
  //   75: solo + verified mount       -> NOT flagged (D6 no-op).
  //   76: concurrent + marker whose worktreePath MISMATCHES the live symlink
  //       -> NOT flagged (verification fails, same posture as row68).
  //   77: concurrent + symlink mount with NO marker at all -> NOT flagged
  //       (the primitive stays narrow; containment, not this primitive,
  //       denies an unverified escape).
  {
    const mountTarget = mkFixture("bee-wcg1-companion-mount-");
    execFileSync("git", ["init", "-q"], { cwd: mountTarget, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(mountTarget, "foo.js"), "// companion file\n");
    const target = "repo/foo.js";

    const verifiedRoot = mkWcgRoot("bee-wcg1-companion-verified-");
    fs.symlinkSync(mountTarget, path.join(verifiedRoot, "repo"));
    fs.writeFileSync(
      path.join(verifiedRoot, ".bee", "companion-session.json"),
      `${JSON.stringify({ sessionId: "s1", worktreePath: mountTarget, mountPath: "repo" }, null, 2)}\n`,
    );
    check(isSharedNestedCheckoutTarget(verifiedRoot, path.join(verifiedRoot, target)) === false, "row75: verified companion mount with NO concurrent session is NOT flagged (D6 no-op)", `solo`);
    addLiveSession(verifiedRoot);
    check(isSharedNestedCheckoutTarget(verifiedRoot, path.join(verifiedRoot, target)) === true, "row74: verified companion mount + concurrent session IS flagged", `concurrent`);

    const mismatchRoot = mkWcgRoot("bee-wcg1-companion-mismatch-");
    const otherReal = mkFixture("bee-wcg1-companion-other-");
    fs.symlinkSync(mountTarget, path.join(mismatchRoot, "repo"));
    fs.writeFileSync(
      path.join(mismatchRoot, ".bee", "companion-session.json"),
      `${JSON.stringify({ sessionId: "s1", worktreePath: otherReal, mountPath: "repo" }, null, 2)}\n`,
    );
    addLiveSession(mismatchRoot);
    check(isSharedNestedCheckoutTarget(mismatchRoot, path.join(mismatchRoot, target)) === false, "row76: a marker whose worktreePath does NOT match the live symlink is NOT flagged (verification fails)", `mismatch`);

    const noMarkerRoot = mkWcgRoot("bee-wcg1-companion-no-marker-");
    fs.symlinkSync(mountTarget, path.join(noMarkerRoot, "repo"));
    addLiveSession(noMarkerRoot);
    check(isSharedNestedCheckoutTarget(noMarkerRoot, path.join(noMarkerRoot, target)) === false, "row77: a symlink mount with NO marker is NOT flagged by the primitive (containment's job, not this primitive's)", `no-marker`);
  }

  // --- 78-82. worktree-concurrency-guard, cell wcg-2: isSharedNestedCheckoutTarget
  // WIRED into the write-guard hook's Edit/Write and Bash dispatch (D1b/D3/D5).
  // Exercised through the real hook child process (not in-process), because
  // this is the behavior change: a write that used to succeed is now refused.
  // The guard fires BEFORE checkWrite, only for a physically-contained
  // (canonicalRelPath-resolved) target that is a genuinely shared nested
  // checkout AND is NOT covered by a verified companion marker; a verified
  // companion mount reached through its sanctioned symlink stays exempt (D1b:
  // "no verified companion marker covers it"). Concurrency excludes the acting
  // session, so a solo session is a pure no-op (D6).

  // Row 78 (RED-FIRST, behavior_change): an Edit into a plain nested `.git`
  // inside a shared checkout, with another live session, is DENIED. Pre-wiring
  // this write SUCCEEDED (status 0) — STR65's exact unguarded incident shape.
  {
    const root = buildFixture("bee-wcg2-edit-plain-nested-concurrent-");
    const nested = path.join(root, "repo");
    fs.mkdirSync(nested, { recursive: true });
    execFileSync("git", ["init", "-q"], { cwd: nested, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(nested, "foo.js"), "// nested plain\n");
    addLiveSession(root); // another session is live -> concurrent
    const r78 = await runHookPayload(
      { tool_name: "Edit", tool_input: { file_path: "repo/foo.js", old_string: "x", new_string: "y" }, session_id: "me" },
      root,
    );
    check(r78.status === 2, "row78: an Edit into a plain nested .git while another session is live is DENIED (was allowed pre-wiring — STR65 shape)", `status=${r78.status} stderr=${r78.stderr}`);
    // Row 79: the denial teaches the paved-road escape (D3/D4): a FRESH
    // `bee worktree new --with-companion`, never an in-place conversion.
    check(/bee worktree new --with-companion/.test(r78.stderr), "row79: the denial names `bee worktree new --with-companion` as the fix", r78.stderr);
    check(/fresh|new (companion )?worktree/i.test(r78.stderr), "row79: the fix is worded as opening a FRESH/new worktree, not upgrading the current one in place", r78.stderr);
  }

  // Row 80 (D6 backward-compat + session exclusion): the SAME Edit, but the
  // only live session is the acting session itself — isConcurrentMode excludes
  // it, so the guard is a pure no-op and the write is ALLOWED.
  {
    const root = buildFixture("bee-wcg2-edit-plain-nested-solo-");
    const nested = path.join(root, "repo");
    fs.mkdirSync(nested, { recursive: true });
    execFileSync("git", ["init", "-q"], { cwd: nested, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(nested, "foo.js"), "// nested plain\n");
    addLiveSession(root, "me"); // only the acting session is live -> solo
    const r80 = await runHookPayload(
      { tool_name: "Edit", tool_input: { file_path: "repo/foo.js", old_string: "x", new_string: "y" }, session_id: "me" },
      root,
    );
    check(r80.status === 0, "row80: the same plain-nested Edit is ALLOWED when the only live session is the acting one (D6 no-op / session exclusion)", `status=${r80.status} stderr=${r80.stderr}`);
  }

  // Row 81 (Bash dispatch branch): a Bash-extracted write target into a plain
  // nested `.git` while concurrent is DENIED too — proves the guard covers the
  // Bash branch, not only Edit/Write.
  {
    const root = buildFixture("bee-wcg2-bash-plain-nested-concurrent-");
    const nested = path.join(root, "repo");
    fs.mkdirSync(nested, { recursive: true });
    execFileSync("git", ["init", "-q"], { cwd: nested, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(nested, "foo.js"), "// nested plain\n");
    addLiveSession(root);
    const r81 = await runHookPayload(
      { tool_name: "Bash", tool_input: { command: "cp new.js repo/foo.js" }, session_id: "me" },
      root,
    );
    check(r81.status === 2, "row81: a Bash write into a plain nested .git while concurrent is DENIED (Bash branch wired)", `status=${r81.status} stderr=${r81.stderr}`);
  }

  // Row 82 (paved-road preserved, D1b exemption): a write into a VERIFIED
  // companion mount reached through its declared symlink stays ALLOWED even
  // when concurrent — the verified marker covers it, so the new guard never
  // fires (the whole point of `--with-companion`). This is the exemption that
  // separates a sanctioned companion write from an unguarded shared-checkout
  // write; without it the guard would break its own paved road.
  {
    const mountTarget = mkFixture("bee-wcg2-companion-mount-");
    execFileSync("git", ["init", "-q"], { cwd: mountTarget, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(mountTarget, "foo.js"), "// companion file\n");
    const root = buildFixture("bee-wcg2-companion-verified-concurrent-");
    fs.symlinkSync(mountTarget, path.join(root, "repo"));
    fs.writeFileSync(
      path.join(root, ".bee", "companion-session.json"),
      `${JSON.stringify({ sessionId: "s1", worktreePath: mountTarget, mountPath: "repo" }, null, 2)}\n`,
    );
    addLiveSession(root);
    const r82 = await runHookPayload(
      { tool_name: "Edit", tool_input: { file_path: "repo/foo.js", old_string: "x", new_string: "y" }, session_id: "me" },
      root,
    );
    check(r82.status === 0, "row82: a write into a verified companion mount stays ALLOWED even when concurrent (verified marker covers it — paved road preserved)", `status=${r82.status} stderr=${r82.stderr}`);
  }

  // --- 83-85. worktree-concurrency-guard, cell wcg-fix-2 (P1 review finding
  // #2): the shared-checkout detection primitive FAILING must fail CLOSED, not
  // open. plan.md's Test Matrix requires the "failure mode of the check itself"
  // (a corrupt marker / broken symlink / unreadable nested .git) to deny, not
  // silently allow because detection errored — most likely exactly during a
  // real race. Before this fix a thrown isSharedNestedCheckoutTarget propagated
  // to the hook's outer catch-all and returned 0 (allow); now the call site
  // wraps it so a genuine exception denies with a typed message.

  // Row 83 (RED-FIRST, behavior_change): an Edit into a physically-contained
  // target for which isSharedNestedCheckoutTarget THROWS is DENIED (exit 2).
  // Pre-fix this write was ALLOWED (exit 0) — the outer catch-all swallowed the
  // throw and failed open, the exact fail-open bug this cell closes.
  {
    const root = buildCallThrowingGuardsFixture("bee-wcgfix2-detect-throw-");
    const r83 = await runHookPayload(
      { tool_name: "Edit", tool_input: { file_path: "src/foo.js", old_string: "x", new_string: "y" }, session_id: "me" },
      root,
    );
    check(r83.status === 2, "row83: an Edit whose shared-checkout detection THROWS is DENIED (was allowed pre-fix — detection now fails CLOSED)", `status=${r83.status} stderr=${r83.stderr}`);
    // Row 84: the deny is the typed detection-error refusal, distinct from the
    // ordinary shared-checkout refusal — it names the errored check and the fix.
    check(/detection/i.test(r83.stderr), "row84: the deny message identifies a detection error (typed, not a silent allow)", r83.stderr);
    check(/fail(s|ed)? closed/i.test(r83.stderr), "row84: the deny message states the check fails CLOSED on error", r83.stderr);
    check(/bee worktree new --with-companion/.test(r83.stderr), "row84: the deny still points at the paved-road fix", r83.stderr);
    // The underlying error is still logged (diagnosable), same as row7's path,
    // even though the verdict is now a deny rather than a fail-open allow.
    const crashLog = path.join(root, ".bee", "logs", "hooks.jsonl");
    const crashEvent = readLastJsonl(crashLog);
    check(
      crashEvent && typeof crashEvent.error === "string" && crashEvent.error.includes("boom"),
      "row84: the underlying detection error is logged for diagnosis",
      JSON.stringify(crashEvent),
    );
  }

  // Row 85 (negative control — proves teeth): the SAME contained target, but
  // with the REAL (non-throwing) guards and no shared nested checkout, still
  // ALLOWS (exit 0). Only an actual thrown exception flips the verdict; a
  // legitimate "nothing shared here" answer is unchanged.
  {
    const root = buildFixture("bee-wcgfix2-detect-ok-control-");
    const r85 = await runHookPayload(
      { tool_name: "Edit", tool_input: { file_path: "src/foo.js", old_string: "x", new_string: "y" }, session_id: "me" },
      root,
    );
    check(r85.status === 0, "row85: the same contained Edit with real, non-throwing detection and nothing shared still ALLOWS (only a thrown error denies)", `status=${r85.status} stderr=${r85.stderr}`);
  }

  // --- 86-87. worktree-concurrency-guard-controlroot-port (Port-D4): the
  // concurrency check must consult opts.controlRoot, NOT the physical `root`
  // that the filesystem scan itself uses — the two can differ when a linked
  // worktree shares one coordination root. Proved DIFFERENTIALLY (both
  // directions), not by a single assertion: a live session record under
  // controlRoot alone must flag it, and the SAME live session sitting under
  // root alone (controlRoot has none) must NOT — the verdict must flip
  // depending on which root the concurrency check actually reads, or a bug
  // that silently kept reading `root` would still pass a one-sided test.
  {
    const physicalRoot = mkWcgRoot("bee-portd4-diff-controlroot-");
    const controlRootDir = mkWcgRoot("bee-portd4-diff-sibling-controlroot-");
    const nested = path.join(physicalRoot, "repo");
    fs.mkdirSync(nested, { recursive: true });
    execFileSync("git", ["init", "-q"], { cwd: nested, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(nested, "foo.js"), "// nested plain\n");
    const target = path.join(nested, "foo.js");

    // Row 86: a live session under controlRootDir ONLY (physicalRoot has
    // none of its own) still flags the target — proves the check reads
    // controlRoot, not root.
    addLiveSession(controlRootDir);
    check(
      isSharedNestedCheckoutTarget(physicalRoot, target, { controlRoot: controlRootDir }) === true,
      "row86: a live session under controlRoot alone (root has none) IS flagged (concurrency reads controlRoot)",
      `controlRoot=${controlRootDir} root=${physicalRoot}`,
    );
  }
  {
    const physicalRoot = mkWcgRoot("bee-portd4-diff-root-only-");
    const controlRootDir = mkWcgRoot("bee-portd4-diff-sibling-empty-");
    const nested = path.join(physicalRoot, "repo");
    fs.mkdirSync(nested, { recursive: true });
    execFileSync("git", ["init", "-q"], { cwd: nested, stdio: ["ignore", "pipe", "pipe"] });
    fs.writeFileSync(path.join(nested, "foo.js"), "// nested plain\n");
    const target = path.join(nested, "foo.js");

    // Row 87: the SAME live session, but planted under physicalRoot instead
    // (controlRootDir has none) — the target is NOT flagged. If the code
    // silently read `root` instead of `opts.controlRoot`, this would flag
    // (false positive) and expose the exact bug Port-D4 exists to prevent.
    addLiveSession(physicalRoot);
    check(
      isSharedNestedCheckoutTarget(physicalRoot, target, { controlRoot: controlRootDir }) === false,
      "row87: the SAME live session planted under root alone (controlRoot has none) is NOT flagged (concurrency ignores root)",
      `controlRoot=${controlRootDir} root=${physicalRoot}`,
    );
  }

  // --- 70-71. internals-reach guard (state-query-surface, cell sqs-a, D
  // 3fbe2f79): two-direction negative control. An inline `node -e` reach that
  // imports a bin/lib/ module is DENIED; a file-based `node <path>.mjs` run
  // that imports the same lib module (exactly how legitimate tests and
  // scripts use it) is never touched by this guard.
  const r70 = await runHookPayload(
    {
      tool_name: "Bash",
      tool_input: { command: `node -e "import('.${path.sep}.bee/bin/lib/cells.mjs').then(() => {})"` },
    },
    root,
  );
  check(r70.status === 2, "row70: inline `node -e` importing bin/lib/cells.mjs is denied (exit 2)",
    `status=${r70.status} stderr=${r70.stderr}`);
  check(r70.stderr.includes("bee status --json"), "row70: stderr names the paved read bee status --json", r70.stderr);
  check(r70.stderr.includes("bee <group> --help --json"), "row70: stderr names the paved read bee <group> --help --json", r70.stderr);

  const r71 = await runHookPayload(
    { tool_name: "Bash", tool_input: { command: "node skills/bee-hive/templates/tests/test_guards.mjs" } },
    root,
  );
  check(r71.status === 0, "row71: file-based `node <path>.mjs` run of a test that legitimately imports lib/guards.mjs is never blocked", `status=${r71.status} stderr=${r71.stderr}`);

  process.stdout.write(`\n${failures === 0 ? "ALL PASS" : `${failures} FAILURE(S)`}\n`);
  process.exitCode = failures === 0 ? 0 : 1;
}

// worktree-concurrency-guard (cell wcg-1) in-process fixture helpers: a
// minimal checkout root (no state.json / copied lib needed — the primitive
// reads only .bee/sessions, .bee/companion-session.json, .gitmodules, and
// nested `.git` nodes), plus a single live concurrent session record so
// isConcurrentMode(root) returns true.
function mkWcgRoot(prefix) {
  const root = mkFixture(prefix);
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  return root;
}

function addLiveSession(root, id = "other-live") {
  const dir = path.join(root, ".bee", "sessions");
  fs.mkdirSync(dir, { recursive: true });
  const nowIso = new Date().toISOString();
  fs.writeFileSync(
    path.join(dir, `${id}.json`),
    `${JSON.stringify({ id, started_at: nowIso, last_heartbeat: nowIso }, null, 2)}\n`,
  );
}

await main();
